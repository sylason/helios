# Checklist: Position Synchronization & Trailing Stop Loss Invariance

- [x] **Bước 1:** Sửa logic kế toán `bars_held` trong `trading_bot/bot_engine/src/execution/paper_trader.rs` dựa trên `exit_timestamp - entry_timestamp`.
- [x] **Bước 2:** Sửa logic ánh xạ `mapped_entry_bar` trong `trading_bot/bot_engine/src/optimizer/helios_bridge.rs` sử dụng Binary Search theo `entry_timestamp`.
- [x] **Bước 3:** Chuẩn hóa `RiskManager::manage_trailing_stop` trong `strategies_backtests/helios/src/strategy/core_v3/risk.rs` để bảo toàn Initial Stop Loss khi vị thế chưa có lãi (`current_pnl <= 0.0`).
- [ ] **Bước 4:** Chạy kiểm thử tự động `cargo check` và `cargo test` cho toàn bộ dự án (`bot_engine`, `helios`, `ferro_trade`).
