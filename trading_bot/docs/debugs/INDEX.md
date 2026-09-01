# 📚 Mục Lục Hệ Thống Debug & Kiểm Toán Kỹ Thuật (Debug & Audit Index)

**Dự án:** `trading_bot` (Binance Futures Execution Platform & Helios Integration)  
**Đường dẫn thư mục:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\debugs\`  

---

## 🗂️ Cấu Trúc Các Chủ Đề Debug Chuyên Sâu

Hệ thống tài liệu debug được phân chia thành 6 thư mục chủ đề chuẩn hóa, có liên kết chéo và kèm theo các bằng chứng kiểm chứng trực tiếp từ mã nguồn:

### 1. 📁 [`01_decoupling_and_config/`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/01_decoupling_and_config/audit_report.md)
- **Nội dung:** Kiểm toán tính tách rời giữa `bot_engine` và cấu hình `HeliosConfig`, loại bỏ phụ thuộc cứng và xử lý serialization.

### 2. 📁 [`02_execution_and_position/`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/02_execution_and_position/audit_report.md)
- **Nội dung:** Kiểm toán chuyên sâu `AssetActor`, `PaperTrader`, `OrderExecutor`, `AccountManager`, phát hiện bẫy Deadlock cờ `is_submitting_order` và hardcoded equity.
- 📄 [`asset_actor_deep_audit.md`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/02_execution_and_position/asset_actor_deep_audit.md)

### 3. 📁 [`03_data_engine_and_storage/`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/03_data_engine_and_storage/audit_report.md)
- **Nội dung:** Kiểm toán hệ thống lưu trữ SQLite, cơ chế nạp nến lịch sử, WebSocket Manager và kiểm soát API Weight.

### 4. 📁 [`04_code_quality_and_encoding/`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/04_code_quality_and_encoding/audit_report.md)
- **Nội dung:** Rà soát chuẩn mã hóa ký tự UTF-8, cấu trúc dữ liệu Zero-cost và Clean Code.

### 5. 📁 [`05_position_sync_and_sl_invariance/`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/05_position_sync_and_sl_invariance/TODO.md)
- **Nội dung:** Đồng bộ hóa vị thế theo Timestamp tuyệt đối và bảo toàn Initial Stop Loss khi vị thế đang âm danh nghĩa.
- 📄 [01_bars_held_and_timestamp_sync.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/05_position_sync_and_sl_invariance/01_bars_held_and_timestamp_sync.md): Bẫy kế toán `bars_held` và `mapped_entry_bar`.
- 📄 [02_initial_sl_invariance_proof.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/05_position_sync_and_sl_invariance/02_initial_sl_invariance_proof.md): Bẫy bóp nghẹt Stop Loss khi PnL âm.

### 6. 📁 [`06_mtf_streaming_and_bridge_desync/`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/06_mtf_streaming_and_bridge_desync/TODO.md)
- **Nội dung:** Kiểm chứng các lỗi sai lệch trong giao tiếp giữa `bot_engine` và chiến lược `Helios` / `ferro_trade`.
- 📄 [01_frozen_mtf_data_proof.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/06_mtf_streaming_and_bridge_desync/01_frozen_mtf_data_proof.md): Bẫy đóng băng dữ liệu nến đa khung thời gian trong `HeliosBridge`.
- 📄 [02_actor_broker_lifecycle_desync.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/06_mtf_streaming_and_bridge_desync/02_actor_broker_lifecycle_desync.md): Lệch pha vòng đời kiểm tra SL/TP giữa `AssetActor` và `Broker`.
- 📄 [03_dna_hotswap_decay_jump.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/06_mtf_streaming_and_bridge_desync/03_dna_hotswap_decay_jump.md): Bẫy đóng lệnh đột ngột khi DNA Hot-Swapping.
- 📄 [04_paper_trader_exit_reason_misclassification.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/06_mtf_streaming_and_bridge_desync/04_paper_trader_exit_reason_misclassification.md): Sai lệch phân loại `ExitReason::HardStopLoss` trong Paper Trader.

---

## 📌 Master TODO
Xem tiến độ tổng thể tại [TODO.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/TODO.md).
