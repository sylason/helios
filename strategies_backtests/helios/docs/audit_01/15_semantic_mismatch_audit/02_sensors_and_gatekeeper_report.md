# BÁO CÁO KIỂM TOÁN CHỈ BÁO, CẢM BIẾN & CẦU DAO GATEKEEPER

> **Hệ thống:** Helios V3.3  
> **Chuyên đề:** Kiểm toán Lệch Ngữ Nghĩa (Semantic Mismatch & MTF Traps)  
> **Đối tượng:** `config.rs`, `core_v3/mod.rs` (`init`), `gatekeeper.rs`, `matrix.rs`, `scanners/`

---

## 1. Executive Summary (Tóm Tắt Báo Cáo)

Sau khi rà soát từng dòng mã nguồn, đối chiếu toán học và vi cấu trúc dòng dữ liệu (Data Lineage) từ lúc resample nến đến khi nạp vào cảm biến và ra quyết định tại Gatekeeper / Scanners, kiểm toán đã phát hiện **2 lỗi CRITICAL (nguy cơ làm tê liệt hệ thống)**, **4 lỗi HIGH** và **4 lỗi MEDIUM**.

Nghiêm trọng nhất là:
1. **Bẫy Flatline Mù Mới Trong Gatekeeper (`CRITICAL`):** Đem so sánh tỷ lệ biến động của **1 cây nến vi mô 1m** (`micro_natr_ratio = current_atr_sl / price` $\approx 0.03\% - 0.07\%$ trên BTC/ETH) với **sàn chi phí toàn bộ vòng đời vị thế** (`effective_min_natr` $\approx 0.18\% - 0.40\%$). Hậu quả: Gatekeeper phủ quyết oan **90% - 95% số nến bình thường**, làm bot bị đóng băng giao dịch!
2. **Bẫy Đo Đạc Cọc Cạch Trong TrendPB (`CRITICAL`):** Đo khoảng cách từ giá nến 1m đến KAMA vĩ mô ($15\text{m}/1\text{h}/4\text{h}$) bằng đơn vị **ATR vi mô 1m** (`delta_kama = (close - kama_mid) / atr_sl`), nhưng lại so sánh với ngưỡng chặn hẹp `max_pullback_dist_atr` ($0.50 \to 3.50$). Khi KAMA chọn khung 1h hoặc 4h, khoảng cách này thường xuyên $> 3.5$ ATR vi mô, khiến TrendPB **bị triệt tiêu hoàn toàn (Dead Zone)**!
3. **Cảm biến mồ côi `vol_sma` bị bỏ quên (`HIGH`):** Struct `HeliosSensors` có trường `vol_sma`, `ReversalScanner` có đọc `vol_sma`, nhưng hàm `init()` trong `core_v3/mod.rs` **hoàn toàn không khởi tạo**. Dẫn đến `vol_ratio` luôn bằng $1.0$ cố định, làm tê liệt thành phần Volume kiệt sức của Reversal Scanner.

---

## 2. BẢNG TỔNG KẾT KIỂM TOÁN CHỈ BÁO & CẢM BIẾN

| STT | Tên Chỉ Báo / Tham Số | Timeframe Cho Phép | Biến / Ngưỡng So Sánh | Vấn Đề Lệch Ngữ Nghĩa Phát Hiện | Mức Độ Rủi Ro | Đề Xuất Khắc Phục |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | **`Gatekeeper / Flatline Guard`** | `AtrParams` (1m, 3m, 5m) | `micro_natr_ratio` vs `effective_min_natr` (0.0018 -> 0.0040) | **Lệch thang đo biến động nến vs chi phí vị thế:** Tỷ lệ biến động của **1 nến 1m** (~0.04% trên BTC) không thể gánh nổi toàn bộ phí sàn 2 chiều + trượt giá + min Alpha (0.18% - 0.40%). Hệ thống bị Flatline Veto oan 95% thời gian. | 🔴 **CRITICAL** | **Phương án A:** Dùng `sensors.natr` (khung 15m/1h/4h) quy về thập phân `macro_natr / 100.0` để so sánh với `effective_min_natr` (đúng bản chất chi phí vị thế).<br>**Phương án B:** Hoặc nhân `micro_natr_ratio` với căn bậc hai thời gian nắm giữ trung bình: $\text{ratio} \times \sqrt{T_{\text{hold}}}$. |
| **2** | **`TrendPB / delta_kama`** | `KamaParams` (15m, 1h, 4h) vs `AtrParams` (1m, 3m, 5m) | `delta_kama` vs `max_pullback_dist_atr` (0.50 -> 3.50) | **Lấy thước đo vi mô đo khoảng cách vĩ mô:** `kama_mid` thuộc khung lớn (15m/1h/4h), nhưng lại chia cho `current_atr` vi mô 1m. Khi KAMA là 4h, khoảng cách giá đến KAMA có thể lên tới 5-10 ATR 1m (dù chỉ là 0.5 ATR 4h), vượt quá trần 3.50 -> **TrendPB bị vô hiệu hóa hoàn toàn (Dead Zone) khi KAMA chọn 1h/4h.** | 🔴 **CRITICAL** | Chuẩn hóa `delta_kama` theo **ATR của chính khung KAMA** (ví dụ lưu `sensors.kama_atr` từ khung KAMA), hoặc tính tỷ lệ phi thứ nguyên `(close - kama_mid) / kama_mid` kết hợp NATR khung KAMA. |
| **3** | **`Gatekeeper / Chaos Whipsaw`** | `NAtrParams` (15m, 1h, 4h), `KamaParams` (15m, 1h, 4h) | `chaos_index` vs `chaos_veto_threshold` (0.50 -> 0.85) | **Bẫy làm phẳng nến bậc thang MTF (Flat Step Percentile):** `rolling_percentile` với cửa sổ 100 nến 1m chạy trên chuỗi MTF 4h (240 nến 1m có giá trị y hệt nhau). Trong 100 nến 1m cùng nến 4h, mọi phần tử bằng nhau khiến percentile kẹt cứng ở 0.50, làm `chaos_index = 0.5 * 0.5 = 0.25` phẳng lì, **cầu dao bão bị mù khi KAMA/NATR chọn khung 4h.** | 🟠 **HIGH** | Tính `rolling_percentile` trên mảng nến gốc của khung thời gian lớn **trước khi rải về 1m**, hoặc cửa sổ `lookback` phải được scale theo timeframe ($\text{window} = \text{bars} \times \text{ratio}$). |
| **4** | **`Sensors / vol_sma`** | `VolMaParams` (default 5m) | `sensors.vol_sma` trong `ReversalScanner` | **Cảm biến mồ côi không được khởi tạo (Uninitialized Sensor):** Struct `HeliosSensors` có `vol_sma`, `ReversalScanner` đọc `vol_sma`, nhưng hàm `init()` trong `core_v3/mod.rs` **quên gọi `ctx.volume_sma`**. Khiến `vol_ratio` luôn fallback về 1.0, làm tê liệt trụ cột Volume kiệt sức của Reversal Scanner. | 🟠 **HIGH** | Bổ sung dòng khởi tạo trong `core_v3/mod.rs`: `s.vol_sma = ctx.volume_sma(ind.vol_ma.timeframe, "VolSMA", ind.vol_ma.period);` |
| **5** | **`Matrix / bb_middle_slope_adj`** | `BbandsParams` (5m, 15m, 1h, 4h) | `bb_slope = (m_curr - m_prev) / (5 * atr)` | **Bẫy vi phân chuỗi bậc thang MTF (Step Derivative Trap):** Lấy hiệu số 5 nến 1m (`k_bars = 5`) trên đường `bb_middle` (khung 15m/1h/4h rải về 1m). Trong 95% thời gian, 5 nến 1m liên tiếp có cùng giá trị BB Middle nên độ dốc bằng 0.0, chỉ giật spike tại mép đóng nến lớn. | 🟠 **HIGH** | Tính độ dốc trực tiếp trên mảng nến gốc khung BBands rồi mới map về 1m (tương tự như cách `ctx.kama_slope` hoạt động). |
| **6** | **`Absorption / VWMA vs BB Middle`** | `VwmaParams` (5m..4h) vs `BbandsParams` (5m..4h) | `vwma < bb_middle` (Long), `vwma > bb_middle` (Short) | **Lệch khung thời gian giữa 2 trục so sánh:** `vwma` và `bbands` có gen Timeframe độc lập. Nếu GA chọn VWMA 5m và BBands 4h, điều kiện so sánh giá trung bình 1.6 giờ với giá trung bình 80 giờ. Trong downtrend 4h, VWMA 5m luôn < BB 4h (Long luôn mở, Short bị khóa 100%). | 🟠 **HIGH** | Khóa cứng đồng bộ `ind.vwma.timeframe = ind.bbands.timeframe` (tương tự như đã làm với Keltner Channel). |
| **7** | **`TrendPB / max_boundary_z`** | `PriceZScoreParams` (1m, 3m, 5m) | `z_score > max_boundary_z` (0.20 -> 0.80 sigma) | **Ngưỡng Z-score vi mô quá hẹp chặn đứng điểm vào lệnh:** Trong phân phối chuẩn, $Z > 0.30$ chiếm ~38% thời gian. Khi giá vừa nảy lên từ KAMA tạo nến xanh thuận trend, Z-score 1m lập tức vượt 0.3 - 0.5 sigma -> Bị Boundary Guard cấm oan. | 🟠 **HIGH** | Nâng dải DNA `max_boundary_z` lên `[0.80, 2.50]` sigma (đúng bản chất chống đu đỉnh bùng nổ, không chặn nhịp bật nảy bình thường). |
| **8** | **`Init / kama_slope ATR Period`** | `KamaParams` (15m, 1h, 4h) vs `AtrParams` (1m, 3m, 5m) | `ind.atr.period` (5 -> 60) truyền vào `ctx.kama_slope` | **Mượn chu kỳ ATR vi mô gán vào khung vĩ mô:** `kama_slope` trên khung 4h nhưng lại dùng `ind.atr.period` (vốn cấu hình cho ATR vi mô 1m). Nếu `ind.atr.period = 60`, hàm ATR 4h cần tới 240 giờ (10 ngày) dữ liệu để warmup. | 🟡 **MED** | Tách riêng tham số `atr_period` cho KAMA hoặc cố định chu kỳ chuẩn (ví dụ 14 bars) trên khung KAMA. |
| **9** | **`Reversal / displacement`** | `KamaParams` (15m, 1h, 4h) vs `AtrParams` (1m, 3m, 5m) | `(close - kama_mid).abs() >= min_disp * current_atr` | **Bắt dao rơi non khi KAMA ở khung 4h:** `displacement` tính bằng ATR vi mô 1m. Khi KAMA ở 4h, giá mới lệch 0.5 ATR 4h (sóng mới bắt đầu) nhưng đã bằng 5 ATR 1m, khiến Reversal Scanner tưởng là kiệt sức và bắt đỉnh/đáy quá sớm. | 🟡 **MED** | Chuẩn hóa `displacement` theo ATR của khung KAMA thay vì ATR 1m vi mô. |
| **10** | **`Config / KeltnerParams timeframe`** | `KeltnerParams` (Default 5m) | `ind.keltner.timeframe` bị skip và ghi đè | **Cấu hình rác gây hiểu lầm:** `KeltnerParams` có trường `timeframe` nhưng bị `#[dna(skip)]` và bị ghi đè cứng bằng `bbands.timeframe` trong `mod.rs`. Gây khó hiểu khi đọc config. | 🟢 **LOW** | Loại bỏ trường `timeframe` trong `KeltnerParams` hoặc thêm comment giải thích rõ ràng kiến trúc khóa đồng bộ. |

---

## 3. CHI TIẾT ĐÀO SÂU 3 VẤN ĐỀ TRỌNG TÂM

### 3.1. Phân Tích Lỗi Critical Số 1: Flatline Guard Veto Oan (Gatekeeper)
- **Công thức hiện tại trong `gatekeeper.rs` (dòng 115-123):**
  $$\text{micro\_natr\_ratio} = \frac{\text{current\_atr\_sl}}{\text{price}}$$
  $$\text{effective\_min\_natr} = 2 \times \text{taker\_fee\_rate} + \text{slippage} + \text{min\_natr\_threshold}$$
- **Số liệu định lượng thực tế:**
  - `taker_fee_rate` = 0.04% $\Rightarrow 2 \times \text{fee} = 0.08\%$.
  - `slippage` = 0.02%.
  - `min_natr_threshold` (DNA): $0.08\% \to 0.30\%$.
  - $\Rightarrow \text{effective\_min\_natr} = 0.18\% \to 0.40\%$.
  - Trong khi đó, `current_atr_sl` là ATR của nến vi mô 1m/3m/5m (do `ind.atr.timeframe` thuộc `["1m", "3m", "5m"]`).
  - Trên BTC ($70,000), biên độ trung bình nến 1m dao động $20 - $45 $\Rightarrow \text{micro\_natr\_ratio} = \frac{30}{70000} \approx 0.043\%$.
- **Hệ quả kinh tế lượng:** $0.043\% < 0.18\% \Rightarrow$ Gatekeeper kích hoạt `VOLATILITY_FLATLINE` veto gần như **toàn bộ các nến bình thường**. Hệ thống bị tê liệt hoàn toàn trừ những cây nến giật bão cực mạnh.

### 3.2. Phân Tích Lỗi Critical Số 2: TrendPB Dead Zone Do Lệch Scale ATR
- **Công thức trong `trend_pb.rs` (dòng 79-81):**
  $$\text{delta\_kama} = \frac{\text{close}_{1\text{m}} - \text{kama\_mid}}{\text{current\_atr}_{1\text{m}}}$$
  $$\text{Điều kiện Long:} -\text{inval\_floor} \le \text{delta\_kama} \le \text{max\_dist}$$
  Với `max_pullback_dist_atr` có dải DNA từ $0.50 \to 3.50$.
- **Bản chất lệch ngữ nghĩa:**
  - `kama_mid` nằm trên khung 15m, 1h, hoặc 4h.
  - Trên BTC, KAMA 4h phản ánh xu hướng lớn. Khoảng cách bình thường từ giá 1m đến KAMA 4h là $150 - $300.
  - Khi chia cho $\text{ATR}_{1\text{m}} \approx \$30$: $\text{delta\_kama} = \frac{200}{30} = 6.67$.
  - Do $6.67 > 3.50$ (ngưỡng tối đa của `max_dist`), điều kiện lọc bị vi phạm $\Rightarrow$ **Bị loại bỏ 100% khi KAMA chọn khung 1h/4h**.

### 3.3. Phân Tích Lỗi High Số 4: Cảm Biến Mồ Côi `vol_sma`
- `HeliosSensors` định nghĩa `pub vol_sma: Arc<Vec<f64>>`.
- Trong `scanners/reversal.rs` dòng 55:
  ```rust
  let vol_sma = sensors.vol_sma.get(i).copied().unwrap_or(vol_current);
  let vol_ratio = if vol_sma > 0.0 { vol_current / vol_sma } else { 1.0 };
  ```
- Tuy nhiên, trong `core_v3/mod.rs` hàm `init()`, không hề có dòng code nào gọi `ctx.volume_sma(...)`.
- Vì `sensors.vol_sma` rỗng (`len == 0`), `get(i)` luôn trả về `None`, dẫn đến `unwrap_or(vol_current)` luôn được chọn.
- Kết quả: `vol_ratio` luôn bằng $1.0$ cố định trên mọi nến! Chỉ số khối lượng kiệt sức của Reversal Scanner bị vô hiệu hóa hoàn toàn.

---

## 4. KHUYẾN NGHỊ THỨ TỰ SỬA LỖI (ACTION PLAN)

1. **Khắc phục ngay Lỗi 1 (Flatline Guard):** Đưa `macro_natr` vào làm chuẩn đo biến động thực tế của tài sản hoặc quy đổi thang đo tương thích.
2. **Khắc phục ngay Lỗi 2 (TrendPB delta_kama):** Chuẩn hóa khoảng cách KAMA theo ATR của khung KAMA.
3. **Khởi tạo `sensors.vol_sma` trong `mod.rs` (Lỗi 4):** Thêm 1 dòng gọi `ctx.volume_sma`.
4. **Đồng bộ hóa Timeframe VWMA theo BBands (Lỗi 6):** Khóa `vwma.timeframe = bbands.timeframe` trong `init()`.
5. **Nâng ngưỡng `max_boundary_z` trong `TrendPB` (Lỗi 7):** Mở rộng dải DNA từ `[0.2, 0.8]` lên `[0.8, 2.5]`.
