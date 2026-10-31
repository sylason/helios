# 📌 TODO: SỬA LỖI ĐỂ CHẠY THÔNG SUỐT TEST `WALK_FORWARD_SMART_SEED`

Tiến độ triển khai sửa lỗi theo quy trình Baby Steps.

---

## Giai đoạn 1: Đồng Bộ Hóa Đầy Đủ Timeframe 30m & Bảo Vệ Context
- [x] **Task 1.1:** Kiểm tra và đồng bộ hóa đầy đủ danh sách `["3m", "5m", "10m", "15m", "30m", "1h", "2h", "3h", "4h"]` trên toàn bộ các điểm nạp cache:
  * `HELIOS_REQUIRED_TIMEFRAMES` trong [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs).
  * `ProspectorConfig::default()` trong [`ferro_trade/src/optimizer/prospector.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/prospector.rs).
  * File cấu hình [`helios_optimizer_config.json`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_optimizer_config.json).
- [x] **Task 1.2:** Cập nhật [`ferro_trade/src/backtest/context.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs:136) thay thế `expect` bằng cơ chế ghi nhận `indicator_manager.mark_infeasible("MTF_Cache_Missing")` an toàn (trả về NaN), chống sập tiến trình GA.

---

## Giai đoạn 2: Sửa Bẫy Ngắt Sớm `is_warmup_overflow` Tại `ferro_trade::engine`
- [x] **Task 2.1:** Thêm `eval_start_bar: Option<usize>` vào `BacktestBuilder` tại [`ferro_trade/src/backtest/builder.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/builder.rs).
- [x] **Task 2.2:** Thêm `eval_start_bar: Option<usize>` vào `Backtest` tại [`ferro_trade/src/backtest/engine.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/engine.rs).
- [x] **Task 2.3:** Cập nhật logic `is_warmup_overflow` trong `engine.rs`: Vô hiệu hóa quy tắc trần $40\%$ khi `eval_start_bar.is_some()`.
- [x] **Task 2.4:** Gán `start_bar = eval_start_bar.unwrap_or(warmup + 1)`, đồng bộ `bar_index` trong `EarlyExitContext` và lấp đầy `equity_curve` đuôi khi bị break sớm.
- [x] **Task 2.5:** Chạy unit tests `cargo test --manifest-path ferro_trade/Cargo.toml --lib` xác nhận 56/56 tests PASS 100%.

---

## Giai đoạn 3: Cập Nhật WFE Gate & Diagnostic Backtest Trong `bot_engine`
- [x] **Task 3.1:** Thêm tham số `eval_start_bar: Option<usize>` vào `diagnostic_backtest_with_eval_and_config` trong [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs).
- [x] **Task 3.2:** Truyền `eval_start_bar` khi gọi chẩn đoán OOS trong [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs).
- [x] **Task 3.3:** Sửa tiêu chí WFE Gate trong `prospector_runner.rs`: Bỏ trừ khống `slippage_drag_roe`, duyệt `SOLID PASS` khi `val_trades >= 1`, `val_ret >= 0.0%`, `val_dd <= 15.0%`. Giữ Champion cũ khi fail.
- [x] **Task 3.4:** Cập nhật và chạy unit tests: `cargo test --manifest-path trading_bot/bot_engine/Cargo.toml --lib test_wfe_gate` và `test_top_k_waterfall` PASS 100%.

---

## Giai đoạn 4: Xác Minh Thực Nghiệm (Người Dùng Tự Chạy Test)
- [ ] **Task 4.1:** Người dùng chạy `cargo run --release --example walk_forward_smart_seed` tại thư mục `trading_bot/bot_engine`.
- [ ] **Task 4.2:** Báo cáo kết quả và log để đối chiếu:
  1. Hỗ trợ đầy đủ nến 30m, không còn panic MTF cache.
  2. Candidate #1 được WFE Gate đánh giá công bằng trên 3 ngày OOS và cấp `SOLID PASS` khi có lãi.
- [ ] **Task 4.3:** Cập nhật `trading_bot/docs/errors.md` và cung cấp lệnh git commit cho người dùng.
