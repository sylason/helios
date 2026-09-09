# ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-3.4 (CACHE COLLISION TRONG KAMA SLOPE)

> **Mã định danh:** `FLAW-3.4`  
> **Mức độ nghiêm trọng:** `CRITICAL` (Xung đột bộ nhớ Cache chỉ báo, trả về dữ liệu sai chu kỳ ATR giữa các cá thể GA và các khung thời gian)  
> **Tập tin liên quan:**  
> - `ferro_trade/src/indicators/identifier.rs` (Định danh `IndicatorType::KamaSlope`)  
> - `ferro_trade/src/indicators/factory/bulk.rs` (MTF Factory Wrapper cho KAMA Slope)  

---

## 1. Bản Chất Kỹ Thuật Của Lỗi Cache Collision

### 1.1. Cơ Chế Đánh Khóa Cache (CacheKey) Trong `ferro_trade`
Trong `ferro_trade/src/indicators/identifier.rs`:
```rust
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    pub timeframe: crate::data::Timeframe,
    pub inputs: SmallVec<[DataColumn; 4]>,
    pub indicator: IndicatorType,
}
```
Bộ nhớ đệm toàn cục `GlobalCache` và `IndicatorManager` dựa hoàn toàn vào `CacheKey` (thông qua `Hash` và `Eq`) để tái sử dụng các chuỗi chỉ báo đã tính toán.

### 1.2. Lỗ Hổng Bỏ Quên Tham Số `atr_period`
Hiện tại, `IndicatorType::KamaSlope` được định nghĩa như sau:
```rust
// ferro_trade/src/indicators/identifier.rs:119
KamaSlope { er: usize, fast: usize, slow: usize, slope_period: usize },
```
Nhưng trong `BulkIndicatorFactory::kama_slope`:
```rust
// ferro_trade/src/indicators/factory/bulk.rs:77-88
fn kama_slope(&mut self, tf: Timeframe, label: &str, er_period: usize, fast: usize, slow: usize, slope_period: usize, atr_period: usize) -> (Arc<Vec<f64>>, Arc<Vec<f64>>) {
    let results = self.add_multi_indicator_mtf_typed(
        tf, &["Close", "High", "Low"],
        IndicatorType::KamaSlope { er: er_period, fast, slow, slope_period }, // <--- THIẾU atr_period!
        er_period.max(atr_period) + slope_period,
        move |inputs| {
            let (raw, adj) = calculate::bulk::kama_slope(inputs[0], inputs[1], inputs[2], er_period, fast, slow, slope_period, atr_period) ...
        }
    );
    (results[0].clone(), results[1].clone())
}
```

### 1.3. Hậu Quả Thực Tế
Chỉ báo `kama_slope` trả về cặp giá trị: `(raw_slope, adj_slope)` với:
$$\text{adj\_slope} = \frac{\text{raw\_slope}}{\text{ATR}(\text{atr\_period})}$$
Khi thuật toán Di truyền (GA Optimizer) tối ưu hóa gen `indicators.atr.period` (hoặc khi hai chiến lược/khung thời gian dùng cùng `er_period, fast, slow, slope_period` nhưng khác `atr_period`):
1. Cá thể A chạy với `atr_period = 14` $\rightarrow$ Tính toán và lưu kết quả vào Cache với Key `{ er, fast, slow, slope_period }`.
2. Cá thể B chạy với `atr_period = 50` $\rightarrow$ Tra cứu CacheKey, thấy trùng khớp 100% $\rightarrow$ **Lấy luôn kết quả của cá thể A (`atr_period = 14`)**!
3. Cá thể B bị tính toán sai lệch hoàn toàn `kama_slope_adj`, làm tê liệt bộ lọc xu hướng `trend_dir` và scanner `trend_pb`.

---

## 2. Giải Pháp Triển Khai

1. Bổ sung `atr_period: usize` vào biến thể `IndicatorType::KamaSlope`:
   ```rust
   KamaSlope { er: usize, fast: usize, slow: usize, slope_period: usize, atr_period: usize },
   ```
2. Cập nhật nơi khởi tạo trong `BulkIndicatorFactory::kama_slope`:
   ```rust
   IndicatorType::KamaSlope { er: er_period, fast, slow, slope_period, atr_period },
   ```
3. Đảm bảo tính toán độc lập 100%: Hai lệnh gọi với `atr_period` khác nhau sẽ tạo ra 2 `CacheKey` khác nhau, không bao giờ bị ghi đè hay lấy nhầm dữ liệu.
