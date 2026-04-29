import requests

class ExternalServices:
    def __init__(self, analyzer_url: str, vision_url: str):
        self.analyzer_url = analyzer_url
        self.vision_url = vision_url

    def get_analyzer_data(self, symbol: str):
        """
        🦊 ดึงข้อมูลจาก Rust Analyzer (Port 3497)
        อัปเกรด: รับค่า Order Book Imbalance สำหรับโหมด Binary Pulse ค่ะ
        """
        try:
            response = requests.get(f"{self.analyzer_url}/{symbol}", timeout=3)
            if response.status_code == 200:
                data = response.json()
                return {
                    "rsi": float(data.get("rsi", 50.0)),
                    "should_ask_ai": data.get("should_ask_ai", True),
                    "info_string": data.get("info_string", "No Data"),
                    "ob_imbalance": float(data.get("orderbook_imbalance", 1.0)),
                    "ob_status": data.get("orderbook_status", "NEUTRAL")
                }
            return {"rsi": 50.0, "should_ask_ai": True, "info_string": "Analyzer Error", "ob_imbalance": 1.0}
        except Exception as e:
            return {"rsi": 50.0, "should_ask_ai": True, "info_string": f"Offline: {str(e)}", "ob_imbalance": 1.0}

    def get_vision_data(self, symbol: str):
        """
        🦊 ดึงข้อมูลจาก Vision (Port 8005)
        """
        try:
            clean_symbol = symbol.replace("-", "")
            response = requests.get(f"{self.vision_url}/{clean_symbol}", timeout=5)
            if response.status_code == 200:
                v = response.json()
                macro = v.get("macro_1d", {})
                micro = v.get("micro_1h", {})
                return {
                    "macro_trend": macro.get("trend", "UNKNOWN"),
                    "gap_pct": v.get("gap_pct", 0.0),
                    "ai_signal": v.get("ai_signal", "WAIT")
                }
            return {"ai_signal": "ERROR", "macro_trend": "UNKNOWN"}
        except:
            return {"ai_signal": "OFFLINE", "macro_trend": "UNKNOWN"}
