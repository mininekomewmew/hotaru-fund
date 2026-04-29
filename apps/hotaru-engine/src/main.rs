use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Local;

const STATE_FILE: &str = "data/paper_state.json";
const LOG_FILE: &str = "data/trade_log.txt";
const ORACLE_DB: &str = "data/oracle_data.db"; // 🦊 เชื่อมต่อคลังความรู้ Oracle ค่ะ

// 🌡️ ฟังก์ชันดึงค่า Fear & Greed ล่าสุดจาก Rust Oracle
fn get_latest_fng_score() -> i32 {
    if let Ok(conn) = rusqlite::Connection::open(ORACLE_DB) {
        if let Ok(mut stmt) = conn.prepare("SELECT fng FROM market_status ORDER BY id DESC LIMIT 1") {
            let fng_str: String = stmt.query_row([], |row| row.get(0)).unwrap_or("50 (Neutral)".to_string());
            let score_part = fng_str.split(' ').next().unwrap_or("50");
            return score_part.parse::<i32>().unwrap_or(50);
        }
    }
    50
}

fn log_trade(message: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", now, message);
    println!("{}", message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct State {
    usdt_balance: f64,
    holdings: HashMap<String, f64>,
    entry_prices: HashMap<String, f64>,
    #[serde(default)]
    highest_prices: HashMap<String, f64>,
    #[serde(default)] 
    robin_hood_spent: f64, 
}

fn load_state() -> State {
    if let Ok(data) = fs::read_to_string(STATE_FILE) {
        if let Ok(state) = serde_json::from_str(&data) { return state; }
    }
    State {
        usdt_balance: 100.0, holdings: HashMap::new(), entry_prices: HashMap::new(),
        highest_prices: HashMap::new(), robin_hood_spent: 0.0,
    }
}

fn save_state(state: &State) {
    let data = serde_json::to_string_pretty(state).unwrap();
    let _ = fs::write(STATE_FILE, data);
}

#[derive(Debug, Deserialize)]
struct AiDecision { action: String, message: String, #[serde(default)] size: f64 }

#[derive(Debug, Deserialize)]
struct AnalyzerResponse { rsi: String, should_ask_ai: bool, info_string: String, orderbook_imbalance: f64 }

async fn get_analyzer_data(client: &Client, symbol: &str) -> (f64, bool, String, f64) {
    let url = format!("http://127.0.0.1:3497/analyze/{}", symbol);
    if let Ok(res) = client.get(&url).send().await {
        if let Ok(data) = res.json::<AnalyzerResponse>().await {
            let rsi_val = data.rsi.parse::<f64>().unwrap_or(50.0);
            return (rsi_val, data.should_ask_ai, data.info_string, data.orderbook_imbalance);
        }
    }
    (50.0, true, "Error".to_string(), 1.0)
}

async fn update_analyzer_memory(client: &Client, symbol: &str, price: f64, rsi: f64) {
    let url = format!("http://127.0.0.1:3497/update_ai_memory/{}", symbol);
    let payload = json!({ "price": price, "rsi": rsi });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn ask_ai(client: &Client, symbol: &str, price: f64, entry_price: f64, info: &str, pos_count: usize) -> AiDecision {
    let payload = json!({ "symbol": symbol, "price": price, "entry_price": entry_price, "summary": info, "current_positions": pos_count });
    if let Ok(res) = client.post("http://127.0.0.1:8000/analyze").json(&payload).send().await {
        if let Ok(decision) = res.json::<AiDecision>().await { return decision; }
    }
    AiDecision { action: "HOLD".to_string(), message: "AI Link Error".to_string(), size: 0.0 }
}

async fn get_current_price(client: &Client, symbol: &str) -> f64 {
    let url = format!("https://api.kucoin.com/api/v1/market/orderbook/level1?symbol={}", symbol);
    if let Ok(res) = client.get(&url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(price_str) = json["data"]["price"].as_str() { return price_str.parse().unwrap_or(0.0); }
        }
    }
    0.0
}

async fn get_top_usdt_pairs(client: &Client, limit: usize) -> Vec<String> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    if let Ok(res) = client.get(url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(ticker_array) = json["data"]["ticker"].as_array() {
                let mut pairs: Vec<(String, f64)> = ticker_array.iter().filter_map(|t| {
                    let sym = t["symbol"].as_str()?;
                    if !sym.ends_with("-USDT") { return None; }
                    let change = t["changeRate"].as_str()?.parse::<f64>().unwrap_or(0.0);
                    Some((sym.to_string(), change))
                }).collect();
                pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                return pairs.into_iter().take(limit).map(|(s, _)| s).collect();
            }
        }
    }
    vec!["BTC-USDT".to_string()]
}

#[tokio::main]
async fn main() {
    log_trade("🏹 [ENGINE START] ปฏิบัติการกู้ชีพพอร์ต! โหมด Dynamic Profit + Sentiment Gate เริ่มทำงานค่ะ!");
    let client = Client::new();

    loop {
        let mut state = load_state();
        let fng_score = get_latest_fng_score(); 
        
        let mut watch_list: Vec<String> = state.holdings.keys().cloned().collect();
        let top_coins = get_top_usdt_pairs(&client, 12).await;
        for coin in top_coins { if !watch_list.contains(&coin) { watch_list.push(coin); } }

        for symbol in &watch_list {
            let current_price = get_current_price(&client, symbol).await;
            if current_price == 0.0 { continue; }
            let entry_price = *state.entry_prices.get(symbol).unwrap_or(&0.0);

            if entry_price > 0.0 {
                let current_pnl = ((current_price - entry_price) / entry_price) * 100.0;
                let mut peak_price = *state.highest_prices.get(symbol).unwrap_or(&current_price);
                if current_price > peak_price { 
                    peak_price = current_price;
                    state.highest_prices.insert(symbol.clone(), current_price); 
                }
                let peak_pnl = ((peak_price - entry_price) / entry_price) * 100.0;

                let trail_dist = if peak_pnl < 1.5 { 0.2 } else if peak_pnl < 3.5 { 0.4 } else { 0.8 };
                if current_pnl > 0.5 && current_price < peak_price * (1.0 - (trail_dist / 100.0)) {
                    if let Some(qty) = state.holdings.remove(symbol) {
                        state.entry_prices.remove(symbol);
                        state.highest_prices.remove(symbol);
                        state.usdt_balance += qty * current_price;
                        log_trade(&format!("💰 [DYNAMIC PROFIT LOCKED] {} @ {:.4} | PNL: {:.2}%", symbol, current_price, current_pnl));
                        continue;
                    }
                }
                if current_pnl <= -1.5 {
                    if let Some(qty) = state.holdings.remove(symbol) {
                        state.entry_prices.remove(symbol);
                        state.highest_prices.remove(symbol);
                        state.usdt_balance += qty * current_price;
                        log_trade(&format!("🚨 [HARD STOP LOSS] {} @ {:.4} | PNL: {:.2}%", symbol, current_price, current_pnl));
                        continue;
                    }
                }
            }

            let (rsi, suggest_ask, info_str, _) = get_analyzer_data(&client, symbol).await;
            let mut pass_gate = true;
            if entry_price == 0.0 && fng_score < 20 {
                pass_gate = false;
            }

            if pass_gate && (entry_price == 0.0 || suggest_ask) {
                let d = ask_ai(&client, symbol, current_price, entry_price, &info_str, state.holdings.len()).await;
                update_analyzer_memory(&client, symbol, current_price, rsi).await;

                if d.action == "BUY" && entry_price == 0.0 {
                    let buy_amount = state.usdt_balance * 0.35;
                    if state.usdt_balance >= buy_amount && state.holdings.len() < 6 {
                        let qty = buy_amount / current_price;
                        state.usdt_balance -= buy_amount;
                        state.holdings.insert(symbol.clone(), qty);
                        state.entry_prices.insert(symbol.clone(), current_price);
                        state.highest_prices.insert(symbol.clone(), current_price);
                        log_trade(&format!("🟢 [BUY] {} @ {:.4} | Amount: {:.2} USDT", symbol, current_price, buy_amount));
                    }
                }
            }
        }
        save_state(&state);
        sleep(Duration::from_secs(45)).await;
    }
}
