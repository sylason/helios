# 04 — Đánh Giá Tầng Playbook Arbitration

## 1. Luồng Logic

```
candidates = [(Trending, score), (Squeeze, score), (Ranging, score), ...]
sorted by score DESC
best_squad = candidates[0]
final_score = best_squad.score × stability_index
IF final_score ≤ min_score_threshold → Wait
ELSE → map best_squad → specific Playbook
```

## 2. Mapping Regime → Playbook

| Regime Winner | Điều kiện phụ | Playbook Output |
|---------------|---------------|-----------------|
| Squeeze | RSI extreme? | SqueezeAggressive / SqueezeConfirmed |
| Trending | Parabolic? → Wait | (MomentumIgnition nếu RSI ∈ band) / TrendPullback |
| Ranging | — | RangeOscillator |
| Absorption | RSI bias? | AbsorptionScout / Wait |
| Anomaly | — | AnomalyFade |
| Parabolic | — | ClimaxReversal |

## 3. 🔴 Phát Hiện Vấn Đề

### ISSUE-P1: Trending Luôn Thắng → Các Nhánh Khác Dead Code

Vì `scores.trend ≈ 0.99` (xem Báo Cáo #3), `best_squad` luôn = Trending.
→ Chỉ có nhánh `MarketRegime::Trending` được thực thi.
→ 5 nhánh còn lại (Squeeze, Ranging, Absorption, Anomaly, Parabolic) là **dead code thực tế**.

### ISSUE-P2: Trending → MomentumIgnition vs TrendPullback

```rust
let rsi_thresh = config.tactical_sniper.momentum.rsi_threshold;  // DNA [0.6, 0.85]
let rsi_climax = config.tactical_sniper.momentum.rsi_climax;      // DNA [0.86, 0.99]

let is_momentum = (matrix_rsi > rsi_thresh && matrix_rsi < rsi_climax)
               || (matrix_rsi < (1.0 - rsi_thresh) && matrix_rsi > (1.0 - rsi_climax));
```

Vùng kích hoạt MomentumIgnition:
- LONG band: RSI ∈ [0.6, 0.99] (rộng)
- SHORT band: RSI ∈ [0.01, 0.4] (rộng)
- Phần giữa [0.4, 0.6]: TrendPullback

**Nhận xét:** Logic phân nhánh này hợp lý nhưng phụ thuộc 100% vào RSI percentile. Không có indicator nào khác tham gia quyết định → **quá đơn giản cho việc phân loại Momentum vs Pullback**.

### ISSUE-P3: Parabolic Filter Trong Trending — Đánh Ngược Nhau

```rust
let parabolic_thresh = config.parabolic.accel_angle_threshold.min(1.0);  // DNA [0.8, 1.0]
let is_parabolic = trend_angle > parabolic_thresh;
if trend_dir == Neutral || is_parabolic { return Wait; }
```

- Khi `is_parabolic = true` → Trending trả về Wait
- NHƯNG cùng lúc, Parabolic score ≈ 0.79 (< Trend ≈ 0.99) → Parabolic không bao giờ thắng
- **Kết quả:** Khi thị trường thực sự parabolic, HỆ THỐNG ĐỨNG YÊN! Không có Playbook nào xử lý!
- Đây là một **khoảng trống chiến thuật (tactical gap)** nghiêm trọng.

### ISSUE-P4: Absorption Cần RSI Bias → Lọc Thêm Một Lần

```rust
MarketRegime::Absorption => {
    if matrix_rsi > bias_threshold || matrix_rsi < (1.0 - bias_threshold) {
        AbsorptionScout
    } else {
        Wait  // ← Lại Wait!
    }
}
```

- Ngay cả khi Absorption thắng Regime (rất hiếm), nó vẫn có thể trả về Wait nếu RSI nằm giữa.
- bias_threshold DNA [0.55, 0.95] → vùng chết RSI ∈ [0.05, 0.45]
- **Cổng lọc chồng cổng lọc → "phễu cạn" tiếp tục.**

## 4. Kết Luận

> Tầng Playbook hoạt động đúng logic. Vấn đề không nằm ở đây mà ở **đầu vào (Regime scores bị méo)**. Tuy nhiên, việc thêm các cổng lọc phụ bên trong (Parabolic check, RSI bias) tạo thêm lỗ hổng "tactical gap" khi không Playbook nào xử lý được tình huống.
