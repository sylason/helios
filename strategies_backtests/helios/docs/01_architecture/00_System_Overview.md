# 00. Tổng quan Hệ thống (System Overview)

Tài liệu này phản ánh **chính xác 100%** kiến trúc hiện tại đang được lập trình trong mã nguồn của Helios (thư mục `src/strategy`).

## 1. Dòng chảy Dữ liệu (Data Pipeline)
Helios xử lý dữ liệu qua 4 cấp độ tuần tự tại mỗi cây nến (Tick):

1. **Sensors (Cảm biến):** Lấy dữ liệu thô (OHLCV) từ `ferro_trade` và tính toán ra 18+ mảng giá trị của các chỉ báo kỹ thuật (Indicator).
2. **Matrix (Ma trận Nhận thức):** Chuyển đổi các giá trị Indicator thô thành điểm số chuẩn hóa `[0, 1]` hoặc `[-1, 1]` bằng thuật toán `Rolling Percentile` và `Z-Score`.
3. **Regime & Playbook (Đánh giá Trận địa & Lên kế hoạch):** Dựa vào Matrix để chấm điểm 6 trạng thái thị trường. Trạng thái nào điểm cao nhất sẽ kích hoạt Đội Đặc nhiệm (Playbook) tương ứng.
4. **Tactical Sniper (Thực thi):** Đội Đặc nhiệm soi lại các tín hiệu vi mô (MACD cắt, giá chạm Band) để quyết định `Long`, `Short` hay `Wait`.

## 2. Giải phẫu `HeliosStrategy` (trong `mod.rs`)

Struct chính của chiến lược bao gồm 4 phần:
- `config`: Bộ tham số khổng lồ (~100 genes) được Optimizer tiêm vào qua DNA.
- `memory`: Bộ nhớ lưu lại các trạng thái trước đó (ví dụ: đang ở trong Regime nào, Playbook nào mở lệnh).
- `sensors`: Nơi lưu trữ toàn bộ chuỗi dữ liệu Indicator.
- `matrix`: Nơi lưu trữ 13 điểm số nhận thức.

> [!WARNING] BUG NGHIÊM TRỌNG TRONG CODE HIỆN TẠI
> Tại file `src/strategy/mod.rs`, toàn bộ nội dung của hàm `next()` (từ dòng 283 đến 410) **đang bị comment out**. 
> Điều này có nghĩa là chiến lược Helios hiện tại **KHÔNG HỀ CHẠY** bất kỳ logic nào trong lúc backtest. Việc Optimizer trả ra kết quả có thể là do nó đang chạy một phiên bản cũ hoặc file khác.

## 3. Kiến trúc Đa khung thời gian (MTF) Ẩn
Trong mã nguồn, logic ra quyết định chỉ gọi data tại một vị trí `[i]` duy nhất (khung 1 phút). Tuy nhiên, tính năng Đa khung thời gian đã được thiết kế ngầm bằng cách:
- Cấu hình cho mỗi Indicator một Timeframe riêng rẽ trong file JSON.
- `ferro_trade` tự động tính toán Indicator ở khung lớn, sau đó `forward fill` (điền dồn) dữ liệu xuống khung 1 phút để tránh sai lệch tick (lookahead bias).
- Nhờ vậy, dù Code chiến lược chỉ đọc ở index `[i]`, bản chất nó đang đọc giá trị của khung H1, H4.
