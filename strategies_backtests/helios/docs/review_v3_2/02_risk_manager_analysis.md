# 🛡️ 02 — Phân Tích Risk Manager (risk.rs)

> File: `src/strategy/core_v3/risk.rs`

---

## I. TRAILING STOP — Progressive Decay Logic

### Code Logic Hiện Tại (Tóm Lược)

```rust
// Bước 1: Đếm "effective_stagnant_bars" từ đỉnh (peak_bar) đến hiện tại
// Chỉ đếm nến mà squeeze_multiplier <= squeeze_thresh
for j in peak_bar..=i {
    if matrix.squeeze_multiplier[j] <= squeeze_thresh {
        effective_stagnant_bars += 1;
    }
}

// Bước 2: Tính số chu kỳ decay
let decay_cycles = effective_stagnant_bars / time_countdown_bars;

// Bước 3: Nếu lệnh lỗ VÀ đã qua 1 chu kỳ -> đóng lệnh ngay
if decay_cycles > 0 && current_pnl <= 0.0 {
    tick.close(None);
    continue;
}

// Bước 4: Progressive Decay (siết SL theo chu kỳ)
if decay_cycles > 0 {
    let progressive_factor = tighten_step_multiplier.powi(decay_cycles as i32);
    current_multiplier *= progressive_factor;
    
    // Mega-trend protection: nếu PnL > mega_pnl_trigger, giữ TSL ở mức normal
    let min_tight = if is_trend_or_momentum && current_pnl > mega_pnl_trigger {
        risk_cfg.tsl_atr_normal
    } else {
        risk_cfg.tsl_atr_tight
    };
    
    current_multiplier = current_multiplier.max(min_tight);
}
```

---

## II. LỖI: Mega-Trend Protection Vô Hiệu Hóa Decay

### Mô Tả Vấn Đề

Trade #5 (LONG MOMENTUM_IGN):
- Duration: **16658 nến**
- PnL: +29.06% — đây là lớn hơn bất kỳ `mega_trend_pnl_trigger` hợp lý nào

**Kịch bản xảy ra:**
```
decay_cycles = effective_stagnant_bars / time_countdown_bars
           = (giả sử) 5000 / 300 = 16 chu kỳ

progressive_factor = 0.8^16 = 0.028 (rất nhỏ)
current_multiplier = tsl_atr_normal * 0.028 = gần 0

→ Nhưng mega_trend protection giữ min_tight = tsl_atr_normal
→ current_multiplier = tsl_atr_normal (không siết được)
```

**Kết quả:** Khi `mega_trend_pnl_trigger` được kích hoạt (PnL đủ lớn), Progressive Decay bị **hoàn toàn vô hiệu hóa** — TSL không bao giờ siết hơn mức `tsl_atr_normal`.

### Đề Xuất Sửa

Mega-trend protection nên có **giới hạn trên về số chu kỳ**, không phải vô hạn:
```rust
// Sau N chu kỳ mega-trend, bắt đầu siết từ từ thay vì dùng tsl_atr_normal cứng
let mega_cycles_grace = 10; // Ví dụ: cho phép 10 chu kỳ không siết
let min_tight = if is_trend_or_momentum && current_pnl > mega_pnl_trigger 
    && decay_cycles <= mega_cycles_grace {
    risk_cfg.tsl_atr_normal
} else {
    risk_cfg.tsl_atr_tight
};
```

---

## III. LỖI: Không Có Recovery Period Trong V3

### So Sánh V2 vs V3

| Feature                     | V2 (mod.rs)              | V3 (mod.rs)    |
|-----------------------------|--------------------------|----------------|
| Recovery Period Counter     | ✅ `recovery_period_counter` | ❌ Không có |
| Volatility Benchmark        | ✅ `volatility_benchmark` | ❌ Không có  |
| Peak Price Tracking         | ✅ `memory.peak_price`   | ✅ Tích hợp trong risk.rs |
| Boxing Recovery             | ✅ Sau lệnh thua, đếm ngược | ❌ Không có |

### Hậu Quả Quan Sát Được

Bars 31939–31974 (35 nến):
- Trades #7, 8, 9, 10, 12, 14 đều có duration 2–10 nến — vào lệnh rồi thua ngay
- Sau mỗi lệnh thua, hệ thống **lập tức scan lại và vào lệnh mới**
- Không có khoảng thời gian hồi phục: `pos.is_open()` → `return` chỉ chặn khi có lệnh đang mở

### Vấn Đề Sâu Hơn

V3 `mod.rs` dòng 139-140:
```rust
if pos.is_open() {
    return; // Chặn nhồi lệnh chỉ khi có vị thế đang mở
}
```

Khi lệnh đóng: **không có cooldown nào**. Hệ thống coi mỗi nến đóng là cơ hội scan bình thường.

### Đề Xuất Sửa

Thêm `cooldown_counter` vào `HeliosStrategyV3` hoặc dùng một field `usize`:
```rust
// Trong mod.rs, sau khi lệnh đóng:
if !pos.is_open() && self.cooldown_bars > 0 {
    self.cooldown_bars -= 1;
    return; // Không scan trong thời gian hồi phục
}
```
Hoặc đơn giản hơn: dùng `trade.exit_bar` để kiểm tra khoảng cách tối thiểu.

---

## IV. LỖI: Peak Scan Complexity O(n²) Theo Duration

### Mô Tả

```rust
// risk.rs dòng 94-114: vòng lặp tìm peak_val và peak_bar
for j in entry_bar..=i {
    if highs[j] > max_high { ... }
}
// risk.rs dòng 118-122: vòng lặp đếm effective_stagnant_bars
for j in peak_bar..=i {
    if matrix.squeeze_multiplier[j] <= squeeze_thresh {
        effective_stagnant_bars += 1;
    }
}
```

Với trade duration = **16658 nến**, mỗi nến mới phải scan qua toàn bộ 16658 nến cũ để tìm peak.
**Tổng tính toán:** O(n²) ≈ 277 triệu phép so sánh chỉ cho 1 lệnh.

### Hậu Quả

- Làm chậm đáng kể backtest khi có lệnh duration dài
- Không phải lỗi logic nhưng là vấn đề hiệu năng cần cải thiện

### Đề Xuất Sửa

Cache `peak_val` và `peak_bar` trong struct bằng cách thêm field state vào `HeliosStrategyV3`:
```rust
// Dùng HashMap<trade_id, (peak_val, peak_bar, effective_stagnant)>
// Cập nhật mỗi nến thay vì scan lại từ đầu
```

---

## V. THIẾU: Base SL ATR Không Được Dùng

`risk_cfg.base_sl_atr` được khai báo trong `RiskStopParams` nhưng trong `risk.rs`:
```rust
// get_initial_sl dùng multiplier truyền từ ngoài
pub fn get_initial_sl(tick, sensors, is_long, multiplier: f64) -> f64 {
    price ± (current_atr * multiplier)
}

// Và trong mod.rs:
RiskManager::get_initial_sl(tick, &self.sensors, is_long,
    self.config.risk_management.stop_management.tsl_atr_normal
)
```

`base_sl_atr` không được dùng. `tsl_atr_normal` được dùng thay thế — nhưng đây là **trailing multiplier, không phải initial SL multiplier**. Hai khái niệm khác nhau.

---

## Bảng Tổng Hợp Risk Manager

| ID   | Vấn Đề                                        | Mức Độ   | Sửa Ở Đâu   |
|------|-----------------------------------------------|----------|-------------|
| R-01 | Mega-trend protection vô hiệu hóa decay mãi mãi | 🔴 NGUY  | risk.rs     |
| R-02 | Không có Recovery Period sau lệnh thua         | 🔴 NGUY  | mod.rs      |
| R-03 | Peak scan O(n²) cho long-duration trades       | 🟡 HIỆU NĂNG | risk.rs  |
| R-04 | `base_sl_atr` dead config — không được dùng   | 🟠 CẢI TIẾN | mod.rs + risk.rs |
