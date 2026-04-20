import os
import re
import json
from cerebras.cloud.sdk import Cerebras

class HotaruBrain:
    def __init__(self, api_keys: list):
        # 🦊 เก็บกุญแจทุกดอกไว้ในคลัง
        self.clients = [Cerebras(api_key=key.strip()) for key in api_keys if key.strip()]
        self.current_index = 0

    def _get_next_client(self):
        if not self.clients:
            return None
        # 🔄 สลับกุญแจไปเรื่อยๆ แบบ Round-Robin
        client = self.clients[self.current_index]
        self.current_index = (self.current_index + 1) % len(self.clients)
        return client

    def decide(self, symbol, price, pnl, fng, combined_info, is_holding):
        client = self._get_next_client()
        if not client:
            return {"action": "HOLD", "message": "No Cerebras API Keys found"}

        mode_text = "POS" if is_holding else "SCAN"
        context_rules = self._get_rules(is_holding)

        system_prompt = f"""You are Hotaru Quant AI. {symbol} | Mode: {mode_text} | PNL: {pnl:.2f}% | F&G: {fng}
Indicators: {combined_info}

Rules:
{context_rules}

Return ONLY JSON: {{"action": "BUY/SELL/HOLD", "message": "reason"}}"""

        try:
            completion = client.chat.completions.create(
                messages=[{"role": "system", "content": system_prompt},
                          {"role": "user", "content": f"Price: {price}"}],
                model="llama3.1-8b", 
                temperature=0.1
            )
            match = re.search(r'\{.*\}', completion.choices[0].message.content.strip(), re.DOTALL)
            return json.loads(match.group(0)) if match else {"action": "HOLD", "message": "Parse Error"}
        except Exception as e:
            # ⚠️ ถ้ากุญแจนี้มีปัญหา (เช่น ติด Rate Limit) ให้ลองใช้กุญแจถัดไปทันที (Recursive Call ครั้งเดียว)
            if "Rate Limit" in str(e) or "429" in str(e):
                return self.decide(symbol, price, pnl, fng, combined_info, is_holding)
            return {"action": "HOLD", "message": f"AI Error: {str(e)}"}

    def _get_rules(self, is_holding):
        if is_holding:
            return """
            [POSITION MANAGEMENT]
            - SELL: PNL > 1.5% OR (PNL > 0.5% and RSI > 70).
            - STOP: PNL < -2% (Hard Exit).
            - HOLD: If Trend Bullish.
            """
        return """
            [HYBRID SCANNER]
            - BUY: Gap > 0.3% (Arbitrage - KuCoin cheaper).
            - BUY: Macro BULLISH 🐂 AND Micro RSI < 40 (Scalp).
            - HOLD: Otherwise.
            """
