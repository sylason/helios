# 06 — Đánh Giá Tầng Risk Management

## 1. Tổng Quan

Risk management gồm 3 cơ chế:
1. **Initial SL** (tại entry): `atr_raw × base_sl_atr × playbook_sl_factor`
2. **Breakeven Lock**: Khi profit ≥ RR target → dời SL về entry price
3. **Trailing Stop**: Peak-based, elastic theo playbook

## 2. Cấu Trúc SL Factor

| Playbook | SL Factor DNA | Vai trò |
|----------|-------------|---------|
| Squeeze | squeeze_sl_factor | Entry + Trailing |
| Trend | trend_sl_factor | Entry + Trailing |
| Momentum | momentum_sl_factor | Entry + Trailing |
| Range | range_sl_factor | Entry + Trailing |
| Reversal | reversal_sl_factor | Entry + Trailing |
| Anomaly | anomaly_sl_factor | Entry + Trailing |
| Absorption | **0.7 hardcoded** | 🔴 Entry only, no DNA |

## 3. Trailing Stop Logic

```rust
let multiplier = match memory.opening_playbook {
    AnomalyFade | Squeeze* => tsl_atr_loose,       // Nới rộng
    TrendPullback | MomentumIgnition => {
        if parabolic => tsl_atr_tight              // Siết chặt
        else => tsl_atr_normal                      // Bình thường
    },
    _ => tsl_atr_normal                             // Default
};

// new_sl = peak ± (smooth_atr × multiplier)
```

## 4. 🔴 Phát Hiện Vấn Đề

### ISSUE-K1: `smooth_atr` = Giá Trị Tức Thời, Không Smooth

```rust
// mod.rs dòng 294
self.memory.volatility_benchmark = current_atr;

// risk.rs dòng 23
let smooth_atr = memory.volatility_benchmark;
```

- Tên biến là `smooth_atr` / `volatility_benchmark` nhưng thực tế nó chỉ là **ATR của nến hiện tại**
- KHÔNG có cơ chế EMA/SMA nào để làm mượt → SL nhảy theo từng nến
- Trên khung 1-phút, ATR dao động rất mạnh → Trailing SL bị giật liên tục

### ISSUE-K2: `peak_price` Tracking Dùng High/Low Chứ Không Phải Close

```rust
// mod.rs dòng 299-307
let high = tick.highs()[i];
let low = tick.lows()[i];
if pos.is_long {
    self.memory.peak_price = Some(current_peak.max(high));
} else {
    self.memory.peak_price = Some(current_peak.min(low));
}
```

- Peak theo High/Low → dễ bị ảnh hưởng bởi "râu nến" (wicks)
- Trên khung 1-phút, wicks rất phổ biến → peak có thể bị đẩy cao/thấp bất thường
- Trailing SL bám theo peak sai → bảo vệ lời kém

### ISSUE-K3: Breakeven Target RR Quá Cao

```
be_profit_target_rr DNA: [min không rõ, max không rõ]
Rank 1 result: be_profit_target_rr = 6.62
```

- Cần đạt 6.62× risk reward trước khi dời về hòa vốn
- Với base_sl_atr ≈ 0.89 và ATR khung 1-phút rất nhỏ → mục tiêu RR cao
- **Breakeven gần như không bao giờ kích hoạt** cho các lệnh bình thường
- Chỉ các lệnh "siêu may mắn" (như trade #58 +67%) mới đạt được

### ISSUE-K4: Không Có Time-Based Exit

- Không có cơ chế thoát lệnh khi lệnh mở quá lâu (>500 nến = >8 giờ)
- Nhiều lệnh trong output mở 800-900 nến → giữ tiền chết, bỏ lỡ cơ hội
- Trade #12: mở 857 nến ≈ 14 giờ → trong thời gian đó không vào lệnh nào khác

## 5. Kết Luận

> Risk management hoạt động cơ bản đúng nhưng:
> 1. `smooth_atr` không smooth → trailing giật
> 2. Peak tracking dùng high/low → nhạy với wicks
> 3. Breakeven target quá cao → gần như disabled
> 4. Không có time-stop → lệnh "zombie" chiếm slot
