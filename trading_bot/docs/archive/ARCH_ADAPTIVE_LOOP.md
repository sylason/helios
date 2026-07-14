# 🔄 Architecture: Adaptive Loop & Asset Actor

Thiết kế mô tả cách các "tiểu bot" tự động tối ưu hóa và thích nghi với thị trường bằng Rust Actors.

---

## 1. Trạng thái của Asset Actor (FSM States)
Mỗi Asset chạy trên một luồng độc lập với các trạng thái:
1. **InitialSync:** Đang tải/gộp dữ liệu 30 ngày.
2. **Optimizing:** Đang chạy Genetic Algorithm để tìm Params.
3. **Trading:** Đang áp dụng Params tối ưu để trade real-time.
4. **Observing (Safety Mode):** Params không đạt tiêu chuẩn (Profit Factor < 1.1). Bot chỉ theo dõi dữ liệu, không vào lệnh.
5. **Stopped:** Dừng do lỗi hệ thống hoặc lệnh Panic.

## 2. Cơ chế Tự tối ưu hóa (Self-Optimization)
Tự động kích hoạt Re-optimization mà không cần hỏi người dùng khi:
- **Market Alerts:** Helios phát hiện Breakout, Flash Crash, hoặc Thao túng.
- **Performance Leak:** Thua lỗ thực tế vượt quá sai số cho phép của Backtest.
- **Extreme Volatility:** Biến động thị trường vượt ngưỡng lịch sử của Window 30 ngày.
- **Scheduled Refresh:** Mặc định mỗi 4-8 tiếng để làm mới bộ tham số.

## 3. Quy trình Hot-swap & Swap Logic
Khi tìm thấy bộ Params mới từ `ferro_trade`:
- **Swap Logic (Always Better):** Chỉ cần bộ Params mới có kết quả tốt hơn (dù chỉ 0.1% Winrate hay Profit Factor), Bot sẽ thực hiện lệnh `Hot-swap` ngay lập tức.
- **Active Position Protection:** Việc hoán đổi (Swap) diễn ra kể cả khi đang có vị thế mở để cập nhật ngay ngưỡng Trailing SL/TP mới tối ưu hơn.
- **Fail-Safe:** Nếu Optimize không tìm thấy bộ Params nào đạt yêu cầu tối thiểu, Bot sẽ giữ nguyên bộ Params cũ và tiếp tục vận hành.

## 4. Điều phối tài nguyên (Optimization Orchestrator)
Để tránh chiếm dụng hết CPU của máy tính cá nhân:
- **Thread Isolation:** Sử dụng tham số `OPT_NUM_THREADS` từ cấu hình để giới hạn số luồng CPU mà mỗi tác vụ Optimize được phép sử dụng.
- **Priority Queue (Risk-based):** Hàng đợi Optimize được sắp xếp theo độ ưu tiên:
    1. **High:** Các Asset đang có vị thế mở (`Active Position`) hoặc đang gặp sụt giảm vốn (`Drawdown`).
    2. **Medium:** Các Asset có thanh khoản cao (Liquidity).
    3. **Low:** Thứ tự yêu cầu (FIFO).
- **Concurrency Limit:** Sử dụng `Semaphore` để giới hạn số lượng Asset được Optimize đồng thời (ví dụ: tối đa 2 cặp tiền cùng lúc).

## 5. Quy trình khởi động song song (Parallel Startup)
Để rút ngắn thời gian chờ đợi khi mở App:
1. **WebSocket Connect:** Kết nối ngay lập tức để nhận giá real-time (nhưng chưa vào lệnh).
2. **Background Optimize:** Chạy chu kỳ Optimize đầu tiên ở luồng nền bằng dữ liệu 30 ngày vừa tải về.
3. **Live Activation:** Ngay khi có kết quả Optimize đầu tiên, Bot tự động kích hoạt trạng thái `Trading` và bắt đầu nhận tín hiệu vào lệnh từ `helios`.
