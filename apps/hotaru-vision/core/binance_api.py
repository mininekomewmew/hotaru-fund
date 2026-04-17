import httpx
from fastapi import HTTPException

class BinanceClient:
    """คลาสจัดการ API ของ Binance"""
    
    def __init__(self):
        self.base_url = "https://api.binance.com/api/v3/klines"

    async def fetch_close_prices(self, symbol: str, interval: str, limit: int) -> list[float]:
        params = {"symbol": symbol.upper(), "interval": interval, "limit": limit}
        
        async with httpx.AsyncClient() as client:
            response = await client.get(self.base_url, params=params)
            
            if response.status_code != 200:
                raise HTTPException(status_code=400, detail=f"Binance Error: {response.text}")
            
            data = response.json()
            # ดึงเฉพาะราคาปิด (ตำแหน่งที่ 4)
            return [float(candle[4]) for candle in data]
