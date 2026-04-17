use rusqlite::{Connection, Result};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

fn init_db() -> Result<Connection> {
    let db_path = "data/oracle_data.db";
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS market_status (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            fng TEXT,
            btc_status TEXT
        )",
        [],
    )?;
    Ok(conn)
}

#[tokio::main]
async fn main() {
    println!("🔮 [ORACLE START] อัปเกรด Oracle เป็นโหมด 'ตาทิพย์รอบทิศ' (All Binance Pairs)...");
    let conn = init_db().expect("❌ ไม่สามารถสร้าง DB สำหรับ Oracle ได้");
    let client = reqwest::Client::new();

    loop {
        let (fng, all_binance) = tokio::join!(
            fetch_fear_and_greed(),
            fetch_all_binance_tickers()
        );

        let mut binance_map = serde_json::Map::new();
        let mut btc_status = String::from("Unknown");

        if let Some(arr) = all_binance.as_array() {
            for item in arr {
                if let (Some(sym), Some(price), Some(vol)) = (
                    item["symbol"].as_str(), 
                    item["lastPrice"].as_str(), 
                    item["volume"].as_str()
                ) {
                    if sym.ends_with("USDT") {
                        binance_map.insert(sym.to_string(), json!({ "price": price, "volume": vol }));
                        if sym == "BTCUSDT" {
                            btc_status = format!("${} | Vol: {}", price, vol);
                        }
                    }
                }
            }
        }

        println!("--------------------------------------------------");
        println!("🌡️ F&G: {}", fng);
        println!("🪙 อัปเดตข้อมูล Binance แล้ว {} เหรียญ", binance_map.len());
        println!("--------------------------------------------------");

        // Save DB
        let _ = conn.execute(
            "INSERT INTO market_status (fng, btc_status) VALUES (?1, ?2)",
            &[&fng, &btc_status],
        );

        // ส่งข้อมูล "ทุกเหรียญ" ไปให้สมอง AI เลือกใช้
        let payload = json!({
            "fng": fng,
            "binance_market": binance_map
        });

        let _ = client.post("http://127.0.0.1:8000/update_sentiment")
            .json(&payload)
            .send()
            .await;

        sleep(Duration::from_secs(60)).await;
    }
}

async fn fetch_fear_and_greed() -> String {
    let url = "https://api.alternative.me/fng/?limit=1";
    if let Ok(res) = reqwest::get(url).await {
        if let Ok(json) = res.json::<Value>().await {
            let value = json["data"][0]["value"].as_str().unwrap_or("50");
            let class = json["data"][0]["value_classification"].as_str().unwrap_or("Neutral");
            return format!("{} ({})", value, class);
        }
    }
    "50 (Neutral)".to_string()
}

// 🟢 ดึงข้อมูลทุกเหรียญในตลาดรวดเดียว
async fn fetch_all_binance_tickers() -> Value {
    let url = "https://api.binance.com/api/v3/ticker/24hr";
    if let Ok(res) = reqwest::get(url).await {
        if let Ok(json) = res.json::<Value>().await {
            return json;
        }
    }
    json!([])
}
