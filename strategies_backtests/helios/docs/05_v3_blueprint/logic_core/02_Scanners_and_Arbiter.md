# Khối Logic Quyết định: Scanners & Arbiter (V3)

Tài liệu này giải thích cách hoạt động của hệ thống Quét tín hiệu đa nhiệm (Scanners) và cách Trọng tài (Arbiter) dùng toán học để phân xử xung đột tín hiệu.

## 1. Tactical Scanners (Các Đội Đặc Nhiệm Tìm Kiếm)

Thay vì bị "đóng băng" bởi Regime như V2, các Scanner trong V3 hoạt động hoàn toàn độc lập, chạy liên tục trên từng nến. Mỗi Scanner đại diện cho một "Trường phái Giao dịch" khác nhau:

### 1.1. Trend Pullback Scanner (Trường phái Thuận Xu Hướng)
- **Cơ chế:** Tìm kiếm các cú hồi (Pullback) trong một xu hướng lớn.
- **Logic kích hoạt:** Nếu giá thoái lui về KAMA Mid hoặc đường Baseline, đồng thời Stochastics rơi vào trạng thái Oversold, Scanner này sẽ tính toán một `Raw Pattern Score` cao.
- **Mục tiêu:** Buy the dip / Sell the rally. 

### 1.2. Momentum Ignition Scanner (Trường phái Bùng nổ)
- **Cơ chế:** Tìm kiếm sự phá vỡ cấu trúc nén (Squeeze Breakout) kèm Volume lớn.
- **Logic kích hoạt:** Bollinger Bands bắt đầu mở rộng vọt ra khỏi Keltner Channels, Volume Force > Ngưỡng bùng nổ, MACD xác nhận cắt lên (Histogram dốc lên).
- **Mục tiêu:** Bắt nhịp sóng 3 (sóng đẩy mạnh nhất) của Elliott.

### 1.3. Range Oscillator Scanner (Trường phái Đánh Biên/Tích Lũy)
- **Cơ chế:** Đánh Ping-Pong giữa 2 biên khi thị trường không có xu hướng.
- **Logic kích hoạt:** Giá trôi dạt về biên trên hoặc biên dưới của Bollinger Bands (cách biên 10-25%), kết hợp sự đảo chiều của RSI/Stochastics.
- **Mục tiêu:** Tích nhặt lợi nhuận nhỏ trong giai đoạn ru ngủ.

### 1.4. Climax / Absorption Scanner (Trường phái Bắt Đỉnh/Đáy)
- **Cơ chế:** Tìm kiếm sự cạn kiệt đà giá hoặc sự can thiệp của Cá Mập.
- **Logic kích hoạt:** Giá ở trạng thái Parabolic (Z-Score cực cao), nhưng Volume đột ngột sụt giảm (Exhaustion) hoặc giá không tăng thêm dù Volume cực đại (Absorption - chặn mua/chặn bán).
- **Mục tiêu:** Vào lệnh ngược chiều ngay tại điểm kiệt sức của phe đối lập.

---

## 2. The Arbiter (Trọng Tài Toán Học)

Do cả 4 Scanner chạy song song, rất có thể ở một cây nến, Range Scanner báo MUA và Momentum Scanner lại báo BÁN. Arbiter sẽ phân xử điều này thông qua **Hệ số Thế trận (Regime Multiplier)**.

### 2.1. Tính Hệ số Thế trận (Regime Multiplier)
Từ Market Matrix, Arbiter tính ra 4 hệ số đại diện cho tính chất môi trường hiện tại (Thang 0.0 - 1.0):
- `Trend_Multiplier`: Tỉ lệ thuận với Trend Strength.
- `Squeeze_Multiplier`: Tỉ lệ thuận với Squeeze Intensity.
- `Range_Multiplier`: Phần bù của Trend và Squeeze.
- `Anomaly_Multiplier`: Tỉ lệ thuận với Z-Score và Chaos Ratio.

### 2.2. Tính Điểm Niềm Tin (Confidence Score)
Khi nhận `Proposal` từ các Scanner, Arbiter nhân `Raw Pattern Score` của chúng với Hệ số tương ứng.
- Ví dụ 1: `Trend_PB_Scanner` nộp Proposal MUA với RawScore = 0.9. Thị trường đang Sideway (`Trend_Multiplier` = 0.2). -> `Confidence = 0.9 * 0.2 = 0.18`.
- Ví dụ 2: `Range_Scanner` nộp Proposal BÁN với RawScore = 0.85. Thị trường Sideway (`Range_Multiplier` = 0.95). -> `Confidence = 0.85 * 0.95 = 0.807`.

### 2.3. Quy tắc Duyệt Lệnh (The Verdict)
1. Arbiter lọc bỏ tất cả các Proposal có `Confidence < Min_Approval_Threshold` (Ví dụ 0.6).
2. Trong ví dụ trên, lệnh của `Trend_PB` (0.18) bị loại, lệnh của `Range` (0.807) được giữ lại.
3. Nếu có nhiều lệnh vượt ngưỡng, Arbiter sẽ duyệt lệnh có `Confidence` cao nhất để chuyển giao cho **Risk Manager**.

**Kết quả:** Hệ thống vẫn linh hoạt bắt được mọi tín hiệu mà không làm mất đi lớp khiên bảo vệ khỏi môi trường độc hại. Lệnh xấu tự động bị triệt tiêu bởi Toán học.
