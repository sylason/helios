# 📐 ĐẶC TẢ KỸ THUẬT: ĐẠI TU TOÀN DIỆN CHIẾN ĐỘI TREND PULLBACK (`trend_pb.rs`)

---

## 🎯 1. Mục Tiêu Kỹ Thuật
1. **Loại bỏ 100% logic phi tuyến Over-engineered & Magic Numbers:** Xóa bỏ hoàn toàn hàm mũ sigmoid `exp(-0.1 * ...)` và 3 trọng số cứng `40% - 40% - 20%` trong `trend_pb.rs`.
2. **Thiết lập 3 Trụ Cột Toán Học Định Lượng (100% Khách quan):**
   - **Khoảng cách tương đối KAMA ($\delta_{\text{KAMA}}$):** Định nghĩa không gian hồi quy chuẩn hóa qua ATR.
   - **Vùng hạ nhiệt xung lực (Stochastic & RSI Cool-down):** Đo lường nhịp hạ nhiệt lành mạnh, không ép RSI sập sâu.
   - **Đạo hàm vận tốc giá hướng lên (CLV & Gradient):** Xác nhận lực cầu kiểm soát nến thoát khỏi nhịp hồi.
3. **Tháo gỡ tử huyệt "Câm Lặng 0 Lệnh" trong `matrix.rs`:** Khắc phục tình trạng ADX hạ nhiệt trong nhịp hồi dìm chết $w_{\text{trend}}$ và Veto nhầm `TrendPB`.
4. **DNA hóa 100%:** Nâng `HeliosConfigV3::DNA_WIDTH` từ 121 lên **123 chiều**.

---

## 🔬 2. Công Thức Toán Học Chi Tiết

### A. Trụ Cột 1: Không Gian Hồi Quy KAMA Chuẩn Hóa
$$\delta_{\text{KAMA}}[i] = \frac{\text{Close}[i] - \text{KAMA}[i]}{\text{ATR}[i]}$$

*   **Điều Kiện Long (Xu hướng Tăng `kama_slope_adj > kama_slope_threshold`):**
    $$-\text{invalidation\_buffer\_atr} \le \delta_{\text{KAMA}}[i] \le \text{max\_pullback\_dist\_atr}$$
    - Nếu $\delta_{\text{KAMA}}[i] < -\text{invalidation\_buffer\_atr}$: Nến đã đóng cửa gãy thủng KAMA $\implies$ Điểm $= 0.0$ (chống bắt dao rơi).
    - Nếu $\delta_{\text{KAMA}}[i] > \text{max\_pullback\_dist\_atr}$: Giá bay quá xa KAMA $\implies$ Điểm $= 0.0$ (chống mua đu đỉnh).
    - Hàm điểm không gian KAMA:
      $$f_{\text{dist\_long}} = 1.0 - \frac{|\delta_{\text{KAMA}}[i]|}{\text{max\_pullback\_dist\_atr}} \in [0.0, 1.0]$$

*   **Điều Kiện Short (Xu hướng Giảm `kama_slope_adj < -kama_slope_threshold`):**
    $$-\text{max\_pullback\_dist\_atr} \le \delta_{\text{KAMA}}[i] \le \text{invalidation\_buffer\_atr}$$
    - Nếu $\delta_{\text{KAMA}}[i] > \text{invalidation\_buffer\_atr}$: Nến đóng cửa vượt thủng kháng cự KAMA $\implies$ Điểm $= 0.0$ (chống cản tàu).
    - Nếu $\delta_{\text{KAMA}}[i] < -\text{max\_pullback\_dist\_atr}$: Giá rơi quá xa KAMA $\implies$ Điểm $= 0.0$ (chống bán đuổi đáy).

---

### B. Trụ Cột 2: Vùng Hạ Nhiệt Định Lượng (Cool-Down Zone)
*   **Stochastic K hạ nhiệt:**
    - Long: $\text{Stoch\_K}[i] \le \text{stoch\_pullback\_thresh}$ (mặc định $30.0 - 45.0$).
      $$f_{\text{stoch\_long}} = 1.0 - \frac{\text{Stoch\_K}[i]}{\text{stoch\_pullback\_thresh}} \in [0.0, 1.0]$$
    - Short: $\text{Stoch\_K}[i] \ge (100.0 - \text{stoch\_pullback\_thresh})$.
      $$f_{\text{stoch\_short}} = \frac{\text{Stoch\_K}[i] - (100.0 - \text{stoch\_pullback\_thresh})}{\text{stoch\_pullback\_thresh}} \in [0.0, 1.0]$$
*   **RSI Hưng Phấn Cap (Trần An Toàn):**
    - Long: $\text{RSI}[i] \le \text{rsi\_max\_cap}$ (mặc định $68.0$, bảo đảm không mua tại đỉnh hưng phấn).
    - Short: $\text{RSI}[i] \ge (100.0 - \text{rsi\_max\_cap})$ (mặc định $32.0$).

---

### C. Trụ Cột 3: Đạo Hàm Vận Tốc Giá Đổi Dấu (CLV & Directional Gradient)
$$\text{CLV}[i] = \frac{2 \times \text{Close}[i] - (\text{High}[i] + \text{Low}[i])}{\max(\text{High}[i] - \text{Low}[i], 10^{-6})} \in [-1.0, +1.0]$$

*   **Tiêu chuẩn phê chuẩn Gradient Long:**
    - $\text{CLV}[i] \ge 0.15$ (giá đóng cửa nằm ở phần trên của cây nến).
    - $\text{Close}[i] \ge \text{Open}[i]$ (nến xanh hoặc doji rút chân, tuyệt đối cấm nến đỏ Marubozu).
*   **Tiêu chuẩn phê chuẩn Gradient Short:**
    - $\text{CLV}[i] \le -0.15$ (giá đóng cửa nằm ở phần dưới của cây nến).
    - $\text{Close}[i] \le \text{Open}[i]$ (nến đỏ hoặc doji râu trên, tuyệt đối cấm nến xanh bốc đầu).

---

### D. Tổng Hợp Điểm Số Mềm (Soft Confidence Score)
$$\text{raw\_score} = \frac{f_{\text{slope}} + f_{\text{dist}} + f_{\text{stoch}}}{3.0}$$
*   Trong đó $f_{\text{slope}} = \tanh(|\text{kama\_slope\_adj}[i]| / \text{kama\_slope\_threshold})$.
*   Nếu không thỏa mãn bất kỳ chốt chặn nào (CLV sai hướng, $\delta_{\text{KAMA}}$ ngoài hộp, RSI quá hưng phấn) $\implies \mathbf{raw\_score \equiv 0.0}$.

---

### E. Tháo Gỡ Tử Huyệt ADX Trong `matrix.rs`
Trong `matrix.rs` (dòng 128-129, 163-167):
```rust
// TÍNH TOÁN TREND MULTIPLIER KẾT HỢP ĐỘ DỐC KAMA (Chống ADX hạ nhiệt trong nhịp hồi)
let t_adx = trend_strength[i].max(0.0).min(1.0);
let slope_strength = (sensors.kama_slope_adj[i].abs() / 0.08).tanh().max(0.0).min(1.0);
let t_mul = t_adx.max(slope_strength);
trend_multiplier.push(t_mul);
```

---

## 🧬 3. Cấu Hình DNA Dynamic 1% -> 99% (`config.rs`)

Struct `TacticalPullbackParams` được chuẩn hóa:

```rust
pub struct TacticalPullbackParams {
    /// 1. Sàn Stop Loss ROE khởi tạo tối đa (4% -> 40%, step = 1.5%)
    #[dna(min = 0.04, max = 0.40, step = 0.015)]
    pub max_initial_sl_pct: f64,

    /// 2. Độ dốc tối thiểu của KAMA (0.01 -> 0.40, step = 0.02)
    #[dna(min = 0.01, max = 0.40, step = 0.02)]
    pub kama_slope_threshold: f64,

    /// 3. Ngưỡng Stochastic hồi quy (10.0 -> 50.0, step = 2.0)
    #[dna(min = 10.0, max = 50.0, step = 2.0)]
    pub stoch_pullback_thresh: f64,

    /// 4. Trần RSI an toàn tránh hưng phấn đỉnh sóng (50.0 -> 80.0, step = 2.0)
    #[dna(min = 50.0, max = 80.0, step = 2.0)]
    pub rsi_max_cap: f64,

    /// 5. [GENE MỚI 1] Khoảng cách tối đa cho phép từ KAMA qua ATR (0.5 -> 3.5 ATR, step = 0.20)
    #[dna(min = 0.50, max = 3.50, step = 0.20)]
    pub max_pullback_dist_atr: f64,

    /// 6. [GENE MỚI 2] Ngưỡng cắt đứt hỗ trợ gãy trend KAMA (0.10 -> 1.50 ATR, step = 0.10)
    #[dna(min = 0.10, max = 1.50, step = 0.10)]
    pub invalidation_buffer_atr: f64,

    /// 7. Ngưỡng kích hoạt mềm tối thiểu (0.20 -> 0.85, step = 0.05)
    #[dna(min = 0.20, max = 0.85, step = 0.05)]
    pub min_activation_score: f64,

    /// 8. Đệm ATR bảo vệ Stop Loss cấu trúc (0.10 -> 3.00 ATR, step = 0.15)
    #[dna(min = 0.10, max = 3.00, step = 0.15)]
    pub struct_sl_buffer_atr: f64,
}
```

---

## 🧪 4. Danh Sách Unit Tests Kiểm Chứng (Test Matrix)

1. `test_trend_pb_healthy_pullback_approved`: Nến hồi về gần KAMA trong trend dốc lên + Stoch hạ nhiệt + CLV dương $\implies$ Proposal Long được phê duyệt.
2. `test_trend_pb_breakdown_invalidation_blocked`: Giá sập gãy thủng KAMA $> \text{invalidation\_buffer}$ $\implies$ Điểm $= 0.0$, triệt tiêu bắt dao rơi.
3. `test_trend_pb_parabolic_chasing_blocked`: Giá bay xa KAMA $> \text{max\_dist}$ $\implies$ Điểm $= 0.0$, triệt tiêu mua đu đỉnh.
4. `test_trend_pb_marubozu_knife_blocked`: Nến đỏ đặc cắm đầu rơi ($\text{CLV} < 0.0$) $\implies$ Điểm $= 0.0$, không chặn đầu xe tải.
5. `test_trend_pb_short_healthy_pullback_approved`: Kiểm chứng đối xứng cho chiều Short khi KAMA dốc xuống.
6. `test_trend_pb_dna_width_123_integrity`: Đảm bảo `HeliosConfigV3::DNA_WIDTH == 123`.
