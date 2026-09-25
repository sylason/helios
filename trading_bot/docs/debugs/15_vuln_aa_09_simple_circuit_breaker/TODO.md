# DANH MỤC CÔNG VIỆC: KHẮC PHỤC VULN-AA-09, VULN-AA-10, VULN-AA-11

---

## 📋 Giai Đoạn 1: Thảo Luận & Chốt Thiết Kế (Design & Alignment)
- [x] Phân tích bản chất toán học & nghiệp vụ của `VULN-AA-09` (Deadlock 2.64 ngày).
- [x] Phân tích sự liên đới hữu cơ giữa `VULN-AA-09`, `VULN-AA-10` và `VULN-AA-11`.
- [x] Viết tài liệu kỹ thuật [`TECHNICAL_SPEC.md`](./TECHNICAL_SPEC.md) và lập bảng đối chất các phương án.
- [x] Thảo luận chi tiết và chốt phương án cho `VULN-AA-10`: **Đồng thuận Giải Pháp 1 - Loại bỏ hoàn toàn Early Release ATR, Cầu Dao thuần TTL**.
- [x] Thảo luận chi tiết và chốt phương án cho `VULN-AA-11`: **Đồng thuận Phương Pháp 4 (Unified Lifecycle Sentinel) - Hybrid Engine Drain + Snapshot Fallback**.

---

## 🛠️ Giai Đoạn 2: Triển Khai Mã Nguồn (Baby Steps Implementation)

### 1. Chuẩn Hóa Cốt Lõi: Helios Strategy Cooldown Tracker (Comment Out An Toàn)
- [x] `strategies_backtests/helios/src/strategy/core_v3/mod.rs`:
  - Comment out khối kiểm tra `self.cooldown_tracker.should_block` tại dòng 358-374 (kèm ghi thích rõ lý do nhường quyền Cầu Dao cho Bot Engine, bảo toàn logic để test khi cần).
- [x] Chạy `cargo check --manifest-path strategies_backtests/helios/Cargo.toml` xác minh biên dịch: **Exit Code 0 (Passed)**.

### 2. Tinh Giản Cầu Dao: Bot Engine Cooldown Guard
- [x] `trading_bot/bot_engine/src/controller/asset_actor.rs`:
  - Rút gọn struct `EngineCooldownGuard`: xóa bỏ `last_losing_price`, `last_losing_side`, `locked_side`, `last_exit_price`. Chỉ giữ `locked_until_bar` và `consecutive_losses`.
  - Cập nhật `calculate_exponential_cooldown`: tính trần động $\text{base\_bars} \times 4$.
  - Tối giản hàm `should_veto`: chỉ kiểm tra `current_bar_idx < self.locked_until_bar`.
  - Xóa bỏ hàm `calculate_rolling_atr` (`VULN-AA-10`).
- [x] Chạy `cargo check --manifest-path trading_bot/bot_engine/Cargo.toml` xác minh biên dịch: **Exit Code 0 (Passed)**.

### 3. Hạ Tầng Engine Drain: Hỗ Trợ Rút Lệnh Đóng Chuẩn (`VULN-AA-11`)
- [x] `trading_bot/bot_engine/src/execution/engine.rs`:
  - Bổ sung phương thức `fn take_last_closed_trade(&mut self) -> Option<crate::execution::paper_trader::PaperTradeRecord> { None }` vào trait `ExecutionEngine`.
- [x] `trading_bot/bot_engine/src/execution/paper_trader.rs`:
  - Thêm trường `pub last_closed_trade: Option<PaperTradeRecord>` vào `PaperTrader`.
  - Trong `close_position`: gán `self.last_closed_trade = Some(trade.clone())`.
  - Bổ sung hàm `pub fn take_last_closed_trade(&mut self) -> Option<PaperTradeRecord> { self.last_closed_trade.take() }`.
- [x] `trading_bot/bot_engine/src/execution/paper_engine.rs`:
  - Triển khai `take_last_closed_trade(&mut self)` ủy quyền cho `self.paper_trader.take_last_closed_trade()`.
- [x] `trading_bot/bot_engine/src/execution/live_engine.rs`:
  - Kế thừa mặc định `take_last_closed_trade(&mut self) -> Option<PaperTradeRecord> { None }` (dùng snapshot fallback).
- [x] Chạy `cargo check --manifest-path trading_bot/bot_engine/Cargo.toml` xác minh biên dịch: **Exit Code 0 (Passed)**.

### 4. Gom Nhất Thể Điểm Chốt: Unified Lifecycle Sentinel (`VULN-AA-11`)
- [x] `trading_bot/bot_engine/src/controller/asset_actor.rs`:
  - Thêm trường `previous_position: Option<LivePositionState>` vào `AssetActor`.
  - Xây dựng hàm chốt hạ duy nhất:
    `fn sync_and_handle_closed_positions(&mut self, current_bar_idx: usize, current_close: f64) -> bool`
    (Trả về `true` nếu vị thế vừa đóng trong nến này).
  - Thay thế toàn bộ logic đóng lệnh phân mảnh tại:
    + Dòng intra-candle SL/TP quét nến.
    + Dòng post-action Strategy đóng lệnh.
    + Dòng WebSocket AccountEvent vị thế về 0.
  - Áp dụng chốt chặn Same-Bar Revenge Re-entry: Nếu vị thế vừa đóng trong nến hiện tại $\implies$ Veto ngay lập tức lệnh mới cùng nến.
- [x] Chạy `cargo check --manifest-path trading_bot/bot_engine/Cargo.toml` xác minh biên dịch: **Exit Code 0 (Passed)**.

---

## 🧪 Giai Đoạn 3: Kiểm Thử & Xác Minh (Verification)
- [x] Chạy unit tests `cargo test --manifest-path strategies_backtests/helios/Cargo.toml --lib`: **65/65 tests passed**.
- [x] Chạy unit tests `cargo test --manifest-path trading_bot/bot_engine/Cargo.toml --lib`: **39/39 tests passed**.
- [ ] Chạy `walk_forward_smart_seed` kiểm chứng 1 chu kỳ: xác nhận không còn kẹt lệnh 2.64 ngày, log in mạch lạc.
