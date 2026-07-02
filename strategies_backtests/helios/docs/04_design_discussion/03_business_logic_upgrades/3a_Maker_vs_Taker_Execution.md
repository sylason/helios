# 3a. Thực thi lệnh: Maker vs Taker

Để tối ưu lợi nhuận với đòn bẩy cao, sự khác biệt giữa phí Maker và Taker là ranh giới giữa Lợi nhuận dài hạn và Phá sản.

## Tình trạng hiện tại
Hệ thống gọi hàm `tick.buy()` và `tick.sell()` mặc định là Market Order (Taker).
Taker Fee trên các sàn Crypto (như Binance) thường ở mức `0.05%`.
Với đòn bẩy 100x, bạn trả `0.05% * 100 = 5%` vốn cho mỗi lần vào lệnh. Tổng cộng 10% vốn bay màu cho cả 2 chiều (Mở và Đóng), chưa tính Spread và Slippage. 

Một chiến lược winrate 50% sẽ cạn kiệt tài khoản cực kỳ nhanh.

## Đề xuất: Execution Gateway chuyên nghiệp

Chúng ta cần xây dựng một Module Thực thi (Execution Gateway) với các chiến thuật rải lệnh (Limit Orders):

### 1. Limit Chasing (Săn mồi rượt đuổi)
Khi có tín hiệu bóp cò, thay vì đánh Market, bot đặt lệnh Limit ngay tại giá `Bid` tốt nhất.
Nếu giá bỏ chạy, bot sẽ tự động hủy lệnh cũ và đẩy lệnh Limit mới lên theo (Chasing) trong một khoảng thời gian giới hạn (Ví dụ: 3 giây).

### 2. Taker for Stop Loss (Cắt lỗ dứt khoát)
- **Vào lệnh (Entry):** 100% dùng Limit Order (Maker) để nhận hoàn phí (hoặc phí cực thấp).
- **Chốt lời (Take Profit):** Dùng Limit Order.
- **Dừng lỗ (Stop Loss):** BẮT BUỘC dùng Market Order (Taker) để đảm bảo thoát khỏi thị trường ngay lập tức, tránh bị trượt SL dẫn đến thanh lý toàn phần.

### 3. Khái niệm "Tổ ong" (Layered Bidding)
Trong chiến thuật `TrendPullback` hoặc `AbsorptionScout`, không vào toàn bộ vị thế tại một mức giá. Chia khối lượng thành 3-5 lệnh Limit nhỏ rải dọc theo vùng KAMA hoặc ranh giới Dải Bollinger.
Điều này giúp bạn có mức giá trung bình cực tốt và tận dụng được các râu nến quét thanh khoản (Liquidity Wicks) của cá mập.
