use reqwest::Client;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

// 📍 เปลี่ยน Path ให้ชี้ไปที่ไฟล์ DB ของเตง (ใช้ Absolute Path ดีที่สุดค่ะ)
const DB_PATH: &str = "data/historical_data.db"; 

// 🪙 รายชื่อเหรียญที่เราต้องการดูดกราฟ
const PAIRS: &[&str] = &[
    "BTC-USDT", "ETH-USDT", "SOL-USDT", "XRP-USDT", 
    "DOGE-USDT", "BNB-USDT", "SHIB-USDT", "PEPE-USDT"
];

#[tokio::main]
async fn main() {
    println!("🚀 [SCRAPER START] ระบบดูดข้อมูลกราฟ (Rust) เริ่มทำงาน!");
    
    // เปิดการเชื่อมต่อ Database
    let conn = Connection::open(DB_PATH).expect("❌ เปิดไฟล์ Database ไม่ได้!");
    
    // 🛡️ ท่าไม้ตาย: สร้าง Unique Index ป้องกันข้อมูลซ้ำซ้อนเวลาดึงกราฟเหลื่อมทับกัน
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_klines_sym_time ON klines(symbol, timestamp)",
        [],
    ).expect("สร้าง Index ไม่สำเร็จ!");

    let client = Client::new();

    loop {
        println!("🔄 กำลังดึงกราฟ 15 นาทีล่าสุดจาก KuCoin...");
        
        for symbol in PAIRS {
            let url = format!("https://api.kucoin.com/api/v1/market/candles?type=15min&symbol={}", symbol);
            
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(json) = res.json::<Value>().await {
                    if let Some(data_array) = json["data"].as_array() {
                        
                        // เอาแค่ 5 แท่งล่าสุดต่อรอบก็พอ เพื่อความรวดเร็วและประหยัดเน็ต
                        for item in data_array.iter().take(5) {
                            let ts = item[0].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
                            let open = item[1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let close = item[2].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let high = item[3].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let low = item[4].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            let vol = item[5].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

                            // บันทึกลง DB ถ้าเวลาซ้ำมันจะอัปเดตแท่งเดิมให้สมบูรณ์ขึ้น!
                            let _ = conn.execute(
                                "INSERT OR REPLACE INTO klines (symbol, timestamp, open, close, high, low, volume) 
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                params![symbol, ts, open, close, high, low, vol],
                            );
                        }
                    }
                }
            }
            // 🛑 พักหายใจ 1 วินาที ป้องกัน KuCoin รำคาญแล้วแบน IP เราค่ะ!
            sleep(Duration::from_secs(1)).await;
        }
        
        println!("✅ อัปเดตข้อมูลลง Database เรียบร้อย! พัก 1 นาทีก่อนดึงรอบต่อไป...");
        sleep(Duration::from_secs(60)).await; // วนลูปดูดกราฟทุกๆ 1 นาที
    }
}
