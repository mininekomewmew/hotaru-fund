import os
import re
import json
import time
from cerebras.cloud.sdk import Cerebras
from core.learning_engine import LearningEngine # 🚀 นำเข้าขุมปัญญาใหม่!

class HotaruBrain:
    def __init__(self, api_keys: list):
        self.clients = [Cerebras(api_key=key.strip()) for key in api_keys if key.strip()]
        self.current_index = 0
        self.exhausted_keys = set()
        self.learner = LearningEngine() # 🦊 ครูฝึกส่วนตัวของโฮตารุค่ะ

    def _get_next_client(self):
        if not self.clients: return None
        attempts = 0
        while attempts < len(self.clients):
            client = self.clients[self.current_index]
            if self.current_index not in self.exhausted_keys:
                self.current_index = (self.current_index + 1) % len(self.clients)
                return client
            self.current_index = (self.current_index + 1) % len(self.clients)
            attempts += 1
        self.exhausted_keys.clear()
        return self.clients[0]

    def calculate_size(self, balance, volatility, trend_strength, regime="SIDEWAY"):
        base_size = balance * 0.35 
        multiplier = 1.0
        if regime == "BULL": multiplier *= 1.2
        elif regime == "BEAR": multiplier *= 0.8
        return min(base_size * multiplier, balance * 0.5)

    def decide(self, symbol, price, pnl, fng, combined_info, is_holding, balance=100.0, current_positions=0, latest_news="N/A", retry_count=0):
        client = self._get_next_client()
        if not client: return {"action": "HOLD", "message": "No Keys"}

        real_pnl = pnl - 0.15 
        analyzer = combined_info.get("analyzer", {})
        vision = combined_info.get("vision", {})
        
        # 🦊 [SELF-EVOLUTION] ดึงบทเรียนในอดีตมาสอน AI ค่ะ!
        past_lessons = self.learner.get_past_experiences(limit=3)
        lessons_str = "\n".join([f"- {l}" for r, l in enumerate(past_lessons)]) if past_lessons else "No past data yet."

        ob_imbalance = analyzer.get("ob_imbalance", 1.0)
        macro_trend = vision.get("macro_trend", "UNKNOWN")
        
        mode = "OPUS_SWING"
        if ob_imbalance > 1.8 or ob_imbalance < 0.5: mode = "BINARY_PULSE"
        regime = "BULL" if "BULLISH" in macro_trend else "BEAR" if "BEARISH" in macro_trend else "SIDEWAY"

        context_rules = self._get_rules(is_holding, mode)

        system_prompt = f"""You are Hotaru Quant AI (Learning Mode).
Mode: {mode} | Regime: {regime} | Positions: {current_positions}/6
Symbol: {symbol} | Price: {price} | Real PNL: {real_pnl:.2f}% | F&G: {fng}

[PAST LESSONS & EXPERIENCE]
{lessons_str}

[DATA]
- Technical: {analyzer.get('info_string', 'N/A')}
- OB Imbalance: {ob_imbalance:.2f}
- Vision Signal: {vision.get('ai_signal', 'WAIT')} | News: {latest_news}

[STRATEGY RULES]
{context_rules}

Return ONLY JSON: {{"action": "BUY/SELL/HOLD", "message": "reason", "confidence": 0.0-1.0}}"""

        try:
            completion = client.chat.completions.create(
                messages=[{"role": "system", "content": system_prompt}, {"role": "user", "content": f"Price: {price}"}],
                model="llama3.1-8b", temperature=0.1
            )
            match = re.search(r'\{.*\}', completion.choices[0].message.content.strip(), re.DOTALL)
            decision = json.loads(match.group(0)) if match else {"action": "HOLD", "message": "Parse Error"}
            
            # 📝 [LEARNING] ถ้ามีการขาย ให้บันทึกบทเรียนทันทีค่ะ (เดี๋ยวเราไปเรียก Record ใน main.py นะค๊ะ)
            decision["size"] = self.calculate_size(balance, 0.02, decision.get("confidence", 0.5), regime=regime) if decision.get("action") == "BUY" else 0
            return decision

        except Exception as e:
            if ("Rate Limit" in str(e) or "429" in str(e)) and retry_count < len(self.clients):
                self.exhausted_keys.add((self.current_index - 1) % len(self.clients))
                return self.decide(symbol, price, pnl, fng, combined_info, is_holding, balance, current_positions, latest_news, retry_count + 1)
            return {"action": "HOLD", "message": f"AI Error: {str(e)}"}

    def _get_rules(self, is_holding, mode="OPUS_SWING"):
        if mode == "BINARY_PULSE":
            if is_holding: return "[BINARY] TP: 0.5-1.0%. Sell if OB reverses. SL: -0.5%."
            return "[BINARY] Buy if OB > 1.8. Quick 5-15 min scalp."
        if is_holding: return "[GREEDY] TP: 2.0%. BE: 1.0%. Sell if weakness. SL: -1.5% (Engine enforced)."
        return "[GREEDY] BUY if BULLISH and RSI 35-55. Aim for Small Steady Wins."
