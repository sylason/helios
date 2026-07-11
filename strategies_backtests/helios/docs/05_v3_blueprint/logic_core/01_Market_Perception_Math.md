# Cốt lõi Nhận thức (Market Perception Math)

Tài liệu này giải thích lý do tại sao Helios chọn sử dụng các Indicators cụ thể, ý nghĩa toán học đằng sau chúng và cách chúng được "nhào nặn" thành các "Lực lượng" (Forces) đại diện cho nhận thức thị trường trong lớp **Market Matrix**.

## 1. Triết lý Thiết kế: Không dự đoán, Chỉ đo đạc
Thị trường tài chính là một hệ thống phi tuyến tính (non-linear). Thay vì dùng Indicator để dự đoán giá tương lai, Helios dùng chúng để đo đạc **Vận tốc (Velocity)**, **Gia tốc (Acceleration)**, **Động lượng (Momentum)** và **Kháng cự ma sát (Volatility/Squeeze)** ở thời điểm hiện tại. Bằng cách số hóa thị trường thành các dạng "vật lý", hệ thống có thể tạo ra các quyết định khách quan.

---

## 2. Các Đơn vị Đo lường (Indicators) và Nhiệm vụ

### 2.1. ADX (Average Directional Index) & Linear Regression
- **Mục đích:** Đo lường sức mạnh (Trend Strength) và góc nghiêng (Angle) của xu hướng.
- **Tại sao dùng ADX?** ADX độc lập với hướng di chuyển (giá tăng hay giảm thì ADX mạnh đều tăng). Nó cho biết phe bò hay phe gấu đang kiểm soát cuộc chơi một cách dứt khoát hay đang giằng co.
- **Tại sao dùng LinReg Angle?** ADX có độ trễ lớn. Đường Linear Regression (Hồi quy tuyến tính) tạo ra một góc nghiêng trực quan bằng độ (Degrees, từ -90 đến 90). Nếu góc quá dốc (ví dụ > 60 độ), thị trường đang rơi vào trạng thái Parabolic (chạy nước rút), báo hiệu nguy cơ sụp đổ nhanh, cần phải siết chặt Stoploss thay vì mua đuổi.

### 2.2. KAMA (Kaufman's Adaptive Moving Average)
- **Mục đích:** Nhận diện Cấu trúc nền tảng và Đo lường độ nhiễu.
- **Tại sao lại dùng KAMA thay vì SMA/EMA?** KAMA tự động điều chỉnh độ mượt (smoothing) dựa trên Độ nhiễu thị trường (Efficiency Ratio - ER). Khi giá chạy ngang (nhiễu cao), KAMA nằm im phăng phắc, không chạy theo giá. Khi giá có xu hướng mạnh, KAMA bám theo cực sát. Nhờ KAMA, Bot không bị lừa bởi các cú Spike (râu nến) giả mạo.

### 2.3. Bollinger Bands & Keltner Channels (Squeeze)
- **Mục đích:** Đo đạc năng lượng tích lũy (Volatility Contraction).
- **Tại sao kết hợp cả 2?** Khái niệm "Squeeze" (John Carter) xảy ra khi Bollinger Bands (đo bằng độ lệch chuẩn) chui lọt vào bên trong Keltner Channels (đo bằng ATR - dải thực tế). Khi điều này xảy ra, thị trường đang nén lại như một chiếc lò xo. Một khi lò xo bung ra (BB mở rộng ra ngoài KC), nó tạo ra một xung lực cực mạnh. Helios dùng nó để kích nổ `Momentum_Ignition`.

### 2.4. MACD & RSI & MFI
- **Mục đích:** Đo Động lượng (Momentum) và Xung lực Khối lượng (Volume Force).
- **Tại sao MACD?** Khoảng cách giữa 2 đường EMA đo đạc "Gia tốc" của giá.
- **Tại sao RSI & MFI?** RSI đo vận tốc thay đổi giá thuần túy, trong khi MFI đo lường vận tốc đó dưới sự bảo chứng của Khối lượng (Volume). Nếu giá tăng (RSI tăng) nhưng dòng tiền (MFI) không vào, đó là sự Tích tụ cạn kiệt (Absorption/Climax), báo hiệu đảo chiều (Reversal).

### 2.5. ATR (Average True Range)
- **Mục đích:** Thước đo rủi ro thuần túy.
- **Tại sao ATR quan trọng?** Nó cho biết mỗi nến giá dao động bao nhiêu USD. Đặt Stoploss bằng ATR thay vì bằng % cố định giúp Stoploss co giãn tự nhiên theo hơi thở của thị trường.

---

## 3. Market Matrix (Ma Trận Nhận Thức)
Các chỉ báo trên là các con số rời rạc và có thang đo khác nhau (ADX thang 100, Góc thang 90 độ, MACD có thể âm/dương vô cực). Để **Trọng tài (Arbiter)** và **Scanner** hiểu được, Helios đưa tất cả qua quá trình Chuẩn hóa (Normalization) để tạo ra các Lực (0.0 đến 1.0):

- **Trend Strength Force (0.0 - 1.0):** Tổng hợp từ ADX và KAMA slope.
- **Momentum Force (-1.0 đến 1.0):** Tích hợp từ Histogram của MACD và RSI.
- **Squeeze Intensity (0.0 - 1.0):** Tỉ lệ độ hẹp của Bollinger Bands so với Keltner.
- **Chaos Ratio (0.0 - 1.0):** Đo lường sự hỗn loạn dựa trên Efficiency Ratio của KAMA.
- **Volume Force (0.0 - 1.0):** Tỉ lệ Volume hiện tại so với Trung bình Volume 100 phiên (Volume Spike).

Nhờ Ma trận này, mọi Scanner trong hệ thống đều dùng chung một bộ ngôn ngữ toán học, giúp GA cực kỳ dễ dàng tìm ra "Tổ hợp gen hoàn hảo" của thị trường.
