# 📑 ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-3.3 (CHAOS RATIO NAN RESILIENCE)

> **Mục tiêu:** Khôi phục chuẩn toán học `NaN` cho chuỗi warmup của `chaos_ratio` trong `ferro_trade`, đồng thời sửa cơ chế fallback trong `matrix.rs` để không làm méo mó Simplex Normalization.

---

## 1. BỐI CẢNH & NGUYÊN NHÂN GỐC

### 1.1. Tại `ferro_trade/src/indicators/calculate/bulk.rs`
- **Hiện trạng:**
  ```rust
  if clean_natr.len() < sma_period {
      return Ok(vec![1.0; clean_natr.len()]);
  }
  // ...
  if v.is_nan() || sma_val.is_nan() || sma_val < 1e-9 { 
      1.0 
  } else { 
      v / sma_val 
  }
  ```
- **Hậu quả:** 
  - Engine `ferro_trade::IndicatorManager` đếm số lượng `NaN` liên tục ở đầu mảng (`actual_warmup`) để xác định `warmup_period`.
  - Việc ép về `1.0` khiến `actual_warmup = 0`, làm mất cơ chế bảo vệ warmup tự động của engine.

### 1.2. Tại `strategies_backtests/helios/src/strategy/core_v3/matrix.rs`
- **Hiện trạng:**
  ```rust
  let chaos_ratio = normalization::rolling_percentile(&sensors.chaos_ratio, config.matrix.squeeze_lookback);
  // ...
  let c_ratio = if chaos_ratio[i].is_nan() { 1.0 } else { chaos_ratio[i] };
  ```
- **Hậu quả:**
  - `chaos_ratio` trong `matrix.rs` là **Rolling Percentile** ($[0.0, 1.0]$).
  - Gán `c_ratio = 1.0` khi gặp `NaN` vô tình gán mức **100% Bão Tố (Maximum Chaos)** cho các nến chưa đủ nến warmup.
  - `w_volatile_raw = c_ratio * v_force` bị thổi phồng, đè bẹp `w_range_raw` và làm lệch vector Simplex.

---

## 2. GIẢI PHÁP THIẾT KẾ QUANT

### 2.1. Chuẩn hóa tại `bulk.rs`
- Các phần tử warmup (`clean_natr.len() < sma_period` hoặc `v.is_nan() || sma_val.is_nan() || sma_val < 1e-9`) **BẮT BUỘC TRẢ VỀ `f64::NAN`**.
- Chỉ tính `v / sma_val` khi cả 2 giá trị đều hợp lệ và `sma_val >= 1e-9`.

### 2.2. Chuẩn hóa tại `matrix.rs`
- Tại dòng tính `ano_score`:
  ```rust
  let c_val = if chaos_ratio[i].is_nan() { 0.0 } else { chaos_ratio[i] };
  let ano_score = 0.5 * z_abs + 0.5 * c_val;
  ```
- Tại dòng tính `c_ratio`:
  ```rust
  let c_ratio = if chaos_ratio[i].is_nan() { 0.0 } else { chaos_ratio[i] };
  ```
- Ý nghĩa Quant: Nếu chưa đủ dữ liệu tính Chaos Percentile, coi như thị trường **không có bão (0% Chaos)** thay vì báo động giả 100% bão tố.

---

## 3. KẾ HOẠCH KIỂM THỬ
1. Cập nhật unit test `test_chaos_ratio_no_nan_short_and_warmup` thành `test_chaos_ratio_returns_nan_during_warmup`: xác nhận các nến đầu là `NaN` và các nến sau là số dương.
2. Kiểm tra `cargo test --package ferro_trade --lib` đảm bảo 100% pass.
3. Kiểm tra `cargo test --package helios --lib` đảm bảo ma trận bối cảnh tính toán chính xác.
