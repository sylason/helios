# 🏗️ Đề xuất Kiến trúc V3: "Signal-First" (Tín hiệu Trước, Phân loại Sau)

## Tóm tắt Vấn đề
Helios V2 sử dụng kiến trúc "Regime-First": Phân loại thị trường trước → Chọn chiến thuật sau.
- Kết quả Optimizer: **Chỉ 2/7 playbook từng được kích hoạt** (Squeeze, Absorption).
- 30-43 trades trong 43,200 nến → Bot chỉ **hoạt động ~0.07% thời gian**.
- Range, Momentum, Anomaly, Parabolic, Reversal = **DEAD CODE** trong thực tế.

---

## Kiến trúc V3 Đề xuất

```
Tick Data (43,200 nến/tháng)
    │
    ▼
┌─────────────────────────────────────────────────┐
│         SCANNER LAYER (Quét Song Song)          │
│                                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │ Breakout │ │ Pullback │ │ MeanRev  │       │
│  │ Scanner  │ │ Scanner  │ │ Scanner  │       │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘       │
│       │             │             │             │
│  ┌────┴─────┐ ┌────┴─────┐ ┌────┴─────┐       │
│  │ Momentum │ │ Reversal │ │ Anomaly  │       │
│  │ Scanner  │ │ Scanner  │ │ Scanner  │       │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘       │
│       │             │             │             │
│       └──────┬──────┴──────┬──────┘             │
│              ▼                                  │
│     [Signal Candidates Pool]                    │
│     Danh sách các tín hiệu kèm Confidence      │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│        ARBITER (Trọng Tài Duy Nhất)             │
│                                                 │
│  1. Lọc bỏ tín hiệu mâu thuẫn nhau            │
│  2. Xếp hạng theo Confidence Score              │
│  3. Kiểm tra Risk/Sizing                        │
│  4. Chọn TỐI ĐA 1 tín hiệu tốt nhất           │
│                                                 │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
              📍 VÀO LỆNH
```

### Nguyên lý Hoạt động

**Mỗi tick, TẤT CẢ 6 Scanner đều chạy song song.** Không có tầng Fuse hay Regime chặn trước.

Mỗi Scanner kiểm tra **2-3 điều kiện lõi** của riêng nó:
- **Breakout Scanner:** BB đang nở + Volume > threshold → Signal(Long/Short, confidence)
- **Pullback Scanner:** Price near KAMA + Stoch reversal → Signal(Long/Short, confidence)
- **MeanReversion Scanner:** Price near BB band + MACD pivot → Signal(Long/Short, confidence)
- **Momentum Scanner:** RSI breakout zone + MACD acceleration → Signal(Long/Short, confidence)
- **Reversal Scanner:** Extreme Z-Score + MACD collapse → Signal(Long/Short, confidence)
- **Anomaly Scanner:** Volume spike + Rejection wick → Signal(Long/Short, confidence)

**Confidence Score** được tính từ:
```
confidence = base_score * environment_quality * volume_confirmation
```
- `base_score`: Mức độ rõ ràng của tín hiệu (VD: RSI ở 15 rõ ràng hơn RSI ở 35)
- `environment_quality`: Tương tự stability_index cũ, nhưng KHÔNG chặn mà chỉ giảm điểm
- `volume_confirmation`: Volume có xác nhận tín hiệu không

---

## Chi tiết Từng Scanner

### 1. Breakout Scanner (Thay thế Squeeze + SqueezeConfirmed)

```rust
fn scan_breakout(sensors, matrix, config) -> Option<Signal> {
    // Điều kiện 1: BB đang nở (không cần biết có Squeeze trước đó hay không)
    let is_expanding = current_bb_width > prev_bb_width * expansion_thresh;
    
    // Điều kiện 2: Volume xác nhận
    let vol_ok = matrix.volume_force[i] > vol_thresh;
    
    if !is_expanding || !vol_ok { return None; }
    
    // Hướng: Dùng KAMA và MACD
    let direction = if close > kama && macd_hist > 0.0 { Long }
                    else if close < kama && macd_hist < 0.0 { Short }
                    else { return None; };
    
    // Confidence: Mức độ nở càng mạnh, càng tin cậy
    let expansion_ratio = current_bb_width / prev_bb_width;
    let confidence = (expansion_ratio - 1.0).min(1.0) * vol_factor;
    
    Some(Signal { direction, confidence, scanner: Breakout })
}
```

**So với V2:** Không cần Regime = Squeeze trước. Bất cứ khi nào BB nở + Volume mạnh = Signal.

### 2. MeanReversion Scanner (Thay thế RangeOscillator)

```rust
fn scan_mean_reversion(sensors, matrix, config) -> Option<Signal> {
    let bb_range = bb_upper - bb_lower;
    if bb_range < min_width * close { return None; }
    
    // Vị trí giá trong BB (0.0 = Lower, 1.0 = Upper)
    let price_position = (close - bb_lower) / bb_range;
    
    // LONG: Giá ở vùng dưới (0.0 - 0.25) + Stoch oversold + MACD pivot up
    if price_position < buy_zone && stoch_k < stoch_oversold && macd_pivot_up {
        let confidence = (buy_zone - price_position) / buy_zone; // Càng gần đáy càng tin
        return Some(Signal { direction: Long, confidence, scanner: MeanReversion });
    }
    
    // SHORT: Giá ở vùng trên (0.75 - 1.0) + Stoch overbought + MACD pivot down
    if price_position > sell_zone && stoch_k > stoch_overbought && macd_pivot_down {
        let confidence = (price_position - sell_zone) / (1.0 - sell_zone);
        return Some(Signal { direction: Short, confidence, scanner: MeanReversion });
    }
    
    None
}
```

**So với V2:** Không đòi giá vượt BB (xác suất 2.5%). Chỉ cần giá ở vùng 1/4 trên/dưới (xác suất ~25%).

### 3. Pullback Scanner (Thay thế TrendPullback)

```rust
fn scan_pullback(sensors, matrix, config) -> Option<Signal> {
    // Điều kiện xu hướng: ADX phải đủ mạnh
    let is_trending = matrix.trend_strength[i] > adx_min_thresh;
    if !is_trending { return None; }
    
    // Giá hồi về KAMA trong khi xu hướng vẫn còn
    let kama_dist = (close - kama).abs() / close;
    let is_near_kama = kama_dist < kama_tolerance;
    
    // Stoch xác nhận vùng mua/bán
    let is_long_setup = close > kama && stoch_k < stoch_thresh && macd_pivot_up;
    let is_short_setup = close < kama && stoch_k > (1.0 - stoch_thresh) && macd_pivot_down;
    
    if is_near_kama && is_long_setup {
        let confidence = matrix.trend_strength[i]; // Xu hướng càng mạnh, pullback càng tin cậy
        return Some(Signal { direction: Long, confidence, scanner: Pullback });
    }
    // ... tương tự cho Short
    None
}
```

### 4. Momentum Scanner (Thay thế MomentumIgnition — HIỆN LÀ DEAD CODE)

```rust
fn scan_momentum(sensors, matrix, config) -> Option<Signal> {
    // RSI phải ở vùng mạnh nhưng chưa cực đoan
    let rsi_ok_long = matrix.momentum[i] > rsi_thresh && matrix.momentum[i] < rsi_climax;
    
    // MACD phải đang tăng tốc (không chỉ tăng, mà tăng NHANH HƠN)
    let is_accelerating = macd_hist[i] > macd_hist[i-n] * accel_factor;
    
    // Volume xác nhận
    let vol_ok = matrix.volume_force[i] > vol_spike_min;
    
    if rsi_ok_long && is_accelerating && vol_ok {
        let confidence = (matrix.momentum[i] - rsi_thresh) / (rsi_climax - rsi_thresh);
        return Some(Signal { direction: Long, confidence, scanner: Momentum });
    }
    None
}
```

**So với V2:** Momentum Scanner giờ THỰC SỰ ĐƯỢC CHẠY, không bị Dead Code.

### 5-6. Reversal + Anomaly Scanner
Giữ nguyên logic nhưng không cần qua Regime Scoring nữa. Z-Score cực đoan + MACD collapse = Signal trực tiếp.

---

## Arbiter (Trọng Tài)

```rust
fn arbitrate(signals: Vec<Signal>, tick: &TickContext, config: &Config) -> Option<TradeAction> {
    if signals.is_empty() || tick.position().is_open() { return None; }
    
    // 1. Lọc mâu thuẫn: Nếu có cả Long và Short → Chỉ giữ cái confidence cao hơn
    // 2. Xếp hạng theo confidence giảm dần
    signals.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    
    let best = &signals[0];
    
    // 3. Tính toán Risk (SL/TP/Size)
    let sl_dist = atr * sl_factor_for(best.scanner);
    let size = calculate_risk_based_size(equity, risk_pct, sl_dist, leverage);
    
    if size <= 0.0 { return None; } // Không đủ vốn
    
    Some(TradeAction { direction: best.direction, size, sl_dist, scanner: best.scanner })
}
```

---

## So sánh Số liệu Dự kiến

| Metric | V2 (Regime-First) | V3 (Signal-First) |
|:-------|:-------------------|:-------------------|
| Trades/tháng | 30-43 | 150-300 (ước tính) |
| Playbooks hoạt động | 2/7 | 6/6 |
| Thời gian có vị thế | ~5% | ~30-50% |
| Số genes | ~100 | ~50-60 |
| Tầng lọc | 4 (nối tiếp) | 2 (scan + arbiter) |
| Code lines (logic) | ~700 | ~400 |
| DRY violations | 14 | 0 |

---

## Lộ trình Chuyển đổi V2 → V3

### Phase 1: Giữ nguyên code V2, sửa bugs critical
- Sửa Parabolic unit mismatch
- Kích hoạt MomentumIgnition
- Sửa Range trigger (price position thay vì vượt BB)
- Thêm Risk-Based Sizing

### Phase 2: Viết Scanner Layer mới
- Tạo `src/strategy/scanners/` module
- Port logic từ `tactical.rs` sang từng Scanner
- Mỗi Scanner trả về `Option<Signal>` thay vì tự gọi `tick.buy()`

### Phase 3: Viết Arbiter + Loại bỏ Regime
- Gộp Environment Fuse vào confidence multiplier
- Xóa `regime.rs` và `playbook.rs`
- Arbiter chọn Signal tốt nhất

### Phase 4: Tối ưu Optimizer
- Giảm genes còn ~50
- Freeze các Scanner kém hiệu quả
- Chạy lại GA trên dataset dài hơn (3-6 tháng)
