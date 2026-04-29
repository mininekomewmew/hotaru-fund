use reqwest::Client;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

// 📍 Path ของ Database
const DB_PATH: &str = "data/historical_data.db";

// 🦊 ฟังก์ชันสำหรับดึงรายชื่อเหรียญที่ "วอลุ่มดี-วิ่งแรง" จาก KuCoin แบบ Real-time
async fn scan_hot_symbols(client: &Client) -> Vec<String> {
    println!("🔍 [SCANNER] กำลังกวาดหาเหรียญที่น่าสนใจในตลาด...");
    let mut hot_symbols = Vec::new();
    
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    if let Ok(res) = client.get(url).send().await {
        if let Ok(json) = res.json::<Value>().await {
            if let Some(tickers) = json["data"]["ticker"].as_array() {
                let mut candidates: Vec<(String, f64, f64)> = tickers.iter()
                    .filter_map(|t| {
                        let symbol = t["symbol"].as_str()?.to_string();
                        if symbol.ends_with("-USDT") && !symbol.contains("3L") && !symbol.contains("3S") {
                            let vol = t["vol"].as_str()?.parse::<f64>().unwrap_or(0.0);
                            let change = t["changeRate"].as_str()?.parse::<f64>().unwrap_or(0.0);
                            if vol > 100000.0 { 
                                return Some((symbol, vol, change));
                            }
                        }
                        None
                    })
                    .collect();

                candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

                for (sym, _, _) in candidates.iter().take(20) {
                    hot_symbols.push(sym.clone());
                }
            }
        }
    }
    
    if hot_symbols.is_empty() {
        hot_symbols = vec!["BTC-USDT".to_string(), "ETH-USDT".to_string(), "SOL-USDT".to_string()];
    }
    
    println!("🎯 [SCANNER] เจอเหรียญน่าสนใจ 20 อันดับแรกแล้วค่ะ!");
    hot_symbols
}

#[tokio::main]
async fn main() {
    println!("🚀 [SCRAPER START] ระบบ Smart Scanner (Rust) เริ่มทำงาน!");
    
    let conn = Connection::open(DB_PATH).expect("❌ เปิดไฟล์ Database ไม่ได้!");
    
    // 🛡️ แยกการสร้างตารางและ Index ออกจากกันเพื่อความปลอดภัยค่ะ!
    conn.execute(
        "CREATE TABLE IF NOT EXISTS klines (
            symbol TEXT,
            timestamp INTEGER,
            open REAL,
            close REAL,
            high REAL,
            low REAL,
            volume REAL
        )",
        [],
    ).expect("❌ สร้างตาราง klines ไม่สำเร็จ!");

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_klines_sym_time ON klines(symbol, timestamp)",
        [],
    ).expect("❌ สร้าง Index ไม่สำเร็จ!");

    println!("✅ Database Initialized Successfully!");
    
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    loop {
        let pairs = scan_hot_symbols(&client).await;
        
        println!("🔄 กำลังดึงกราฟของเหรียญที่เลือกมาให้เตงนะคะ...");
        
        for symbol in &pairs {
            let url = format!("https://api.kucoin.com/api/v1/market/candles?type=15min&symbol={}", symbol);
            
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(json) = res.json::<Value>().await {
                    if let Some(data_array) = json["data"].as_array() {
                        for item in data_array.iter().take(10) {
                            let ts = item[0].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
                            let open = item[1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let close = item[2].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let high = item[3].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let low = item[4].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let vol = item[5].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

                            let _ = conn.execute(
                                "INSERT OR REPLACE INTO klines (symbol, timestamp, open, close, high, low, volume) 
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                params![symbol, ts, open, close, high, low, vol],
                            );
                        }
                    }
                }
            }
            sleep(Duration::from_millis(150)).await;
        }
        
        println!("✅ อัปเดตข้อมูลเหรียญซิ่งลง Database เรียบร้อย! พัก 1 นาทีก่อนสแกนรอบใหม่...");
        sleep(Duration::from_secs(60)).await;
    }
}
