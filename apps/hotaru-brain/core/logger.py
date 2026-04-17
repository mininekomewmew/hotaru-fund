import os
from datetime import datetime

class BrainLogger:
    def __init__(self, log_path: str):
        self.log_path = log_path

    def log_decision(self, symbol, price, pnl, decision, indicators):
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        log_entry = (
            f"[{now}] SYMBOL: {symbol} @ {price} | PNL: {pnl:.2f}%\n"
            f"🌡️ INDICATORS: {indicators}\n"
            f"🧠 AI DECISION: {decision['action']} | REASON: {decision['message']}\n"
            f"{'-' * 80}\n"
        )
        try:
            with open(self.log_path, "a", encoding="utf-8") as f:
                f.write(log_entry)
        except Exception as e:
            print(f"❌ Logger Error: {e}")
