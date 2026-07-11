# 02 — Đánh Giá Tầng Sensor + Matrix

## 1. Danh Sách Indicator & Mục Đích

| Indicator | Timeframe DNA | Vai trò thiết kế | Tầng sử dụng |
|-----------|-----------|-------------------|---------------|
| NATR | natr.timeframe | Đo biến động % → chaos | Matrix (chaos, chaos_ratio) |
| ChaosRatio | natr.timeframe | NATR/SMA(NATR) → spike detection | Matrix (chaos_ratio) |
| VolMA | vol_ma.timeframe | Trung bình khối lượng | Matrix (volume_force) |
| ER | kama.timeframe | Hiệu quả hướng đi → entropy | Matrix (entropy) |
| MFI | mfi.timeframe | Dòng tiền → liquidity | Matrix (liquidity) |
| ADX | adx.timeframe | Sức mạnh xu hướng | Matrix (trend_strength) |
| LinReg Slope | linreg.timeframe | Hướng xu hướng | Matrix (trend_dir, trend_angle_adj) |
| LinReg Angle | linreg.timeframe | Góc xu hướng (degrees) | Matrix (trend_angle) — **KHÔNG DÙNG** |
| KAMA | kama.timeframe | Đường trung tâm thích nghi | Tactical (kama_mid filter) |
| BB (Upper/Lower) | bbands.timeframe | Dải biến động → squeeze, bb_width | Matrix + Tactical |
| KC (Upper/Lower) | keltner.timeframe | Kênh Keltner → squeeze | Matrix (squeeze_intensity) |
| RSI | rsi.timeframe | Sức mạnh tương đối | Matrix (momentum) |
| MACD (Main/Signal/Hist) | macd.timeframe | Động lượng → xác nhận trigger | Tactical (tất cả playbook) |
| Stochastic (K/D/Cross) | stochastic.timeframe | Overbought/Oversold | Tactical (pullback, range) |
| ATR | atr.timeframe | Khoảng cách SL thô | Risk (trailing, initial SL) |
| Z-Score | z_score_lookback | Vị trí giá thống kê | Matrix (price_z_score) |

## 2. Quy Trình Chuẩn Hóa (Matrix)

| Biến Matrix | Công thức | Đầu ra | Lookback DNA |
|-------------|-----------|--------|-------------|
| trend_strength | ADX / 100 | [0, 1] (linear) | — |
| chaos | NATR / 100 | [0, 0.1+] (linear) | — |
| trend_dir | rolling_pct(adj_slope) → 2x-1 | [-1, 1] | trend_lookback |
| trend_angle_adj | rolling_pct(slope/ATR) | [0, 1] | trend_lookback |
| squeeze_intensity | rolling_pct(overlap_raw) | [0, 1] | squeeze_lookback |
| bb_width_norm | rolling_pct(bb_width_raw) | [0, 1] | squeeze_lookback |
| momentum | rolling_pct(RSI) | [0, 1] | momentum_lookback |
| liquidity | rolling_pct(MFI) | [0, 1] | momentum_lookback |
| chaos_ratio | rolling_pct(NATR/SMA_NATR) | [0, 1] | squeeze_lookback |
| entropy | rolling_pct(ER) | [0, 1] | trend_lookback |
| volume_force | rolling_pct(vol/vol_ma) | [0, 1] | volume_lookback |
| price_z_score | Z-Score → tanh(z/sensitivity) | [-1, 1] | z_score_lookback |

## 3. 🔴 Phát Hiện Vấn Đề

### BUG-S1: `trend_strength` KHÔNG phải percentile
- `trend_strength = ADX / 100` → giá trị linear, KHÔNG phải percentile
- Nhưng `regime.rs` dùng nó chia cho `adx_moderate` (dải [0.05, 0.35]) → kết quả rất lớn
- **Hệ quả:** `adx_norm` thường >> 1.0 → `scores.trend` bị tanh bão hòa ≈ 1.0
- **Đây là lý do Trending luôn thắng trong cuộc đấu Regime!**

### BUG-S2: `chaos` KHÔNG phải percentile
- `chaos = NATR / 100` → giá trị linear rất nhỏ (thường 0.001 - 0.01)
- `sensor.rs` so sánh `chaos < config.chaos_z_threshold` (dải [0.005, 0.065])
- Với crypto biến động mạnh, NATR ~ 0.5-2.0 → chaos ~ 0.005-0.02
- **Hệ quả:** So sánh thường TRUE → Environment Fuse thường mở cửa
- Nhưng `chaos_score = (1.0 - chaos/threshold)` → chaos_score ≈ 0.7-0.95 (gần như luôn cao)
- **Cầu chì Chaos gần như vô hiệu!**

### BUG-S3: `trend_angle` (raw degrees) được lưu nhưng KHÔNG bao giờ dùng
- `matrix.rs` line 90: `let trend_angle = sensors.linreg_angle.as_ref().clone()` — sao chép y nguyên
- Nhưng `mod.rs` luôn truyền `trend_angle_adj` (percentile) vào mọi hàm
- `trend_angle` (dạng degrees) hoàn toàn là dead data, chiếm bộ nhớ vô ích

### ⚠️ CẢNH BÁO: Asymmetry Percentile vs Linear
- **Percentile:** squeeze_intensity, bb_width_norm, momentum, liquidity, chaos_ratio, entropy, volume_force, trend_dir, trend_angle_adj
- **Linear (raw scaled):** trend_strength (ADX/100), chaos (NATR/100)
- **Tanh:** price_z_score

Sự không đồng nhất này khiến `regime.rs` phải xử lý các đầu vào ở hệ quy chiếu khác nhau, tạo ra các công thức "so sánh táo với cam".
