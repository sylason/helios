# Checklist & Quản Lý Công Việc: Position Synchronization & Trailing SL Invariance

**Thư mục:** `trading_bot/docs/debugs/05_position_sync_and_sl_invariance/`  
**Trạng thái:** Đang hoàn thiện kiểm thử thực nghiệm

---

## 📋 Danh Sách Nhiệm Vụ (Task Checklist)

- [x] **Nhiệm vụ 1: Sửa logic kế toán `bars_held` trong `PaperTrader`**
  - **Mô tả:** Đổi cách tính từ `exit_bar_index - entry_bar_index` sang `(exit_timestamp - entry_timestamp) / 60_000` để kháng lỗi Rolling Buffer trượt.
  - **File sửa:** [`trading_bot/bot_engine/src/execution/paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)

- [x] **Nhiệm vụ 2: Chuẩn hóa ánh xạ `mapped_entry_bar` trong `HeliosBridge`**
  - **Mô tả:** Dùng thuật toán Binary Search $O(\log N)$ trên `entry_timestamp` thay vì phép trừ offset tương đối `saturating_sub(slice_start)`.
  - **File sửa:** [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs)

- [x] **Nhiệm vụ 3: Thiết lập cơ chế Initial Stop Loss Invariance trong `RiskManager`**
  - **Mô tả:** Ràng buộc `current_pnl > 0.0` trước khi cho phép dời Trailing Stop Loss, triệt tiêu nguy cơ bóp nghẹt vị thế khi PnL âm.
  - **File sửa:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)

- [ ] **Nhiệm vụ 4: Viết và chạy Unit Test kiểm tra biên độ SL**
  - **Mô tả:** Viết test case mô phỏng trường hợp giá âm rung lắc xem mức SL có bị dời sai lệch không.
  - **Vị trí test:** `strategies_backtests/helios/tests/` & `trading_bot/bot_engine/tests/`

- [ ] **Nhiệm vụ 5: Kiểm tra hồi quy toàn diện (Regression Test)**
  - **Mô tả:** Chạy `cargo check` và `cargo test` trên toàn bộ 3 workspace (`ferro_trade`, `helios`, `trading_bot`).
