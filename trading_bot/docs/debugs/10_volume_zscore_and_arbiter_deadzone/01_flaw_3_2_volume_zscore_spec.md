# 📐 ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-3.2 (VOLUME Z-SCORE CHUẨN QUANT)

> **Mã lỗi kiểm toán:** `FLAW-3.2`  
> **Mức độ rủi ro:** **CRITICAL** (Sai lệch toán học cốt lõi khiến 4 Scanner và Ma trận Bối cảnh nhận diện sai dòng tiền).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/10_volume_zscore_and_arbiter_deadzone/01_flaw_3_2_volume_zscore_spec.md`  

---

## 1. Bản Chất Vấn Đề & Bằng Chứng Mã Nguồn

### 1.1. Hiện trạng lỗi trong Code
Trong [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:94-101`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L94-L101):
```rust
// VOLUME FORCE CHUẨN QUANT: 0.5 * Tanh(Vol_Z_Score) + 0.5 * (MFI / 100)
let vol_z_scale = normalization::tanh_scale(&sensors.z_score, 0.5);
let mut volume_force = Vec::with_capacity(len);
for i in 0..len {
    let mfi_norm = (sensors.mfi[i] / 100.0).max(0.0).min(1.0);
    let vf = 0.5 * (vol_z_scale[i].abs()) + 0.5 * mfi_norm;
    volume_force.push(vf.max(0.0).min(1.0));
}
```

Truy ngược nguồn gốc của `sensors.z_score` trong [`core_v3/mod.rs:144`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L144):
```rust
s.z_score = ctx.z_score(ind.kama.timeframe, "ZScore", ind.z_score_lookback);
```

Và định nghĩa trong [`ferro_trade/src/indicators/factory/bulk.rs:291-296`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/factory/bulk.rs#L291-L296):
```rust
fn z_score(&mut self, tf: Timeframe, label: &str, period: usize) -> Arc<Vec<f64>> {
    self.add_indicator_mtf(tf, &["Close"], label, &[period as f64], period,
        move |inputs| {
            let len = inputs[0].len();
            vec![calculate::bulk::z_score(inputs[0], period).unwrap_or_else(|_| vec![f64::NAN; len])]
        })
}
```

👉 **Bằng chứng rõ ràng:** `ctx.z_score` tính toán Z-Score trên cột `"Close"` (Giá đóng cửa). Khi đưa vào `matrix.rs`, code đã lấy Z-Score của Giá để tính `Volume Force`!

### 1.2. Hậu quả thực tế
1. **Ảo giác thanh khoản:** Khi giá biến động mạnh nhưng volume cạn kiệt (Fake pump/dump), `volume_force` lại bị thổi phồng lên cao, khiến bot kích hoạt vào lệnh bừa bãi.
2. **Mù lòa hấp thụ (Absorption Blindness):** Khi cá mập gom hàng với khối lượng đột biến nhưng giá đi ngang, `Z_price ≈ 0` $\rightarrow$ `volume_force ≈ 0`, làm tê liệt scanner `absorption.rs`.
3. **Méo mó bối cảnh:** `w_volatile_raw = chaos_ratio * volume_force` và `vol_exp_score` bị tính sai hoàn toàn.

---

## 2. Thiết Kế Kỹ Thuật Giải Quyết

### 2.1. Cập nhật `ferro_trade`
Thêm hàm tính Z-Score trên cột Khối lượng `"Volume"` vào `BulkIndicatorFactory`:
```rust
fn z_score_volume(&mut self, tf: Timeframe, label: &str, period: usize) -> Arc<Vec<f64>> {
    self.add_indicator_mtf(tf, &["Volume"], label, &[period as f64], period,
        move |inputs| {
            let len = inputs[0].len();
            vec![calculate::bulk::z_score(inputs[0], period).unwrap_or_else(|_| vec![f64::NAN; len])]
        })
}
```

### 2.2. Cập nhật `HeliosSensors` (`sensors.rs`)
Bổ sung trường lưu trữ chuyên biệt cho Khối lượng:
```rust
pub struct HeliosSensors {
    // ...
    pub z_score: Arc<Vec<f64>>,       // Z-Score Giá (Dành cho Reversal & Anomaly)
    pub vol_z_score: Arc<Vec<f64>>,   // Z-Score Khối Lượng (Dành cho Volume Force & Absorption)
    // ...
}
```

### 2.3. Cập nhật Khởi tạo Cảm biến (`core_v3/mod.rs`)
```rust
s.z_score = ctx.z_score(ind.kama.timeframe, "PriceZScore", ind.z_score_lookback);
s.vol_z_score = ctx.z_score_volume(ind.kama.timeframe, "VolZScore", ind.z_score_lookback);
```

### 2.4. Cập nhật Ma Trận Thị Trường (`matrix.rs`)
```rust
// TÍNH VOLUME FORCE TỪ ĐÚNG Z-SCORE KHỐI LƯỢNG
let vol_z_scale = normalization::tanh_scale(&sensors.vol_z_score, 0.5);
let mut volume_force = Vec::with_capacity(len);
for i in 0..len {
    let mfi_norm = (sensors.mfi[i] / 100.0).max(0.0).min(1.0);
    let vf = 0.5 * (vol_z_scale[i].abs()) + 0.5 * mfi_norm;
    volume_force.push(vf.max(0.0).min(1.0));
}
```

---

## 3. Kế Hoạch Kiểm Thử (Verification Plan)
- [ ] **Unit Test 1 (`ferro_trade`):** Kiểm tra `z_score_volume` với mảng volume có đột biến (spike) $10\times$, xác nhận Z-Score $> 3.0$.
- [ ] **Unit Test 2 (`helios`):** Kiểm tra `volume_force` trong `matrix.rs` khi giá phẳng nhưng volume tăng vọt, xác nhận `volume_force > 0.70`.
- [ ] **Biên dịch toàn hệ sinh thái:** `cargo test --package helios --lib` và `cargo test --package ferro_trade --lib`.
