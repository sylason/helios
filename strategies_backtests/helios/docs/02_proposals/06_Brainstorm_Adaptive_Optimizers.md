# 🧠 Brainstorm: Adaptive Optimizers & The Curse of Dimensionality

**Context**: Giải quyết bài toán không gian tìm kiếm bùng nổ khi áp dụng Genetic Algorithm (GA) cho các hệ thống giao dịch có >100 tham số, với **ràng buộc tối thượng: KHÔNG hardcode bất kỳ tham số indicator nào** nhằm bảo toàn tính linh hoạt tối đa trước các trạng thái thị trường (Market Regimes).

---

## 1. Vấn đề cốt lõi: Lời nguyền chiều dữ liệu 🌪️

Khi số lượng tham số vượt quá 100, không gian tìm kiếm của GA mở rộng theo cấp số nhân (Combinatorial Explosion). Thuật toán dễ bị mắc kẹt tại *Local Optima*, tốc độ hội tụ siêu chậm và dẫn đến hiện tượng **Overfitting** chết người. 

Tuy nhiên, việc "cố định" (hardcode) các tham số như chu kỳ MACD hay RSI để giảm bớt chiều dữ liệu là hành động "tự sát" về mặt logic học:
- **Thị trường mang tính phi tuyến:** Chu kỳ 14 ngày của RSI có thể đúng ở năm 2020 nhưng sai hoàn toàn ở năm 2026.
- **Sự đa dạng của Regimes:** Optimizer cần quyền tự do để đo lường "nhịp thở" riêng của từng loại tài sản (Crypto, Forex, Stock) trong các hoàn cảnh khác nhau.

**Câu hỏi đặt ra:** Làm sao để giảm chiều không gian tối ưu hóa mà không tước đoạt đi sự tự do của hệ thống? 
Dưới đây là các kiến trúc đề xuất cho **Strategy V3**.

---

## 2. Các Đề xuất Kiến trúc Tối ưu (Strategy V3 Blueprints) 📐

### A. Gene Grouping & Master Parameters (Tương quan Phả hệ) 🧬
Thay vì để GA mò mẫm 100 tham số một cách rời rạc và độc lập, hãy cấu trúc chúng lại theo **đặc tính vật lý của thị trường**.
- **Cơ chế:** Khởi tạo các `Master Parameters` (Tham số gốc) đại diện cho một "tần số" hoặc "chu kỳ" của thị trường. Các tham số indicator con (`Slave Parameters`) sẽ được sinh ra thông qua một hàm tỷ lệ (Ratio) so với Master.
- **Ví dụ:**
  - `Master_Trend_Period` = GA(10..100)
  - `MACD_Fast` = Master_Trend_Period * 0.5
  - `MACD_Slow` = Master_Trend_Period * 1.0
  - `RSI_Period` = Master_Trend_Period * 0.25
- **Hiệu quả:** Giảm số lượng gene cần tiến hóa từ 20 xuống 1, nhưng vẫn giữ được sự đồng bộ về pha giữa các công cụ. GA lúc này chỉ cần làm một việc: Tìm ra **"Tần số chính" (Market Rhythm)**.

### B. Kiến trúc Adaptive Thresholds qua Rolling Percentile 🌊
Sự lỗi thời nhanh nhất của các parameters thường nằm ở việc sử dụng **ngưỡng giá trị tĩnh (Static Thresholds)** (ví dụ: `Volume > 1000` hoặc `RSI > 70`).
- **Cơ chế:** Tiêu diệt hoàn toàn các hằng số tĩnh bằng cách sử dụng **Z-score** hoặc **Rolling Percentile** (Phân vị trượt).
- **Ví dụ:** Thay vì tối ưu ngưỡng cắt của RSI từ 60 đến 90, hãy để hệ thống so sánh RSI hiện tại với phân vị thứ 80 của chính nó trong chu kỳ `N` nến qua: 
  `RSI_Current > Rolling_Percentile(RSI, window=N, percentile=0.8)`
- **Hiệu quả:** Optimizer không cần phải mò mẫm một hằng số vô nghĩa nữa. Hệ thống sẽ tự thích ứng (Auto-adapts) cực mạnh với các pha biến động (high-vol) hoặc đi ngang (low-vol) của thị trường. Tham số cần tối ưu lúc này chỉ là "Độ dài chu kỳ quan sát" và "Độ hiếm của sự kiện".

### C. Phased Optimization Pipeline (Tối ưu hóa Phân tầng) 🏗️
Đừng ném toàn bộ 100 tham số vào một nồi lẩu thập cẩm. Hãy chia nhỏ quá trình tiến hóa thành các giai đoạn nối tiếp.
- **Giai đoạn 1 (Macro & Regime):** Chỉ chạy GA để tìm ra các bộ lọc xu hướng chính (Core Trend Filters) và chu kỳ dài hạn. Mục tiêu: Nhận diện đúng Market State.
- **Giai đoạn 2 (Micro & Triggers):** Sau khi cố định kết quả tốt nhất của Giai đoạn 1, tiếp tục chạy GA để tối ưu hóa các điểm vào/ra lệnh ngắn hạn (Oscillators, Breakout Thresholds).
- **Giai đoạn 3 (Risk & Sizing):** Dùng bộ tín hiệu đã hình thành để tối ưu hóa chiến thuật Quản lý vốn (Stop-loss, Take-profit, Trailing).
- **Hiệu quả:** Cắt giảm không gian tìm kiếm khổng lồ $O(N^{100})$ xuống thành tổng các không gian nhỏ $O(N^{30}) + O(N^{40}) + O(N^{30})$. Tăng tốc độ hội tụ và giảm thiểu nhiễu loạn chéo.

---

## 3. Lời kết & Định hướng Triển khai 🚀

Để thiết kế **Strategy V3**, chúng ta tuyệt đối không thỏa hiệp bằng cách hardcode. Chúng ta sẽ "hack" cấu trúc không gian của GA:

1. 🛠️ **Refactor kiến trúc Parameter:** Định nghĩa lại Data Structures trong Rust để cho phép khởi tạo `Master-Slave Relationships`.
2. ⚙️ **Tích hợp Adaptive Metrics vào Lõi (ferro_trade):** Tiêu chuẩn hóa các thư viện tính toán để Z-score và Percentile trở thành công cụ cơ bản (first-class citizens) dễ dàng truy xuất với hiệu năng cao.
3. 🔄 **Xây dựng Pipeline Đa tầng (Event Bus):** Thiết kế quá trình backtest thành các workflow nối tiếp (Phased), thay vì chạy nguyên khối.

> **Khẩu quyết:** Đừng bắt Optimizer học thuộc lòng một con số tĩnh, hãy dạy nó cách tự tìm ra thước đo đúng cho mọi thời điểm.
