# 3c. Kỹ thuật Nhồi lệnh (Pyramiding The Winners)

Theo quy tắc gốc (`HELIOS_PROTOCOL.md`), Helios đang tuân thủ **Nguyên tắc "Độc tôn" (No Pyramiding)** - Tuyệt đối không nhồi lệnh.

## Tại sao Cấm Nhồi Lệnh là an toàn nhưng lãng phí?
Cấm nhồi lệnh giúp Helios không bị FOMO ngược (nhồi thêm khi đang lỗ - Martingale).
Tuy nhiên, trong đòn bẩy cao, lợi nhuận khổng lồ không đến từ việc ăn nhiều lệnh nhỏ, mà đến từ việc **nhồi lệnh khi đang chiến thắng (Scale-in Winners)** để bắt trọn những con sóng Parabolic lịch sử mà không tăng thêm rủi ro gốc.

## Kỹ thuật "Kim tự tháp không rủi ro" (Risk-Free Pyramiding)

Helios nên hỗ trợ một module Pyramiding đặc biệt, hoạt động theo quy tắc sau:

1. **Khởi tạo:** Mở Vị thế 1 (V1) với rủi ro 2% vốn.
2. **Phá vỡ cản:** Giá chạy đúng hướng và đạt ngưỡng Breakeven (Dời SL của V1 về điểm Entry). Lúc này V1 = Risk Free (Rủi ro = 0).
3. **Tín hiệu tiếp diễn:** Nếu xuất hiện tín hiệu TrendPullback thuận chiều, mở tiếp Vị thế 2 (V2) với rủi ro 2% vốn.
4. **Đồng bộ Dừng lỗ:** Dời Dừng lỗ của V1 lên **BẰNG** Dừng lỗ của V2.

**Kết quả kỳ diệu:** 
Lúc này bạn đang ôm khối lượng gấp đôi, lợi nhuận tiềm năng x2.
Nhưng nếu giá quay đầu cắn SL?
- V2 sẽ lỗ 2%.
- V1 sẽ **Chốt lãi** đúng bằng số tiền V2 lỗ.
- Tổng rủi ro vẫn luôn chỉ là **2% vốn**!

Đó là cách các Institutional Traders đánh quỹ. Họ dùng tiền lãi của lệnh trước để làm Margin bảo vệ lệnh sau. Đây là module cực kỳ giá trị để đưa vào hệ thống Helios V3.
