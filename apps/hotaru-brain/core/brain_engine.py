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
            return """
            [POSITION MANAGEMENT MODE]
            - SELL: If PNL > 5% and Micro RSI > 75 (Take Profit), or if Macro Trend turns BEARISH.
            - STOP LOSS: If PNL < -3%, evaluate if the support level is broken.
            - HOLD: If Trend is still Bullish and RSI is not overextended. Let profits run!
            """
        return """
            [MARKET SCANNING MODE]
            - BUY: If Macro(1D) is BULLISH 🐂 AND (Micro RSI < 45 OR Setup is 'PERFECT BUY').
            - BUY: If Vision signal says 'PERFECT BUY', ignore minor RSI fluctuations and ENTRY.
            - AGGRESSION: At 0.00% PNL, don't be afraid. If the macro trend is up, every dip is an opportunity.
            - HOLD: Only if both Macro and Micro trends are Bearish or unclear.
            """
