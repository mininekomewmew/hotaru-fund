import sqlite3
import os
import re

class LearningEngine:
    def __init__(self, db_path="data/historical_data.db"):
        self.db_path = db_path

    def record_lesson(self, symbol, pnl, reasoning, scenario):
        """
        🦊 บันทึกบทเรียนลงในความจำระยะยาว
        """
        try:
            conn = sqlite3.connect(self.db_path)
            cur = conn.cursor()
            
            # สรุปบทเรียนเบื้องต้น
            lesson = ""
            if pnl > 1.5:
                lesson = f"SUCCESS: High conviction setup for {symbol}. Profit Locked effectively."
            elif pnl < -1.0:
                lesson = f"FAILURE: Tighten exit for {symbol}. Avoid entering when trend is similar to this."
            
            cur.execute(
                "INSERT INTO hotaru_memory (symbol, pnl, reasoning, scenario_tag, lessons_learned) VALUES (?, ?, ?, ?, ?)",
                (symbol, pnl, reasoning, scenario, lesson)
            )
            conn.commit()
            conn.close()
            print(f"📝 [LEARNING] จดจำบทเรียนเหรียญ {symbol} (PNL: {pnl:.2f}%) เรียบร้อยค่ะ!")
        except Exception as e:
            print(f"❌ [LEARNING ERROR] {str(e)}")

    def get_past_experiences(self, limit=3):
        """
        🦊 ดึงบทเรียนที่เด่นๆ (กำไรเยอะสุด หรือ ขาดทุนเยอะสุด) มาส่งให้ AI ดูค่ะ
        """
        try:
            conn = sqlite3.connect(self.db_path)
            cur = conn.cursor()
            cur.execute("SELECT lessons_learned FROM hotaru_memory ORDER BY ABS(pnl) DESC LIMIT ?", (limit,))
            experiences = [r[0] for r in cur.fetchall()]
            conn.close()
            return experiences
        except:
            return []
