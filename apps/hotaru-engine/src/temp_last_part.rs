                    let pnl_pct = ((current_price - entry_price) / entry_price) * 100.0;
                    log_trade(&format!("💰 [AI SELL] {} @ {:.4} | PNL: {:.2}% | Reason: {}", symbol, current_price, pnl_pct, decision.message));

                    if pnl_pct < -1.0 {
                        cut_loss_happened = true;
                    }
                }
            }
        }

        save_state(&state);
        println!("⏳ เช็ครอบนี้เสร็จแล้ว พักผ่อน 60 วินาที...");
        sleep(Duration::from_secs(60)).await; 
    }
}
