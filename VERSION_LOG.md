# 🦊 Hotaru Fund - Version Log

## [v1.1.0] - 2026-04-21 (01:55 UTC)
### 🚀 สรุปการปรับปรุงระบบ (System Fixes & Improvements)

#### 1. 🧠 Hotaru Brain (AI Core)
- **Fix:** แก้ไขปัญหา **"Vision Offline"** ใน `service_clients.py` โดยการปรับปรุง Schema ให้ตรงกับที่ `hotaru-vision` ส่งมา (Key: `micro_1h` มีเพียง `rsi`) 
- **Improvement:** เพิ่มระบบ Error Handling ป้องกันแอปค้าง (Crash) เมื่อข้อมูล indicator บางตัวหายไป
- **Debug:** เพิ่ม Log การรับสัญญาณ `/analyze` เพื่อให้ตรวจสอบได้ทันทีเมื่อมีการยิง Request เข้ามา

#### 2. ⚙️ Hotaru Engine (Rust Engine)
- **Optimization:** ปรับเกณฑ์ **SMART PRE-FILTER** (RSI) จากเดิม **45.0 เป็น 52.0** เพื่อให้บอทเริ่มทำงานได้ในสภาพตลาดปัจจุบัน (RSI พื้นฐานสูงขึ้น)
- **Fix:** แก้ไขปัญหาการยิง Request ไปหา Brain ที่พอร์ต 8000 ให้แม่นยำขึ้น
- **Logging:** เพิ่มระบบแจ้งเตือนสถานะการสื่อสารกับ AI (📡 [ENGINE] -> 🧠 [BRAIN]) ให้แสดงผลในหน้าจอทันที

#### 3. 🏹 Hotaru Scraper (Data Miner)
- **Expansion:** เพิ่มรายชื่อเหรียญขุดข้อมูลกราฟ (PAIRS) จาก 8 ตัว เป็น 16 ตัว ครอบคลุมเหรียญมาแรงล่าสุด:
  - `ORDI`, `AAVE`, `NEAR`, `SUI`, `LINK`, `AVAX`, `ADA`, `FET`
- **Goal:** ป้องกันปัญหา RSI 50.00 (Default) สำหรับเหรียญที่ไม่มีข้อมูลใน Database

#### 4. 👁️ Hotaru Vision (MTF Eye)
- **Stability:** ยืนยันการเชื่อมต่อกับ Binance API และระบบ Arbitrage Signal ยังทำงานปกติ 100%

---
**สถานะปัจจุบัน:** ทุกระบบออนไลน์ (Online) และกำลังทำการสแกนตลาดเพื่อหาจุดเข้าซื้อที่ RSI ย่อตัวลงมาตามกลยุทธ์ "Scalp Buy in Bullish Trend" ค่ะ! 🛡️✨🚀
