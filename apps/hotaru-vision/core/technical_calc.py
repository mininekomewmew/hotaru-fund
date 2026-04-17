from typing import List

class TechnicalAnalyzer:
    """คลาสจัดการ Indicator ทั้งหมด"""
    
    @staticmethod
    def calculate_rsi(prices: List[float], period: int = 14) -> float:
        if len(prices) < period + 1: return 50.0
        
        gains = [max(0, prices[i] - prices[i-1]) for i in range(1, len(prices))]
        losses = [max(0, prices[i-1] - prices[i]) for i in range(1, len(prices))]
        
        avg_gain = sum(gains[-period:]) / period
        avg_loss = sum(losses[-period:]) / period
        
        if avg_loss == 0: return 100.0
        rs = avg_gain / avg_loss
        return round(100 - (100 / (1 + rs)), 2)

    @staticmethod
    def calculate_sma(prices: List[float], period: int) -> float:
        if len(prices) < period:
            return prices[-1] if prices else 0.0
        return sum(prices[-period:]) / period
