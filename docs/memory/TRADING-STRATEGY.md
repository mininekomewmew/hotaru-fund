# 🛡️ Hotaru Fund - Trading Strategy & Hard Rules
**Version:** 1.0 (Opus-Inspired)
**Mission:** เอาชนะตลาดด้วยวินัยและระบบอัตโนมัติ (Autonomous Scalping)

## 🚫 Hard Rules (กฎเหล็กห้ามละเมิด)
1. **No Leverage:** เทรด Spot เท่านั้น ห้ามยุ่งกับ Futures หรือ Token ที่มี Leverage (3L, 3S)
2. **Position Limit:** ถือครองเหรียญได้สูงสุด **5-6 ตำแหน่ง** พร้อมกันเท่านั้น
3. **Position Sizing:** ลงทุนสูงสุดไม่เกิน **20% ของเงินต้น** ต่อ 1 ออเดอร์
4. **Hard Stop-Loss:** ตัดขาดทุนทันทีที่ **-7%** จากราคาเข้า (No Hoping, No Averaging Down)
5. **Trailing Stop:** ทันทีที่เปิดออเดอร์ ต้องมี Trailing Stop เริ่มต้นที่ **10%**

## 📈 Buy-Side Gate (เงื่อนไขก่อนเข้าซื้อ)
- ยอดรวม Position หลังซื้อต้องไม่เกิน 6
- เหรียญต้องมี Volume 24h > 100,000 USDT (ผ่านการสแกนจาก Rust Hunter)
- ต้องมีการระบุ "Catalyst" หรือเหตุผลที่เข้าซื้อลงใน Research Log ก่อนเสมอ
- Risk/Reward Ratio ต้องมีอย่างน้อย **2:1**

## 📉 Sell-Side Rules (เงื่อนไขการขาย)

## 📉 Sell-Side Rules (ฉบับงกกำไร - Greedy Scalper)
- **Early Take Profit:** เริ่มพิจารณาขายทันทีเมื่อกำไร > **1.0%**
- **Momentum Drop:** หากกำไร > 0.5% และเห็นราคาเริ่ม "หักหัวลง" หรือ RSI เริ่มตัดลง ให้ขายล็อคกำไรทันที (Lock-in Gains)
- **Break Even:** เมื่อกำไรถึง **1.0%** ให้ขยับ Stop Loss มาที่จุดคุ้มทุน (0%) ทันที
- **Hard Stop-Loss:** ปรับลดความเสี่ยงเหลือสูงสุด **-3.0%** (เพื่อไม่ให้เสียเงินต้นเยอะเกินไป)
- **Binary Pulse Target:** ในโหมดสั้น เน้นจบงานที่ **0.5% - 0.8%** ภายใน 5-10 นาที

## ⚡ New Mode: Binary Pulse (HFT-Inspired)
**Status:** Experimental (Integrating Polymarket Engine Logic)
**Target:** 5-15 Minute Scalping

### 🔍 Strategy Logic
- **Order Book Imbalance:** ติดตามแรงซื้อ (Bids) และแรงขาย (Asks) ในระดับ Level 2 หากฝั่งใดฝั่งหนึ่งหนากว่าอย่างเห็นได้ชัด (> 1.5x) ให้ถือเป็นสัญญาณชีพจร (Pulse)
- **Short-Term Targets:** TP: 0.5% - 1.0% | SL: 0.3% - 0.5%
- **Execution:** เน้นเหรียญที่มี "Hot Pulse" จาก Scanner และมีส่วนต่างราคา (Gap) ที่น่าสนใจ

### 🔄 Dual-Mode Integration
- บอทจะสลับเป็น **Binary Pulse** เมื่อตลาดมีความผันผวนสูง (High Volatility) และวอลุ่มเข้าหนาแน่น
- บอทจะคงโหมด **Opus-Swing** ไว้สำหรับเหรียญที่มีเทรนด์ขาขึ้นชัดเจนและมั่นคง
---

