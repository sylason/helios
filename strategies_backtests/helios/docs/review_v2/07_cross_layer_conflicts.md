# 07 — Xung Đột Liên Tầng & Hiệu Ứng Triệt Tiêu Tín Hiệu

## 1. Bản Đồ Xung Đột

```
                    ┌──────────────────────────────────────────────┐
                    │         XUNG ĐỘT CHÍNH (CRITICAL)           │
                    └──────────────────────────────────────────────┘

    [Matrix]                    [Regime]                   [Playbook]
  trend_strength              Trend Score                 Trending wins
  = ADX/100                   ≈ 0.99 (bão hòa)           100% of the time
  (LINEAR,                        ▲                           │
   not pct)                       │                           ▼
       ╔══════════════════════════╧═══╗            ┌─────────────────┐
       ║  ROOT CAUSE: adx_norm >> 1  ║            │  Squeeze: DEAD  │
       ║  regime.rs dòng 65          ║            │  Ranging: DEAD  │
       ╚═════════════════════════════╝            │  Absorption: DEAD│
                                                   │  Anomaly: DEAD  │
                                                   │  Parabolic: DEAD│
                                                   └─────────────────┘
```

## 2. Chuỗi Triệt Tiêu Chi Tiết

### Chain #1: Absorption Kill Chain
```
1. matrix.rs: bb_width_norm = rolling_percentile(bb_width_raw)     ← OK [0,1]
2. regime.rs: IF bb_width_norm < range_width_max                    ← range_width_max [0.1, 0.4]
3. → Absorption score > 0 chỉ khi BB hẹp nhất 10-40% lịch sử      ← Bước lọc #1
4. → Nhưng Trend score ≈ 0.99                                       ← Bước chặn #2
5. → Absorption phải > 0.99 để thắng Trend                          ← GẦN NHƯ KHÔNG THỂ
6. → AbsorptionScout KHÔNG BAO GIỜ được chọn
```

### Chain #2: Squeeze Kill Chain
```
1. matrix.rs: squeeze_intensity = rolling_percentile(sqz_raw)      ← OK [0,1]
2. regime.rs: squeeze_score = (intensity / threshold).tanh()        ← OK khi intensity cao
3. → Squeeze score có thể đạt 0.8-0.9                               ← Khá tốt
4. → NHƯNG Trend score ≈ 0.99                                       ← Bước chặn
5. → Squeeze phải > 0.99 để thắng → Không thể                      ← CHẾT
6. → Ngay cả nếu thắng, tactical trigger yêu cầu is_expanding      ← Bước lọc #2
7. → vol_expansion_thresh [1.005, 1.05] → BB phải nở 0.5-5%        ← Có thể
8. → + volume_confirm > thresh + price vs KAMA + MACD               ← Bước lọc #3
```

### Chain #3: Ranging Kill Chain
```
1. regime.rs: ranging = 1.0 - max(all others)                      ← DESIGN FLAW
2. → Vì trend ≈ 0.99 → ranging ≈ 0.01                              ← CHẾT NGAY TỪ ĐẦU
3. → RangeOscillator KHÔNG BAO GIỜ có cơ hội
```

### Chain #4: Parabolic Paradox
```
1. regime.rs: parabolic_score = (trend_angle_adj / 0.85).tanh()    ← Max ≈ 0.83
2. → Không bao giờ thắng Trend (≈ 0.99)                             ← CHẾT
3. playbook.rs: Khi Trending wins VÀ is_parabolic → Wait            ← CHẶN NỐT
4. → Kết quả: Khi thị trường Parabolic, KHÔNG AI XỬ LÝ!           ← TACTICAL GAP
```

### Chain #5: Anomaly Kill Chain
```
1. regime.rs: anomaly_score = (z/thresh + chaos).tanh()             ← Có thể cao khi z extreme
2. → Nhưng z extreme hiếm gặp trên 1-min                            ← Bước lọc tự nhiên
3. → Ngay cả khi z extreme, Trend ≈ 0.99 vẫn thắng                 ← Bước chặn
4. → Ngay cả nếu Anomaly thắng, tactical cần 4 AND conditions      ← Bước lọc cuối
```

## 3. Xung Đột Giữa Các Tầng Khác

### Conflict #6: Environment Fuse vs Regime — Nhân Đôi Lọc Volume

- **Sensor.rs:** `volume_force > min_liquidity_ratio` → tăng stability_index
- **Playbook.rs:** `final_score = raw_score × stability_index`
- **Tactical:** MomentumIgnition yêu cầu `volume_force > volume_spike_min`
- → Volume bị kiểm tra **3 LẦN** ở 3 tầng khác nhau → Overkill

### Conflict #7: MACD Raw vs Percentile — Hệ Quy Chiếu Lẫn Lộn

- **Matrix:** momentum = rolling_percentile(RSI) → [0,1]
- **Tactical:** trực tiếp dùng `sensors.macd_hist[i]` → raw value (có thể ±100)
- **Tactical (Squeeze):** `macd_hist > -0.01` → hardcoded threshold cho giá trị raw
- → Cùng một file dùng 2 hệ quy chiếu khác nhau → Inconsistent

### Conflict #8: Single Position Lock → Zombie Trade Problem

- **Risk:** Trailing stop giữ lệnh mở 500-900 nến
- **Tactical:** `!tick.position().is_open()` → chặn mọi lệnh mới
- → Một lệnh "zombie" (lãi nhỏ nhưng không trigger trailing) chiếm slot cả ngày
- → Bỏ lỡ 100+ cơ hội tiềm năng

## 4. Bảng Tổng Hợp Impact

| Xung đột | Severity | Impact | Playbook bị ảnh hưởng |
|-----------|----------|--------|----------------------|
| Trend bão hòa (ROOT) | 🔴 CRITICAL | Giết 5/7 playbook | ALL trừ Trend/Mom |
| Absorption cổng IF + Trend | 🔴 HIGH | AbsorptionScout = 0 trades | AbsorptionScout |
| Ranging = 1-max | 🔴 HIGH | RangeOscillator = 0 trades | RangeOscillator |
| Parabolic paradox | 🟡 MEDIUM | Tactical gap khi parabolic | ClimaxReversal |
| Volume triple-check | 🟡 LOW | Lọc quá mức nhưng không chết | MomentumIgnition |
| MACD raw/pct mismatch | 🟡 LOW | Squeeze filter vô nghĩa | Squeeze* |
| Single position lock | 🟡 MEDIUM | Bỏ lỡ cơ hội | ALL |
