# 📜 NHẬT KÝ LỖI KIẾN TRÚC & SAI LỆCH THIẾT KẾ (ERRORS.MD)

## 🚨 LỖI #001: Chỉ báo mồ côi có Period = 0 làm tràn Warmup triệt tiêu vòng lặp Backtest

### 1. Hiện tượng & Triệu chứng (Symptoms)
- Trong log `WFSS_macro_veto_expo_cooldown.log`:
  - 23 chu kỳ GA Re-Optimize và Single Backtest Eval đều trả về `Return: 0.00% | Trades: 0 | Max DD: -0.00%`.
  - Toàn bộ 200 cá thể qua các thế hệ đều có kết quả giống hệt nhau và kết thúc cực nhanh (~6-7 giây cho 8 thế hệ).
  - WFE Validation Gate báo `✨ [WFE GATE: PASS] Champion DNA Validated! OOS Trades: 0 -> Approved for Deployment`.
  - Ngược lại, Paper Trading (`evaluate_live_tick`) vẫn khớp và thực thi 23 lệnh thực tế trên cùng một bộ dữ liệu.

### 2. Nguyên nhân gốc rễ (Root Cause)
- **Mắt xích 1:** Khi tinh giản cấu trúc DNA (Bước 3), các gen cấu hình của `rsi_anomaly` và `stoch_range` được comment out khỏi `config.rs`.
- **Mắt xích 2:** Struct `RsiParams` và `StochasticParams` trong cấu hình rơi về `Default::default()` với `period = 0`, `k_period = 0`, `d_period = 0`.
- **Mắt xích 3:** Trong `HeliosStrategyV3::init`, hàm vẫn gọi `ctx.rsi()` và `ctx.stoch()` với `period = 0`. Các hàm toán học chỉ báo không thể tính toán với cửa sổ bằng 0 nên trả về vector chứa 100% giá trị `NaN` ($63,200 / 63,200$).
- **Mắt xích 4:** `IndicatorManager::add_raw_cached` dò tìm phần tử hợp lệ đầu tiên để xác định warmup:
  ```rust
  let mut actual_warmup = 0;
  for &val in arc_vec.iter() {
      if !val.is_nan() { break; }
      actual_warmup += 1;
  }
  self.warmup_period = self.warmup_period.max(warmup).max(actual_warmup);
  ```
  Vì toàn bộ mảng là `NaN`, `actual_warmup` bằng đúng `len - 1` ($63,199$ nến).
- **Mắt xích 5:** Động cơ Backtest `ferro_trade::run_logic` lặp theo:
  `for i in (warmup + 1)..full_len` $\to$ `63,200..63,200` $\to$ **0 vòng lặp được thực thi**! Chiến lược không bao giờ được gọi `strategy.next()` trong Backtest.
- **Mắt xích 6:** `evaluate_live_tick` thoát được bẫy này vì nó chỉ đánh giá duy nhất nến cuối cùng `i = full_len - 1` và điều kiện `if full_len <= warmup` bị false ($63,200 \le 63,199$ là false). 4 Playbook active không dùng `rsi_anomaly` hay `stoch_range` nên vẫn phát sinh lệnh bình thường.

### 3. Giải pháp khắc phục (Resolution)
1. Dọn dẹp dứt điểm các cuộc gọi `ctx.rsi(...)` cho `rsi_anomaly` và `ctx.stoch(...)` cho `stoch_range` trong `HeliosStrategyV3::init`.
2. Khóa cổng WFE Gate trong `prospector_runner.rs`: Bắt buộc `val_trades >= 1` và `ret > 0.0` mới cho phép deploy.
3. Bổ sung cơ chế bảo vệ trong `IndicatorManager` để cảnh báo nếu `actual_warmup >= len`.
