# Sự Ngắt Kết Nối Với Bộ Gen (DNA Disconnect) & Magic Numbers

Đây là sự thụt lùi (Regression) nghiêm trọng nhất của phiên bản `core_v3` so với `core_v2`.

## 🚨 Vấn Đề Hiện Tại
Hàm cốt lõi của mọi Scanner được định nghĩa như sau:
```rust
fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3) -> Option<Proposal>;
```
Hàm này **hoàn toàn vắng mặt `HeliosConfig`**. 

## Hậu Quả: Tràn Ngập Các "Con Số Ma Thuật" (Magic Numbers)
Vì không thể lấy các tham số tối ưu từ DNA của cấu hình, người code đã phải điền tay toàn bộ các ngưỡng giới hạn (Thresholds). Hệ lụy là Thuật toán Optimizer (Prospector) đã trở nên vô dụng hoàn toàn với các chiến thuật này, do các thông số trên DNA không còn liên kết tới logic thực thi.

### Bằng chứng 1: `trend_pb.rs`
```rust
// Thay vì lấy: config.tactical_sniper.pullback.stoch_pullback_thresh
if close <= kama * 1.005 && stoch_k < 30.0 && stoch_k > stoch_d {
    raw_score = 0.8 + (30.0 - stoch_k) / 150.0;
}
```
Các con số `1.005`, `30.0`, `150.0` là các con số được Fix cứng. Dù bộ Optimizer có tìm ra chuỗi Gen tốt hơn thì Code vẫn phớt lờ.

### Bằng chứng 2: `range_osc.rs`
```rust
// Mất liên kết tới config.tactical_sniper.range.stoch_oversold
if close <= bb_l * 1.002 && stoch_k < 20.0 && stoch_k > stoch_d {
    raw_score = 0.7 + (20.0 - stoch_k) / 100.0;
}
```

### Bằng chứng 3: `momentum_ignition.rs`
```rust
if is_breakout && vol_force > 1.5 {
    raw_score = 0.7 + (vol_force - 1.5) * 0.1;
}
```

## 🛠️ Giải Pháp Đề Xuất
1. Sửa lại Trait `TacticalScanner` trong `arbiter.rs` để truyền thêm Context về cấu hình:
```rust
fn scan(&self, 
        config: &HeliosConfig, // Bổ sung cực kỳ cấp bách
        tick: &TickContext, 
        sensors: &HeliosSensors, 
        matrix: &MarketMatrixV3) -> Option<Proposal>;
```
2. Cập nhật lại toàn bộ nội dung của các Scanner để lấy thông số Threshold trực tiếp từ `config.tactical_sniper...`.
3. Tái xây dựng lại công thức tính `raw_score` sao cho nó tỷ lệ thuận với mức độ thỏa mãn của Pattern so với Threshold từ DNA.
