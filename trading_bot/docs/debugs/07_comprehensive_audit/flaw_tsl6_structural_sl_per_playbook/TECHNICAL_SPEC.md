# 📑 TÀI LIỆU KỸ THUẬT: FLAW-TSL.6 - STRUCTURAL STOP LOSS PER-PLAYBOOK & DYNAMIC DNA EXPANSION

> **Mã lỗi:** FLAW-TSL.6  
> **Dự án liên quan:** `strategies_backtests/helios`  
> **Thư mục tài liệu:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_tsl6_structural_sl_per_playbook/`  
> **Trạng thái:** Sẵn sàng triển khai (Ready for Implementation)

---

## 1. BỐI CẢNH & PHÂN TÍCH BẢN CHẤT LỖI LOGIC

### 1.1. Hiện trạng thực tế trong Helios V3
Trong kiến trúc chiến lược hiện tại:
- **`Reversal` và `Anomaly`**: Đã tự tính mức `suggested_sl` theo râu nến (`low - wick_buffer * ATR`).
- **4 Scanners còn lại (`TrendPullback`, `MomentumIgnition`, `RangeOscillator`, `Absorption`)**:
  - Đều trả về `suggested_sl: None`!
  - Khi Arbiter phê duyệt tín hiệu, `mod.rs:297-307` buộc phải fallback về công thức tính SL mù theo khoảng cách:
    $$\text{Raw SL} = \text{Close} \pm (\text{base\_sl\_atr} \times \text{ATR})$$
- **Hậu quả:**
  1. **Mất bản sắc kỹ thuật:** Lệnh Breakout không cắt lỗ ở đáy nến nổ; Lệnh Range không cắt lỗ ở dải Bollinger; Lệnh Trend Pullback không cắt lỗ ở đường hỗ trợ KAMA.
  2. **Méo mó tỷ lệ Risk/Reward (R:R):** Lệnh Range sóng ngắn ăn $1.0\% - 2.0\%$ nhưng lại gánh mức SL dùng chung của Trend ($3.0 - 5.0 \text{ ATR}$), dẫn đến R:R âm nặng và gồng lỗ vô ích khi mẫu hình đã bị phá vỡ.

---

## 2. THIẾT KẾ KIẾN TRÚC GIẢI PHÁP (PHƯƠNG ÁN B DYNAMIC)

### 2.1. Cài Đặt 4 Gene DNA Mới Trong `config.rs`
Mỗi Scanner được trang bị một hệ số đệm biến động ATR độc lập:

1. **`TacticalPullbackParams` (`trend_pb`):**
   ```rust
   /// Đệm ATR bảo vệ qua KAMA / đáy nhịp hồi (0.10 -> 4.00 ATR, Step 0.15 -> 27 nấc)
   #[dna(min = 0.10, max = 4.00, step = 0.15)]
   #[serde(default = "default_pullback_sl_buffer_atr")]
   pub struct_sl_buffer_atr: f64,
   ```
2. **`TacticalMomentumParams` (`momentum_ignition`):**
   ```rust
   /// Đệm ATR dưới đáy nến bùng nổ xung lực (0.10 -> 3.00 ATR, Step 0.15 -> 20 nấc)
   #[dna(min = 0.10, max = 3.00, step = 0.15)]
   #[serde(default = "default_momentum_sl_buffer_atr")]
   pub struct_sl_buffer_atr: f64,
   ```
3. **`TacticalRangeParams` (`range_osc`):**
   ```rust
   /// Đệm ATR ngoài biên dải Bollinger Bands (0.10 -> 3.00 ATR, Step 0.15 -> 20 nấc)
   #[dna(min = 0.10, max = 3.00, step = 0.15)]
   #[serde(default = "default_range_sl_buffer_atr")]
   pub struct_sl_buffer_atr: f64,
   ```
4. **`TacticalAbsorptionParams` (`absorption`):**
   ```rust
   /// Đệm ATR dưới vùng nén hấp thụ thanh khoản (0.25 -> 10.00 ATR, Step 0.25 -> 40 nấc)
   #[dna(min = 0.25, max = 10.00, step = 0.25)]
   #[serde(default = "default_absorption_sl_buffer_atr")]
   pub struct_sl_buffer_atr: f64,
   ```

* **Mở rộng không gian DNA:** Tổng số chiều DNA của Helios V3 tăng từ **113 chiều lên 117 chiều**.

### 2.2. Công Thức Tính Structural SL Cho Từng Scanner
Mỗi Scanner tính toán `suggested_sl` bám sát cản vi mô của mẫu hình:

1. **`TrendPullback`:**
   - Long: $\text{SL} = \min(\text{Low}[i], \text{KAMA}[i]) - (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
   - Short: $\text{SL} = \max(\text{High}[i], \text{KAMA}[i]) + (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
2. **`MomentumIgnition`:**
   - Long: $\text{SL} = \text{Low}[i] - (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
   - Short: $\text{SL} = \text{High}[i] + (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
3. **`RangeOscillator`:**
   - Long: $\text{SL} = \min(\text{Low}[i], \text{BB\_Lower}[i]) - (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
   - Short: $\text{SL} = \max(\text{High}[i], \text{BB\_Upper}[i]) + (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
4. **`AbsorptionScout` (Neo biên hộp nén Squeeze - BB Lower/Upper):**
   - Long: $\text{SL} = \min(\text{Low}[i], \text{BB\_Lower}[i]) - (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$
   - Short: $\text{SL} = \max(\text{High}[i], \text{BB\_Upper}[i]) + (\text{current\_atr} \times \text{struct\_sl\_buffer\_atr})$

### 2.3. Chốt Chặn An Toàn (Sanity Guard & Twin Sentinels)
1. **Sanity Guard Trong Scanner:**
   - Đảm bảo tính hợp lý hình học trước khi trả về `Proposal`:
     - Long: Nếu $\text{SL} \ge \text{Close} \implies \text{SL} = \text{Close} - \max(\text{buffer} \times \text{ATR},\ \text{Close} \times 0.002)$.
     - Short: Nếu $\text{SL} \le \text{Close} \implies \text{SL} = \text{Close} + \max(\text{buffer} \times \text{ATR},\ \text{Close} \times 0.002)$.
2. **Twin Sentinels Trong `clamp_sl_price`:**
   - **Sàn rủi ro `min_initial_sl_pct`:** Đảm bảo khoảng cách tối thiểu chống cắn spread sàn.
   - **Trần rủi ro `playbook_max_sl_roe`:** Khống chế mức lỗ tối đa khi gặp nến râu quét dị thường (Fat-tail Wick).

---

## 3. DANH MỤC FILE THAY ĐỔI
1. `strategies_backtests/helios/src/strategy/core_v3/config.rs` (Thêm 4 gene DNA).
2. `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs` (Tính structural SL).
3. `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs` (Tính structural SL).
4. `strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs` (Tính structural SL).
5. `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs` (Tính structural SL).
6. `strategies_backtests/helios/src/strategy/core_v3/mod.rs` (Cập nhật test DNA_WIDTH = 117 và unit tests).
