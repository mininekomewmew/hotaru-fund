import json
import requests
import time
import os
from datetime import datetime

# 🦊 โฮตารุช่วยหา Path ของโปรเจกต์แบบ Dynamic ให้ค่ะ ย้ายเครื่องไปไหนก็รันได้!
BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
STATE_FILE = os.path.join(BASE_DIR, "data/paper_state.json")
LOG_FILE = os.path.join(BASE_DIR, "data/trade_log.txt")
INITIAL_CAPITAL = 100.0  

# 🎨 โค้ดสี
C_GREEN = '\033[92m'
C_RED = '\033[91m'
C_YELLOW = '\033[93m'
C_CYAN = '\033[96m'
C_MAGENTA = '\033[95m'
C_BLUE = '\033[94m'
C_WHITE = '\033[97m'
C_BOLD = '\033[1m'
C_RESET = '\033[0m'

def get_price(symbol):
    try:
        url = f"https://api.kucoin.com/api/v1/market/orderbook/level1?symbol={symbol}"
        res = requests.get(url, timeout=3).json()
        return float(res['data']['price'])
    except:
        return 0.0

def format_uptime(start, now):
    duration = now - start
    days = duration.days
    hours, rem = divmod(duration.seconds, 3600)
    mins, _ = divmod(rem, 60)
    # 🟢 ตัดวินาทีทิ้ง โชว์แค่ Days, hr, minutes
    return f"{days} Days {hours:02d} hr {mins:02d} minutes"

def get_start_time_from_log():
    try:
        with open(LOG_FILE, 'r', encoding='utf-8') as f:
            for line in f:
                if "[" in line and "]" in line:
                    time_str = line.split("]")[0].strip("[")
                    return datetime.strptime(time_str, "%Y-%m-%d %H:%M:%S")
    except Exception:
        pass
    return datetime.now()

def main():
    start_time = get_start_time_from_log()

    while True:
        try:
            with open(STATE_FILE, 'r') as f:
                state = json.load(f)
        except:
            print(f"{C_YELLOW}⏳ กำลังรอไฟล์ paper_state.json...{C_RESET}")
            time.sleep(5)
            continue

        usdt_balance = state.get("usdt_balance", 0.0)
        holdings = state.get("holdings", {})
        entry_prices = state.get("entry_prices", {})

        total_holdings_value = 0.0
        total_invested_cost = 0.0
        coins_data = []

        for sym, qty in holdings.items():
            current_price = get_price(sym)
            if current_price == 0.0: continue
            
            entry = entry_prices.get(sym, 0.0)
            cost = qty * entry
            value = qty * current_price
            
            total_invested_cost += cost
            total_holdings_value += value
            
            pnl_pct = ((current_price - entry) / entry) * 100 if entry > 0 else 0.0
            pnl_usdt = value - cost
            
            coins_data.append({
                "sym": sym, "entry": entry, "current": current_price,
                "pnl_pct": pnl_pct, "pnl_usdt": pnl_usdt, "value": value, "cost": cost
            })

        current_portfolio_value = usdt_balance + total_holdings_value
        net_profit_usdt = current_portfolio_value - INITIAL_CAPITAL
        net_profit_pct = (net_profit_usdt / INITIAL_CAPITAL) * 100

        coins_data.sort(key=lambda x: x['pnl_pct'], reverse=True)

        os.system('cls' if os.name == 'nt' else 'clear')
        now = datetime.now()
        now_str = now.strftime("%Y-%m-%d %H:%M:%S")
        uptime_str = format_uptime(start_time, now)
        start_str = start_time.strftime("%Y-%m-%d %H:%M:%S")

        # 🌟 เปิดโล่งด้านขวา ป้องกันอีโมจิดันกรอบเบี้ยว
        print(f"{C_MAGENTA}{C_BOLD}╔═══════════════════════════════════════════════════════════════{C_RESET}")
        print(f"{C_MAGENTA}║{C_RESET} {C_CYAN}✨🦊 HOTARU QUANT MONITOR (LIVE) 🦊✨{C_RESET}")
        print(f"{C_MAGENTA}╠═══════════════════════════════════════════════════════════════{C_RESET}")
        print(f"{C_MAGENTA}║{C_RESET} {C_YELLOW}🗓️  Started Since : {start_str}{C_RESET}")
        print(f"{C_MAGENTA}║{C_RESET} {C_YELLOW}⏳  Total Uptime  : {uptime_str}{C_RESET}")
        print(f"{C_MAGENTA}╚═══════════════════════════════════════════════════════════════{C_RESET}")
        
        color = C_GREEN if net_profit_usdt >= 0 else C_RED
        sign = "+" if net_profit_usdt >= 0 else ""
        
        print(f"\n{C_BLUE}📊 PORTFOLIO SUMMARY{C_RESET}")
        print("-" * 65)
        print(f" 🏦 {C_WHITE}Initial Capital{C_RESET} : 100.00 USDT")
        print(f" 💎 {C_WHITE}Current Value{C_RESET}   : {color}{C_BOLD}{current_portfolio_value:.2f} USDT{C_RESET}")
        print(f" 🚀 {C_WHITE}NET PROFIT{C_RESET}      : {color}{C_BOLD}{sign}{net_profit_usdt:.2f} USDT ({sign}{net_profit_pct:.2f}%){C_RESET}")
        print("-" * 65)
        
        print(f" 💰 {C_WHITE}Cash Balance{C_RESET}    : {C_CYAN}{usdt_balance:.2f} USDT{C_RESET}")
        print(f" 📦 {C_WHITE}Active Trades{C_RESET}   : {C_CYAN}{len(holdings)} positions{C_RESET}")
        print("=" * 65)
        
        # 🌟 ฮาร์ดโค้ดระยะห่างตาราง (บังคับให้ช่องไฟเท่ากัน 100%)
        print(f"{C_MAGENTA}Symbol       | PNL (%)    | Status Bar      | Value (USDT){C_RESET}")
        print("-" * 65)

        for c in coins_data:
            c_color = C_GREEN if c['pnl_pct'] >= 0 else C_RED
            c_sign = "+" if c['pnl_pct'] >= 0 else ""
            
            # ปั้น String แบบล็อคจำนวนตัวอักษร
            sym_str = c['sym'].ljust(12)
            pnl_str = f"{c_sign}{c['pnl_pct']:.2f}%".ljust(10)
            val_str = f"{c['value']:.2f} $".ljust(14)
            
            # ปั้นกราฟแท่ง (15 ตัวอักษรเป๊ะๆ)
            filled = min(int((abs(c['pnl_pct']) / 5.0) * 15), 15)
            empty = 15 - filled
            bar_text = ('█' * filled) + ('░' * empty)
            
            # ประกอบร่าง (เส้น | จะอยู่ตรงกันแน่นอน)
            print(f"{C_WHITE}{sym_str}{C_RESET} | {c_color}{pnl_str}{C_RESET} | {c_color}{bar_text}{C_RESET} | {C_YELLOW}{val_str}{C_RESET}")

        print("=" * 65)
        print(f" 💳 {C_WHITE}Total Invested Cost{C_RESET}  : {total_invested_cost:.2f} USDT")
        print(f" 📈 {C_WHITE}Current Assets Value{C_RESET} : {total_holdings_value:.2f} USDT")
        print("-" * 65)
        print(f"{C_CYAN}✨ Last Updated: {now_str} (Auto-refresh 10s) ✨{C_RESET}")
        
        time.sleep(10)

if __name__ == "__main__":
    main()
