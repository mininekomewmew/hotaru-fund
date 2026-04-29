import os
import json
import uvicorn
from fastapi import FastAPI
from core.models import MarketData, OracleData, NewsData
from core.logger import BrainLogger
from core.service_clients import ExternalServices
from core.brain_engine import HotaruBrain

app = FastAPI(title="🧠 Hotaru Brain - Unified AI Engine")

# --- Configuration ---
BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA_DIR = os.path.join(BASE_DIR, "data")
LOG_FILE = os.path.join(DATA_DIR, "ai_reasoning.log")

CEREBRAS_API_KEYS = os.environ.get("CEREBRAS_API_KEY", "").split(",")

# --- Initialize Classes ---
logger = BrainLogger(LOG_FILE)
services = ExternalServices(
    analyzer_url="http://127.0.0.1:3497/analyze",
    vision_url="http://127.0.0.1:8005/analyze"
)
brain = HotaruBrain(CEREBRAS_API_KEYS)

# --- Global Market Status ---
global_market_status = {
    "fng": "50 (Neutral)", 
    "binance_market": {},
    "latest_news": "No major news reported yet."
}

@app.get("/")
async def root():
    return {"status": "Hotaru Brain is Online!", "keys_active": len(brain.clients)}

@app.post("/update_sentiment")
async def update_sentiment(data: OracleData):
    global_market_status["fng"] = data.fng
    global_market_status["binance_market"] = data.binance_market
    print(f"📊 [BRAIN] Sentiment Updated: F&G {data.fng}")
    return {"status": "updated"}

@app.post("/news_event")
async def receive_news(data: NewsData):
    """
    🦊 ช่องทางรับข่าวด่วนจาก News-Sentry (Rust)
    และบันทึกไว้ในความทรงจำหลักเพื่อใช้ร่วมกับการวิเคราะห์กราฟค่ะ ✨🗞️
    """
    global_market_status["latest_news"] = f"[{data.impact_keyword.upper()}] {data.news_title}"
    print(f"📰 [BRAIN] New Impactful News Received: {global_market_status['latest_news']}")
    return {"status": "news_received"}

@app.post("/analyze")
async def analyze_market(data: MarketData):
    print(f"📥 [BRAIN] Received analyze request for {data.symbol} @ {data.price}")
    try:
        state_path = os.path.join(DATA_DIR, "paper_state.json")
        current_balance = 100.0
        if os.path.exists(state_path):
            with open(state_path, "r") as f:
                state = json.load(f)
                current_balance = state.get("usdt_balance", 100.0)

        is_holding = data.entry_price > 0
        pnl_pct = ((data.price - data.entry_price) / data.entry_price * 100) if is_holding else 0

        # 1. Get Data from Sub-services (Technical & Order Book & Vision)
        analyzer_data = services.get_analyzer_data(data.symbol)
        vision_data = services.get_vision_data(data.symbol)

        combined_info_data = {
            "analyzer": analyzer_data,
            "vision": vision_data,
            "volatility": 0.02
        }

        # 2. Ask the Brain (ส่ง Balance + ตำแหน่งปัจจุบัน + ข่าวด่วน ไปด้วย!)
        decision = brain.decide(
            data.symbol, data.price, pnl_pct, 
            global_market_status["fng"], combined_info_data, is_holding,
            balance=current_balance,
            current_positions=data.current_positions,
            latest_news=global_market_status["latest_news"] # 🗞️ หัวใจของการเทรดด้วยข่าวค่ะ!
        )

        # 4. Log the reasoning
        log_info = f"{analyzer_data.get('info_string', '')} | Vision: {vision_data.get('ai_signal', '')}"
        logger.log_decision(data.symbol, data.price, pnl_pct, decision, log_info)
        print(f"🧠 [BRAIN] Decision for {data.symbol}: {decision['action']} | Size: {decision.get('size', 0):.2f} USDT")

        return decision

    except Exception as e:
        error_msg = {"action": "HOLD", "message": f"Critical Brain Error: {str(e)}"}
        logger.log_decision(data.symbol, data.price, 0, error_msg, "ERROR")
        return error_msg

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
