import httpx
from fastapi import HTTPException

class BinanceClient:
    """คลาสจัดการ API ของ Binance"""
    
    def __init__(self):
        self.base_url = "https://api.binance.com/api/v3/klines"

    async def fetch_close_prices(self, symbol: str, interval: str, limit: int) -> list[float]:
        params = {"symbol": symbol.upper(), "interval": interval, "limit": limit}
        
        async with httpx.AsyncClient() as client:
            try:
                response = await client.get(self.base_url, params=params, timeout=5)
                
                if response.status_code != 200:
                    # 🦊 ถ้าหาใน Binance ไม่เจอ (เหรียญซิ่ง!) ให้ดึงราคาล่าสุดจาก KuCoin มาทำลิสต์ราคาแทนค่ะ
                    current_price = await self.fetch_kucoin_price(symbol.replace("USDT", "-USDT"))
                    if current_price > 0:
                        return [current_price] * limit
                    return [0.0] * limit
                
                data = response.json()
                return [float(candle[4]) for candle in data]
            except Exception:
                return [0.0] * limit

    async def fetch_kucoin_klines(self, symbol: str, interval: str, limit: int) -> list[float]:
        """ดึงกราฟจาก KuCoin โดยตรง (เพื่อความแม่นยำของเหรียญซิ่ง)"""
        # KuCoin interval mapping: 1d -> 1day, 1h -> 1hour
        kc_interval = "1day" if interval == "1d" else "1hour"
        url = f"https://api.kucoin.com/api/v1/market/candles?symbol={symbol.upper()}&type={kc_interval}&limit={limit}"
        async with httpx.AsyncClient() as client:
            try:
                res = await client.get(url, timeout=5)
                data = res.json()["data"]
                # KuCoin candles: [time, open, close, high, low, volume, turnover]
                # ราคาปิดอยู่ที่ index 2
                return [float(candle[2]) for candle in data]
            except Exception:
                return []

    async def fetch_kucoin_price(self, symbol: str) -> float:
        """ดึงราคาปัจจุบันจาก KuCoin เพื่อหา Gap"""
        url = f"https://api.kucoin.com/api/v1/market/orderbook/level1?symbol={symbol.upper()}"
        async with httpx.AsyncClient() as client:
            try:
                res = await client.get(url, timeout=5)
                data = res.json()
                return float(data['data']['price'])
            except Exception:
                return 0.0
