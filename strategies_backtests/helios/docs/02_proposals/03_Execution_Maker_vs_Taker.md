# Đề xuất 03: Maker Limit hay Taker Market?

Trả lời cho phân vân của bạn ở Q3 về việc: Cần tốc độ cao (Market) nhưng sợ phí/trượt giá.

## Phân tích Đánh đổi (Trade-off)

- **Market Order (Taker):** Khớp lệnh ngay lập tức. Đảm bảo 100% bắt được cú Squeeze Breakout hoặc Nuke. **Nhược điểm:** Phí sàn cao (0.05%), Trượt giá (Slippage) có thể làm giảm lợi nhuận.
- **Limit Order (Maker):** Không mất phí, điểm vào đẹp. **Nhược điểm:** Khi thị trường biến động mạnh, giá bỏ chạy quá nhanh, lệnh Limit sẽ không bao giờ được khớp (Miss the boat).

## Đề xuất cho Day Trading Đòn bẩy Cao

Vì bạn đang giao dịch trong môi trường tốc độ cao và rủi ro lớn, việc **Vào trúng thời điểm** quan trọng hơn việc tiết kiệm một chút phí giao dịch ban đầu.

**Giải pháp đề nghị:**
1. **Lối Vào (Entry): DÙNG MARKET ORDER (Taker).**
   - Đội Squeeze, Momentum, và Nuke bắt buộc phải bóp cò bằng Market Order. Khởi đầu một con sóng lớn không có chỗ cho sự do dự đợi giá lùi về khớp Limit.
   - Bù đắp: Ta sẽ config Optimizer trừ đi (Penalty) một khoản Phí + Trượt giá cố định (ví dụ 0.1%) cho mỗi lệnh. Optimizer sẽ tự động học cách **chỉ chọn những lệnh có biên độ lợi nhuận đủ lớn để bù đắp phí**, bỏ qua các tín hiệu nhiễu nhỏ.

2. **Lối Ra (Exit / Take Profit): DÙNG LIMIT ORDER (Maker).**
   - Nếu bạn có ngưỡng Take Profit cố định (RR), hãy đặt nó bằng lệnh Limit. Sàn sẽ hoàn phí (Rebate) hoặc miễn phí cho bạn.

3. **Cắt lỗ (Stop Loss): DÙNG STOP-MARKET (Taker).**
   - Chạy trốn khỏi một chuyến tàu đắm thì không được tính toán phí. Chạm SL là phải cắt bằng Market ngay lập tức.
