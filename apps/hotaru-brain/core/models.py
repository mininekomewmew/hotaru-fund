from pydantic import BaseModel
from typing import List, Dict

class MarketData(BaseModel):
    symbol: str
    price: float
    entry_price: float
    summary: str
    current_positions: int = 0  # 💎 เพิ่มด่านตรวจจำนวนเหรียญค่ะ

class OracleData(BaseModel):
    fng: str
    binance_market: dict

class NewsData(BaseModel):
    news_title: str
    impact_keyword: str
    timestamp: str
