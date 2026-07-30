# 🎯 04 — Action Plan (Kế Hoạch Sửa Lỗi)

> **Nguyên tắc:** Sửa từng lỗi một theo thứ tự ưu tiên. Mỗi lần sửa xong: `cargo check` → ghi kết quả.
> **Không sửa bất kỳ dải tham số nào thuộc về IndicatorsConfig.**

---

## NHÓM 1: 🔴 NGUY — Sửa Ngay

### Fix-01: ABSORPTION Scanner — Ngưỡng Kích Hoạt Quá Lỏng

**File:** `src/strategy/core_v3/scanners/absorption.rs`

**Vấn đề:** `if raw_score > 0.0` cho phép bất kỳ giá trị siêu nhỏ vào lệnh.

**Sửa:**
- Thêm field `min_activation_score` vào `TacticalAbsorptionParams` trong `config.rs`
- Dải: `[0.10, 0.60, step = 0.025]`
- Đổi điều kiện thành: `if raw_score >= abs_cfg.min_activation_score`

**Ước tính:** ~10 dòng code, 1 field mới trong config.

---

### Fix-02: Recovery Period — Chặn Nhồi Lệnh Sau Thua

**File:** `src/strategy/core_v3/mod.rs`

**Vấn đề:** Sau khi đóng lệnh lỗ, hệ thống scan ngay lập tức không có cooldown.

**Sửa:**
- Thêm field `cooldown_bars_remaining: usize` vào `HeliosStrategyV3`
- Thêm field `cooldown_duration` (DNA) vào `RiskStopParams` trong `config.rs`
- Dải: `[0.0, 500.0, step = 30.0]`, default = 0 (tắt)
- Trong `fn next()`: sau khi detect vừa đóng lệnh lỗ → set `cooldown_bars_remaining = cooldown_duration`

**Ước tính:** ~20 dòng code, 1 field mới trong config.

---

### Fix-03: Mega-Trend Protection — Giới Hạn Số Chu Kỳ

**File:** `src/strategy/core_v3/risk.rs`

**Vấn đề:** Mega-trend protection vô hiệu hóa Progressive Decay mãi mãi khi PnL lớn.

**Sửa:**
- Thêm field `mega_trend_max_cycles` vào `RiskStopParams` trong `config.rs`
- Dải: `[5.0, 50.0, step = 5.0]`
- Trong risk.rs: chỉ bảo vệ khi `decay_cycles <= mega_trend_max_cycles`

**Ước tính:** ~5 dòng code, 1 field mới trong config.

---

### Fix-04: Reversal Multiplier — Không Bị Triệt Tiêu Khi Trend Cực Đoan

**File:** `src/strategy/core_v3/matrix.rs`

**Vấn đề:** `rev_score = (1.0 - t_mul) * (...)` làm reversal bằng 0 khi trend rất mạnh.

**Sửa:** Thêm "extreme trend bonus" — khi `t_mul > 0.85`, reversal score không bị nhân với `(1 - t_mul)` mà được giữ ổn định ở mức threshold:
```rust
let trend_factor = if t_mul > 0.85 { 0.15 } else { 1.0 - t_mul };
let rev_score = trend_factor * ((momentum_extremity * rev_m_w) + (z_abs * (1.0 - rev_m_w)));
```

Hoặc đơn giản hơn: tách reversal multiplier ra khỏi trend factor, dùng thẳng `momentum_extremity` và `z_abs`.

**Ước tính:** ~5 dòng code, không cần thêm config.

---

## NHÓM 2: 🟡 QUAN TRỌNG — Sửa Sau

### Fix-05: ABSORPTION — Thêm Điều Kiện Lọc Trend

**File:** `src/strategy/core_v3/scanners/absorption.rs`

**Vấn đề:** Absorption vào lệnh cả khi đang có trend mạnh.

**Sửa:**
- Truyền thêm `matrix.trend_multiplier[i]` vào scanner
- Thêm điều kiện: `if trend_multiplier > abs_cfg.trend_filter_threshold { return None; }`
- Field mới: `trend_filter_threshold` trong `TacticalAbsorptionParams`, dải `[0.50, 0.95, step = 0.05]`

---

### Fix-06: Dead Config Fields — Dọn Dẹp

**Files:** `config.rs`, scanner files

**Các fields không được đọc:**
- `TacticalAbsorptionParams.macd_lookback` → không đọc trong absorption.rs
- `TacticalMomentumParams.rsi_threshold`, `rsi_climax`, `macd_lookback`, `accel_factor` → không đọc trong momentum_ignition.rs
- `TacticalAnomalyParams.min_rejection_pct`, `macd_lookback` → không đọc trong anomaly.rs
- `TacticalReversalParams.exhaustion_vol_drop`, `macd_lookback` → không đọc trong reversal.rs
- `TacticalPullbackParams.macd_lookback`, `rsi_buy_thresh` → không đọc trong trend_pb.rs
- `TacticalRangeParams.volume_max_threshold`, `min_width_pct` → không đọc trong range_osc.rs
- `MatrixWeightsConfig` — `squeeze_multiplier` field trong Matrix không được dùng trong Arbiter
- `RiskStopParams.base_sl_atr` → không được dùng đúng mục đích

**Sửa:** Hoặc đọc chúng trong scanner tương ứng, hoặc xóa chúng.

---

### Fix-07: Range Multiplier — Thêm BB Width Context

**File:** `src/strategy/core_v3/matrix.rs`

**Sửa:**
```rust
// Range = yếu trend VÀ BB width ở ngưỡng trung bình (không quá hẹp/rộng)
let bb_in_range = (bb_width_norm[i] > 0.01 && bb_width_norm[i] < 0.05) as u8 as f64;
let range_score = (1.0 - t_mul) * bb_in_range;
range_multiplier.push(range_score);
```

---

### Fix-08: `squeeze_multiplier` — Gán Cho Scanner Phù Hợp

**File:** `src/strategy/core_v3/arbiter.rs`

`squeeze_multiplier` hiện không được dùng. Có thể gán cho TREND_PB (trend pullback trong squeeze) hoặc bỏ field đó.

---

## NHÓM 3: 🟠 CẢI TIẾN — Làm Sau Khi Rank 1 Ổn Định

### Fix-09: Momentum Ignition — Thêm Follow-Through Confirmation

**File:** `src/strategy/core_v3/scanners/momentum_ignition.rs`

Xác nhận breakout bằng cách kiểm tra nến trước đó cũng đóng ngoài KC (không phải chỉ nến hiện tại).

### Fix-10: ANOMALY vs REVERSAL — Phân Biệt Rõ Hơn

**Mục tiêu:** Anomaly = bắt cú quét tàn bạo + wick rõ ràng. Reversal = phân kỳ dài hạn + kiệt sức.
Cần kích hoạt `min_rejection_pct` trong anomaly.rs để đảm bảo wick đủ lớn.

### Fix-11: TREND_PB — Stoch Cross Confirmation

Thêm kiểm tra: cross phải xảy ra trong N nến gần nhất (không phải chỉ vị trí hiện tại).

---

## Thứ Tự Thực Hiện

```
Fix-01 → Fix-02 → Fix-03 → cargo check → Run Optimizer
                ↓ (nếu cần)
Fix-04 → Fix-05 → cargo check → Run Optimizer
                ↓ (nếu cần)
Fix-06 (dead code cleanup) → Fix-07 → Fix-08 → cargo check
```

---

## Tracking

| Fix   | Mô Tả                              | Status |
|-------|------------------------------------|--------|
| Fix-01 | Absorption min_activation_score   | [x]    |
| Fix-02 | Recovery Period cooldown           | [x]    |
| Fix-03 | Mega-trend max_cycles              | [x]    |
| Fix-04 | Reversal Multiplier extreme trend  | [x]    |
| Fix-05 | Absorption trend filter            | [x]    |
| Fix-06 | Dead config cleanup                | [x]    |
| Fix-07 | Range Multiplier (Reverted v3_8)   | [x]    |
| Fix-08 | Squeeze Multiplier assignment      | [ ]    |
| Fix-09 | Momentum follow-through            | [ ]    |
| Fix-10 | Anomaly min_rejection_pct          | [ ]    |
| Fix-11 | TrendPB stoch cross confirmation   | [ ]    |
