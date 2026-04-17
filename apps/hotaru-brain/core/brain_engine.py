import os
import re
import json
from cerebras.cloud.sdk import Cerebras

class HotaruBrain:
    def __init__(self, api_key: str):
        self.client = Cerebras(api_key=api_key) if api_key else None

    def decide(self, symbol, price, pnl, fng, combined_info, is_holding):
        if not self.client:
            return {"action": "HOLD", "message": "No Cerebras API Key"}

        mode_text = "Position Management" if is_holding else "New Entry Scanning"
        context_rules = self._get_rules(is_holding)

        system_prompt = f"""You are Hotaru, an Elite Quant Trader AI.
Currently processing: {symbol}. Mode: {mode_text}. PNL: {pnl:.2f}%. F&G: {fng}.
Indicators: {combined_info}.

{context_rules}

Return ONLY valid JSON: {{"action": "BUY/SELL/HOLD", "message": "reason"}}"""

        try:
            completion = self.client.chat.completions.create(
                messages=[{"role": "system", "content": system_prompt},
                          {"role": "user", "content": f"Symbol: {symbol}, Price: {price}"}],
                model="llama3.1-8b", 
                temperature=0.1
            )
            match = re.search(r'\{.*\}', completion.choices[0].message.content.strip(), re.DOTALL)
            return json.loads(match.group(0)) if match else {"action": "HOLD", "message": "Parse Error"}
        except Exception as e:
            return {"action": "HOLD", "message": f"AI Error: {str(e)}"}

    def _get_rules(self, is_holding):
        if is_holding:
            return "HOLDING MODE: SELL if RSI > 70 or Vision is Bearish. HOLD to let winners run."
        return "ENTRY MODE: BUY only if Vision Macro is Bullish and RSI is Oversold. Otherwise HOLD."
