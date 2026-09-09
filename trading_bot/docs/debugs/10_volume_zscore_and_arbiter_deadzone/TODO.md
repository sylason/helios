# 📋 DANH MỤC CÔNG VIỆC: SỬA LỖI FLAW-3.2 (VOLUME Z-SCORE) & FLAW-4.1 (ARBITER DEAD ZONE)

> **Thư mục quản lý:** `trading_bot/docs/debugs/10_volume_zscore_and_arbiter_deadzone/`  
> **Các tài liệu kỹ thuật liên quan:**  
> - `01_flaw_3_2_volume_zscore_spec.md`  
> - `02_flaw_4_1_arbiter_confidence_spec.md`  

---

## 🎯 DANH SÁCH NHIỆM VỤ THEO BABY STEPS

### 🌿 BƯỚC 1: SỬA FLAW-3.2 TRONG `ferro_trade` & `helios`
- [x] **Task 1.1:** Thêm chỉ báo `z_score_volume` vào `ferro_trade/src/indicators/factory/bulk.rs` (tính Z-Score trên cột `"Volume"`).
- [x] **Task 1.2:** Bổ sung trường `pub vol_z_score: Arc<Vec<f64>>` vào `HeliosSensors` trong `strategies_backtests/helios/src/strategy/sensors.rs`.
- [x] **Task 1.3:** Đăng ký và nạp `s.vol_z_score = ctx.z_score_volume(...)` trong `strategies_backtests/helios/src/strategy/core_v3/mod.rs`.
- [x] **Task 1.4:** Cập nhật `matrix.rs` sử dụng `sensors.vol_z_score` để tính `volume_force`.
- [x] **Task 1.5:** Chạy `cargo check --package ferro_trade` và `cargo check --package helios`.

---

### 🌿 BƯỚC 2: SỬA FLAW-4.1 & KÍCH HOẠT GEN MA `scanner_weight` TRONG `arbiter.rs`
- [x] **Task 2.1:** Cập nhật hàm tính `confidence` trong `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs` theo công thức Weighted Geometric Mean:
  - $C = \sqrt{w_{regime} \times M}$
  - Zero Guard: `if raw_score <= 0.0 || C <= 0.0 { return 0.0; }`
  - `confidence = raw_score.powf(w) * C.powf(1.0 - w)`
- [x] **Task 2.2:** Viết bộ Unit Tests trong `arbiter.rs` kiểm chứng:
  - `test_zero_guard_prevents_false_signals`
  - `test_full_dynamic_range_coverage`
  - `test_scanner_weight_activation`
- [x] **Task 2.3:** Chạy `cargo test --package helios --lib`.

---

### 🌿 BƯỚC 3: KIỂM THỬ TÍCH HỢP & ĐỐI CHIẾU TOÀN BỘ HỆ THỐNG
- [x] **Task 3.1:** Chạy `cargo test --package bot_engine --lib`.
- [x] **Task 3.2:** Chạy `cargo test --package bot_engine --test trailing_sl_test`.
- [x] **Task 3.3:** Cập nhật `[x] [ĐÃ SỬA ✅]` cho `FLAW-3.2` và `FLAW-4.1` trong `trading_bot/docs/debugs/07_comprehensive_audit/TODO_CRITICAL.md`.
- [x] **Task 3.4:** Báo cáo kết quả và cung cấp lệnh git commit cho người dùng.
