# 📋 DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO) - TẦNG CẢM BIẾN (SENSORS LAYER)

**Module:** `Sensors & Indicator Pipeline`  
**Đường dẫn tài liệu kiểm toán:** [`strategies_backtests/helios/docs/audit_01/07_sensors/technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/07_sensors/technical_audit.md)  
**Ngày cập nhật:** 2026-09-19  

---

## 🎯 TỔNG QUAN PHÂN BỔ CÔNG VIỆC

| Mức Độ Ưu Tiên | Số Lượng Task | Trạng Thái | Mô Tả Trọng Tâm |
| :---: | :---: | :---: | :--- |
| **P0 (Critical)** | 2 | 🔴 Chưa xử lý | Triệt tiêu bẫy Warmup 144,000 nến và loại bỏ nguy cơ Panic crash của `RangeOsc`. |
| **P1 (Major)** | 2 | 🟠 Chưa xử lý | Dọn dẹp chỉ báo tính thừa (`rsi_reversal`, `stoch_trend_d`) và sửa lỗi nuốt chửng NaN trong Matrix. |
| **P2 (Medium)** | 2 | 🟡 Chưa xử lý | Chống Catastrophic Cancellation trong sliding window ER và sửa float epsilon dedup mismatch. |
| **P3 (Minor)** | 1 | 🔵 Chưa xử lý | Tinh giản 17 trường rác/zombie trong struct `HeliosSensors`. |

---

## 🔴 ƯU TIÊN P0: LỖ HỔNG CHÍ TỬ & BÃY CRASH HỆ THỐNG

- [ ] **Task 01: Khắc chế bẫy Warmup Z-Score trên khung 4h (SEN-01)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/config.rs:L66-L69`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L66-L69)  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L216-L217`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L216-L217)  
    + [`ferro_trade/src/backtest/context.rs:L173-L177`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs#L173-L177)
  - **Vấn đề:**  
    DNA cho phép `z_score_lookback` lên tới 600. Khi GA chọn `kama.timeframe = 4h`, bộ dữ liệu 63,200 nến 1m chỉ có ~263 nến 4h. Phép tính $600 > 263$ làm hàm `sma` trả về `Err`, toàn bộ mảng Z-Score biến thành `NaN`. Hệ thống đếm NaN và đẩy Warmup lên $63,199$ nến, làm backtest thực thi 0 lệnh.
  - **Giải pháp triển khai:**
    1. Trong `core_v3/config.rs`, hạ trần DNA: `#[dna(min = 20.0, max = 200.0, step = 10.0)]` cho `z_score_lookback`.
    2. Trong `core_v3/mod.rs::init()`, thêm bộ kẹp an toàn:
       ```rust
       let resampled_ohlcv = ctx.cache.get_resampled_ohlcv(ind.kama.timeframe);
       let max_safe_lookback = resampled_ohlcv.map(|df| df.close.len() / 2).unwrap_or(100);
       let safe_z_lookback = ind.z_score_lookback.min(max_safe_lookback).max(10);
       s.z_score = ctx.z_score(ind.kama.timeframe, "ZScore", safe_z_lookback);
       s.vol_z_score = ctx.z_score_volume(ind.kama.timeframe, "VolZScore", safe_z_lookback);
       ```
    3. Trong `ferro_trade::IndicatorManager`, nếu một chỉ báo có `actual_warmup >= len`, phát cảnh báo `warn!` và không nâng `self.warmup_period` nuốt trọn dữ liệu.
  - **Tiêu chí nghiệm thu (Acceptance Criteria):**  
    + Chạy backtest trên chuỗi nến ngắn 10,000 bars với cấu hình 4h không bao giờ bị dội Warmup kẹp trần.
    + GA không sinh ra cá thể bị chết vòng lặp.

---

- [ ] **Task 02: Triệt tiêu nguy cơ Panic Index Out-of-Bounds trong `range_osc.rs` (SEN-02)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/sensors.rs:L47-L48`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs#L47-L48)  
    + [`strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs:L70-L71`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L70-L71)
  - **Vấn đề:**  
    Trong `core_v3/mod.rs`, cuộc gọi tính `stoch_range` đã bị comment out (mảng rỗng `len = 0`). Tuy nhiên trong `scanners/range_osc.rs`, mã nguồn vẫn truy cập trực tiếp `sensors.stoch_range_k[i]`. Nếu scanner này được bật lại trong Arbiter, bot sẽ crash tức thì.
  - **Giải pháp triển khai:**
    1. Sửa `scanners/range_osc.rs` chuyển sang dùng kênh an toàn: `sensors.stoch_trend_k.get(i).copied().unwrap_or(50.0)`.
    2. Dọn dẹp dứt điểm 2 trường `stoch_range_k` và `stoch_range_d` khỏi `HeliosSensors`.
  - **Tiêu chí nghiệm thu:**  
    + Bật `arbiter.add_scanner(Box::new(scanners::range_osc::RangeOscScanner))` không phát sinh bất kỳ Panic nào.

---

## 🟠 ƯU TIÊN P1: LÃNG PHÍ TÀI NGUYÊN & LỖ HỔNG XỬ LÝ SỐ LIỆU

- [ ] **Task 03: Dọn dẹp chỉ báo ma `rsi_reversal` và `stoch_trend_d` (SEN-03)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:L182-L183, L201`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L182-L183)  
    + [`strategies_backtests/helios/src/strategy/sensors.rs:L40, L46`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs#L40)  
    + [`strategies_backtests/helios/src/strategy/core_v3/config.rs:L34`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L34)
  - **Vấn đề:**  
    `rsi_reversal` và `stoch_trend_d` được tính toán trên đa khung thời gian nhưng không có bất kỳ logic nào trong 4 Playbook đọc giá trị này.
  - **Giải pháp triển khai:**
    1. Xóa bỏ cuộc gọi `ctx.rsi(ind.rsi_reversal.timeframe, ...)` trong `core_v3/mod.rs`.
    2. Bỏ `rsi_reversal` khỏi DNA `IndicatorsConfig` (giảm bớt số chiều không gian tìm kiếm GA).
    3. Xóa trường `rsi_reversal` và `stoch_trend_d` khỏi `HeliosSensors`.
  - **Tiêu chí nghiệm thu:**  
    + Thời gian chạy hàm `init()` trong mỗi epoch GA giảm ~10-15%, RAM tiết kiệm 2 mảng `Vec<f64>` độ dài 63,200 phần tử.

---

- [ ] **Task 04: Sửa bẫy nuốt chửng NaN trong `MarketMatrixV3::from_sensors` (SEN-04)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:L107-L125`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L107-L125)
  - **Vấn đề:**  
    `unwrap_or(50.0)` trên `sensors.mfi.get(i)` không bắt được `NaN` vì `.get(i)` trả về `Some(f64::NAN)`. Phép tính `(NaN).max(0.0)` trong Rust ép giá trị về `0.0` thay vì giá trị trung tính `50.0 / 100.0 = 0.50`. Tương tự với `vol_z_scale` và `price_z_score`.
  - **Giải pháp triển khai:**
    Sử dụng kiểm tra `is_nan()` tường minh:
    ```rust
    let mfi_val = sensors.mfi.get(i).copied().unwrap_or(50.0);
    let mfi_clean = if mfi_val.is_nan() { 50.0 } else { mfi_val };
    let mfi_norm = (mfi_clean / 100.0).clamp(0.0, 1.0);

    let z_val = vol_z_scale.get(i).copied().unwrap_or(0.0);
    let z_clean = if z_val.is_nan() { 0.0 } else { z_val };
    let vf = 0.5 * z_clean.abs() + 0.5 * mfi_norm;
    flow_axis.push(vf.clamp(0.0, 1.0));
    ```
  - **Tiêu chí nghiệm thu:**  
    + Khi MFI đang trong giai đoạn warmup, `flow_axis` nhận giá trị trung tính 0.25 (từ MFI 0.50) thay vì bị sập về 0.0 giả tạo.

---

## 🟡 ƯU TIÊN P2: ĐỘ CHÍNH XÁC SỐ HỌC & GIẢM NHIỄU

- [ ] **Task 05: Chống sai số tích lũy trôi nổi (Catastrophic Cancellation) trong ER (SEN-05)**
  - **Tập tin liên quan:**  
    + [`ferro_trade/src/indicators/calculate/bulk.rs:L157-L167`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L157-L167)
  - **Vấn đề:**  
    Sliding window `volatility_sum = volatility_sum - old_diff + new_diff` tích lũy sai số dấu chấm động 64-bit qua hàng chục nghìn bước lặp, có thể trôi về số âm cực nhỏ khiến ER bị gán 0.0.
  - **Giải pháp triển khai:**
    1. Thêm bảo vệ không âm: `volatility_sum = (volatility_sum - old_diff + new_diff).max(0.0);`.
    2. Định kỳ mỗi 1,000 nến, tính lại `volatility_sum` bằng vòng lặp chuẩn để reset sai số làm tròn.
  - **Tiêu chí nghiệm thu:**  
    + Chạy kiểm thử trên chuỗi 1,000,000 nến tổng hợp, sai số giữa sliding window và brute-force sum luôn $< 10^{-12}$.

---

- [ ] **Task 06: Đồng bộ hóa Float Epsilon trong `rolling_percentile` (SEN-06)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/utils/normalization.rs:L76-L89`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/utils/normalization.rs#L76-L89)
  - **Vấn đề:**  
    `dedup_by` dùng `abs() < f64::EPSILON` trong khi `binary_search_by` dùng `total_cmp`. Nếu 2 giá trị lệch nhau $< \text{EPSILON}$, một giá trị bị xóa khiến `binary_search_by` thất bại, làm nến hợp lệ bị trả về `NaN`.
  - **Giải pháp triển khai:**
    Thay thế logic lọc trùng:
    ```rust
    unique_vals.dedup_by(|a, b| a.total_cmp(b).is_eq());
    ```
  - **Tiêu chí nghiệm thu:**  
    + Không còn bất kỳ nến nào sau giai đoạn warmup bị biến thành `NaN` trong kết quả của `rolling_percentile`.

---

## 🔵 ƯU TIÊN P3: TINH GIẢN KIẾN TRÚC & CLEAN CODE

- [ ] **Task 07: Tinh giản trường dư thừa trong `HeliosSensors` (SEN-07)**
  - **Tập tin liên quan:**  
    + [`strategies_backtests/helios/src/strategy/sensors.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs)
  - **Vấn đề:**  
    Struct `HeliosSensors` chứa 35 trường, trong đó có nhiều trường rác từ V2/V1 không còn sử dụng trong V3.3:
    - `highs`, `lows`, `volumes` (Scanners đã đọc từ `tick.*`).
    - `kama_slope` (chỉ dùng `kama_slope_adj`).
    - `vol_sma` (đã thay bằng `vol_z_score`).
    - `macd_main`, `macd_signal`, `macd_hist` (Helios V3.3 đã loại bỏ MACD).
    - `stoch_k` (thừa).
  - **Giải pháp triển khai:**
    Thu gọn struct `HeliosSensors` từ 35 trường xuống còn đúng 18 trường cốt lõi phục vụ Matrix và 4 Playbook hoạt động thực tế.
  - **Tiêu chí nghiệm thu:**  
    + Toàn bộ crate `helios` và `trading_bot` biên dịch thành công (`cargo check`), mã nguồn tinh gọn, tường minh 100%.
