# Checklist & Quản Lý Công Việc: MTF Streaming & Strategy Bridge Desynchronization

**Thư mục:** `trading_bot/docs/debugs/06_mtf_streaming_and_bridge_desync/`  
**Trạng thái:** Đã kiểm chứng đầy đủ bằng mã nguồn

---

## 📋 Danh Sách Nhiệm Vụ (Task Checklist)

- [x] **Nhiệm vụ 1: Khắc phục Lỗi Đóng Băng MTF trong `HeliosBridge`**
  - **Mô tả:** Đồng bộ hóa cơ chế `precompute_timeframes` tươi mới mỗi tick thay vì cache theo bucket, đảm bảo tính đúng đắn 1:1 với `diagnostic_backtest`.
  - **File liên quan:** [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs)

- [ ] **Nhiệm vụ 2: Đồng bộ Hóa Vòng Đời Kiểm Tra Rủi Ro giữa `AssetActor` và `Broker`**
  - **Mô tả:** Đảm bảo Strategy có thể đánh giá và dời Trailing Stop Loss trên nến hiện tại trước khi lệnh bị quét oan theo mức SL cũ.
  - **File liên quan:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)

- [ ] **Nhiệm vụ 3: Sửa Lỗi Phân Loại `ExitReason` trong `PaperTrader`**
  - **Mô tả:** So sánh mức SL kích hoạt với `initial_sl_price` thay vì `entry_price` để ghi nhận đúng các lần dời Trailing SL giảm thiểu rủi ro.
  - **File liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)

- [ ] **Nhiệm vụ 4: Bảo Lưu Tham Số Rủi Ro Khi DNA Hot-Swapping**
  - **Mô tả:** Gắn cố định các tham số quản trị rủi ro vào `LivePositionState` khi vào lệnh, ngăn chặn hiện tượng nhảy vọt `decay_cycles` làm đóng lệnh đột ngột khi nạp DNA mới.
  - **File liên quan:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)

- [ ] **Nhiệm vụ 5: Kiểm Tra Hồi Quy và Xác Minh Thực Nghiệm**
  - **Mô tả:** Chạy lại kịch bản `walk_forward_smart_seed.rs` và kiểm tra xem kết quả Paper Trader có đồng bộ và khớp sát với Backtest chẩn đoán không.
