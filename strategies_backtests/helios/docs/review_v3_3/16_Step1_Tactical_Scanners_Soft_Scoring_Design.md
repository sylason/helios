# HELIOS V3.3 ARCHITECTURE REVIEW: PART 16 - STEP 1 TACTICAL SCANNERS SOFT SCORING & DNA GENE DESIGN

Tài liệu này ghi nhận chi tiết toàn bộ thiết kế tái cấu trúc cho **Bước 1: 6 Tactical Scanners** trong Helios V3.3. Thiết kế này chuyển hóa $100\%$ các cổng lọc nhị phân cứng (`if A && B && C`) sang **Hàm Kích Hoạt Mềm Tích Lũy Continuous (Gaussian / Sigmoid / Tanh Soft Scoring)**, đồng thời mở rộng toàn bộ ngưỡng kỹ thuật thành **Dải Tham Số Gene DNA** để thuật toán Di truyền (Genetic Algorithm Optimizer) tự do tối ưu hóa.

---

## 🏛️ 1. NGUYÊN TẮC THIẾT KẾ BƯỚC 1

1. **Triệt tiêu 100% Magic Numbers (Không hardcode bất kỳ ngưỡng nào)**: Mọi tham số ngưỡng, độ rộng dải dung sai, hệ số dốc, và điểm kích hoạt tối thiểu đều được khai báo có macro `#[dna(...)]` trong `TacticalSniperConfig` (`src/strategy/core_v3/config.rs`).
2. **Không gian Điểm Liên Tục ($P_j \in [0.0, 1.0]$)**: Thay vì ngắt lệnh `return None` khi một tiêu chí phụ bị lệch nhẹ, mỗi Scanner tính điểm hoàn hảo của mẫu hình (Pattern Congruence Score) thông qua hàm Gaussian Kernel và hàm Sigmoid mượt mà.
3. **Phát Đề Xuất Độc Lập (`Proposal`)**: Mỗi Scanner khi đạt điểm $P_j \ge \text{min\_activation\_score}$ sẽ độc lập gửi `Proposal` về cho Hội đồng Trọng tài `HeliosArbiter`.

---

## 🎯 2. THIẾT KẾ CHI TIẾT 6 TACTICAL SCANNERS

### 2.1. 🛡️ `RANGE_OSC` (Range Oscillator Scanner - Đánh Biên Hộp Sideway)

- **Nguyên lý**: Đánh bật lại dạng bóng bàn (Ping-Pong) khi giá tiếp cận mép dải Bollinger Bands trong thị trường đi ngang.
- **Công thức Chấm Điểm Mềm ($P_{\text{range}}$)**:
  1. *Độ tiệm cận dải biên BB ($f_{\text{price}}$)*:
     $$f_{\text{price}} = \exp\left( -\frac{(\text{Close} - \text{BB}_{\text{edge}})^2}{2 \cdot (\text{bb\_proximity\_pct} \cdot \text{Close})^2} \right)$$
     *(Với $\text{BB}_{\text{edge}} = \text{BB}_{\text{Lower}}$ cho Long; $\text{BB}_{\text{edge}} = \text{BB}_{\text{Upper}}$ cho Short)*.
  2. *Mức độ Quá Bán / Quá Mua Stochastic ($f_{\text{stoch}}$)*:
     - Cho Long: $f_{\text{stoch}} = \frac{1}{1 + e^{-0.1 \cdot (\text{stoch\_oversold} - \text{stoch\_k})}}$
     - Cho Short: $f_{\text{stoch}} = \frac{1}{1 + e^{-0.1 \cdot (\text{stoch\_k} - \text{stoch\_overbought})}}$
  3. *Độ Móc Động Lượng Stochastic ($f_{\text{hook}}$)*:
     - Cho Long: $f_{\text{hook}} = \frac{1}{1 + e^{-0.5 \cdot (\text{stoch\_k} - \text{stoch\_d})}}$
     - Cho Short: $f_{\text{hook}} = \frac{1}{1 + e^{-0.5 \cdot (\text{stoch\_d} - \text{stoch\_k})}}$
  4. *Điểm Tổng Hợp*:
     $$P_{\text{range}} = 0.50 \cdot f_{\text{price}} + 0.30 \cdot f_{\text{stoch}} + 0.20 \cdot f_{\text{hook}}$$
  5. *Take Profit*: Đặt Take Profit tĩnh cố định tại dải Band đối diện ($\text{TP} = \text{BB}_{\text{Upper}}$ cho Long, $\text{TP} = \text{BB}_{\text{Lower}}$ cho Short).
- **Dải Gene DNA (`TacticalRangeParams`)**:
  - `bb_proximity_pct`: `#[dna(min = 0.001, max = 0.015, step = 0.001)]`
  - `stoch_oversold`: `#[dna(min = 10.0, max = 40.0, step = 2.0)]`
  - `stoch_overbought`: `#[dna(min = 60.0, max = 90.0, step = 2.0)]`
  - `min_activation_score`: `#[dna(min = 0.25, max = 0.75, step = 0.05)]`
  - `min_width_pct`: `#[dna(min = 0.005, max = 0.050, step = 0.002)]`
  - `volume_max_threshold`: `#[dna(min = 0.10, max = 0.90, step = 0.05)]`

---

### 2.2. 🏄 `TREND_PB` (Trend Pullback Scanner - Bắt Sóng Hồi Trong Xu Hướng)

- **Nguyên lý**: Đánh bám sóng (Trend Following) ở các nhịp hồi kỹ thuật lành mạnh về sát trục KAMA.
- **Công thức Chấm Điểm Mềm ($P_{\text{trend}}$)**:
  1. *Độ dốc xu hướng KAMA Slope ($f_{\text{slope}}$)*:
     $$f_{\text{slope}} = \tanh\left(\frac{|\text{kama\_slope\_adj}|}{\text{kama\_slope\_threshold}}\right)$$
  2. *Độ sâu nhịp hồi Stochastic ($f_{\text{stoch}}$)*:
     - Cho Long: $f_{\text{stoch}} = \frac{1}{1 + e^{-0.1 \cdot (\text{stoch\_pullback\_thresh} - \text{stoch\_k})}}$
     - Cho Short: $f_{\text{stoch}} = \frac{1}{1 + e^{-0.1 \cdot (\text{stoch\_k} - (100.0 - \text{stoch\_pullback\_thresh}))}}$
  3. *RSI an toàn ($f_{\text{rsi}}$)*:
     - Cho Long: $f_{\text{rsi}} = \frac{1}{1 + e^{-0.1 \cdot (\text{rsi\_buy\_thresh} - \text{rsi})}}$
     - Cho Short: $f_{\text{rsi}} = \frac{1}{1 + e^{-0.1 \cdot (\text{rsi} - (100.0 - \text{rsi\_buy\_thresh}))}}$
  4. *Điểm Tổng Hợp*:
     $$P_{\text{trend}} = 0.40 \cdot f_{\text{slope}} + 0.40 \cdot f_{\text{stoch}} + 0.20 \cdot f_{\text{rsi}}$$
- **Dải Gene DNA (`TacticalPullbackParams`)**:
  - `kama_slope_threshold`: `#[dna(min = 0.02, max = 0.30, step = 0.02)]`
  - `stoch_pullback_thresh`: `#[dna(min = 20.0, max = 55.0, step = 3.0)]`
  - `rsi_buy_thresh`: `#[dna(min = 40.0, max = 65.0, step = 2.0)]`
  - `min_activation_score`: `#[dna(min = 0.25, max = 0.75, step = 0.05)]`

---

### 2.3. 🚀 `MOMENTUM_IGN` (Momentum Ignition Scanner - Bắt Xung Lực Bùng Nổ)

- **Nguyên lý**: Đuổi theo các con sóng đẩy bùng nổ vượt ra ngoài dải Bollinger Bands kèm theo sự bứt phá của RSI và dòng tiền.
- **Công thức Chấm Điểm Mềm ($P_{\text{mom}}$)**:
  1. *Độ bục dải Bollinger Bands ($f_{\text{break}}$)*:
     - Cho Long: $f_{\text{break}} = \frac{1}{1 + e^{-2.0 \cdot \frac{\text{Close} - \text{BB}_{\text{Upper}}}{\text{ATR}}}}$
     - Cho Short: $f_{\text{break}} = \frac{1}{1 + e^{-2.0 \cdot \frac{\text{BB}_{\text{Lower}} - \text{Close}}{\text{ATR}}}}$
  2. *RSI Xung Lực Bùng Nổ ($f_{\text{rsi}}$)*:
     - Cho Long: $f_{\text{rsi}} = \frac{1}{1 + e^{-0.15 \cdot (\text{rsi\_mom} - \text{rsi\_threshold})}}$
     - Cho Short: $f_{\text{rsi}} = \frac{1}{1 + e^{-0.15 \cdot ((100.0 - \text{rsi\_threshold}) - \text{rsi\_mom})}}$
  3. *Lực Đẩy Khối Lượng ($f_{\text{vol}}$)*:
     $$f_{\text{vol}} = \tanh\left(\frac{\text{volume\_force}}{\text{vol\_z\_threshold}}\right)$$
  4. *Điểm Tổng Hảo*:
     $$P_{\text{mom}} = 0.40 \cdot f_{\text{break}} + 0.35 \cdot f_{\text{rsi}} + 0.25 \cdot f_{\text{vol}}$$
- **Dải Gene DNA (`TacticalMomentumParams`)**:
  - `rsi_threshold`: `#[dna(min = 55.0, max = 80.0, step = 2.0)]`
  - `vol_z_threshold`: `#[dna(min = 0.20, max = 0.80, step = 0.05)]`
  - `min_activation_score`: `#[dna(min = 0.25, max = 0.75, step = 0.05)]`

---

### 2.4. 🧲 `REVERSAL` (Climax Reversal Scanner - Bắt Đỉnh / Đáy Kiệt Sức)

- **Nguyên lý**: Bắt cú đảo chiều (Mean Reversion) khi giá bị kéo dãn quá đà (Parabolic Overextension) kết hợp với hiện tượng cạn kiệt khối lượng.
- **Công thức Chấm Điểm Mềm ($P_{\text{rev}}$)**:
  1. *Độ Lệch Chuẩn Z-Score ($f_{\text{z}}$)*:
     $$f_{\text{z}} = \tanh\left(\frac{|\text{Price\_Z\_Score}|}{\text{z\_score\_threshold}}\right)$$
  2. *Khoảng Cách KAMA Displacement ($f_{\text{disp}}$)*:
     $$f_{\text{disp}} = \tanh\left(\frac{|\text{Close} - \text{KAMA}_{\text{Mid}}|}{\text{min\_reversal\_displacement} \cdot \text{ATR}}\right)$$
  3. *Cạn Kiệt Khối Lượng Đẩy ($f_{\text{vol}}$)*:
     $$f_{\text{vol}} = 1.0 - \tanh\left(\frac{\text{volume\_force}}{\text{exhaustion\_vol\_drop}}\right)$$
  4. *Điểm Tổng Hợp*:
     $$P_{\text{rev}} = 0.40 \cdot f_{\text{z}} + 0.40 \cdot f_{\text{disp}} + 0.20 \cdot f_{\text{vol}}$$
  5. *Take Profit*: Đặt chốt lời tĩnh cố định ngay tại trục trung vị $\text{TP} = \text{KAMA}_{\text{Mid}}$.
- **Dải Gene DNA (`TacticalReversalParams`)**:
  - `z_score_threshold`: `#[dna(min = 1.2, max = 3.5, step = 0.1)]`
  - `min_reversal_displacement`: `#[dna(min = 0.8, max = 3.0, step = 0.1)]`
  - `exhaustion_vol_drop`: `#[dna(min = 0.10, max = 0.60, step = 0.05)]`
  - `wick_sl_buffer_atr`: `#[dna(min = 0.10, max = 2.00, step = 0.10)]`
  - `min_activation_score`: `#[dna(min = 0.25, max = 0.75, step = 0.05)]`

---

### 2.5. ⚡ `ANOMALY` (Anomaly Fade Scanner - Bắt Cú Hồi Sau Flash Crash)

- **Nguyên lý**: Bắt cú bật hồi kỹ thuật ngay sau sự kiện quét thanh khoản (Liquidity Sweep / Pinbar Rejection).
- **Công thức Chấm Điểm Mềm ($P_{\text{ano}}$)**:
  1. *Tỷ lệ râu nến từ chối giá ($f_{\text{wick}}$)*:
     - Cho Long: $f_{\text{wick}} = \tanh\left(\frac{\text{Lower\_Wick}}{\text{wick\_atr\_threshold} \cdot \text{ATR}}\right)$
     - Cho Short: $f_{\text{wick}} = \tanh\left(\frac{\text{Upper\_Wick}}{\text{wick\_atr\_threshold} \cdot \text{ATR}}\right)$
  2. *Độ Lệch Chuẩn Z-Score ($f_{\text{z}}$)*:
     $$f_{\text{z}} = \tanh\left(\frac{|\text{Price\_Z\_Score}|}{\text{z\_score\_threshold}}\right)$$
  3. *Điểm Tổng Hợp*:
     $$P_{\text{ano}} = 0.60 \cdot f_{\text{wick}} + 0.40 \cdot f_{\text{z}}$$
  4. *Stop Loss*: Đặt ngoài chóp râu nến: $\text{SL} = \text{Extreme\_Wick} \pm (\text{ATR} \times \text{wick\_sl\_buffer\_atr})$.
- **Dải Gene DNA (`TacticalAnomalyParams`)**:
  - `wick_atr_threshold`: `#[dna(min = 0.5, max = 2.5, step = 0.1)]`
  - `z_score_threshold`: `#[dna(min = 1.0, max = 3.0, step = 0.1)]`
  - `wick_sl_buffer_atr`: `#[dna(min = 0.10, max = 1.50, step = 0.10)]`
  - `min_activation_score`: `#[dna(min = 0.25, max = 0.75, step = 0.05)]`

---

### 2.6. 🧲 `ABSORPTION` (Absorption Scout Scanner - Dò Dấu Chân Cá Mập)

- **Nguyên lý**: Phát hiện dấu chân gom hàng / phân phối của tổ chức lớn trong các vùng nén biên độ hẹp trước khi giá bùng nổ.
- **Công thức Chấm Điểm Mềm Chuẩn Hóa ($P_{\text{abs}}$)**:
  1. *Cường độ nén Squeeze Intensity ($f_{\text{sqz}}$)*:
     $$f_{\text{sqz}} = \tanh\left(\frac{\text{squeeze\_intensity}}{\text{squeeze\_threshold}}\right)$$
  2. *Lực gom Khối Lượng ($f_{\text{vol}}$)*:
     $$f_{\text{vol}} = \tanh\left(\frac{\text{volume\_force}}{\text{vol\_z\_threshold}}\right)$$
  3. *Điểm Tổng Hợp*:
     $$P_{\text{abs}} = 0.50 \cdot f_{\text{sqz}} + 0.50 \cdot f_{\text{vol}}$$
  4. *Xác Định Hướng Lệnh*: Dựa trên độ lệch liên tục giữa $\text{VWMA}$ và $\text{BB}_{\text{Middle}}$:
     - Nếu $\text{VWMA} \ge \text{BB}_{\text{Middle}} \implies \text{Long}$
     - Nếu $\text{VWMA} < \text{BB}_{\text{Middle}} \implies \text{Short}$
- **Dải Gene DNA (`TacticalAbsorptionParams`)**:
  - `squeeze_threshold`: `#[dna(min = 0.20, max = 0.70, step = 0.05)]`
  - `vol_z_threshold`: `#[dna(min = 0.20, max = 0.80, step = 0.05)]`
  - `min_activation_score`: `#[dna(min = 0.25, max = 0.75, step = 0.05)]`

---

## 🔍 3. ƯU ĐIỂM, KHUYẾT ĐIỂM TIỀM ẨN & CƠ CHẾ PHÒNG VỆ CỦA BƯỚC 1

### 3.1. Ưu Điểm (Pros)
1. **Khắc phục triệt để lỗi Phễu Cạn**: Hồi sinh $100\%$ các Playbooks bị khóa cứng, cho phép toàn bộ 6 Scanners tham gia quét tín hiệu.
2. **Loại bỏ 100% Magic Numbers**: Mở rộng toàn bộ ngưỡng kỹ thuật thành Gene DNA cho GA Optimizer tự động may đo.
3. **Tạo Gradient liên tục**: Cung cấp phản hồi điểm số mịn mượt cho thuật toán Di truyền tối ưu hóa.

### 3.2. Khuyết Điểm Tiềm Ẩn & Tình Huống Thất Bại (Cons & Failure Modes)
1. **Rủi ro tích lũy tín hiệu yếu (Weak Signal Accumulation)**: Khi cả 3 yếu tố đều ở mức trung bình ($0.5$), tổng điểm vẫn đạt $0.50$ và có thể phát sinh đề xuất chất lượng thấp.
2. **Xung đột tín hiệu ngược chiều (Contradictory Proposals)**: Trong nến giật râu 2 đầu, `RANGE_OSC` có thể cho điểm Long cao trong khi `MOMENTUM_IGN` cho điểm Short cao.
3. **Độ nhạy dải dung sai Gaussian**: Nếu Optimizer chọn dải dung sai `bb_proximity_pct` quá rộng, scanner có thể chấm điểm cao cho cả nến ở giữa dải băng.

### 3.3. Cơ Chế Phản Ứng & Phòng Vệ Của Hệ Thống (Counter-Measures)
1. **Màng Lọc Arbiter & Veto (Bước 3)**: Điểm $P_j$ bị nhân với Trọng số Bối cảnh $\overline{w}_{\text{regime}}$ và Multiplier dòng tiền $\implies$ Dìm điểm và loại bỏ các tín hiệu yếu.
2. **Max Tracking & Strict Exclusive Order**: Chỉ chọn 1 đề xuất duy nhất có điểm số cao nhất và chỉ giữ 1 chiều vị thế, triệt tiêu xung đột lệnh đối nghịch.
3. **Kẹp trần SL $\le 4.0\%$ & Progressive Time Decay**: Cắt lỗ nhỏ an toàn nếu vào lệnh sai và kích hoạt Stagnant Capital Release đóng chủ động nếu giá đi ngang sau 2-4 tiếng.
