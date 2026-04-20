module.exports = {
  apps : [
    {
      name: "hotaru-scraper",
      script: "./apps/hotaru-scraper/target/release/hotaru-scraper",
      restart_delay: 5000 
    },
    {
      name: "hotaru-analyzer",
      // 📍 โฮตารุแก้เป็นขีดล่าง ( _ ) แล้วค่ะ
      script: "./apps/hotaru-analyzer/target/release/hotaru_analyzer",
    },
    {
      name: "hotaru-oracle",
      // 📍 โฮตารุแก้ชื่อให้ตรงกับไฟล์เป๊ะๆ เป็น hotaru_oracle (ขีดล่าง)
      script: "./apps/hotaru-oracle/target/release/hotaru_oracle", 
      restart_delay: 5000 
    },
    {
      name: "hotaru-engine",
      script: "./apps/hotaru-engine/target/release/hotaru-engine",
    },
    {
      name: "hotaru-brain",
      script: "uvicorn",
      args: "main:app --host 0.0.0.0 --port 8000",
      cwd: "./apps/hotaru-brain", 
      interpreter: "python3",     
      restart_delay: 5000
    },
    // 🌟 เอาแอปใหม่มาต่อท้ายตรงนี้เลยค่า!
    {
      name: "hotaru-vision",
      script: "python3",
      args: "-m uvicorn main:app --host 127.0.0.1 --port 8005", // รันผ่าน uvicorn ที่พอร์ต 8005
      cwd: "./apps/hotaru-vision", 
      restart_delay: 5000
    }

  ]
};
