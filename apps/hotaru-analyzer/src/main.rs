use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use rusqlite::{Connection, OpenFlags};
use serde_json::json;

// ==========================================
// 📈 1. ฟังก์ชันคำนวณ RSI (เหมือนเดิมเป๊ะ)
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

// ==========================================
// 📊 2. ฟังก์ชันคำนวณ EMA & MACD (อาวุธใหม่!)
// ==========================================
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

// ==========================================
// 🕸️ 3. ฟังก์ชันคำนวณ Bollinger Bands (ตาข่ายดักก้นเหว)
// ==========================================
fn calculate_bb(closes: &[f64], period: usize) -> (f64, f64, f64) {
    if closes.len() < period { return (0.0, 0.0, 0.0); }
    let slice = &closes[(closes.len() - period)..];
    let sum: f64 = slice.iter().sum();
    let sma = sum / period as f64;
    
    let mut variance = 0.0;
    for &val in slice { variance += (val - sma).powi(2); }
    variance /= period as f64;
    let std_dev = variance.sqrt();
    
    (sma - 2.0 * std_dev, sma, sma + 2.0 * std_dev)
}

// ==========================================
// 🚀 4. ระบบ API หลัก
// ==========================================
async fn analyze(path: web::Path<String>) -> impl Responder {
    let symbol = path.into_inner();
    println!("🔍 [ANALYZER] กำลังวิเคราะห์ข้อมูล: {}", symbol);
    let db_path = "data/historical_data.db";
    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            println!("❌ เปิด DB ไม่ได้: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "symbol": symbol, "rsi": 50.0, "status": "DB_ERROR", "message": e.to_string()
            }));
        }
    };

    // 📍 จุดที่ 2: เปลี่ยนคำว่า candles เป็น "klines" ตรง SELECT ค่ะ!
    let mut stmt = match conn.prepare("SELECT close FROM klines WHERE symbol = ? ORDER BY timestamp DESC LIMIT 100") {
        Ok(s) => s,
        Err(e) => {
            println!("❌ ค้นหาตารางไม่เจอ: {}", e);
            return HttpResponse::InternalServerError().json(json!({"status": "QUERY_ERROR"}))
        }
    };

    let mut closes = Vec::new();
    let rows = stmt.query_map([&symbol], |row| row.get::<_, f64>(0));
    if let Ok(mapped_rows) = rows {
        for row in mapped_rows.flatten() { closes.push(row); }
    }

    if closes.len() < 26 { // MACD ต้องการอย่างน้อย 26 แท่ง
        return HttpResponse::Ok().json(json!({
            "symbol": symbol, "rsi": 50.0, "status": "NOT_ENOUGH_DATA",
            "recently_oversold": false, "info_string": "Data insufficient"
        }));
    }

    closes.reverse(); // พลิก อดีต -> ปัจจุบัน
    let current_price = *closes.last().unwrap_or(&0.0);

    // 🧠 คำนวณ Indicator โหดๆ
    let rsi = calculate_rsi(&closes, 14);
    let (macd, macd_signal) = calculate_macd(&closes);
    let (bb_lower, _, bb_upper) = calculate_bb(&closes, 20);

    // 🔎 ประเมินสถานะตลาด
    let rsi_status = if rsi <= 35.0 { "OVERSOLD" } else if rsi >= 70.0 { "OVERBOUGHT" } else { "NEUTRAL" };
    let macd_status = if macd > macd_signal { "BULLISH (UP)" } else { "BEARISH (DOWN)" };
    let bb_status = if current_price < bb_lower { "BELOW LOWER BAND (DIP)" }
                    else if current_price > bb_upper { "ABOVE UPPER BAND (HIGH)" }
                    else { "INSIDE BAND" };

    // จำลอง Time Machine สั้นๆ (อันเดิมของเตง)
    let mut recently_oversold = false;
    for i in (closes.len().saturating_sub(5))..closes.len() {
        let past_closes = &closes[0..=i];
        if calculate_rsi(past_closes, 14) < 30.0 {
            recently_oversold = true;
            break;
        }
    }

    // 💎 สร้างประโยควิเคราะห์แบบสุดยอดส่งให้สมอง AI!
    let info_string = format!("RSI: {:.2} ({}) | MACD: {} | BB: {}", rsi, rsi_status, macd_status, bb_status);

    HttpResponse::Ok().json(json!({
        "symbol": symbol,
        "rsi": format!("{:.2}", rsi),
        "status": rsi_status,
        "recently_oversold": recently_oversold,
        "info_string": info_string // 🟢 ตัวแปรใหม่ ส่งข้อมูล Indicator เทพๆ ออกไป!
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🔮 [ANALYZER START] God-Tier Quant Analyzer (Rust) รันที่พอร์ต 3497 แล้ว!");
    HttpServer::new(|| {
        App::new().route("/analyze/{symbol}", web::get().to(analyze))
    })
    .bind(("127.0.0.1", 3497))?
    .run()
    .await
}
