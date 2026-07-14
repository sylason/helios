import duckdb
import pandas as pd

def check_missing_candles(db_path: str, symbol: str):
    conn = duckdb.connect(db_path)
    
    # Lấy toàn bộ nến của symbol, sắp xếp theo thời gian
    query = f"""
        SELECT timestamp 
        FROM candles_1m 
        WHERE symbol = '{symbol}' 
        ORDER BY timestamp ASC
    """
    df = conn.execute(query).df()
    
    if df.empty:
        print(f"Không có dữ liệu cho {symbol}")
        return
        
    print(f"Tổng số nến trong DB: {len(df)}")
    print(f"Nến đầu tiên: {pd.to_datetime(df['timestamp'].iloc[0], unit='ms')}")
    print(f"Nến cuối cùng: {pd.to_datetime(df['timestamp'].iloc[-1], unit='ms')}")
    
    # Tính toán khoảng cách (diff) giữa các nến liên tiếp
    # Nến 1m = 60,000 ms
    df['diff'] = df['timestamp'].diff()
    
    # Tìm các khoảng trống (gap) > 60000ms
    gaps = df[df['diff'] > 60000].copy()
    
    if gaps.empty:
        print("✅ TUYỆT VỜI! Database liên tục 100%, không bị thiếu (gap) bất kỳ cây nến 1m nào.")
    else:
        print(f"❌ PHÁT HIỆN {len(gaps)} KHOẢNG TRỐNG (GAP):")
        for idx, row in gaps.iterrows():
            prev_time = pd.to_datetime(df.loc[idx - 1, 'timestamp'], unit='ms')
            curr_time = pd.to_datetime(row['timestamp'], unit='ms')
            missing_minutes = int((row['diff'] / 60000) - 1)
            print(f"  - Từ {prev_time} đến {curr_time}: Thiếu {missing_minutes} nến")

if __name__ == "__main__":
    check_missing_candles("storage/trading_bot.db", "ETHUSDT")
