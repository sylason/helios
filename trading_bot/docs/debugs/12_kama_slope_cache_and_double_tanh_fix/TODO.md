# 📋 DANH MỤC CÔNG VIỆC: SỬA LỖI FLAW-3.4 (KAMA SLOPE CACHE) & FLAW-4.5 (DOUBLE TANH SCANNERS)

> **Thư mục quản lý:** `trading_bot/docs/debugs/12_kama_slope_cache_and_double_tanh_fix/`  
> **Các tài liệu kỹ thuật liên quan:**  
> - `01_flaw_3_4_kama_slope_cache_spec.md`  
> - `02_flaw_4_5_double_tanh_spec.md`  

---

## 🎯 DANH SÁCH NHIỆM VỤ THEO BABY STEPS

### 🌿 BƯỚC 1: SỬA FLAW-3.4 TRONG `ferro_trade`
- [x] **Task 1.1:** Bổ sung `atr_period: usize` vào enum `IndicatorType::KamaSlope` tại `ferro_trade/src/indicators/identifier.rs`.
- [x] **Task 1.2:** Cập nhật khởi tạo `IndicatorType::KamaSlope` trong `BulkIndicatorFactory::kama_slope` tại `ferro_trade/src/indicators/factory/bulk.rs`.
- [x] **Task 1.3:** Viết Unit Test trong `ferro_trade` xác minh 2 lần gọi `kama_slope` với `atr_period` khác nhau tạo ra 2 kết quả độc lập, không bị xung đột CacheKey (`test_kama_slope_cache_key_differentiation_by_atr_period`).
- [x] **Task 1.4:** Chạy `cargo test --package ferro_trade --lib` (50/50 PASSED).

---

### 🌿 BƯỚC 2: SỬA FLAW-4.5 TRONG `strategies_backtests/helios`
- [x] **Task 2.1:** Cập nhật `reversal.rs` chuyển sang đọc `z_score = sensors.z_score[i]`.
- [x] **Task 2.2:** Cập nhật `anomaly.rs` chuyển sang đọc `z_score = sensors.z_score[i]`.
- [x] **Task 2.3:** Viết Unit Test trong `helios` xác minh điểm $f_z$ và tổng `score` của `reversal` và `anomaly` đạt các mốc cao $\ge 0.65$ khi thị trường lệch $-3.0\sigma$ (`test_reversal_and_anomaly_use_raw_z_score_not_double_tanh`).
- [x] **Task 2.4:** Chạy `cargo test --package helios --lib` (13/13 PASSED).

---

### 🌿 BƯỚC 3: KIỂM THỬ TÍCH HỢP & ĐỐI CHIẾU TOÀN BỘ HỆ THỐNG
- [x] **Task 3.1:** Chạy `cargo test --package ferro_trade --lib` (50/50 PASSED).
- [x] **Task 3.2:** Chạy `cargo test --package helios --lib` (13/13 PASSED).
- [x] **Task 3.3:** Chạy `cargo test --package bot_engine --lib` (31/31 PASSED, 1 ignored testnet).
- [x] **Task 3.4:** Chạy `cargo test --package bot_engine --test trailing_sl_test` (4/4 PASSED).
- [x] **Task 3.5:** Cập nhật `[x] [ĐÃ SỬA ✅]` cho `FLAW-3.4` và `FLAW-4.5` trong `trading_bot/docs/debugs/07_comprehensive_audit/TODO_CRITICAL.md`.
- [x] **Task 3.6:** Báo cáo kết quả và cung cấp lệnh git commit cho người dùng.
