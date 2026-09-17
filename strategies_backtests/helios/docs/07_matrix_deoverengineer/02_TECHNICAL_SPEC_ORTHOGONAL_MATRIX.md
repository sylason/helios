# 📐 ĐẶC TẢ KỸ THUẬT: MA TRẬN TRỰC GIAO ĐỘC LẬP & ARBITER TINH GIẢN (KISS & CLEAN)

---

## 🏛️ 1. TRIẾT LÝ THIẾT KẾ CỐT LÕI
1. **Trực Giao Độc Lập (Orthogonality):** 
   - `MarketMatrixV3` không đóng vai trò "người phán xử", mà chỉ là **Thước Đo Khách Quan Cung Cấp Số Liệu Vật Lý Phi Thứ Nguyên (Dimensionless Metrics)**.
   - Các trục đo lường (Xu hướng, Nén dải, Khối lượng, Hỗn loạn) tồn tại song song độc lập. Tuyệt đối **không ép chúng vào trò chơi Zero-Sum Simplex** ($\sum \equiv 1.0$) khiến trạng thái này bóp nghẹt trạng thái khác.
2. **Phi Thứ Nguyên Hóa 100% (Universal Dimensionless Scaling):**
   - Mọi biến động giá và độ dốc (Slope) đều được chia chuẩn hóa theo **ATR** hiện tại, loại bỏ hoàn toàn các hằng số chia tuyệt đối (`0.08`, `0.50`), hoạt động chính xác 1:1 trên mọi loại tài sản từ \$0.00001 đến \$100,000.
3. **Phân Định Trách Nhiệm Rõ Ràng (Separation of Concerns):**
   - **Scanner:** Tự kiểm tra điều kiện hình học và tự Veto rủi ro nội tại (Local Veto).
   - **Arbiter:** Đấu thầu tuyến tính, công bằng, chọn Proposal có tỷ lệ R:R và điểm tự tin thực chất cao nhất.

---

## 📐 2. CÔNG THỨC TOÁN HỌC 4 TRỤC MA TRẬN MỚI

### Trục 1: Xu Hướng & Độ Dốc Chuẩn Hóa Theo ATR (Trend Direction & Strength)
- **Độ dốc KAMA phi thứ nguyên (KAMA Slope per ATR):**
  $$\delta_{\text{KAMA}}[i] = \frac{\text{kama\_mid}[i] - \text{kama\_mid}[i-1]}{\text{ATR}[i]}$$
- **Hướng xu hướng chuẩn hóa $[-1.0, 1.0]$:**
  $$\text{trend\_dir}[i] = \tanh\left(\frac{\delta_{\text{KAMA}}[i]}{\text{scale\_atr}}\right) \in [-1.0, 1.0]$$
  *(Với $\text{scale\_atr}$ là gen DNA `kama_slope_scale_atr`)*.
- **Độ mạnh xu hướng $[0.0, 1.0]$:**
  $$\text{trend\_strength}[i] = \tanh\left(\frac{\text{ADX}[i]}{\text{adx\_thresh}}\right) \in [0.0, 1.0]$$
  *(Với $\text{adx\_thresh}$ là gen DNA `adx_trend_threshold`)*.

### Trục 2: Độ Nén Dải Biến Động (Volatility Squeeze Intensity)
- Giữ nguyên công thức hình học chuẩn của Bollinger Bands nằm trong Keltner Channel:
  $$\text{overlap} = (KC_u - BB_u) + (BB_l - KC_l)$$
  $$\text{sqz\_raw} = \text{clamp}\left(\frac{\text{overlap}}{KC_u - KC_l}, 0.0, 1.0\right)$$
  $$\text{squeeze\_intensity} = \text{RollingPercentile}(\text{sqz\_raw}, \text{lookback})$$

### Trục 3: Xung Lực Thanh Khoản (Volume Force)
- Không cộng gộp MFI và Volume Z-Score theo tỷ lệ 50/50 cảm tính.
- Chuẩn hóa Volume Z-Score độc lập qua hàm Tanh co giãn:
  $$\text{volume\_force}[i] = \tanh\left(\frac{|\text{vol\_z\_score}[i]|}{\text{vol\_z\_sens}}\right) \in [0.0, 1.0]$$
  *(MFI được giữ nguyên ở `sensors.mfi` cho các Scanner cần dòng tiền tham chiếu riêng)*.

### Trục 4: Độ Hỗn Loạn / Bão Whipsaw (Chaos Ratio)
- Giữ nguyên tỷ lệ biến động tương đối:
  $$\text{chaos\_ratio}[i] = \text{RollingPercentile}\left(\frac{\text{NATR}[i]}{\text{SMA}(\text{NATR})[i]}, \text{lookback}\right) \in [0.0, 1.0]$$

---

## ⚖️ 3. ARBITER TINH GIẢN (KISS CONFIDENCE AUCTION)

### Bỏ cơ chế bọc phi tuyến 3 tầng:
$$\text{XÓA BỎ: } \text{confidence} = (\text{raw})^w \times \left(\sqrt{w_{\text{regime}} \times \text{multiplier}}\right)^{1-w}$$

### Thay thế bằng Đấu Thầu Tuyến Tính Rõ Ràng ($O(1)$):
1. **Thu thập Proposal từ các Scanner còn active:**
   - `TREND_PB`
   - `ABSORPTION`
   - `REVERSAL`
   - `MOMENTUM_IGNITION`
   *(Giữ nguyên code `range_osc` và `anomaly` nhưng không để chúng cướp thầu)*.
2. **Chấm điểm Tự tin Độc lập:**
   $$\text{confidence} = \text{proposal.raw\_score} \times (1.0 - \text{chaos\_penalty})$$
   - Trong đó $\text{chaos\_penalty} = \text{chaos\_ratio}[i] \times \text{config.chaos\_weight}$ (Thị trường càng hỗn loạn Whipsaw thì điểm tự tin càng bị trừ lùi).
3. **Phê duyệt & Giải quyết Xung đột:**
   - Điều kiện tối thiểu: $\text{confidence} \ge \text{config.min\_confidence}$.
   - Nếu có nhiều Scanner cùng phát tín hiệu đồng thuận (ví dụ cùng Long): Chọn lệnh có **Khoảng cách Stop Loss gần hơn** để tối ưu tỷ lệ $R:R$.

---

## 🧬 4. CẤU HÌNH DNA TRONG `config.rs`

### Gen Mới Bổ Sung Vào `MatrixConfig`:
- `adx_trend_threshold: f64`: Dải $[15.0, 35.0]$, Step $1.0$, Default $25.0$.
- `kama_slope_scale_atr: f64`: Dải $[0.05, 0.50]$, Step $0.05$, Default $0.20$.
- `vol_z_sensitivity: f64`: Dải $[0.50, 2.50]$, Step $0.10$, Default $1.00$.

### Gen Cũ Cần Comment Out Trong `ArbiterConfig`:
- `// pub min_confidence_trending: f64,`
- `// pub min_confidence_ranging: f64,`
- `// pub min_confidence_compression: f64,`
- `// pub min_confidence_chaos: f64,`
- `// pub min_multiplier_veto: f64,`
$\implies$ **Tổng số gen DNA giảm từ 123 xuống 121 chiều**, không gian tìm kiếm gọn gàng hơn.

---

## 🧪 5. DANH MỤC UNIT TESTS BẮT BUỘC

1. `test_kama_slope_atr_scale_asset_agnostic`: Kiểm chứng độ dốc KAMA chia theo ATR cho kết quả tương đương giữa ETH (\$2500) và DOGE (\$0.10).
2. `test_matrix_independence_no_zero_sum`: Kiểm chứng khi Trend tăng mạnh, điểm đo Squeeze và Volume Force không bị suy giảm vô cớ.
3. `test_arbiter_fair_auction_unblocked_reversal`: Kiểm chứng khi thị trường có sóng mạnh, tín hiệu Reversal ở pha kiệt sức không bị Veto oan ức.
4. `test_chaos_penalty_scales_down_confidence`: Kiểm chứng khi bão Whipsaw bùng nổ, điểm tự tin của mọi Scanner đều bị giảm trừ để phòng thủ.
