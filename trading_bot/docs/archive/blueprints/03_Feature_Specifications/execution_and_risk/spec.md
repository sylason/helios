# Execution & Risk Management Specification

## 1. Trách nhiệm (Responsibilities)
- Quản lý API Key (Testnet/Mainnet).
- Xử lý 4 giai đoạn Order: Demo Local (Paper Trade) -> Demo Testnet -> Live.
- Quản trị Rủi ro Tốc độ cao bằng Trailing Stoploss.

## 2. Cắt lỗ Động (Trailing Stoploss) làm Chốt chặn chính
Trái tim của quản trị rủi ro ở dự án này không phải là Stoploss tĩnh, mà là **Trailing Stoploss**.
- Thay vì đợi nến 1 phút đóng lại để Strategy xử lý, một Task độc lập (Observer) sẽ subscribe luồng `markPrice` của Binance.
- Nếu giá chạm đường Trailing Stoploss (được cập nhật động theo biến động giá), Observer lập tức gọi lệnh Market Close.
- Việc này giúp tối ưu hóa lợi nhuận và hạn chế tối đa rủi ro trước những cú râu nến (Wick) đâm xuyên chớp nhoáng.

## 3. Lỗi Mạng & Mất Kết Nối (Disconnect Policy)
- Nếu API K-line bị rớt, bot giữ nguyên vị thế và thực hiện Exponential Backoff Reconnect.
- **Panic Close (Đóng khẩn cấp)**: KHÔNG đóng vị thế ngay khi vừa rớt mạng. Bot dựa vào Trailing Stoploss. CHỈ khi mất kết nối với `markPrice` (không theo dõi được Trailing SL) quá **5 phút**, hệ thống mới Trigger Panic Close (đóng toàn bộ vị thế) để an toàn.

## 4. Paper Trading Local
Để phục vụ Phase 4 của Demo:
- Execution module có cờ (Flag) `is_paper_trade = true`.
- Khi Flag này bật, thay vì dùng REST request đẩy lệnh, Module chỉ tính giá Khớp lệnh ảo dựa vào `markPrice` ngay thời điểm đó (có cộng thêm một mức độ Slippage - trượt giá ảo định trước).
- Ghi nhận Transaction vào Logs / Console `println!` để review.
