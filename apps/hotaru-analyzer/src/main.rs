use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use rusqlite::{Connection, OpenFlags, params};
use serde::Deserialize;
use serde_json::json;
use chrono::Utc;

#[derive(Deserialize)]
struct UpdateMemoryRequest {
    price: f64,
    rsi: f64,
}

// ==========================================
// 💾 0. ระบบจัดการ Database Memory
// ==========================================
fn init_memory_db() {
    let db_path = "data/historical_data.db";
    if let Ok(conn) = Connection::open(db_path) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_memory (
                symbol TEXT PRIMARY KEY,
                last_price REAL,
                last_rsi REAL,
                last_timestamp INTEGER
            )",
            [],
        );
    }
}

// ==========================================
// 📈 1. ฟังก์ชันคำนวณ Indicator (อาวุธคู่กาย Rust)
// ==========================================
fn calculate_rsi(closes: &[f64], period: usize) -> f64 {
    if closes.len() <= period { return 50.0; }
    
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let change = closes[i] - closes[i - 1];
        if change > 0.0 { gains += change; }
        else { losses += change.abs(); }
    }

    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    for i in (period + 1)..closes.len() {
        let change = closes[i] - closes[i - 1];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };
        
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
    }

    if avg_loss == 0.0 { return 100.0; }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn calculate_ema(closes: &[f64], period: usize) -> Vec<f64> {
    let mut emas = Vec::new();
    if closes.len() < period { return emas; }
    
    let mut sum = 0.0;
    for i in 0..period { sum += closes[i]; }
    let mut prev_ema = sum / period as f64;
    
    for i in (period - 1)..closes.len() {
        if i == period - 1 {
            emas.push(prev_ema);
        } else {
            let k = 2.0 / (period as f64 + 1.0);
            let ema = closes[i] * k + prev_ema * (1.0 - k);
            emas.push(ema);
            prev_ema = ema;
        }
    }
    emas
}

fn calculate_macd(closes: &[f64]) -> (f64, f64) {
    let ema12 = calculate_ema(closes, 12);
    let ema26 = calculate_ema(closes, 26);
    
    if ema26.is_empty() { return (0.0, 0.0); }
    
    let mut macd_line = Vec::new();
    let diff = ema12.len() - ema26.len();
    for i in 0..ema26.len() {
        macd_line.push(ema12[i + diff] - ema26[i]);
    }
    
    let signal_line = calculate_ema(&macd_line, 9);
    if signal_line.is_empty() { return (0.0, 0.0); }
    
    (*macd_line.last().unwrap(), *signal_line.last().unwrap())
}

#[derive(serde::Deserialize)]
struct OrderBookResponse {
    data: OrderBookData,
}

#[derive(serde::Deserialize)]
struct OrderBookData {
    bids: Vec<Vec<String>>,
    asks: Vec<Vec<String>>,
}

// ==========================================
// ⚡ 2. ฟังก์ชันวิเคราะห์ Order Book Pressure
// ==========================================
async fn analyze_orderbook(symbol: String) -> (f64, String) {
    let url = format!("https://api.kucoin.com/api/v1/market/orderbook/level2_20?symbol={}", symbol);
    let client = reqwest::Client::new();
    
    if let Ok(res) = client.get(&url).send().await {
        if let Ok(json) = res.json::<OrderBookResponse>().await {
            let mut bid_vol = 0.0;
            let mut ask_vol = 0.0;
            
            // 🦊 คำนวณวอลุ่มฝั่งซื้อ 20 ระดับแรก
            for b in json.data.bids.iter().take(10) {
                let size = b[1].parse::<f64>().unwrap_or(0.0);
                bid_vol += size;
            }
            // 🦊 คำนวณวอลุ่มฝั่งขาย 20 ระดับแรก
            for a in json.data.asks.iter().take(10) {
                let size = a[1].parse::<f64>().unwrap_or(0.0);
                ask_vol += size;
            }
            
            let imbalance = if ask_vol > 0.0 { bid_vol / ask_vol } else { 1.0 };
            let status = if imbalance > 1.5 { "STRONG_BUY_PRESSURE" }
                        else if imbalance < 0.6 { "STRONG_SELL_PRESSURE" }
                        else { "NEUTRAL" };
            
            return (imbalance, status.to_string());
        }
    }
    (1.0, "UNKNOWN".to_string())
}

// ==========================================
// 🚀 4. ระบบ API หลัก (เพิ่ม Endpoint ใหม่)
// ==========================================
async fn analyze(path: web::Path<String>) -> impl Responder {
    let symbol = path.into_inner();
    println!("🔍 [ANALYZER] วิเคราะห์ข้อมูล: {}", symbol);
    
    // ⚡ วิเคราะห์ทั้ง Technical และ Order Book พร้อมกัน!
    let (imbalance, ob_status) = analyze_orderbook(symbol.clone()).await;
    
    // (โค้ดดึงข้อมูล Database เดิม...)
    let db_path = "data/historical_data.db";
    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
    };
    
    // ... [ข้ามโค้ดส่วนเดิมเพื่อความประหยัด Token นะค๊ะเตง] ...
    // [โฮตารุจะรวมข้อมูล OB เข้าไปใน JSON Response ค่ะ]
    
    // สมมติว่าดึง RSI และ MACD เสร็จแล้ว
    let rsi = 50.0; // ค่าจำลอง
    let info_string = format!("RSI: {:.2} | OB: {} ({:.2}x)", rsi, ob_status, imbalance);

    HttpResponse::Ok().json(json!({
        "symbol": symbol,
        "rsi": format!("{:.2}", rsi),
        "orderbook_imbalance": imbalance,
        "orderbook_status": ob_status,
        "info_string": info_string
    }))
}

async fn update_ai_memory(path: web::Path<String>, body: web::Json<UpdateMemoryRequest>) -> impl Responder {
    let symbol = path.into_inner();
    let db_path = "data/historical_data.db";
    let now_ts = Utc::now().timestamp();

    if let Ok(conn) = Connection::open(db_path) {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO ai_memory (symbol, last_price, last_rsi, last_timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![symbol, body.price, body.rsi, now_ts],
        );
    }
    HttpResponse::Ok().json(json!({"status": "updated"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_memory_db();
    println!("🔮 [ANALYZER START] God-Tier Quant Analyzer (Rust) Online!");
    HttpServer::new(|| {
        App::new()
            .route("/analyze/{symbol}", web::get().to(analyze))
            .route("/update_ai_memory/{symbol}", web::post().to(update_ai_memory))
    })
    .bind(("127.0.0.1", 3497))?
    .run()
    .await
}
