# 05 — Đánh Giá Tầng Tactical Execution

## 1. Tổng Quan 7 Tactical Triggers

| Playbook | Điều kiện vào lệnh (AND) | Nhận xét |
|----------|--------------------------|---------|
| **SqueezeAgg/Conf** | is_expanding AND is_volume_strong AND price vs KAMA AND MACD > -0.01 | 🔴 `macd_hist > -0.01` dùng raw MACD (không chuẩn hóa) |
| **TrendPullback** | price > KAMA AND stoch_k < pullback_thresh AND MACD turning up | ✅ Logic hợp lý |
| **MomentumIgnition** | RSI safe band AND MACD accelerating AND volume_force > spike_min | ✅ Logic hợp lý |
| **RangeOscillator** | price_position < 0.25 AND vol exhausted AND stoch < oversold AND MACD rebound | 🔴 Quá nhiều AND → rất khó trigger |
| **ClimaxReversal** | RSI extreme AND z_score extreme AND momentum collapsing | ⚠️ Emergency Override có thể gây whipsaw |
| **AbsorptionScout** | shark volume AND MACD breaking out AND RSI bias | ✅ Logic hợp lý |
| **AnomalyFade** | z_score extreme AND RSI extreme AND MACD turning AND candle rejection | 🔴 4 điều kiện AND → cực kỳ khó trigger |

## 2. 🔴 Phát Hiện Vấn Đề Chi Tiết

### ISSUE-T1: Squeeze Dùng MACD Histogram Thô (Raw vs Percentile Mismatch)

```rust
// tactical.rs dòng 101
if is_expanding && is_volume_strong && current_close > kama_mid && macd_hist > -0.01 ...
```

- `macd_hist` = giá trị thô từ sensors (có thể âm vài trăm tùy giá coin)
- So sánh với `-0.01` hardcoded → **không có ý nghĩa nhất quán**
- Với ETH ~$3500, MACD hist có thể nằm ở khoảng ±5 → điều kiện `> -0.01` gần như luôn TRUE
- → Điều kiện MACD trong Squeeze là **vô nghĩa**, không lọc gì cả

### ISSUE-T2: RangeOscillator Yêu Cầu 5 Điều Kiện AND

```
price_position < 0.25
AND is_exhausted (vol < thresh)
AND stoch_k < stoch_oversold
AND is_rebounding_up (MACD[i] > MACD[i-1])
AND !position_open
```

- 5 AND → xác suất khoảng `0.25 × 0.5 × 0.3 × 0.5 = 1.9%` mỗi nến
- Trên 43,000 nến × 1.9% ≈ 817 cơ hội lý thuyết
- Nhưng Ranging regime gần như không bao giờ thắng (xem ISSUE-R3) → **0 cơ hội thực tế**

### ISSUE-T3: AnomalyFade Yêu Cầu Râu Nến (Candle Rejection)

```rust
let is_bullish_rejection = (close - low) / (high - low) > min_rejection_pct;
```

- Điều kiện râu nến trên khung 1-phút cực kỳ nhiễu
- Kết hợp với z_score extreme + RSI extreme + MACD turning = **4 AND**
- → AnomalyFade gần như không bao giờ bắn được (ngay cả khi Anomaly regime thắng)

### ISSUE-T4: Tất Cả Playbook Đều Chặn `!tick.position().is_open()`

- Hệ thống chỉ cho phép **MỘT lệnh mở tại một thời điểm**
- Lệnh trung bình mở ~200 nến (xem output) → chiếm ~0.5% tổng thời gian
- NHƯNG trong khi lệnh đang mở, MỌI tín hiệu mới đều bị bỏ qua
- Ngoại lệ: ClimaxReversal và AnomalyFade có Emergency Override (score > 0.98)
- → **Hệ thống bỏ lỡ rất nhiều cơ hội khi đang giữ một lệnh kém**

### ISSUE-T5: Absorption Dùng Hardcoded SL Factor

```rust
// tactical.rs dòng 362
let sl_dist = (atr_raw * risk_cfg.base_sl_atr * 0.7).max(current_close * sl_min_pct);
```

- `0.7` là **magic number hardcoded**, không nằm trong DNA
- Optimizer không thể tối ưu SL cho Absorption riêng biệt
- Các playbook khác dùng `sl_factor` từ config → tại sao Absorption lại ngoại lệ?

## 3. Kết Luận

> Tầng Tactical có logic triggers khá hợp lý CHO MỖI playbook riêng lẻ. Vấn đề chính:
> 1. **Quá nhiều AND conditions** (đặc biệt Range và Anomaly) khiến trigger rate cực thấp
> 2. **Single-position lock** ngăn cản đa dạng hóa lệnh
> 3. **MACD raw vs percentile mismatch** ở Squeeze
> 4. **Magic number 0.7** ở Absorption SL
