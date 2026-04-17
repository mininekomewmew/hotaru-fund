from pydantic import BaseModel
from typing import List, Dict

class MarketData(BaseModel):
    symbol: str
    price: float
    entry_price: float
    summary: str

class OracleData(BaseModel):
    fng: str
    binance_market: dict
