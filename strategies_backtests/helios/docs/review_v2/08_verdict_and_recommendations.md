# 08 — Phán Quyết Cuối Cùng & Kế Hoạch Hành Động

## 1. Phán Quyết

### Nguyên nhân gốc rễ duy nhất (Single Root Cause):

> **`trend_strength = ADX / 100` không phải percentile**, khiến `adx_norm` trong `regime.rs` quá lớn → `scores.trend` bão hòa ở ~0.99 → Trending luôn thắng → 5/7 Playbook bị chết.

### Các vấn đề phụ (amplifiers):
1. `chaos = NATR / 100` cũng không phải percentile → Environment Fuse không đúng hệ
2. `trend_angle` (raw degrees) được lưu nhưng dead data
3. `smooth_atr` không smooth (chỉ là ATR tức thời)
4. Absorption SL factor hardcoded 0.7
5. MACD raw trong Squeeze trigger
6. Single-position lock gây zombie trades

## 2. Kế Hoạch Hành Động — Theo Thứ Tự Ưu Tiên

### 🔴 P0: Fix Root Cause — Chuẩn Hóa trend_strength

**File:** `matrix.rs`  
**Thay đổi:** Chuyển `trend_strength` từ `ADX / 100` sang `rolling_percentile`

```rust
// CŨ (dòng 49):
trend_strength.push(sensors.adx[i] / 100.0);

// MỚI:
// Giữ nguyên raw push, rồi rolling_percentile ở bước 2
adx_raw.push(sensors.adx[i]);
...
let trend_strength = normalization::rolling_percentile(&adx_raw, config.matrix.trend_lookback);
```

**Tác động kỳ vọng:** Trend score không còn bão hòa → Squeeze, Absorption, Ranging, Anomaly, Parabolic đều có cơ hội thắng.

---

### 🔴 P1: Fix chaos — Chuẩn Hóa chaos trong Matrix

**File:** `matrix.rs`  
**Thay đổi:** `chaos` cũng cần rolling_percentile

```rust
// CŨ:
chaos.push(sensors.natr[i] / 100.0);

// MỚI:
natr_raw.push(sensors.natr[i]);
...
let chaos = normalization::rolling_percentile(&natr_raw, config.matrix.squeeze_lookback);
```

**File phụ:** `sensor.rs` — cập nhật `chaos_z_threshold` DNA range sang [0.1, 0.9] vì chaos giờ là percentile.

---

### 🟡 P2: Xóa dead data trend_angle

**File:** `matrix.rs`  
**Thay đổi:** Xóa `trend_angle` (raw degrees) khỏi MarketMatrix struct — không ai dùng nó.

---

### 🟡 P3: Smooth ATR cho Trailing Stop

**File:** `mod.rs`  
**Thay đổi:** Thay `volatility_benchmark = current_atr` bằng EMA:

```rust
let alpha = 0.1; // ~10 nến decay
self.memory.volatility_benchmark = 
    self.memory.volatility_benchmark * (1.0 - alpha) + current_atr * alpha;
```

---

### 🟢 P4: Absorption SL Factor

**File:** `tactical.rs`  
**Thay đổi:** Thay `0.7` hardcoded bằng `risk_cfg.range_sl_factor` (tái sử dụng) hoặc thêm field DNA mới.

---

### 🟢 P5: MACD Percentile trong Squeeze Trigger

**File:** `tactical.rs`  
**Thay đổi:** Thay `macd_hist > -0.01` bằng kiểm tra xu hướng: `macd_hist[i] > macd_hist[i-1]`

---

## 3. Kế Hoạch Kiểm Chứng

Sau khi fix P0 + P1:
1. Chạy `cargo check` → đảm bảo compile
2. Chạy optimizer → kiểm tra:
   - Có xuất hiện các Playbook tag mới (Sqz_Agg, Sqz_Conf, Range_L, Absorp_L, Ano_Fade_L, Climax_Rev_L)?
   - Số lượng trades tăng lên?
   - Max Drawdown có tăng quá mức chấp nhận?
3. So sánh kết quả trước/sau fix

## 4. Tóm Tắt Trực Quan

```
TRƯỚC FIX:
  Trend: ████████████████████████████ 99%
  Squeeze: █                            5%
  Ranging: ▏                            1%
  Absorption: ▏                         0%
  Anomaly: ██                          12%
  Parabolic: ████                      25%

SAU FIX (kỳ vọng):
  Trend: █████████████                 55%
  Squeeze: ██████████                  40%
  Ranging: ████████                    35%
  Absorption: ███████                  30%
  Anomaly: ████                        15%
  Parabolic: ██████                    25%
```

> **Lưu ý:** Số liệu trên là ước tính định tính, không phải kết quả thực tế. Cần chạy optimizer để xác nhận.
