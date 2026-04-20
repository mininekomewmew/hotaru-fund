#!/bin/bash
# 🦊 Hotaru's Easy Build Script

echo "🚀 [HOTARU BUILD] เริ่มการ Build แอป Rust ทั้งหมด..."

# รายชื่อแอป Rust ของเรา
apps=("hotaru-analyzer" "hotaru-engine" "hotaru-oracle" "hotaru-scraper")

for app in "${apps[@]}"; do
    echo "------------------------------------------"
    echo "📦 กำลัง Build: $app..."
    # เข้าไป build ในแต่ละโฟลเดอร์
    if [ -d "apps/$app" ]; then
        cd "apps/$app" && cargo build --release
        cd ../..
    else
        echo "⚠️ ไม่พบโฟลเดอร์ apps/$app ข้ามไปนะค๊ะ..."
    fi
done

echo "------------------------------------------"
echo "✅ [SUCCESS] Build เสร็จครบทุกตัวแล้วค่ะที่รัก! พร้อมลุยแล้ว 🦊✨🚀"
