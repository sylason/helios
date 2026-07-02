# 2c. Lỗ hổng Đa khung thời gian (MTF Confluence Gap)

> [!NOTE]
> Tài liệu thiết kế của Helios có nhắc đến "Trọng số theo Khung thời gian", nhưng phân tích mã nguồn (`tactical.rs`) cho thấy hệ thống đang bị **mù đa khung**.

## Phân tích Mã nguồn Hiện tại
Trong `strategy/mod.rs` và `tactical.rs`, tất cả các tín hiệu từ ATR, RSI, MACD, KAMA đều được lấy từ vector tại vị trí `i` hiện tại.
Mặc dù bạn có cấu hình Period và Timeframe cho Indicator, nhưng tại bước ra quyết định cuối cùng, Helios chỉ đang đưa ra kết luận dựa trên trạng thái duy nhất của khung thời gian nó đang chạy (ví dụ 1 phút).

Điều này vi phạm nguyên tắc cơ bản nhất của Trading có hệ thống: **"Trade ở khung nhỏ, dựa vào bối cảnh khung lớn"**.

## Hiệu ứng "Tín hiệu giả" (Noise)
Khung M1 liên tục tạo ra các thế trận `Squeeze` và `Parabolic` giả.
Nếu M1 báo hiệu Long Breakout (Giá cắt lên KAMA), nhưng trên H1, giá lại đang nằm ngay dưới đường kháng cự siêu mạnh (Downtrend), lệnh Long M1 sẽ thất bại thảm hại.

## Cần thay đổi kiến trúc thế nào?

Helios cần lưu trữ ít nhất 3 Context độc lập:
1. `HTF_Context` (Higher Time Frame - Vĩ mô): Xác định xu hướng chủ đạo (Ví dụ: Bearish).
2. `ITF_Context` (Intermediate Time Frame - Trung hạn): Xác định vùng canh lệnh (Ví dụ: Giá đang kéo về vùng quá mua).
3. `LTF_Context` (Lower Time Frame - Vi mô): Trigger bóp cò (Ví dụ: Chờ MACD gãy xuống).

**Mã giả Ràng buộc Playbook:**
```rust
// Trong playbook.rs
if htf_context.trend == Bearish && ltf_context.regime == Squeeze {
    // CHỈ cho phép Squeeze Breakout chiều SHORT
    // Khóa hoàn toàn chiều LONG
}
```
Việc đồng bộ hóa dữ liệu từ các timeframe khác nhau vào cùng một tick `i` đòi hỏi một cơ chế Data Alignment chính xác để tránh bẫy "Lookahead bias" (Nhìn trộm tương lai).
