use feed_rs::parser;
use reqwest::Client;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use serde_json::json;

// 🪙 รายชื่อสำนักข่าวระดับโลกที่น้องจะไปเฝ้าค่ะ (อัปเกรดเพิ่มแหล่งข่าวชั้นนำ!)
const FEEDS: &[&str] = &[
    "https://feeds.reuters.com/reuters/topNews",
    "https://www.whitehouse.gov/feed/",
    "https://www.federalreserve.gov/feeds/press_all.xml",
    "https://cointelegraph.com/rss",
    "https://www.coindesk.com/arc/outboundfeeds/rss/", // 🚀 CoinDesk
    "https://www.theblock.co/rss.xml",                // 🚀 The Block
    "https://search.cnbc.com/rs/search/view.xml?partnerId=2000&keywords=crypto", // 🚀 CNBC Crypto
    "https://www.bloomberg.com/feeds/bview/register.rss", // 🚀 Bloomberg View
];

// 🔍 คำสำคัญที่ถ้าเจอแล้วต้อง "ตื่นตัว" ทันที!
const HOT_KEYWORDS: &[&str] = &[
    "fed", "rate", "inflation", "cpi", "sec", "etf", "approved", 
    "partnership", "hack", "exploit", "ceasefire", "strike"
];

#[tokio::main]
async fn main() {
    println!("📡 [NEWS-SENTRY START] หน่วยลาดตระเวนข่าวสารความเร็วแสง เริ่มปฏิบัติการ!");
    
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("HotaruNewsSentry/1.0")
        .build()
        .unwrap();

    let mut seen_ids = HashSet::new();

    loop {
        for url in FEEDS {
            if let Ok(response) = client.get(*url).send().await {
                if let Ok(content) = response.bytes().await {
                    if let Ok(feed) = parser::parse(&content[..]) {
                        for entry in feed.entries {
                            let id = entry.id.clone();
                            if !seen_ids.contains(&id) {
                                let title = entry.title.map(|t| t.content).unwrap_or_default();
                                let summary = entry.summary.map(|s| s.content).unwrap_or_default();
                                let full_text = format!("{} {}", title, summary).to_lowercase();

                                // 🔍 ตรวจสอบคำสำคัญ
                                for kw in HOT_KEYWORDS {
                                    if full_text.contains(kw) {
                                        println!("🔥 [HOT NEWS] เจอข่าวสำคัญ: {} (Keyword: {})", title, kw);
                                        
                                        // 🧠 ส่งสัญญาณไปให้ Brain ทันที!
                                        let _ = send_to_brain(&client, &title, kw).await;
                                        break;
                                    }
                                }
                                seen_ids.insert(id);
                            }
                        }
                    }
                }
            }
            // 🛑 พักหายใจสั้นๆ ระหว่างเปลี่ยนสำนักข่าว
            sleep(Duration::from_millis(500)).await;
        }

        // 🛡️ ป้องกันเห็นข่าวเก่าซ้ำซาก (ล้างความจำถ้าเยอะเกินไป)
        if seen_ids.len() > 1000 { seen_ids.clear(); }

        println!("⏳ ลาดตระเวนรอบนี้เสร็จสิ้น... พัก 10 วินาทีก่อนวนรอบใหม่ค่ะ ✨");
        sleep(Duration::from_secs(10)).await;
    }
}

async fn send_to_brain(client: &Client, title: &str, keyword: &str) -> Result<(), reqwest::Error> {
    let payload = json!({
        "news_title": title,
        "impact_keyword": keyword,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // 📡 ส่งไปที่สมอง (เดี๋ยวเราไปเพิ่ม Endpoint นี้ใน Python นะค๊ะที่รัก)
    let _ = client.post("http://127.0.0.1:8000/news_event")
        .json(&payload)
        .send()
        .await;

    Ok(())
}
