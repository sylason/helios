# MỔ XẺ CHUYÊN SÂU MA TRẬN TRẠNG THÁI THỊ TRƯỜNG (MARKET MATRIX V3.3)

> **Ngày thực hiện:** 04/10/2026  
> **Phạm vi kiểm toán:** `core_v3/matrix.rs`, `core_v3/normalization.rs`, `core_v3/config.rs:25-70`.  
> **Mục tiêu:** Kiểm toán tính toàn vẹn toán học, hành vi vi cấu trúc và các điểm mù trong tầng chuẩn hóa ma trận trạng thái (Level 2: Market Matrix V3.3).

---

## 1. VAI TRÒ KIẾN TRÚC & HIỆN TRẠNG 6 TRỤC ĐO

`MarketMatrixV3` chịu trách nhiệm chuẩn hóa các cảm biến thô từ `HeliosSensors` thành các đại lượng phi thứ nguyên $[0.0, 1.0]$ hoặc $[-1.0, 1.0]$:

| Tên biến | Công thức tính toán | Mục đích thiết kế | Mức độ tiêu thụ thực tế |
| :--- | :--- | :--- | :--- |
| **`chaos_index`** | $\text{percentile}(\text{ChaosRatio}) \times (1.0 - \text{percentile}(ER))$ | Nhận diện bão Whipsaw $[0.0, 1.0]$ | ✅ Tiêu thụ bởi `Gatekeeper` |
| **`squeeze_intensity`** | $\text{percentile}\left(\frac{1}{1 + (BB_w / KC_w)^2}, \text{lookback}=20\right)$ | Cường độ nén dải biến động $[0.0, 1.0]$ | ✅ Tiêu thụ bởi `Trend_PB` và `Absorption` |
| **`kama_slope_atr`** | $\text{clamp}\left(\frac{\text{kama\_slope\_adj}}{\text{scale\_atr}}, -1.0, 1.0\right)$ | Độ dốc KAMA chuẩn hóa qua ATR | ✅ Tiêu thụ bởi cả 4 Scanner |
| **`trend_strength`** | $\text{clamp}\left(\frac{ADX}{100.0}, 0.0, 1.0\right)$ | Sức mạnh xu hướng tuyến tính | ❌ **ZOMBIE CODE:** Cả 4 scanner không hề đọc |
| **`bb_width_norm`** | $\text{clamp}\left(\frac{BB_w}{BB_m}, 0.0, 1.0\right)$ | Biên độ dải Bollinger chuẩn hóa | ❌ **ZOMBIE CODE:** Không có scanner nào đọc |
| **`bb_middle_slope_adj`** | $\text{clamp}\left(\frac{\Delta BB_m}{5 \times ATR}, -2.0, 2.0\right)$ | Độ dốc trục giữa Bollinger | ❌ **ZOMBIE CODE:** Chỉ dùng trong thẻ log telemetry |

---

## 2. BỐN LỖ HỔNG TOÁN HỌC GỐC RỄ ĐÃ XÁC MINH

### ❌ Lỗ hổng 1: Bẫy Tê Liệt Của Cửa Sổ Trượt Percentile (The Rolling Percentile Memory Trap)
- **Mã nguồn:** Dòng 81–85 sử dụng `rolling_percentile` với cửa sổ chỉ có 20 nến (`sqz_lb = 20` và `trend_lb = 20`):
  ```rust
  let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
  let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, sqz_lb);
  let entropy = normalization::rolling_percentile(&sensors.efficiency_ratio, trend_lb);
  ```
- **Hệ quả vi cấu trúc:**
  1. **Nén Tích Lũy Kéo Dài (Prolonged Squeeze):** Thị trường nén chặt 30 nến để chuẩn bị nổ vol. Từ nến thứ 21 trở đi, toàn bộ 20 nến trong cửa sổ đều có giá trị nén cao xấp xỉ nhau. Hàm percentile suy biến về trung vị **0.50** (chính unit test dòng 497 thừa nhận điều này). Thị trường nén chặt nhất thì `squeeze_intensity` lại tự tụt từ 0.90 xuống 0.50 $\to$ `Absorption` bỏ qua cơ hội gom, `Trend_PB` tưởng hết chop zone nên nhảy vào sớm.
  2. **Bão Hỗn Loạn Kéo Dài (Prolonged Storm):** Cơn bão kéo dài 40 nến. Sau nến thứ 20, cả cửa sổ đều là nến bão. Nến thứ 25 dù biến động dữ dội nhưng so với 20 nến bão trước đó thì nó là bình thường $\to$ `chaos_index` tụt từ 0.75 xuống dưới 0.40 $\to$ **Gatekeeper tự động mở khóa cho bot lao vào giữa tâm bão!**

### ❌ Lỗ hổng 2: Chỉ Báo Zombie Lãng Phí Bộ Nhớ (Zombie Indicators)
- 3/6 trục đo (`trend_strength`, `bb_width_norm`, `bb_middle_slope_adj`) được tính toán cho 43,100 nến, lưu trong `Arc<Vec<f64>>` nhưng không có scanner nào sử dụng.
- Cảm biến cực tốt như `trend_strength` (ADX) phản ánh sức mạnh xu hướng thì bị bỏ quên, trong khi `Trend_PB` lại chỉ dựa dẫm mù quáng vào 1 biến duy nhất là `kama_slope_atr`.

### ❌ Lỗ hổng 3: Bão Hòa Độ Dốc KAMA Triệt Tiêu Độ Nhạy (Saturation Trap)
- **Mã nguồn (Dòng 87-90):**
  ```rust
  let scale_atr = if config.matrix.kama_slope_scale_atr <= 0.0 { 0.20 } else { config.matrix.kama_slope_scale_atr };
  let kama_slope_atr: Vec<f64> = sensors.kama_slope_adj.iter().map(|&s| {
      if s.is_finite() { (s / scale_atr).clamp(-1.0, 1.0) } else { 0.0 }
  }).collect();
  ```
- Khi trend mạnh trên nến 15m/1h, `kama_slope_adj` thường đạt $0.30 - 1.50$. Chia cho 0.20 rồi kẹp cứng `clamp(-1.0, 1.0)` khiến `kama_slope_atr` liên tục bị kẹp cứng ở **+1.00 hoặc -1.00** hàng chục nến liên tiếp $\to$ Mất khả năng phân biệt giữa xu hướng ổn định và xu hướng kiệt sức parabol.

### ❌ Lỗ hổng 4: Nhiễu Nền Của `chaos_index` Khi Thị Trường Phẳng Lặng (Chop Noise Baseline)
- Khi thị trường đi ngang hoàn toàn vô hại (không bão, không tin tức):
  $ER$ dao động quanh 0 $\implies \text{percentile}(ER) \approx 0.50 \implies noise \approx 0.50$.
  $ChaosRatio$ dao động quanh trung bình $\implies \text{percentile}(ChaosRatio) \approx 0.50$.
  Kết quả: $chaos\_index \approx 0.50 \times 0.50 = \mathbf{0.25 - 0.35}$.
- Ngay cả khi thị trường phẳng lì như mặt hồ, `chaos_index` vẫn luôn hiển thị mức 0.25–0.35. Khoảng cách từ mức nền này đến ngưỡng bão 0.70 là quá hẹp, khiến việc tinh chỉnh ngưỡng Gatekeeper trở nên rất mong manh.

---

## 3. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành phần Matrix | Ưu điểm | Khuyết điểm | Edge Cases nguy hiểm |
| :--- | :--- | :--- | :--- |
| **Safe Math Guards** | Khử 100% nguy cơ panic chia cho 0, NaN, Infinity. | — | — |
| **Squeeze Intensity** | Hàm mềm $1 / (1 + \text{ratio}^2)$ liên tục và mượt mà. | Bị `rolling_percentile(20)` kéo tụt về 0.50 khi nén lâu. | Đi ngang tích lũy 40 nến: Squeeze hiển thị 0.50 (tưởng thị trường bình thường). |
| **KAMA Slope ATR** | Chuẩn hóa phi thứ nguyên theo ATR, asset-agnostic. | Phép chia thô cho 0.20 gây bão hòa cứng tại $\pm 1.00$. | Mất độ nhạy gia tốc xu hướng khi sóng mở rộng mạnh. |
| **Chaos Index** | Kết hợp biến động bùng nổ ($c\_r$) với sự mất phương hướng ($noise$). | Suy biến khi bão dài hơn 20 bars; nhiễu nền cao ở thị trường phẳng. | Cơn bão thanh lý kéo dài 40 phút: Nến 25 Gatekeeper tự tắt báo động. |
