use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Local;

// ==========================================
// 📍 ตั้งค่า Path แบบเจาะจง
// ==========================================
const STATE_FILE: &str = "data/paper_state.json";
const LOG_FILE: &str = "data/trade_log.txt";

// ==========================================
// 📝 0. ระบบ Logger
// ==========================================
fn log_trade(message: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", now, message);
    
    println!("{}", message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

// ==========================================
// 📊 1. โครงสร้างกระเป๋าเงิน
// ==========================================
#[derive(Debug, Serialize, Deserialize)]
struct State {
    usdt_balance: f64,
    holdings: HashMap<String, f64>,
    entry_prices: HashMap<String, f64>,
    #[serde(default)] 
    robin_hood_spent: f64, 
}

const NORMAL_BUY_AMOUNT: f64 = 10.0;
const ROBIN_HOOD_AMOUNT: f64 = 5.0;      
const ROBIN_HOOD_MAX_BUDGET: f64 = 50.0; 

fn load_state() -> State {
    if let Ok(data) = fs::read_to_string(STATE_FILE) {
        match serde_json::from_str(&data) {
            Ok(state) => return state,
            Err(e) => println!("⚠️ [JSON ERROR] ไฟล์ paper_state.json มีปัญหา: {}", e),
        }
    }
    State {
        usdt_balance: 100.0,
        holdings: HashMap::new(),
        entry_prices: HashMap::new(),
        robin_hood_spent: 0.0,
    }
}

fn save_state(state: &State) {
    let data = serde_json::to_string_pretty(state).unwrap();
    let _ = fs::write(STATE_FILE, data);
}

// ==========================================
// 🤖 2. โครงสร้างข้อมูลที่คุยกับระบบภายนอก
// ==========================================
#[derive(Debug, Deserialize)]
struct AiDecision {
    action: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct AnalyzerResponse {
    rsi: String,
    recently_oversold: bool,
}

async fn get_analyzer_data(client: &Client, symbol: &str) -> (f64, bool) {
    let url = format!("http://127.0.0.1:3497/analyze/{}", symbol);
    if let Ok(res) = client.get(&url).send().await {
        if let Ok(data) = res.json::<AnalyzerResponse>().await {
            let rsi_val = data.rsi.parse::<f64>().unwrap_or(50.0);
            return (rsi_val, data.recently_oversold);
        }
    }
    (50.0, false)
}

async fn ask_ai(client: &Client, symbol: &str, price: f64, entry_price: f64) -> AiDecision {
    let payload = json!({
        "symbol": symbol,
        "price": price,
        "entry_price": entry_price,
        "summary": "Checking market condition..."
    });
    let res = client.post("http://127.0.0.1:8000/analyze").json(&payload).send().await;
    if let Ok(response) = res {
        if let Ok(decision) = response.json::<AiDecision>().await {
            return decision;
        }
    }
    AiDecision { action: "HOLD".to_string(), message: "Error contacting AI".to_string() }
}

async fn get_current_price(client: &Client, symbol: &str) -> f64 {
    let url = format!("https://api.kucoin.com/api/v1/market/orderbook/level1?symbol={}", symbol);
    if let Ok(res) = client.get(&url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(price_str) = json["data"]["price"].as_str() {
                return price_str.parse().unwrap_or(0.0);
            }
        }
    }
    0.0
}

// 🟢 ฟังก์ชันใหม่: ดึง Top 10 เหรียญจาก Binance (เพื่อเป็นผู้นำตลาด) แต่เอามาเทรดที่ KuCoin
async fn get_top_usdt_pairs(client: &Client, limit: usize) -> Vec<String> {
    let url = "https://api.binance.com/api/v3/ticker/24hr";
    let mut symbols = Vec::new();

    if let Ok(res) = client.get(url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(ticker_array) = json.as_array() {
                let mut pairs: Vec<(String, f64)> = ticker_array.iter().filter_map(|t| {
                    let sym = t["symbol"].as_str()?;
                    if !sym.ends_with("USDT") { return None; }
                    // ตัดพวกเหรียญแปลกๆ หรือ Stablecoin ออก
                    if sym.contains("UP") || sym.contains("DOWN") || sym.contains("USDC") || sym.contains("BUSD") { return None; }
                    
                    let vol = t["quoteVolume"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    Some((sym.to_string(), vol))
                }).collect();

                // เรียงตาม Volume (มูลค่าซื้อขายรวม)
                pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                
                // แปลงชื่อจาก Binance (BTCUSDT) เป็น KuCoin (BTC-USDT)
                symbols = pairs.into_iter().take(limit).map(|(s, _)| {
                    let base = s.replace("USDT", "");
                    format!("{}-USDT", base)
                }).collect();
            }
        }
    }
    
    if symbols.is_empty() {
        println!("⚠️ [WARNING] ดึง Top 10 จาก Binance ไม่ได้ ใช้เหรียญมาตรฐานแทน...");
        return vec!["BTC-USDT".to_string(), "ETH-USDT".to_string(), "SOL-USDT".to_string()];
    }
    symbols
}

// ==========================================
// 🚀 3. ระบบทำงานหลัก
// ==========================================
#[tokio::main]
async fn main() {
    log_trade("🏹 [ENGINE START] ระบบพร้อม! โหมดสแกนตลาด (Smart Market Scan) ทำงาน!");
    let client = Client::new();

    loop {
        let mut state = load_state();
        let mut cut_loss_happened = false;

        // 1. เอาเหรียญที่มีในพอร์ตใส่ลง List ก่อน
        let mut watch_list: Vec<String> = state.holdings.keys().cloned().collect();
        
        // 2. ดึง Top 10 เหรียญที่มาแรงที่สุดจาก KuCoin มารวมด้วย!
        println!("🔍 [MARKET SCAN] กำลังดึง Top 10 เหรียญมาแรงจากตลาด...");
        let top_coins = get_top_usdt_pairs(&client, 10).await;
        for coin in top_coins {
            if !watch_list.contains(&coin) {
                watch_list.push(coin);
            }
        }

        // เผื่อเน็ตหลุด ดึง API ไม่ได้
        if watch_list.is_empty() {
            watch_list.push("BTC-USDT".to_string());
        }

        let mut best_pnl_in_portfolio = 0.0;
        for (h_sym, _) in &state.holdings {
            let current_price = get_current_price(&client, h_sym).await;
            let entry = *state.entry_prices.get(h_sym).unwrap_or(&0.0);
            if entry > 0.0 {
                let pnl = ((current_price - entry) / entry) * 100.0;
                if pnl > best_pnl_in_portfolio {
                    best_pnl_in_portfolio = pnl;
                }
            }
        }

        let dynamic_cut_loss = if best_pnl_in_portfolio > 1.0 { -0.8 } else { -1.5 };

        let symbols_to_process = watch_list.clone();

        for symbol in &symbols_to_process {
            let current_price = get_current_price(&client, symbol).await;
            if current_price == 0.0 { continue; }
            let entry_price = *state.entry_prices.get(symbol).unwrap_or(&0.0);

            // ✂️ AUTO CUT LOSS
            if entry_price > 0.0 {
                let current_pnl = ((current_price - entry_price) / entry_price) * 100.0;
                if current_pnl <= dynamic_cut_loss {
                    let qty = state.holdings.remove(symbol).unwrap();
                    state.entry_prices.remove(symbol);
                    let revenue = qty * current_price;
                    state.usdt_balance += revenue;
                    
                    log_trade(&format!("🔴 [AUTO SELL] {} @ {:.4} | PNL: {:.2}% | Reason: DYNAMIC CUT LOSS", symbol, current_price, current_pnl));
                    cut_loss_happened = true;
                    continue; 
                }
            }

            // 🛡️ SMART PRE-FILTER
            if entry_price == 0.0 {
                let (rsi, recently_oversold) = get_analyzer_data(&client, symbol).await;
                // 🦊 โฮตารุใจดีขึ้น: ถ้า RSI < 45 ก็ส่งให้ AI ดูได้เลย ไม่ต้องรอให้เน่าก้นเหวค่ะ
                if rsi >= 45.0 && !recently_oversold {
                    println!("💤 [SMART FILTER] {} RSI = {:.2} (ยังไม่ถึงจุดย่อ) -> ไม่กวน AI", symbol, rsi);
                    continue; 
                }
            } else {
                let current_pnl = ((current_price - entry_price) / entry_price) * 100.0;
                if current_pnl < 2.5 {
                    println!("💤 [SMART FILTER] {} PNL = {:.2}% (ยังไม่ถึงเป้าขาย) -> นั่งทับมือ ไม่กวน AI!", symbol, current_pnl);
                    continue; 
                }
            }

            // 🧠 ถาม AI (ถึงตรงนี้ได้คือผ่านเลขามาแล้ว)
            let decision = ask_ai(&client, symbol, current_price, entry_price).await;

            if decision.action == "BUY" && entry_price == 0.0 {
                if state.usdt_balance >= NORMAL_BUY_AMOUNT {
                    let qty = NORMAL_BUY_AMOUNT / current_price;
                    state.usdt_balance -= NORMAL_BUY_AMOUNT;
                    state.holdings.insert(symbol.clone(), qty);
                    state.entry_prices.insert(symbol.clone(), current_price);
                    
                    log_trade(&format!("🟢 [BUY] {} @ {:.4} (Normal 10 USDT)", symbol, current_price));
                }
            } 
            else if decision.action == "SELL" && entry_price > 0.0 {
                if state.holdings.contains_key(symbol) {
                    let qty = state.holdings.remove(symbol).unwrap();
                    state.entry_prices.remove(symbol);
                    let revenue = qty * current_price;
                    state.usdt_balance += revenue;
                    
                    let pnl_pct = ((current_price - entry_price) / entry_price) * 100.0;
                    log_trade(&format!("💰 [AI SELL] {} @ {:.4} | PNL: {:.2}% | Reason: {}", symbol, current_price, pnl_pct, decision.message));

                    if pnl_pct < -1.0 || decision.message.to_uppercase().contains("CUT") {
                        cut_loss_happened = true;
                    }
                }
            }
        }

        // 🏹 ROBIN HOOD
        if cut_loss_happened && state.robin_hood_spent < ROBIN_HOOD_MAX_BUDGET {
            let mut best_coin = String::new();
            let mut best_pnl = 0.0;
            let mut best_price = 0.0;

            for (h_sym, _) in &state.holdings {
                let current_price = get_current_price(&client, h_sym).await;
                let entry = *state.entry_prices.get(h_sym).unwrap_or(&0.0);
                if entry > 0.0 {
                    let pnl = ((current_price - entry) / entry) * 100.0;
                    if pnl > best_pnl {
                        best_pnl = pnl;
                        best_coin = h_sym.clone();
                        best_price = current_price;
                    }
                }
            }

            if best_pnl > 0.5 && state.usdt_balance >= ROBIN_HOOD_AMOUNT && state.robin_hood_spent + ROBIN_HOOD_AMOUNT <= ROBIN_HOOD_MAX_BUDGET {
                if let (Some(&old_qty), Some(&old_entry)) = (state.holdings.get(&best_coin), state.entry_prices.get(&best_coin)) {
                    let bonus_qty = ROBIN_HOOD_AMOUNT / best_price;
                    let total_qty = old_qty + bonus_qty;
                    let new_avg_entry = ((old_qty * old_entry) + (bonus_qty * best_price)) / total_qty;
                    
                    state.usdt_balance -= ROBIN_HOOD_AMOUNT;
                    state.robin_hood_spent += ROBIN_HOOD_AMOUNT;
                    state.holdings.insert(best_coin.clone(), total_qty);
                    state.entry_prices.insert(best_coin.clone(), new_avg_entry);
                    
                    log_trade(&format!("🏹 [ROBIN HOOD] อัดฉีด 5 USDT ให้ {} (PNL ตอนนี้: +{:.2}%) | งบเหลือ: {:.2}$", best_coin, best_pnl, ROBIN_HOOD_MAX_BUDGET - state.robin_hood_spent));
                }
            }
        }

        save_state(&state);
        println!("⏳ เช็ครอบนี้เสร็จแล้ว พักผ่อน 5 นาที...");
        sleep(Duration::from_secs(300)).await; 
    }
}
