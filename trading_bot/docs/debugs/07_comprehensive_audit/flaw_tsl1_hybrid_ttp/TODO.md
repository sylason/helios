# 📋 MASTER TODO: TRIỂN KHAI HYBRID TRAILING TAKE PROFIT (FLAW-TSL.1)

> **Mục tiêu:** Quản trị các đầu việc theo phương pháp Baby Steps cho tính năng Hybrid Trailing Take Profit (Dynamic Structural TP + Peak-Callback Trailing TP).  
> **Thư mục:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_tsl1_hybrid_ttp/TODO.md`  
> **Trạng thái:** `Hoàn thành (Passed 100% Tests)`

---

## 📌 DANH SÁCH ĐẦU VIỆC CHI TIẾT (BABY STEPS)

### Giai Đoạn 1: Thiết Kế & Tài Liệu Hóa (Documentation)
- [x] **Task 1.1:** Khởi tạo thư mục tính năng `trading_bot/docs/debugs/07_comprehensive_audit/flaw_tsl1_hybrid_ttp/`.
- [x] **Task 1.2:** Viết tài liệu đặc tả kỹ thuật `TECHNICAL_SPEC.md` (Toán học, Chốt chặn Guards, Telemetry).
- [x] **Task 1.3:** Viết tài liệu theo dõi tiến độ `TODO.md`.

### Giai Đoạn 2: Cấu Hình & Tham Số Hóa DNA (`config.rs`)
- [x] **Task 2.1:** Bổ sung các trường cấu hình TTP vào `RiskStopParams` trong [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs):
  - `ttp_activation_roe: f64` (Ngưỡng ROE kích hoạt TTP cho Trend, mặc định `0.20` = 20% ROE).
  - `ttp_callback_atr: f64` (Khoảng cách quay đầu từ đỉnh theo ATR, mặc định `0.80` ATR).
  - `dynamic_tp_min_gain_pct: f64` (Sàn bảo vệ lãi tối thiểu khi co dải Bollinger, mặc định `0.002` = 0.20%).
- [x] **Task 2.2:** Cập nhật hàm `default()` cho `RiskStopParams` và kiểm tra macro derive `DnaMappableMacro`.

### Giai Đoạn 3: Triển Khai Nhánh A - Dynamic Structural TP (`risk.rs`)
- [x] **Task 3.1:** Trong [`risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs), hàm `manage_trailing_stop`:
  - Lấy chỉ báo `sensors.bb_upper[i]`, `sensors.bb_lower[i]` cho `RANGE_OSC`.
  - Lấy chỉ báo `sensors.kama_mid[i]` cho `REVERSAL`.
- [x] **Task 3.2:** Triển khai **Min Profit Floor Guard**:
  - Long: $TP \ge Entry \times (1.0 + \text{min\_gain})$.
  - Short: $TP \le Entry \times (1.0 - \text{min\_gain})$.
- [x] **Task 3.3:** Triển khai **Spread Guard**:
  - Long: $TP > SL + 0.5 \times ATR$.
  - Short: $TP < SL - 0.5 \times ATR$.
- [x] **Task 3.4:** Cập nhật mức TP mới vào `tick.update_trade_sl_tp(trade.id, sl_to_send, new_tp_to_update)` khi `sl_changed || tp_changed`.
- [x] **Task 3.5:** Bổ sung log telemetry `[DYNAMIC TP UPDATED]` để theo dõi trực quan.

### Giai Đoạn 4: Triển Khai Nhánh B - Peak-Callback Trailing TP (`risk.rs`)
- [x] **Task 4.1:** Tính toán tỷ lệ lợi nhuận cao nhất đạt được từ đỉnh (`peak_roe`) cho các Playbooks xu hướng (`TREND_PB`, `MOMENTUM_IGN`, `ABSORPTION`, `ANOMALY`).
- [x] **Task 4.2:** Kiểm tra điều kiện kích hoạt `peak_roe >= config.ttp_activation_roe`.
- [x] **Task 4.3:** Tính toán mức giá kích hoạt thoát lệnh `trigger_price = peak_val - (callback_atr * ATR)`.
- [x] **Task 4.4:** Kiểm tra điều kiện đóng lệnh khi giá quay đầu ($Close \le TriggerPrice$ cho Long, $Close \ge TriggerPrice$ cho Short), gọi `tick.close(None)`. Bắn log `[TTP TRIGGERED EXIT]`.

### Giai Đoạn 5: Đồng Bộ Execution Engine & Telemetry
- [x] **Task 5.1:** Bổ sung `ExitReason::TrailingTakeProfit` (ký hiệu `"TTP"`) vào [`paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs). Phân loại khi nhận `Action::ClosePosition` với `pnl > 0.0`.
- [x] **Task 5.2:** Trong [`helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs), bảo toàn `[STATIC TP FROZEN SENSOR]` để phục vụ debug telemetry.

### Giai Đoạn 6: Kiểm Thử Độc Lập & Xác Minh (Verification)
- [x] **Task 6.1:** Viết 4 unit tests độc lập trong `strategies_backtests/helios/src/strategy/core_v3/risk.rs` kiểm thử Dynamic Structural TP, Guards, Peak-Callback TTP, và FLAW-TSL.5.
- [x] **Task 6.2:** Chạy `cargo test --lib` cho `helios` (21/21 passed) và `bot_engine` (34/34 passed) đảm bảo 100% pass.
- [x] **Task 6.3:** Cập nhật `TODO.md` và `TODO_CRITICAL.md`. Cung cấp câu lệnh Git commit thủ công cho người dùng.
