import os
import uvicorn
from fastapi import FastAPI
from core.models import MarketData, OracleData
from core.logger import BrainLogger
from core.service_clients import ExternalServices
from core.brain_engine import HotaruBrain

app = FastAPI(title="🧠 Hotaru Brain - Unified AI Engine")

# --- Configuration ---
# 🦊 โฮตารุหา Path ของโปรเจกต์ให้แบบอัตโนมัติค่ะ
BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA_DIR = os.path.join(BASE_DIR, "data")
LOG_FILE = os.path.join(DATA_DIR, "ai_reasoning.log")

# 🦊 รับ API Keys หลายตัว (เช่น key1,key2,key3)
CEREBRAS_API_KEYS = os.environ.get("CEREBRAS_API_KEY", "").split(",")

# --- Initialize Classes ---
logger = BrainLogger(LOG_FILE)
services = ExternalServices(
    analyzer_url="http://127.0.0.1:3497/analyze",
    vision_url="http://127.0.0.1:8005/analyze"
)
brain = HotaruBrain(CEREBRAS_API_KEYS)

global_market_status = {"fng": "50 (Neutral)", "binance_market": {}}

@app.get("/")
async def root():
    return {"status": "Hotaru Brain is Online!", "keys_active": len(brain.clients)}

@app.post("/update_sentiment")
async def update_sentiment(data: OracleData):
    global_market_status["fng"] = data.fng
    global_market_status["binance_market"] = data.binance_market
    print(f"📊 [BRAIN] Sentiment Updated: F&G {data.fng}")
    return {"status": "updated"}

@app.post("/analyze")
async def analyze_market(data: MarketData):
    try:
        is_holding = data.entry_price > 0
        pnl_pct = ((data.price - data.entry_price) / data.entry_price * 100) if is_holding else 0

        # 1. Get Data from Sub-services
        rsi_text = services.get_analyzer_data(data.symbol)
        vision_text = services.get_vision_data(data.symbol)
        combined_info = f"{rsi_text} | {vision_text}"

        # 2. Ask the Brain
        decision = brain.decide(
            data.symbol, data.price, pnl_pct, 
            global_market_status["fng"], combined_info, is_holding
        )

        # 3. Log the reasoning (บังคับบันทึกเสมอ!)
        logger.log_decision(data.symbol, data.price, pnl_pct, decision, combined_info)
        print(f"🧠 [BRAIN] Decision for {data.symbol}: {decision['action']}")

        return decision
    except Exception as e:
        error_msg = {"action": "HOLD", "message": f"Critical Brain Error: {str(e)}"}
        logger.log_decision(data.symbol, data.price, 0, error_msg, "ERROR")
        return error_msg

if __name__ == "__main__":
    import uvicorn
    # 🦊 บังคับรันที่พอร์ต 8000 เสมอค่ะ
    uvicorn.run(app, host="0.0.0.0", port=8000)
