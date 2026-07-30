# 🔬 01 — Phân Tích 6 Scanner (Scanner Analysis)

> File: `src/strategy/core_v3/scanners/`
> Liên quan đến: `absorption.rs`, `anomaly.rs`, `momentum_ignition.rs`, `range_osc.rs`, `reversal.rs`, `trend_pb.rs`

---

## I. ABSORPTION SCANNER — Thiên Vị Nghiêm Trọng

### Code Logic Hiện Tại

```rust
// absorption.rs
let vol_activation = if 1.0 > vol_thresh {
    ((vol_force - vol_thresh) / (1.0 - vol_thresh)).max(0.0).min(1.0)
} else { 0.0 };
let squeeze_activation = squeeze.max(0.0).min(1.0);

let raw_score = (vol_activation * v_w) + (squeeze_activation * sq_w);

if raw_score > 0.0 {
    // Kích hoạt lệnh nếu raw_score > 0
}
```

### Lỗi Phát Hiện

| Lỗi | Mô Tả | Mức Độ |
|-----|-------|--------|
| **Ngưỡng kích hoạt = 0** | `if raw_score > 0.0` cho phép bất kỳ tín hiệu siêu nhỏ nào vào lệnh | 🔴 NGUY |
| **Không có điều kiện hướng** | Hướng lệnh chỉ dựa trên `macd > macd_signal` — không có điều kiện thị trường rõ ràng | 🔴 NGUY |
| **squeeze_activation = squeeze** | `squeeze_intensity` từ matrix đã nằm trong [0,1], nhưng không có min-threshold, nên giá trị thấp (0.01) vẫn đóng góp vào score | 🟡 QUAN |
| **Không lọc thị trường Trend** | Hệ thống vào lệnh Absorption ngay cả khi đang có Trend mạnh | 🟡 QUAN |
| **MACD chưa có độ trễ** | MACD cross không được kiểm tra trong N nến gần nhất — dễ bị nhiễu | 🟠 CẢI TIẾN |

### Đề Xuất Sửa

```rust
// Bổ sung min_score_threshold (~0.20-0.30) thay vì > 0.0
if raw_score >= abs_cfg.min_activation_score {
    // Kiểm tra thêm Trend: nếu Trend mạnh (trend_multiplier > 0.7)
    // thì không nên vào lệnh Absorption ngược chiều
}
```

---

## II. MOMENTUM IGNITION SCANNER — Logic Breakout Đúng Nhưng Thiếu Lọc

### Code Logic Hiện Tại

```rust
// Breakout direction: chỉ kiểm tra close > kc_u hoặc close < kc_l
if close > kc_u {
    signal_type = Some(SignalType::Long);
} else if close < kc_l {
    signal_type = Some(SignalType::Short);
}
```

### Lỗi Phát Hiện

| Lỗi | Mô Tả | Mức Độ |
|-----|-------|--------|
| **Không có xác nhận Follow-Through** | Chỉ cần 1 nến đóng ngoài KC là đủ — dễ bị false breakout | 🟡 QUAN |
| **vol_raw không scale với vol_ratio** | Trong `matrix.rs`, volume được nhân với `vol_ratio` khi tính `volume_force`, nhưng trong scanner thì `v / ma` trực tiếp — không nhất quán | 🟡 QUAN |
| **Không sử dụng matrix.volatility_expansion_multiplier** | Scanner tự tính score riêng, nhưng lại bỏ qua multiplier context từ matrix — Arbiter thì dùng multiplier để weight, dẫn đến đánh giá kép | 🟠 CẢI TIẾN |

---

## III. TREND PULLBACK SCANNER — Điều Kiện Kích Hoạt Không Rõ Ràng

### Code Logic Hiện Tại

```rust
if dir > dir_thresh { // Bullish Trend
    let score = (stoch_long_activation * st_w) + (kama_long_activation * k_w);
    if score > 0.0 && stoch_k > stoch_d {
        // Long
    }
}
```

### Lỗi Phát Hiện

| Lỗi | Mô Tả | Mức Độ |
|-----|-------|--------|
| **Stoch Cross không kiểm tra hướng chéo** | `stoch_k > stoch_d` chỉ kiểm tra vị trí hiện tại, không xác nhận cross mới xảy ra | 🟡 QUAN |
| **KAMA distance quá nhạy** | `kama_long_dist = if close > kama { close - kama } else { 0.0 }` — nếu giá đang trên KAMA, activation về 0, tức là **bullish trend với giá trên KAMA sẽ không vào lệnh được** — logic ngược | 🔴 NGUY |
| **`macd_lookback` khai báo trong config nhưng không dùng** | Trường `macd_lookback` tồn tại nhưng code không đọc | 🟠 DEAD CODE |

### Phân Tích Sâu — Lỗi KAMA Logic

Khi market đang Bullish Trend:
- `close > kama` thường xuyên → `kama_long_dist > 0` → `kama_long_activation → 0`
- Scanner kỳ vọng giá *hồi về gần KAMA* để vào lệnh Pullback
- Nhưng formula tính ngược: khoảng cách xa thì activation thấp → **đúng về ý nghĩa nhưng cần kiểm tra lại khi close < kama**

> Tức là khi giá đang XÁ KAMA (pullback thực sự đang xảy ra), `kama_long_activation` mới cao — đây có vẻ đúng ý tưởng. **Nhưng**: khi `kama_tol_abs * 2.0` nhỏ, activation drop to 0 rất nhanh — scanner quá nhạy cảm với parameter này.

---

## IV. RANGE OSC SCANNER — Logic Ổn Nhưng Chưa Lọc Regime

### Lỗi Phát Hiện

| Lỗi | Mô Tả | Mức Độ |
|-----|-------|--------|
| **Không lọc Breakout sắp xảy ra** | Có `volume_max_threshold` trong config nhưng scanner không đọc field này | 🟡 QUAN (Dead Config) |
| **Không lọc khi Trend mạnh** | Dù `range_multiplier = 1 - trend_strength` sẽ giảm confidence, scanner không self-gate | 🟠 CẢI TIẾN |
| **BB touch activation quá nhạy** | `bb_tol = bb_range * tol_pct` — nếu `tol_pct = 0.02`, tolerance rất nhỏ → BB activation gần như 0 hoặc 1 | 🟠 |

---

## V. REVERSAL SCANNER — Không Xuất Hiện Trong Trade Log

### Nhận Định

REVERSAL hoàn toàn vắng mặt trong Rank 1. Nguyên nhân có thể:

1. **`reversal_multiplier` từ Matrix thấp** — vì `rev_score = (1.0 - t_mul) * (...)`, nên khi Trend mạnh, reversal multiplier giảm mạnh → confidence bị veto bởi `min_multiplier_veto`
2. **Z-score threshold quá cao** — scanner cần cả `z_score` VÀ `rsi` đều cực đoan cùng lúc → điều kiện hiếm
3. **Tranh lệnh với ANOMALY** — cả hai đều dùng z_score + rsi, nhưng anomaly thêm wick pattern → anomaly thường thắng về raw_score

### Không Phải Lỗi — Nhưng Cần Quan Sát Thêm

Nếu REVERSAL không bao giờ thắng ANOMALY trong điều kiện tương đương, thì REVERSAL là **dead code chiến lược**.

---

## VI. ANOMALY SCANNER — Cạnh Tranh Với Reversal

### Logic Hiện Tại

```rust
let short_base = (z_short_activation * z_w) + (rsi_short_activation * rsi_w);
raw_score = short_base * pattern_w + upper_rejection * wick_w;
```

### Lỗi / Quan Sát

| Lỗi | Mô Tả | Mức Độ |
|-----|-------|--------|
| **`min_rejection_pct` khai báo nhưng không dùng** | Trường `min_rejection_pct` trong config không được đọc trong scanner | 🟠 DEAD CONFIG |
| **Upper/lower rejection luôn trong [0,1]** | Vì là tỷ lệ wick/total_range — hợp lệ. Nhưng không có min threshold để lọc wick nhỏ | 🟠 CẢI TIẾN |
| **Anomaly và Reversal có logic gần như giống nhau** | Hai scanner khác nhau ở chỗ: anomaly thêm wick rejection vào score. Ranh giới hành vi mờ nhạt | 🟡 QUAN |

---

## Bảng Tổng Hợp Scanner

| Scanner        | Hoạt Động? | Lỗi Chính              | Ưu Tiên Sửa |
|----------------|-----------|------------------------|-------------|
| ABSORPTION     | Quá nhiều | Ngưỡng `> 0`, thiếu filter | 🔴 Cao     |
| MOMENTUM_IGN   | Ổn        | Thiếu follow-through    | 🟡 Trung   |
| TREND_PB       | Ít xuất hiện | KAMA logic cần xem lại | 🟡 Trung   |
| RANGE_OSC      | Ít         | Dead config field       | 🟠 Thấp    |
| REVERSAL       | Không có  | Bị triệt tiêu bởi Anomaly | 🟡 Trung |
| ANOMALY        | Thỉnh thoảng | Dead config field     | 🟠 Thấp    |
