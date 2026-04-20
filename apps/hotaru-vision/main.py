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
    # 1. ดึงข้อมูลจากทั้ง 2 ตลาดพร้อมกัน (KuCoin ใช้ Format -USDT, Binance ใช้ USDT)
    binance_symbol = symbol.replace("-", "")
    
    # ดึงราคาปัจจุบันจากทั้งสองที่เพื่อหา Gap
    macro_prices, micro_prices, kucoin_price_data = await asyncio.gather(
        binance.fetch_close_prices(binance_symbol, "1d", 90),
        binance.fetch_close_prices(binance_symbol, "1h", 168),
        binance.fetch_kucoin_price(symbol) # เดี๋ยวโฮตารุไปเพิ่มฟังก์ชันนี้ให้ใน binance_api.py นะค๊ะ
    )

    binance_price = micro_prices[-1]
    kucoin_price = kucoin_price_data
    
    # คำนวณ Gap % (ถ้าน้อยกว่า 0 แปลว่า KuCoin ถูกกว่า)
    gap_pct = ((binance_price - kucoin_price) / binance_price) * 100 if kucoin_price > 0 else 0

    # 2. 🦅 วิเคราะห์เทรนด์เหมือนเดิม
    macro_rsi = calc.calculate_rsi(macro_prices, 14)
    macro_sma50 = calc.calculate_sma(macro_prices, 50)
    macro_trend = "BULLISH 🐂" if binance_price > macro_sma50 else "BEARISH 🐻"

    # 3. 🔬 วิเคราะห์ภาพเล็กและ Volatility
    micro_rsi = calc.calculate_rsi(micro_prices, 14)
    
    # 4. 🧠 สรุปสัญญาณคอมโบ
    setup_quality = "WAIT"
    if gap_pct > 0.3: # KuCoin ถูกกว่า Binance 0.3% ขึ้นไป
        setup_quality = f"💎 ARBITRAGE OPPORTUNITY (Gap: {gap_pct:.2f}%)"
    elif macro_trend == "BULLISH 🐂" and micro_rsi < 40:
        setup_quality = "⚡ SCALP BUY (Dip in Bullish Trend)"

    return {
        "symbol": symbol.upper(),
        "binance_price": binance_price,
        "kucoin_price": kucoin_price,
        "gap_pct": gap_pct,
        "macro_1d": {"trend": macro_trend, "rsi": macro_rsi},
        "micro_1h": {"rsi": micro_rsi},
        "ai_signal": setup_quality
    }

# รันด้วยคำสั่ง: uvicorn main:app --host 127.0.0.1 --port 8005
