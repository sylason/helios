# 🗺️ MA TRẬN RANH GIỚI TÁC CHIẾN 6 PLAYBOOKS & QUẢN TRỊ TƯƠNG TÁC CHÉO

> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_reg10_range_osc_band_walking_trap/PLAYBOOK_BOUNDARY_MATRIX.md`  
> **Mục tiêu cốt lõi:** Định vị chính xác ranh giới hoạt động của từng Playbook, thiết lập hợp đồng giao diện giữa các tầng `Sensors` -> `Matrix` -> `Scanners` -> `Arbiter`, loại bỏ hoàn toàn hiện tượng cướp thầu và ngáng chân nhau (Cannibalization & Preemption).

---

## 1. 🏛️ Kiến Trúc Luồng Dữ Liệu Một Chiều (Unidirectional Dataflow)

Toàn bộ hệ sinh thái Helios V3 vận hành theo luồng xử lý nghiêm ngặt:

$$\mathbf{Sensors} \xrightarrow{\text{Dữ liệu thô}} \mathbf{Matrix} \xrightarrow{\text{Bối cảnh vĩ mô}} \mathbf{Scanners} \xrightarrow{\text{Đề xuất vi mô}} \mathbf{Arbiter} \xrightarrow{\text{Quyết định cuối}} \mathbf{Risk\ Manager}$$

* **Sensors (`sensors.rs`):** Cảm biến đo đạc toán học cấp cơ sở từ chuỗi nến OHLCV (Bollinger Bands, KAMA, Stochastic, ATR, MFI, Z-Score). Tuyệt đối không chứa logic mua/bán.
* **Matrix (`matrix.rs`):** Đánh giá trạng thái vĩ mô qua 4-State Continuous Simplex Vector ($w_{\text{trend}} + w_{\text{range}} + w_{\text{comp}} + w_{\text{chaos}} \equiv 1.0000$) và các Multipliers.
* **Tactical Scanners (`scanners/*.rs`):** 6 chiến đội trinh sát vi mô độc lập, chỉ quét cơ hội khi thị trường rơi đúng vào bản sắc của mình.
* **Signal Arbiter (`arbiter.rs`):** Hội đồng trọng tài tổ chức đấu thầu sòng phẳng dựa trên điểm tự tin hình học có trọng số (Weighted Geometric Mean).
* **Risk Manager (`risk.rs`):** Quản trị vị thế sau khi vào lệnh (Structural SL, Dynamic TP, Trailing Stop, Stagnant Decay).

---

## 2. 📊 BẢNG MA TRẬN RANH GIỚI TÁC CHIẾN 6 PLAYBOOKS

| STT | Playbook / Scanner | Trạng Thái Thị Trường Mục Tiêu | Sensors Cốt Lõi Yêu Cầu | Trọng Số Simplex & Multiplier | Ranh Giới Dẫm Chân & Xung Đột | Cơ Chế Phân Làn & Veto An Toàn |
| :---: | :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | **`TrendPullback`** (`trend_pb.rs`) | Sóng có xu hướng định hướng rõ ràng, dốc mạnh ($ADX \ge 25$). | KAMA Mid, KAMA Slope, Stoch Trend %K, RSI Trend. | $w_{\text{trend\_norm}}$, `trend_multiplier`. | Bị `RangeOsc` ngáng chân cướp lệnh khi thị trường trôi dốc chậm (Grind-down). | Cấm `RangeOsc` khi trục giữa Bollinger Bands có độ dốc. |
| **2** | **`MomentumIgnition`** (`momentum_ignition.rs`) | Nến xung lực cực mạnh, bứt phá dải biên Bollinger / Keltner. | Volume Force, BB Upper/Lower, RSI Momentum. | $w_{\text{trend\_norm}}$, `volatility_expansion_multiplier`. | Bị `RangeOsc` nhảy vào chặn đầu (Fade) ngay tại cây nến bứt phá. | Cấm `RangeOsc` khi nến đâm thủng hoặc đóng ngoài dải BB. |
| **3** | **`AbsorptionScout`** (`absorption.rs`) | Nén tích lũy co thắt (Squeeze), Smart Money gom hàng trước khi nổ vol. | BB Squeeze, Volume Z-Score, VWMA, BB Middle. | $w_{\text{comp\_norm}}$, `absorption_multiplier`. | Tranh chấp với `RangeOsc` tại vùng biên độ nén trung gian ($1.5\% - 2.5\%$). | Phân làn: $\text{Squeeze} < 1.0$ là của `Absorption`; $\text{Squeeze} \ge 1.0$ mới cho `RangeOsc`. |
| **4** | **`RangeOscillator`** (`range_osc.rs`) 🎯 | **Kênh đi ngang phẳng (Flat Horizontal Range), dao động biên êm đềm.** | BB Middle Slope, Rejection Wick, Stoch Range %K/%D. | $w_{\text{range\_norm}}$, `range_multiplier`. | **Thủ phạm chính:** Nhầm sóng trôi dốc thành Range, bắt dao rơi Long liên tục. | **Khóa chặt:** Bắt buộc trục giữa BB Middle phải phẳng ($|\text{Slope}| \le \text{Threshold}$). |
| **5** | **`ClimaxReversal`** (`reversal.rs`) | Sóng tăng/giảm kiệt sức Parabol cực hạn ($Z_{\text{price}} > 2.5\sigma$). | Price Z-Score, ATR Displacement, Volume Exhaustion. | $w_{\text{chaos\_norm}}$, `reversal_multiplier`. | Chồng chéo với `RangeOsc` khi giá chạm mép ngoài dải Bollinger Bands ($2.0\sigma$). | `Reversal` chỉ đánh khi biến động cực đại ($Z > 2.5\sigma$); `RangeOsc` chỉ đánh khi $Z$ bình thường. |
| **6** | **`AnomalyFade`** (`anomaly.rs`) | Râu nến quét thanh khoản dị thường (Pinbar / Fakeout). | Upper/Lower Wick Ratio, Body Ratio, Volume Z-Score. | $w_{\text{chaos\_norm}}$, `anomaly_multiplier`. | Có thể dẫm chân với `Reversal` nếu nến vừa có râu vừa có Z-Score cao. | Ưu tiên `Anomaly` khi râu nến chiếm $\ge 70\%$ tổng chiều dài nến. |

---

## 3. 🛡️ NGUYÊN TẮC BẢO TỒN HẠ TẦNG (NON-REGRESSION PRINCIPLES)

Khi nâng cấp `sensors.rs` và `matrix.rs`, ta áp dụng tuyệt đối 3 nguyên tắc bảo vệ:

1. **Nguyên tắc Chỉ Thêm Không Sửa (Additive-Only):**
   * Giữ nguyên 100% các công thức và mảng số hiện tại: `kama_slope_adj`, `squeeze_intensity`, `vol_z_score`, `chaos_ratio`.
   * Chỉ thêm mới các trường cảm biến chuyên dụng (ví dụ: `bb_middle_slope_adj` để đo độ phẳng trục giữa BB).
2. **Bảo tồn Bất biến Simplex (Simplex Invariant):**
   * Luôn đảm bảo: $w_{\text{trend}} + w_{\text{range}} + w_{\text{comp}} + w_{\text{chaos}} \equiv 1.0000$.
3. **Phân Định Trách Nhiệm Vĩ Mô vs Vi Mô:**
   * `Matrix` chỉ chịu trách nhiệm đo: *Thị trường có đang ở trạng thái phẳng hay không?*
   * `Scanner` chịu trách nhiệm đo: *Tại điểm chạm dải, giá có phản ứng bật ngược hay đang đâm thủng?*
