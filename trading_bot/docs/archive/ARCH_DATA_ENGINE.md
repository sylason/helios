# 🏗️ Architecture: Data Engine (High-Performance Aggregator)

Thiết kế này đảm bảo dữ liệu luôn đầy đủ (15-30 days window) và phản ứng cực nhanh (Tick-level).

---

## 1. Luồng dữ liệu (Data Pipeline)
1. **WebSocket Feed:**
    - **`aggTrade`:** Luồng giao dịch thô từng mili giây.
    - **`markPrice`:** Luồng giá tham chiếu để chốt lời/lỗ và tính Margin.
2. **Local Aggregator (Rust):** 
    - Tổng hợp Tick thành nến 1m. 
    - **Zero-delay:** Chốt giá nến ngay tại mili giây thứ 0 của phút mới, nhanh hơn nến sàn gửi về 1-2 giây.
3. **Tick Buffer (RAM):**
    - Giữ dữ liệu Tick trong **2 phút** gần nhất.
    - Mục đích: Xử lý râu nến (wicks) cho Trailing SL và phân tích biến động siêu ngắn hạn.
    - Cơ chế: Xóa tự động các Tick lỗi thời định kỳ.
4. **Candle Window (RAM & DuckDB):**
    - Giữ **15-30 ngày** nến 1m (khoảng 21,600 - 43,200 dòng).
    - Polars DataFrame giữ dữ liệu trong RAM để Strategy truy cập tức thì.
    - DuckDB làm lớp lưu trữ bền vững để phục hồi khi mở lại App.

## 2. Cơ chế Khởi động & Vá dữ liệu (Stitching)
1. **WS Connect:** Kết nối WebSocket ngay khi mở App để không bỏ lỡ dữ liệu mới.
2. **Gap Analysis:** Kiểm tra DuckDB để xác định khoảng trống (gaps) dữ liệu từ lần cuối đóng App.
3. **REST Backfill:** Gọi API sàn để tải dữ liệu thiếu cho đủ cửa sổ 30 ngày.
4. **Memory Injection:** Đổ toàn bộ dữ liệu vào Polars RAM Buffer và sẵn sàng cho chu kỳ Optimize đầu tiên.

## 3. Tối ưu hóa Lưu trữ
- **Hourly Cleanup:** Một luồng chạy ngầm cứ mỗi 1 giờ sẽ thực hiện xóa dữ liệu nến cũ hơn 30 ngày trong DuckDB.
- **Buffer VecDeque:** Trong Rust, sử dụng `VecDeque` để quản lý sliding window hiệu quả nhất về mặt bộ nhớ.

## 4. Trade Journaling (Lịch sử giao dịch)
Để phục vụ thống kê và quản lý thuế, hệ thống duy trì nhật ký giao dịch:
- **Bản Demo (MVP):** Xuất dữ liệu ra file `.csv` đơn giản.
- **Bản Production:** Lưu vào bảng `trade_log` trong DuckDB.
- **Dữ liệu lưu trữ:** `Symbol, EntryTime, ExitTime, EntryPrice, ExitPrice, Fee, RealizedPnL, Leverage, Reason (SL/TP/Panic)`.
