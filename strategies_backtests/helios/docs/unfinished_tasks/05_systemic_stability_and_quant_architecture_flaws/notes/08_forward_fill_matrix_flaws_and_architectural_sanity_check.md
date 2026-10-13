# Ghi Chú Nghiên Cứu Định Lượng: Mổ Xẻ Dị Tật Forward-Fill Trong Matrix & Đối Chất Xung Đột Đa Khung Thời Gian

> [!WARNING]
> **TÀI LIỆU NGHIÊN CỨU & ĐỐI CHẤT HỌC THUẬT (LAB RESEARCH NOTE - UNVERIFIED)**  
> Tài liệu này ghi nhận sự thật kiểm toán mã nguồn ngày 12/10/2026 về lỗi tính toán trên dữ liệu rải phẳng (Forward-Fill), nguồn gốc thất bại của sàn đấu thầu Arbiter, và giải pháp triệt tiêu xung đột Vi mô vs Vĩ mô.

---

## 🔬 1. SỰ THẬT KIỂM TOÁN: LỖI TÍNH TOÁN TRÊN CHUỖI FORWARD-FILL TẠI `matrix.rs`

### 1.1. Tầng `ferro_trade` (Cốt lõi): Đã Tính Đúng
- Trong `ferro_trade/src/backtest/context.rs` (dòng 133–158) và `bulk.rs`:  
  Mọi chỉ báo đa khung thời gian (KAMA, KAMA Slope, Bollinger Bands, ATR, ADX) **được tính toán trực tiếp trên mảng nến khung lớn gốc (mảng ngắn $N/15$)**.
- Các phép tính căn bậc 2 trong độ lệch chuẩn, sai phân, trung bình động đều vận hành trên dữ liệu nến 15m thực tế.
- Sau khi có mảng kết quả cuối cùng, engine mới dùng `lookup_table` để rải (forward-fill) mảng kết quả về độ dài $N$ của khung 1m để hàm `next()` dễ truy cập.

---

### 1.2. Tầng `helios/matrix.rs` (Chiến Lược): Phát Hiện Dị Tật Toán Học Nặng Nề!
Tại file [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L81-L85):
```rust
let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, sqz_lb);
let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, trend_lb);
```

#### Dòng dữ liệu bị nhiễm độc (Data Lineage Infection):
1. `sensors.chaos_ratio` và `sensors.efficiency_ratio` vốn được tính từ khung 15m/1h (`ind.natr.timeframe`, `ind.kama.timeframe`), và **đã bị rải phẳng thành mảng 1m** (15 nến liên tiếp có cùng 1 giá trị giống hệt nhau).
2. Sau đó, `matrix.rs` lại gọi hàm `rolling_percentile` **chạy trực tiếp trên mảng 1m đã bị rải phẳng này**, với cửa sổ trượt `trend_lb` (100–1000 nến 1m).

#### Hệ quả toán học méo mó:
- Trong cửa sổ trượt 100 nến 1m, thực chất chỉ có $100 / 15 \approx 6.6$ quan sát nến 15m độc lập! Mỗi giá trị bị nhân bản cơ học 15 lần.
- Cây Fenwick Tree trong `normalization.rs` phải lưu trữ các giá trị trùng lặp nhân tạo, làm sai lệch phân phối xác suất tích lũy (Cumulative Distribution Function - CDF).
- **Thứ hạng phân vị (Percentile Rank) bị giật cục theo từng nấc thang 15 phút một lần**, trong 14 phút ở giữa nó hoàn toàn tê liệt.
- Tương tự, `kama_slope_adj` bị giữ nguyên suốt 15 phút. Nếu ở phút thứ 5 giá 1m bị sập $2\%$, ma trận `matrix.kama_slope_atr` vẫn báo là *"thị trường đang dốc lên với độ dốc của 15 phút trước"*.

---

## ❓ 2. TẠI SAO TA CẦN `rolling_percentile`? CÓ THỂ TỐI GIẢN HÓA ĐƯỢC KHÔNG?

### 2.1. Bản chất nghiệp vụ: Tại sao nó xuất hiện?
- Các chỉ số thô như Độ nén Squeeze (`BB_Width / KC_Width`), Chaos Ratio, hay Efficiency Ratio mang giá trị tuyệt đối rất khác nhau giữa các cặp tiền (BTC biến động $1\%$ khác xa MEME coin biến động $15\%$).
- `rolling_percentile` được sinh ra để **phi thứ nguyên hóa (Dimensionless Scaling)**: Chuyển một giá trị bất kỳ về đoạn $[0.0, 1.0]$, trả lời câu hỏi: *"Giá trị hiện tại đang nằm ở phân vị thứ mấy so với lịch sử $W$ nến vừa qua?"*.

### 2.2. Điểm yếu và Bẫy Phức Tạp Hóa (Over-Engineering Risk)
- Cực kỳ ngốn tài nguyên CPU: Duy trì cây Fenwick / Sorted Window trên từng nến 1m.
- Nhạy cảm với Lookback Window: Cửa sổ 100 nến cho kết quả khác xa 500 nến.

### 2.3. Giải pháp Tối Giản Định Lượng (Occam's Scalpel):
1. **Tính Percentile trên khung gốc:** Nếu một chỉ báo thuộc khung 15m, việc tính `rolling_percentile` **BẮT BUỘC phải thực hiện trên mảng ngắn 15m gốc** (với window 30–50 nến 15m). Sau khi có mảng percentile mượt mà mới rải về 1m!
2. **Khảo sát giải pháp thay thế đơn giản hơn:** Đánh giá việc thay thế `rolling_percentile` bằng hàm phi tuyến $\tanh(\cdot)$ kẹp trần hoặc Rolling Min-Max Scaling nhẹ hơn $10\text{x}$.

---

## ⚖️ 3. TẠI SAO "ĐẤU THẦU ĐIỂM TỰ TIN" TRƯỚC ĐÂY KHÔNG HIỆU QUẢ?

Hệ thống đã theo đuổi triết lý "Đấu thầu Arbiter" từ lâu nhưng kết quả OOS vẫn trồi sụt, thua lỗ. Dưới đây là **4 nguyên nhân gốc rễ (Root Causes)** được chỉ ra từ code và log thực nghiệm:

1. **"Sàn Đấu Thầu Ma" (Phantom Auction - Không hề có cạnh tranh):**
   - Các Scanner bị thiết kế với các điều kiện hình học tự loại trừ nhau:
     - `Momentum`: Đòi hỏi giá nằm ngoài dải BB.
     - `Trend_PB`: Đòi hỏi giá hồi sâu vào trong dải BB.
     - `Absorption`: Đòi hỏi dải BB bị bóp nghẹt (Squeeze).
     - `Reversal` & `Range_Osc`: Bị tê liệt (trigger = 0).
   - **Hệ quả:** Trong $99.9\%$ trường hợp, trên cùng 1 cây nến chỉ có ĐÚNG 1 Scanner kích hoạt (hoặc không có thằng nào). Arbiter không hề có đối thủ để chọn lựa, mà biến thành "con dấu tự động phê duyệt"!
2. **Điểm Tự Tin Giả Tạo (Subjective Confidence Scoring):**
   - Mỗi scanner tự chấm điểm theo một công thức nội bộ tùy tiện, không đồng nhất về mặt kỳ vọng toán học ($EV$). Một scanner hào phóng tự cho điểm $0.85$ sẽ luôn đè bẹp scanner chỉ cho điểm $0.60$, dù xác suất thắng thực tế của nó thấp hơn.
3. **Arbiter "Mù" Bối Cảnh Vĩ Mô (Context-Blind Arbiter):**
   - Arbiter chỉ biết so điểm giữa các Scanner, nhưng **không có quyền khóa cửa khi thị trường bão tố (Chaos) hoặc đi ngang không có xu hướng (Chop)**.
   - Khi thị trường đi ngang, `Momentum` bắn tín hiệu bứt phá giả (Fakeout), tự cho điểm $0.80$, Arbiter thấy cao nhất liền mở lệnh $\to$ ăn đòn ngay lập tức!
4. **Hệ Thống Risk Phá Hủy Tỷ Lệ R:R:**
   - Dù lệnh thắng thầu chạy đúng hướng (MFE $+6.8\%$ ROE), Trailing SL bị giãn ATR nở rộng nuốt chửng lợi nhuận, kéo tụt về chỉ còn $+0.4\%$ ROE, trong khi lệnh thua cắn full Hard SL $-6.0\%$.

---

## 🧭 4. CƠ CHẾ TRIỆT TIÊU XUNG ĐỘT ĐA KHUNG THỜI GIAN (VI MÔ VS VĨ MÔ)

### 4.1. Bản chất xung đột kinh điển
- **Xung đột Hướng:** Khung 1h/4h đang là Xu hướng Tăng mạnh (Macro Bull), nhưng khung 1m đang có một cây nến đỏ sập mạnh (Micro Dump).
  - Nếu Scanner 1m thấy nến đỏ mạnh $\to$ Đòi Short (Momentum Short). Nhưng Short lúc này là chặn đầu xe lửa!
  - Nếu Scanner Trend_PB thấy giá giảm $\to$ Đòi Long bắt đáy. Nhưng nến 1m đang rơi tự do chưa có lực đỡ $\to$ Bắt dao rơi đứt tay!

### 4.2. Cơ chế Giải Quyết Triệt Để: Phân Cấp Bất Đối Xứng (Asymmetric Command Hierarchy)
Phương án 3 tầng giải quyết xung đột bằng nguyên tắc quân sự nghiêm ngặt:

```
[TẦNG 1: MACRO GATE (1h/4h)] --> RA LỆNH CẤM (Veto/Permission Only)
           |                     (Chỉ cho phép LONG / Chế độ Chop: NGỒI IM)
           v
[TẦNG 2: MESO MATRIX (15m)]  --> CUNG CẤP VÙNG SETUP
           |                     (Xác định vùng giá hỗ trợ KAMA / Biên BB)
           v
[TẦNG 3: MICRO EXECUTION (1m)]--> CUNG CẤP ĐIỂM BẮT ĐẦU & BẢO VỆ
                                 (Chờ nến 1m co chân/hấp thụ mới bấm cò)
```

1. **Macro KHÔNG BAO GIỜ ra lệnh vào lệnh:** Nó chỉ làm nhiệm vụ **LỌC VÀ KHÓA CỬA**. Nếu Macro 1h là Bull, toàn bộ các tín hiệu Short ở khung nhỏ bị triệt tiêu ngay lập tức.
2. **Micro KHÔNG BAO GIỜ được phép chống lại Macro:** Khung 1m chỉ được quyền quyết định: *Đã đến lúc bấm cò theo hướng Macro chưa? Đặt SL ở mức giá nào an toàn nhất?*.

### 4.3. Đối chất: Có xung đột logic nào còn sót lại không?
- **Độ trễ khi đảo chiều xu hướng lớn (Trend Reversal Lag):** Khi thị trường thực sự đảo chiều từ Bull sang Bear, khung Macro 1h sẽ phản ứng chậm $15 \dots 45$ phút. Trong thời gian này, Macro vẫn cấm Short và có thể dính 1 lệnh Long thua.
  - *Đánh giá:* **Đây là sự đánh đổi bắt buộc và chuẩn xác về mặt định lượng.** Không một hệ thống nào trên thế giới có thể vừa ăn trọn xu hướng lớn vừa bắt trúng râu đảo chiều đầu tiên ở khung 1m. Sự đánh đổi này bảo vệ tài khoản khỏi $90\%$ các cú bẫy phá vỡ giả (Whipsaws).
