# 📋 DANH MỤC CÔNG VIỆC SPRINT 13: FLAW-3.3 (CHAOS NAN) & STOP LOSS / UNDERWATER TSL

> **Thư mục quản lý:** `trading_bot/docs/debugs/13_chaos_nan_resilience_and_underwater_tsl_fix/`  
> **Tài liệu kỹ thuật liên quan:**  
> - `01_flaw_3_3_chaos_nan_resilience_spec.md`  
> - `02_initial_sl_base_atr_and_underwater_tsl_spec.md`  

---

## 🎯 DANH SÁCH NHIỆM VỤ THEO BABY STEPS

### 🌿 BƯỚC 1: SỬA FLAW-3.3 TRONG `ferro_trade` VÀ `helios/matrix.rs`
- [x] **Task 1.1:** Khôi phục trả về `f64::NAN` cho chuỗi warmup của `chaos_ratio` tại `ferro_trade/src/indicators/calculate/bulk.rs`.
- [x] **Task 1.2:** Cập nhật unit test `test_chaos_ratio_no_nan_short_and_warmup` thành `test_chaos_ratio_returns_nan_during_warmup` trong `bulk.rs`.
- [x] **Task 1.3:** Cập nhật `matrix.rs` tại dòng 126 và dòng 136: xử lý fallback khi `chaos_ratio[i].is_nan()` về `0.0` (thị trường 0% Chaos) thay vì `1.0`.
- [x] **Task 1.4:** Chạy kiểm thử `cargo test --package ferro_trade --lib` và `cargo test --package helios --lib`.

---

### 🌿 BƯỚC 2: KHÔI PHỤC `base_sl_atr` & KÍCH HOẠT UNDERWATER TSL
- [x] **Task 2.1:** Cập nhật `mod.rs:302` truyền `self.config.risk_management.stop_management.base_sl_atr` vào `RiskManager::get_initial_sl`.
- [x] **Task 2.2:** Cập nhật `risk.rs:manage_trailing_stop` cho phép Stagnant Time Decay dời siết SL khi `current_pnl <= 0.0` và `decay_cycles > 0`, tích hợp 3 chốt chặn an toàn (Buffer Floor $1.0$ ATR, chống nhảy qua đầu giá, Monotonic Ratchet cấm lùi).
- [x] **Task 2.3:** Viết unit test xác minh `base_sl_atr` được áp dụng đúng và Underwater TSL siết SL an toàn.
- [x] **Task 2.4:** Chạy kiểm thử `cargo test --package helios --lib`.

---

### 🌿 BƯỚC 3: KIỂM THỬ TOÀN DIỆN & TÍCH HỢP HỆ THỐNG
- [x] **Task 3.1:** Chạy `cargo test --package ferro_trade --lib` (50/50 PASSED).
- [x] **Task 3.2:** Chạy `cargo test --package helios --lib` (15/15 PASSED).
- [x] **Task 3.3:** Chạy `cargo test --package bot_engine --lib` (31/31 PASSED, 1 ignored testnet).
- [x] **Task 3.4:** Chạy `cargo test --package bot_engine --test trailing_sl_test` (4/4 PASSED).
- [x] **Task 3.5:** Cập nhật dấu `[x]` vào `TODO.md` và cập nhật `TODO_CRITICAL.md`.
- [x] **Task 3.6:** Thông báo hoàn thành và cung cấp câu lệnh git commit cho người dùng.
