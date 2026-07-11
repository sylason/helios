# 03 — Đánh Giá Tầng Regime Scoring

## 1. Tổng Quan 6 Squad Điểm Số

| Squad | Công thức scoring | Đầu vào chính | Vấn đề |
|-------|-------------------|---------------|---------|
| **Anomaly** | `(z_comp × 0.8 + chaos_comp × 0.2).tanh()` | price_z_score (tanh), chaos_ratio (pct) | z_score_threshold là divisor → score phụ thuộc cấu hình nhiều |
| **Trend** | `(confidence / (min_thresh × buffer)).tanh()` | trend_angle_adj (pct), trend_strength (LINEAR) | 🔴 **trend_strength bão hòa** → Trend luôn ≈ 1.0 |
| **Squeeze** | `(squeeze_intensity / threshold).tanh()` | squeeze_intensity (pct), threshold (dải 0.1-0.95) | Hoạt động đúng |
| **Absorption** | `(volume_force / vol_thresh).tanh()` nếu `bb_width_norm < range_width_max` | bb_width_norm (pct), volume_force (pct) | 🔴 **Cổng IF quá hẹp** |
| **Parabolic** | `(trend_angle_adj / 0.85).tanh()` | trend_angle_adj (pct) | Hoạt động khi adj > 0.6 |
| **Ranging** | `1.0 - max(trend, squeeze, anomaly, absorption, parabolic)` | Tất cả | 🔴 **Luôn nhỏ** vì trend ≈ 1.0 |

## 2. 🔴 Phân Tích Chi Tiết Từng Vấn Đề

### ISSUE-R1: Trend Score Bão Hòa (Root Cause #1)

```rust
// regime.rs dòng 64-73
let angle_norm = trend_angle.abs() / 0.5;    // trend_angle_adj ∈ [0,1] → angle_norm ∈ [0,2]
let adx_norm = trend_strength / adx_moderate; // trend_strength = ADX/100 ∈ [0,1]
                                               // adx_moderate ∈ [0.05, 0.35]
                                               // → adx_norm ∈ [0, 20] !!!
```

**Ví dụ thực tế:**
- ADX = 25 → trend_strength = 0.25
- adx_moderate = 0.07 (trong dải DNA [0.05, 0.35])
- → adx_norm = 0.25 / 0.07 = **3.57**
- confidence = weighted_avg(angle_norm, adx_norm) ≈ **2.5**
- → tanh(2.5 / min_threshold) ≈ **0.99**

**Kết luận:** Trend score gần như luôn bão hòa ở ~0.99, khiến:
- Trend luôn thắng trong cuộc đấu argmax
- Ranging = 1.0 - 0.99 = 0.01 → Ranging chết
- Absorption, Anomaly phải đánh bại 0.99 → gần như bất khả thi

### ISSUE-R2: Absorption Bị Khóa Bởi Cổng IF

```rust
// regime.rs dòng 79-81
if bb_width_norm < reg_cfg.absorption.range_width_max {  // ← CỔNG IF
    scores.absorption = (volume_force / vol_thresh).tanh();
}
// Nếu KHÔNG vào cổng → scores.absorption = 0.0 (default)
```

- `bb_width_norm` giờ là percentile [0, 1]
- `range_width_max` DNA = [0.1, 0.4]
- → Absorption chỉ kích hoạt khi BB width nằm trong 10-40% hẹp nhất lịch sử
- Điều này đúng về mặt ý nghĩa (Absorption = biên độ hẹp + volume cao)
- NHƯNG ngay cả khi vượt cổng IF, score vẫn phải > 0.99 của Trend mới được chọn → **Bất khả thi!**

### ISSUE-R3: Ranging Bị Nghiền Nát

```rust
scores.ranging = (1.0 - max(trend, squeeze, anomaly, absorption, parabolic)).max(0.01).min(1.0);
```

- Vì trend ≈ 0.99 → ranging ≈ 0.01
- Ranging KHÔNG BAO GIỜ thắng Trend → `RangeOscillator` chết

### ISSUE-R4: Parabolic vs Trending — Xung Đột Thiết Kế

- Parabolic score = `(trend_angle_adj / 0.85).tanh()`
- Khi trend_angle_adj = 0.9 → Parabolic = tanh(1.06) ≈ 0.79
- Trend score = ≈ 0.99
- → Parabolic KHÔNG BAO GIỜ thắng Trend
- → `ClimaxReversal` chết

## 3. Kết Luận

> **Nguyên nhân gốc rễ #1:** `trend_strength = ADX/100` không phải percentile, khiến `adx_norm` quá lớn → Trend Score bão hòa. Mọi Regime khác đều bị áp đảo.
>
> **Hệ quả:** Chỉ có Trending regime chiến thắng → chỉ có `TrendPullback` và `MomentumIgnition` (cả hai đều nằm dưới nhánh Trending) được map.
