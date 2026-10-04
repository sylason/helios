# DANH MỤC CÔNG VIỆC BẢO LƯU: LIQUIDATION SIMULATION & DYNAMIC AUTO-DELEVERAGE (TODO.md)

Tài liệu quản lý công việc và test cases khi quay lại nâng cấp phân hệ Mô Phỏng Thanh Lý & Đòn Bẩy Động theo:
👉 [Bản thiết kế kiến trúc (`01_liquidation_simulation_and_auto_deleverage_blueprint.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/02_liquidation_and_auto_deleverage/01_liquidation_simulation_and_auto_deleverage_blueprint.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [ ] **`TASK-LIQ-01`**: **Bổ Sung Cơ Chế Thanh Lý Cưỡng Chế Vào `ferro_trade::Broker`**
  - *Mô tả:* Trong `ferro_trade/src/broker/broker.rs`, tính toán `liquidation_price` cho các vị thế mở và kiểm tra điều kiện vi phạm tại mỗi nến để kích hoạt Margin Call.
  - *Vị trí tác động:* `ferro_trade/src/broker/broker.rs` và `trade.rs`.
  - *Test cases:*
    - `test_broker_liquidates_position_when_price_crosses_liq_barrier`: Đảm bảo vị thế không SL bị force-close khi lỗ vượt mức ký quỹ duy trì.

- [ ] **`TASK-LIQ-02`**: **Tích Hợp Liquidation Watchdog Vào `PaperTrader` Trong `bot_engine`**
  - *Mô tả:* Mô phỏng chính xác việc sàn bắn lệnh thanh lý giải chấp trong môi trường Paper Trade để đo đạc rủi ro sát với sàn Live.
  - *Vị trí tác động:* `trading_bot/bot_engine/src/execution/paper_trader.rs`.
  - *Test cases:*
    - `test_paper_trader_triggers_liquidation_on_flash_crash`: Kiểm tra tài khoản paper bị thanh lý khi giá nhảy gap quét qua mức Liq.

- [ ] **`TASK-LIQ-03`**: **Nạp Tự Động Bảng `leverageBracket` Từ Binance Futures API**
  - *Mô tả:* Gọi endpoint REST `GET /fapi/v1/leverageBracket` khi khởi động bot, parse và lưu trữ MMR theo Tier vào `MarketFilter`.
  - *Vị trí tác động:* `trading_bot/bot_engine/src/models/market_info.rs` và `order_executor.rs`.
  - *Test cases:*
    - `test_fetch_and_parse_leverage_bracket_tier_1_mmr`: Kiểm tra nạp đúng MMR của BTC (0.4%) và Altcoin (1.0% - 2.5%).

- [ ] **`TASK-LIQ-04`**: **Triển Khai Cơ Chế Dynamic Auto-Deleverage Guard**
  - *Mô tả:* Khi khoảng cách SL cấu trúc lớn hơn vùng an toàn thanh lý của đòn bẩy mặc định, tự động tính toán $L_{\text{safe}}$ và gọi API đổi đòn bẩy trước khi gửi lệnh.
  - *Vị trí tác động:* `strategies_backtests/helios/src/strategy/core_v3/risk.rs` và `trading_bot/bot_engine/src/execution/order_executor.rs`.
  - *Test cases:*
    - `test_auto_de_leverage_calculation`: Kiểm tra công thức giải ngược tìm $L_{\text{safe}}$ chính xác.
    - `test_order_executor_sets_safe_leverage_before_order`: Kiểm tra luồng gọi API đổi đòn bẩy an toàn.
