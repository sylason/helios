# 📋 DANH MỤC CÔNG VIỆC: SỬA LỖI FLAW-3.3 (SIMPLEX NaN) & FLAW-4.4 (FALSE WIN COOLDOWN)

> **Thư mục quản lý:** `trading_bot/docs/debugs/11_simplex_nan_and_false_win_fix/`  
> **Các tài liệu kỹ thuật liên quan:**  
> - `01_flaw_3_3_simplex_nan_spec.md`  
> - `02_flaw_4_4_last_closed_trade_spec.md`  

---

## 🎯 DANH SÁCH NHIỆM VỤ THEO BABY STEPS

### 🌿 BƯỚC 1: SỬA FLAW-3.3 TRONG `ferro_trade` & `helios`
- [x] **Task 1.1:** Cập nhật hàm `chaos_ratio` trong `ferro_trade/src/indicators/calculate/bulk.rs`:
  - Trả về `1.0` khi `clean_natr.len() < sma_period`.
  - Bổ sung `sma_val.is_nan()` vào biểu thức guard: `if v.is_nan() || sma_val.is_nan() || sma_val < 1e-9 { 1.0 } else { v / sma_val }`.
- [x] **Task 1.2:** Viết Unit Test trong `calculate/bulk.rs` kiểm chứng chuỗi ngắn và chuỗi warmup không chứa `NaN`.
- [x] **Task 1.3:** Cập nhật `matrix.rs` trong `strategies_backtests/helios`:
  - Bọc Zero & NaN Guard cho mẫu số Simplex `sum`.
  - Fallback chia đều $1/3 - 1/3 - 1/3$ nếu `sum <= 1e-6 || sum.is_nan()`.
- [x] **Task 1.4:** Chạy `cargo test --package ferro_trade --lib` và `cargo test --package helios --lib`.

---

### 🌿 BƯỚC 2: SỬA FLAW-4.4 TRONG `ferro_trade`, `helios` & `bot_engine`
- [x] **Task 2.1:** Bổ sung trường `pub last_closed_trade: Option<&'a Trade>` vào `TickContext` trong `ferro_trade/src/backtest/context.rs`.
- [x] **Task 2.2:** Nạp `last_closed_trade: broker.closed_trades.last()` tại `ferro_trade/src/backtest/engine.rs`.
- [x] **Task 2.3:** Nạp `last_closed_trade: broker.closed_trades.last()` tại `trading_bot/bot_engine/src/optimizer/helios_bridge.rs`.
- [x] **Task 2.4:** Cập nhật logic Cooldown trong `strategies_backtests/helios/src/strategy/core_v3/mod.rs` đọc PnL từ `tick.last_closed_trade.pl(None)`.
- [x] **Task 2.5:** Viết Unit Test mô phỏng nến rút chân (Hammer) xác nhận Cooldown được kích hoạt đúng khi dính Stop Loss.

---

### 🌿 BƯỚC 3: KIỂM THỬ TÍCH HỢP & ĐỐI CHIẾU TOÀN BỘ HỆ THỐNG
- [x] **Task 3.1:** Chạy `cargo test --package ferro_trade --lib` (49/49 PASSED).
- [x] **Task 3.2:** Chạy `cargo test --package helios --lib` (12/12 PASSED).
- [x] **Task 3.3:** Chạy `cargo test --package bot_engine --lib` (31/31 PASSED, 1 ignored testnet).
- [x] **Task 3.4:** Chạy `cargo test --package bot_engine --test trailing_sl_test` (4/4 PASSED).
- [x] **Task 3.5:** Cập nhật `[x] [ĐÃ SỬA ✅]` cho `FLAW-3.3` và `FLAW-4.4` trong `trading_bot/docs/debugs/07_comprehensive_audit/TODO_CRITICAL.md`.
- [x] **Task 3.6:** Báo cáo kết quả và cung cấp lệnh git commit cho người dùng.
