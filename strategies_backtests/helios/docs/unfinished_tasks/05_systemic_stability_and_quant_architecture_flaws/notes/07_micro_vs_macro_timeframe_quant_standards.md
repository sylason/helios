# Ghi Chú Nghiên Cứu Định Lượng: Tiêu Chuẩn Phân Chia Khung Thời Gian Vi Mô (Micro) vs. Vĩ Mô (Macro)

> [!WARNING]
> **TÀI LIỆU NGHIÊN CỨU & KHẢO SÁT (LAB RESEARCH NOTE - UNVERIFIED)**  
> Tài liệu này ghi chép các phân tích nền tảng vật lý và toán học phục vụ thảo luận kiến trúc, chưa phải là đặc tả kỹ thuật nghiệm thu cuối cùng của hệ thống.

---

## 🎯 1. Vấn Đề Nhận Thức Luận (The Epistemic Problem)

Trong giới giao dịch cá nhân (Retail Trading), các khung thời gian như `1m`, `5m`, `15m`, `1h`, `4h`, `1d` thường được coi như những "chân lý hiển nhiên" kế thừa từ biểu đồ nến Nhật cổ điển. 

Tuy nhiên, dưới góc nhìn của **Vật lý Kinh tế lượng (Econophysics)** và **Cấu trúc Vi mô Thị trường (Market Microstructure)**:
- **Thời gian trong thị trường tài chính là một liên tục thể (Continuum):** Không có cây nến nào "tự nhiên" sinh ra là 15 phút hay 1 giờ. Đó hoàn toàn là các cửa sổ lấy mẫu nhân tạo (Artificial Sampling Windows $\Delta t$).
- **Hình học Fractal (Mandelbrot):** Thị trường tài chính có tính chất tự tương tự (Self-similarity). Các mô hình dao động lặp lại ở mọi quy mô thời gian.

**Câu hỏi cốt tử:** *Nếu $\Delta t$ có thể là bất kỳ giá trị nào (1s, 1m, 3m, 7m, 15m, 42m, 1h...), làm sao một hệ thống định lượng (Quant) có thể phân định rạch ròi, khách quan khung nào là Vi mô (Micro), khung nào là Vĩ mô (Macro) mà không bị rơi vào cảm tính giáo điều?*

---

## 📐 2. Bốn Định Mức Vật Lý & Toán Học Khách Quan

Để loại bỏ hoàn toàn tính chủ quan, hệ thống thiết lập 4 định mức định lượng độc lập để phân loại khung thời gian:

### 2.1. Định mức 1: Ngưỡng Nhiễu Vi Cấu Trúc (Microstructure Noise Threshold - $\tau_{noise}$)
- **Hiện tượng vật lý:** Ở các khoảng thời gian cực ngắn ($\Delta t < 1 \dots 3$ phút), biến động giá bị chi phối bởi các yếu tố cơ học của sàn giao dịch:
  1. Độ trễ truyền dẫn mạng (Network & WebSocket Latency: $10 \dots 150\text{ms}$).
  2. Bước giá tối thiểu (Tick size discreteness).
  3. Dao động đập qua đập lại giữa giá Bid và Ask khi khớp lệnh Market (Bid-Ask Bounce).
- **Thước đo toán học:** Hàm tự tương quan bậc 1 của chuỗi lợi suất nến:
  $$\rho_1 = \text{Corr}(r_t, r_{t-1}) \quad \text{với } r_t = \ln\left(\frac{P_t}{P_{t-1}}\right)$$
  - Nếu $\rho_1 < -0.15$ (tự tương quan âm có ý nghĩa thống kê): Thị trường đang nằm trong **Vùng Nhiễu Vi Cấu Trúc**. Biến động chủ yếu là nhiễu trắng và lực nảy cơ học.
- **Quy chuẩn kiến trúc:** 
  - Khung thời gian nào có $\rho_1 < -0.15$ (ở Crypto là $1s \to 1m$) được phân loại là **CỰC VI MÔ (Ultra-Micro / Micro)**.
  - **Giới hạn trách nhiệm:** Tuyệt đối KHÔNG dùng khung này để xác định xu hướng lớn. Khung này chỉ được phép dùng cho: **Khớp lệnh (Execution Timing), Tính trượt giá (Slippage), Kẹp Stop Loss (SL Clamping), và Bảo vệ khẩn cấp (Emergency Breaker)**.

---

### 2.2. Định mức 2: Tỷ Số Phân Tách Bước Sóng (The Spectral Octave Rule)
- **Nguyên lý xử lý tín hiệu:** Theo lý thuyết Fourier và Wavelet, để hai chuỗi tín hiệu không bị trùng lặp thông tin và không gây nhiễu lẫn nhau, bước sóng (chu kỳ lấy mẫu) giữa hai tầng quan sát phải cách nhau ít nhất một quãng tần số (Octave Separation):
  $$\frac{T_{macro}}{T_{micro}} \ge 3 \dots 5$$
- **Hệ quả định lượng:**
  - Nếu chọn $T_{micro} = 5m$ và $T_{macro} = 6m$ ($\text{Tỷ số} = 1.2\text{x}$): Hệ số tương quan Pearson giữa hai chuỗi là $\rho > 0.98$. Tầng Macro không hề cung cấp thêm thông tin mới nào, mà chỉ làm tăng gấp đôi độ trễ tính toán.
  - Khi tỷ số $\frac{T_{macro}}{T_{micro}} \in [3, 5]$ (ví dụ: $1m \to 5m \to 15m \to 1h$): Hệ số tương quan giảm xuống mức lý tưởng ($\rho \approx 0.3 \dots 0.5$).
- **Quy chuẩn kiến trúc:** 
  - Tách bạch rõ 3 tầng quan sát theo bậc bước sóng:
    - **Vi mô (Micro):** $1m$
    - **Trung mô (Meso):** $5m \dots 15m$ ($\text{Bước nhảy} = 5\text{x} \dots 15\text{x}$)
    - **Vĩ mô (Macro):** $1h \dots 4h$ ($\text{Bước nhảy} = 4\text{x} \dots 16\text{x}$ so với Meso)

---

### 2.3. Định mức 3: Thời Gian Bán Rã Của Thông Tin (Information Half-Life - $t_{1/2}$)
- **Bản chất kinh tế:** Một lệnh quét thanh khoản lớn (Whale Absorption) hoặc một chuỗi thanh lý (Liquidation Cascade) mất bao lâu để thị trường "tiêu hóa" và quay về trạng thái cân bằng mới?
- **Thước đo toán học:** Tốc độ suy giảm tự tương quan của dòng tiền và biến động (Autoregressive Decay of Order Flow Imbalance):
  $$I(t) = I_0 \cdot e^{-\lambda t} \implies t_{1/2} = \frac{\ln(2)}{\lambda}$$
- **Thực tế phái sinh Crypto:**
  - Đối với các tài sản thanh khoản cao (BTC, ETH): $t_{1/2}$ của một xung lực dao động trong khoảng $10 \dots 20$ phút.
  - Đối với Altcoins: $t_{1/2}$ dao động trong khoảng $20 \dots 45$ phút.
  - Sau $45 \dots 60$ phút, thông tin của xung lực cũ gần như tiêu biến hoàn toàn, nhường chỗ cho bối cảnh dòng tiền mới.
- **Quy chuẩn kiến trúc:**
  - Khung $\le 15$ phút ($t \le t_{1/2}$) là **Không Gian Chiến Thuật (Tactical Window)**: Nơi các Scanners tìm kiếm các điểm mất cân bằng cung cầu tạm thời.
  - Khung $\ge 1$ giờ ($t \gg t_{1/2}$) là **Không Gian Bối Cảnh (Strategic/Regime Window)**: Nơi các biến động ngẫu nhiên đã bị triệt tiêu, chỉ còn lại xu hướng tích lũy/phân phối của các thế lực lớn.

---

### 2.4. Định mức 4: Ma Trận Phân Định Trách Nhiệm Bất Biến (Strict Separation of Roles)

Dựa trên 3 định mức vật lý trên, hệ sinh thái phân chia trách nhiệm bất biến giữa các tầng:

| Tầng Thời Gian | Khung Đại Diện | Vai Trò Hệ Thống | Câu Hỏi Nghiệp Vụ Duy Nhất | Output Cung Cấp Cho Hệ Thống |
| :--- | :--- | :--- | :--- | :--- |
| **1. VĨ MÔ (Macro)** | $1h - 4h$ | **Bộ Chỉ Huy Chiến Lược (Regime Gatekeeper)** | *Thị trường hiện tại là Xu hướng, Dao động tích lũy, hay Bão tố hỗn loạn?* | Bộ lọc nhị phân / Trạng thái cấp phép (`PERMITTED_PLAYBOOKS`, `CHAOS_VETO`). |
| **2. TRUNG MÔ (Meso)** | $15m$ | **Radar Tác Chiến (Scanners Matrix)** | *Mô hình cơ hội nào đang xuất hiện (Bứt phá, Hấp thụ, Đảo chiều)?* | Điểm tự tin chuẩn hóa ($Confidence \in [0.0, 1.0]$) để đấu thầu công bằng. |
| **3. VI MÔ (Micro)** | $1m$ | **Súng & Khiên Thực Thi (Execution & Risk Guardian)** | *Điểm khớp lệnh tối ưu ở đâu? Đặt SL bao nhiêu? Khi nào Trailing?* | Size vị thế, Giá Limit/Market, Stop Loss Clamping, Trailing Stop. |

---

## 💡 3. Kết Luận Định Lượng

1. **Không có khung thời gian "thần thánh":** Phân chia Vi mô vs. Vĩ mô không phải là cảm tính cá nhân, mà là bài toán tách lọc tần số sóng và bán rã thông tin.
2. **Quy tắc vàng:**
   - **Tầng Vĩ mô (Macro)** nhìn xa nhưng hành động chậm: Chỉ dùng để **MỞ/KHÓA VAN** (Gating).
   - **Tầng Trung mô (Meso)** nhận diện cơ hội: Dùng để **CHẤM ĐIỂM CHIẾN THUẬT** (Scoring).
   - **Tầng Vi mô (Micro)** phản xạ tức thời: Dùng để **BẢO VỆ VỐN VÀ THỰC THI** (Execution & Risk).
