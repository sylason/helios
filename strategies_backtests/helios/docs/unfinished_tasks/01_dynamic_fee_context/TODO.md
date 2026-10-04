# DANH MỤC CÔNG VIỆC BẢO LƯU: DYNAMIC EXCHANGE FEE CONTEXT (TODO.md)

Tài liệu quản lý công việc và test cases khi quay lại nâng cấp phân hệ Phí Sàn Tự Động theo:
👉 [Bản thiết kế kiến trúc (`01_dynamic_exchange_fee_context_blueprint.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/01_dynamic_fee_context/01_dynamic_exchange_fee_context_blueprint.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [ ] **`TASK-FEE-01`**: **Mở Rộng Context Của `ferro_trade` Hỗ Trợ Phí Động**
  - *Mô tả:* Bổ sung `pub commission_rate: f64` vào struct `InitContext` và `TickContext` trong `ferro_trade/src/backtest/context.rs`.
  - *Vị trí tác động:* `ferro_trade/src/backtest/context.rs` và `ferro_trade/src/backtest/engine.rs`.
  - *Test cases:*
    - `test_tick_context_receives_engine_commission_rate`: Đảm bảo `tick.commission_rate` khớp chính xác với giá trị được cấu hình trong `BacktestBuilder`.

- [ ] **`TASK-FEE-02`**: **Cập Nhật `MarketFilter` & SQLite Cache Trong `bot_engine`**
  - *Mô tả:* Truy vấn Binance REST endpoint `GET /fapi/v1/commissionRate` để lấy phí Taker/Maker theo Symbol và VIP level.
  - *Vị trí tác động:* `trading_bot/bot_engine/src/models/market_info.rs` và `sqlite_impl.rs`.
  - *Test cases:*
    - `test_market_filter_stores_dynamic_commission_rate`: Kiểm tra nạp và lưu phí sàn vào DB.

- [ ] **`TASK-FEE-03`**: **Tiêm Phí Sàn Tự Động Qua `HeliosBridge`**
  - *Mô tả:* Khi `AssetActor` gửi nến vào `HeliosBridge`, đọc `commission_rate` từ `MarketFilter` và gán vào `TickContext`.
  - *Vị trí tác động:* `trading_bot/bot_engine/src/controller/helios_bridge.rs`.
  - *Test cases:*
    - `test_helios_bridge_injects_fee_to_tick`: Đảm bảo strategy nhận được phí động thời gian thực.

- [ ] **`TASK-FEE-04`**: **Tái Cấu Trúc `RegimeGatekeeper` Đọc Phí Trực Tiếp Từ Tick**
  - *Mô tả:* Chuyển đổi công thức tính sàn Flatline sang `tick.commission_rate`, loại bỏ dần sự phụ thuộc vào `RiskBaseParams.taker_fee_rate`.
  - *Vị trí tác động:* `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs`.
  - *Test cases:*
    - `test_gatekeeper_flatline_adapts_to_injected_tick_fee`: Kiểm tra sàn Flatline tự nâng lên khi phí tăng từ 0.04% lên 0.08%.
