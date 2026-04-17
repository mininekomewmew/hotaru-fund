import requests

class ExternalServices:
    def __init__(self, analyzer_url: str, vision_url: str):
        self.analyzer_url = analyzer_url
        self.vision_url = vision_url

    def get_analyzer_data(self, symbol: str):
        try:
            response = requests.get(f"{self.analyzer_url}/{symbol}", timeout=3)
            data = response.json()
            return f"Micro RSI: {data.get('rsi', '50.0')} ({data.get('status', 'NEUTRAL')})"
        except:
            return "Micro RSI: Unknown"

    def get_vision_data(self, symbol: str):
        try:
            clean_symbol = symbol.replace("-", "")
            response = requests.get(f"{self.vision_url}/{clean_symbol}", timeout=5)
            if response.status_code == 200:
                v = response.json()
                macro = v["macro_1d"]
                micro = v["micro_1h"]
                return (f"| 👁️ VISION -> Macro: {macro['trend']} (RSI {macro['rsi']}) "
                        f"| Micro: {micro['trend']} ({micro['status']}) "
                        f"| Signal: {v['ai_signal']}")
            return "| 👁️ Vision: Error"
        except:
            return "| 👁️ Vision: Offline"
