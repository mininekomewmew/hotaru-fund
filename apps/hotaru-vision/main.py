import asyncio
from fastapi import FastAPI

# ดึง Class ที่เราเขียนแยกไว้มาใช้งาน
from core.binance_api import BinanceClient
from core.technical_calc import TechnicalAnalyzer

app = FastAPI(title="👁️ Hotaru Vision - MTF Data Center")

# สร้างตัวแทน (Instance) ของ Class ขึ้นมาใช้งาน
binance = BinanceClient()
calc = TechnicalAnalyzer()

@app.get("/analyze/{symbol}")
async def analyze_mtf(symbol: str):
    # 1. ให้สายลับไปดูดข้อมูลมา 2 เลนส์พร้อมกัน
    macro_prices, micro_prices = await asyncio.gather(
        binance.fetch_close_prices(symbol, "1d", 90),
        binance.fetch_close_prices(symbol, "1h", 168)
    )

    current_price = micro_prices[-1]

    # 2. 🦅 วิเคราะห์ภาพใหญ่ (Macro)
    macro_rsi = calc.calculate_rsi(macro_prices, 14)
    macro_sma50 = calc.calculate_sma(macro_prices, 50)
    macro_trend = "BULLISH 🐂" if current_price > macro_sma50 else "BEARISH 🐻"

    # 3. 🔬 วิเคราะห์ภาพเล็ก (Micro)
    micro_rsi = calc.calculate_rsi(micro_prices, 14)
    micro_sma20 = calc.calculate_sma(micro_prices, 20)
    micro_trend = "UPTREND 📈" if current_price > micro_sma20 else "DOWNTREND 📉"
    
    micro_status = "NEUTRAL"
    if micro_rsi < 30: micro_status = "OVERSOLD (DIP) 🟢"
    elif micro_rsi > 70: micro_status = "OVERBOUGHT (HIGH) 🔴"

    # 4. 🧠 หาจุดตัดความได้เปรียบ
    setup_quality = "WAIT"
    if macro_trend == "BULLISH 🐂" and micro_status == "OVERSOLD (DIP) 🟢":
        setup_quality = "🔥 PERFECT BUY (เทรนด์ใหญ่ขึ้น + กราฟเล็กย่อตัว)"
    elif macro_trend == "BEARISH 🐻" and micro_status == "OVERBOUGHT (HIGH) 🔴":
        setup_quality = "❄️ PERFECT SHORT / SELL (เทรนด์ใหญ่ลง + กราฟเล็กเด้ง)"

    return {
        "symbol": symbol.upper(),
        "current_price": current_price,
        "macro_1d": {"trend": macro_trend, "rsi": macro_rsi},
        "micro_1h": {"trend": micro_trend, "rsi": micro_rsi, "status": micro_status},
        "ai_signal": setup_quality
    }

# รันด้วยคำสั่ง: uvicorn main:app --host 127.0.0.1 --port 8005
