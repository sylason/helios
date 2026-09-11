# 📚 TỪ ĐIỂN TÍNH NĂNG TOÀN DIỆN (FEATURE INVENTORY & SPEC AUDIT)

Tài liệu này là **Bản ghi Tính năng Độc bản (Single Source of Truth)** tổng hợp, bóc tách và phân loại $100\%$ các tính năng, cơ chế kỹ thuật, công thức toán và quy trình nghiệp vụ đã được ghi nhận trong toàn bộ tài liệu của `trading_bot/docs/` từ trước đến nay.

---

## 🏷️ QUY ƯỚC PHÂN LOẠI TRẠNG THÁI (STATUS LABELS)

- 🟢 **ACTIVE / FINAL**: Tính năng chuẩn xác ở thiết kế mới nhất (Đã thống nhất và sẵn sàng áp dụng).
- 🟡 **PROPOSED / DISCUSSION**: Tính năng / Ý tưởng cần thảo luận làm rõ thêm hoặc có điểm chưa hợp lý.
- 🔴 **DEPRECATED / OUTDATED**: Tính năng / Giải pháp cũ đã bị kiến trúc mới thay thế (Sẽ chuyển vào `archive/`).
- ⚠️ **CONFLICT / INCONSISTENCY**: Các điểm mâu thuẫn giữa tài liệu cũ và tài liệu mới.

---

## 🗂️ 1. PHÂN HỆ THU THẬP & LƯU TRỮ DỮ LIỆU (DATA ENGINE & STORAGE)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`DE-01`** | **Lưu trữ SQLite Độc Bản (`trading_bot.db`)** | 🟢 ACTIVE | Gom toàn bộ nến lịch sử và DNA records vào 1 file duy nhất `storage/trading_bot.db`. Thiết lập B-Tree Index cho `(symbol, timestamp)` và `(symbol, strategy_id)`. | `docs/engine/TODO.md` |
| **`DE-02`** | **Động cơ Phân tích DuckDB** | 🔴 DEPRECATED | Thiết kế ban đầu dùng DuckDB lưu nến lịch sử. Đã được thay thế hoàn toàn bằng SQLite để tối ưu hóa tính nhỏ gọn Local-First và tránh lock file. | `blueprints/database_schema.md` |
| **`DE-03`** | **Single Shared WsManager (Combined Stream)** | 🟢 ACTIVE | 1 kết nối WebSocket duy nhất tới `wss://fstream.binancefuture.com/stream?streams=...` gom chung toàn bộ các cặp tiền, tránh bị sàn chặn IP. | `engine/specs/shared_ws_and_routing.md` |
| **`DE-04`** | **Bảng Điều Hướng Gói Tin $O(1)$ (`routing_table`)** | 🟢 ACTIVE | `Arc<RwLock<HashMap<String, Sender<Candle>>>>` trong `WsManager` để định tuyến nến trực tiếp cho đúng `AssetActor` mà không làm phiền actor khác. | `engine/specs/shared_ws_and_routing.md` |
| **`DE-05`** | **Đăng ký / Hủy Stream Động (Dynamic WS)** | 🟢 ACTIVE | Gửi payload JSON `SUBSCRIBE` / `UNSUBSCRIBE` trực tiếp trên kết nối WS hiện hữu khi thêm/bớt cặp tiền mà không cần ngắt kết nối. | `engine/specs/shared_ws_and_routing.md` |
| **`DE-06`** | **Tự Tính Nến 1m từ WebSocket `aggTrade`** | 🔴 DEPRECATED | Tự tổng hợp nến từ tick `aggTrade`. Đã bỏ vì tốn CPU/RAM và tăng độ trễ không cần thiết; chuyển sang subscribe thẳng kênh Kline 1m. | `engine/discussions/grill_me_questions_v1.md` |
| **`DE-07`** | **Tự Động Bù Khoảng Trống Nến (REST Gap Filler)** | 🟢 ACTIVE | Mở kết nối WebSocket trước, sau đó gọi REST API tải bù 33 ngày nến ($43.200$ nến 1m) + 200 nến đệm (`buffer_padding_bars`) để lấp đầy khoảng trống nến và đủ 50 nến warmup cho khung 4h. | `engine/specs/engine_facade_and_lifecycle.md`, `engine/specs/binance_execution_flow.md` |
| **`DE-08`** | **Giám Sát Trọng Số API (`WeightMonitor`)** | 🟢 ACTIVE | Đọc header `X-MBX-USED-WEIGHT-1M`, kiểm soát giới hạn 2400 weight/phút. Khi $\ge 2200$, chỉ tạm hoãn (sleep 30s) các cuộc gọi REST API, luồng WebSocket vẫn vận hành bình thường. | `engine/specs/binance_live_execution.md` |
| **`DE-09`** | **Tự Động Dọn Dẹp Nến Cũ Hàng Ngày** | 🟡 PROPOSED | Tự động xóa dữ liệu nến cũ hơn 33 ngày khi mở ứng dụng để tiết kiệm dung lượng ổ cứng. | `engine/discussions/grill_me_questions_v1.md` |
| **`DE-10`** | **User Data Stream & Keep-Alive ListenKey** | 🟢 ACTIVE | Nhận sự kiện `ORDER_TRADE_UPDATE` và `ACCOUNT_UPDATE` qua private WebSocket; tự động duy trì bằng `connection.keepalive_user_data_stream()` định kỳ mỗi 30-60 phút để cập nhật vị thế tức thời trong RAM. | `engine/specs/binance_live_execution.md`, `engine/specs/binance_execution_flow.md` |

---

## 🧬 2. PHÂN HỆ TỐI ƯU HÓA DI TRUYỀN & THÍCH ỨNG (GA OPTIMIZER & DNA GUARD)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`GA-01`** | **Tối Ưu Hóa Cửa Sổ Trượt (Rolling Window 30D)** | 🟢 ACTIVE | Chạy định kỳ thuật toán di truyền (`Prospector GA`) trên 30 ngày nến gần nhất ($43.158$ nến 1m) để liên tục thích nghi với thị trường. *(Đã chuẩn hóa chu kỳ 4h vào `live_demo_runner.rs` qua [T1.4])* | `GEMINI.md`, `tests/paper_forward_test/16_walk_forward_smart_seed_analysis.md` |
| **`GA-02`** | **Bộ Điều Phối JIT Optimizer (`Semaphore(1)`)** | 🟢 ACTIVE | Hàng đợi tuần tự dùng `tokio::sync::Semaphore(1)` để chỉ cho phép 1 task GA chạy tại 1 thời điểm, triệt tiêu nguy cơ nghẽn CPU đa luồng. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`GA-03`** | **Nguyên Tắc Nạp Nến JIT (Just-In-Time Query)** | 🟢 ACTIVE | Chỉ truy vấn 30 ngày nến mới nhất từ SQLite ngay tại thời điểm giành được Semaphore lock trước khi bắt đầu tính toán GA. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`GA-04`** | **Kiểm Tra Hạn Sử Dụng DNA (`DNA Freshness Guard`)** | 🟢 ACTIVE | Khi bật bot: Nếu $\Delta t < 15$p $\rightarrow$ Nạp ngay (<1s) và trade lập tức; Nếu $\Delta t \ge 15$p $\rightarrow$ Huấn luyện GA ban đầu đồng bộ rồi mới trade. *(Đã chuẩn hóa vào `live_demo_runner.rs` qua [T1.3])* | `engine/specs/engine_facade_and_lifecycle.md` |
| **`GA-05`** | **Smart Seeding 0.5% Quần Thể** | 🟢 ACTIVE | Tiêm cá thể Rank 1 Champion từ chu kỳ trước làm hạt giống mồi vào thế hệ 0 (chiếm 0.5%), reset cache để 99.5% là đột biến/LHS mới. *(Đã chuẩn hóa vào `live_demo_runner.rs` qua [T1.4])* | `tests/paper_forward_test/16_walk_forward_smart_seed_analysis.md` |
| **`GA-06`** | **Hot-Swapping DNA Không Cần Khởi Động Lại** | 🟢 ACTIVE | Khi Re-Optimizer tìm ra DNA mới, truyền qua kênh `tokio::sync::watch` cập nhật trực tiếp vào `AssetActor` mà không làm gián đoạn bot. | `controller/asset_actor.rs`, `GEMINI.md` |
| **`GA-07`** | **Bảo Lưu Tham Số Rủi Ro Khi Hot-Swap (Anti-Decay Jump)** | 🟢 ACTIVE | Gắn cố định cấu hình rủi ro vào vị thế khi mở lệnh, ngăn chặn hiện tượng nhảy vọt `decay_cycles` làm đóng lệnh đột ngột khi nạp DNA mới. | `debugs/06_mtf_streaming_and_bridge_desync/03_dna_hotswap_decay_jump.md` |
| **`GA-08`** | **Hàm Mục Tiêu Tối Ưu Mở Rộng (Fitness with PF, SQN & W_Freq)** | 🟢 ACTIVE | Phạt nặng cá thể nếu Profit Factor $PF < 1.15$, thưởng $PF \ge 1.50$ và SQN cao, kết hợp trọng số tần suất lệnh $W_{\text{Freq}}$ (tối ưu mục tiêu $8 - 55$ lệnh/tháng). | `engine/discussions/mega_trend_todo.md` |

---

## 👑 3. PHÂN HỆ LÕI ĐIỀU HÀNH ĐA TÀI SẢN & FSM (ENGINECONTROLLER)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`EC-01`** | **Facade API `EngineController`** | 🟢 ACTIVE | Cấu trúc trung tâm quản lý danh bạ `AssetActorRegistry`, điều phối kết nối WS, JIT Scheduler và lưu trữ SQLite. | `engine/README.md`, `engine/TODO.md` |
| **`EC-02`** | **Máy Trạng Thái Đối Xứng (Symmetric FSM)** | 🟢 ACTIVE | Vòng đời 8 trạng thái: `INACTIVE`, `STARTING (SYNCING / GA_OPTIMIZING)`, `RUNNING_ACTIVE`, `PAUSED`, `GRACEFUL_STOPPING`, `STOPPED`, `ERROR`. Mọi lệnh Bật đều có Tắt/Hủy. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`EC-03`** | **Bộ Lệnh Điều Khiển Vòng Đời Đầy Đủ** | 🟢 ACTIVE | 8 lệnh: `start_asset`, `cancel_start`, `pause_trading`, `resume_trading`, `graceful_stop`, `cancel_graceful_stop`, `force_stop_immediate`, `remove_asset`. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`EC-04`** | **Quản Trị Vốn Lũy Tiến Kèm Vốn Sàn (Compounding Floor)** | 🟢 ACTIVE | $\text{Capital} = \max(\text{INITIAL\_BASE},\ \text{INITIAL\_BASE} + \sum \text{Realized PnL})$. Tự động tăng volume khi có lãi và bảo vệ không tụt dưới vốn cơ sở ban đầu. | `engine/specs/capital_and_risk_compounding.md` |
| **`EC-05`** | **Luồng Điều Chuyển Vốn An Toàn (Capital Rebalancing)** | 🟢 ACTIVE | Cho phép điều chuyển lãi thực giữa các cặp tiền. Nếu cặp đang mở vị thế $\rightarrow$ Phân nhánh an toàn (áp dụng từ lệnh sau vs đóng lệnh ngay). | `engine/specs/capital_and_risk_compounding.md` |
| **`EC-06`** | **Tiếp Quản Vị Thế Có Sẵn (`Adopt & Protect`)** | 🟢 ACTIVE | Khi kích hoạt bot, tự động kiểm tra vị thế mở sẵn trên sàn Binance, nạp vào state và lập tức treo lệnh Trailing SL bảo vệ. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`EC-07`** | **Tự Động Cấu Hình `ISOLATED Margin 20x`** | 🟢 ACTIVE | Tự động gọi API đặt `marginType = ISOLATED` và `leverage = 20x` khi khởi động cặp tiền để cách ly rủi ro tuyệt đối. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`EC-08`** | **Nguyên Tắc Cô Lập Lỗi (Fault Isolation)** | 🟢 ACTIVE | Lỗi ở 1 cặp tiền (ví dụ thiếu ký quỹ) chỉ chuyển cặp đó sang `ERROR`/`PAUSED`, tuyệt đối không làm dừng hay ảnh hưởng đến các cặp tiền khác. | `engine/specs/engine_facade_and_lifecycle.md`, `engine/specs/fault_isolation_error_handler.md` |
| **`EC-09`** | **Xóa Cặp Tiền Khỏi Watchlist (`remove_asset`)** | 🟢 ACTIVE | Ngắt stream WS, gỡ khỏi `routing_table`, giải phóng RAM nhưng giữ nguyên vẹn dữ liệu nến và lịch sử trade trong SQLite. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`EC-10`** | **Quản Lý Đòn Bẩy Động & Nuke Playbook** | 🟡 PROPOSED | Bổ sung `Action::ChangeLeverage { leverage: u32 }` cho phép chiến lược tự nâng đòn bẩy khi xuất hiện mẫu hình nuke thanh khoản cực hạn. | `engine/discussions/dynamic_leverage.md` |
| **`EC-11`** | **Kiến Trúc Quản Lý Lỗi Tập Trung & Bảng Ánh Xạ Mã Lỗi (`BotError`)** | 🟢 ACTIVE | Enum `BotError` (dùng `thiserror`) phân loại chi tiết các nhóm lỗi: `ApiError`, `NetworkError`, `MarginError`, `MarketFilterError`, `StateError` kèm ma trận Retry/Sleep/Log. | `engine/specs/fault_isolation_error_handler.md`, `engine/specs/binance_live_execution.md` |
| **`EC-12`** | **Quy Gom Hằng Số Vận Hành (`EngineConfig Struct`)** | 🟢 ACTIVE | Struct cấu hình tập trung chứa toàn bộ tham số môi trường: `initial_cash`, `commission_rate`, `margin_rate`, `max_live_mem_candles`, `historical_sync_days`, `buffer_padding_bars`. *(Đã chuẩn hóa 1:1 vào `live_demo_runner.rs`)* | `engine/specs/execution_engine_architecture_design.md` |

---

## 🛡️ 4. PHÂN HỆ THỰC THI & PHÒNG NGỰ SÀN (EXECUTION ENGINE & RISK DEFENSE)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`EX-01`** | **Động Cơ Khớp Lệnh Kép (`ExecutionEngineFactory`)** | 🟢 ACTIVE | Đa hình hóa giữa `PaperExecutionEngine` (giả lập PnL, phí sàn, trượt giá) và `BinanceLiveExecutionEngine` (gửi lệnh thực tế tới sàn). *(Đã tích hợp công tắc 3 chế độ `ExecutionMode` vào `live_demo_runner.rs`)* | `engine/specs/execution_engine_architecture_design.md`, `execution/engine.rs` |
| **`EX-02`** | **Mark Price Trailing SL Guard** | 🟢 ACTIVE | Toàn bộ lệnh Algo `STOP_MARKET` (`closePosition: true`) đều dùng `working_type: WorkingTypeEnum::MarkPrice` để chống râu nến ảo quét thanh khoản trong 0.1s. | `engine/specs/trailing_sl_execution.md` |
| **`EX-03`** | **Quy Trình Chống Xung Đột Mạng (Anti-Race Condition)** | 🟢 ACTIVE | Hủy toàn bộ SL cũ $\rightarrow$ Chờ 150ms $\rightarrow$ Vòng lặp Retry 3 lần đặt SL mới; tích hợp hàm định dạng `format_price_decimal` & `format_qty_decimal`. | `engine/specs/trailing_sl_execution.md` |
| **`EX-04`** | **Chuẩn Hóa Số Thập Phân (`MarketFilter`) & Công Thức Order Sizing** | 🟢 ACTIVE | Làm tròn giá theo `tick_size`, khối lượng theo `step_size`; kiểm tra ràng buộc kép: $\text{Order Qty} \ge \text{min\_qty}$ VÀ $\text{Notional} \ge 5.0\text{ USDT}$. | `engine/specs/trailing_sl_execution.md`, `engine/specs/capital_and_risk_compounding.md` |
| **`EX-05`** | **Bảo Toàn Initial SL Khi PnL Âm (SL Invariance)** | 🟢 ACTIVE | Ràng buộc `current_pnl > 0.0` trước khi cho phép Trailing SL dời bám đỉnh, ngăn chặn bóp nghẹt Stop Loss khi vị thế đang âm danh nghĩa. | `debugs/05_position_sync_and_sl_invariance/02_initial_sl_invariance_proof.md` |
| **`EX-06`** | **Sửa Lỗi Phân Loại `ExitReason` trong PaperTrader** | 🟢 ACTIVE | So sánh mức giá SL kích hoạt với `initial_sl_price` thay vì `entry_price` để ghi nhận chính xác các lệnh thoát bằng Trailing SL. | `debugs/06_mtf_streaming_and_bridge_desync/04_paper_trader_exit_reason_misclassification.md` |
| **`EX-07`** | **Cơ Chế Thả Diều Siêu Sóng (Mega Trend Hard Floor)** | 🟡 PROPOSED | Khi $\text{uPnL} \ge \text{trigger}$: Khóa mức sàn bảo hiểm cứng ($\text{SL}_{\text{Floor}} \ge 50\%$ Peak ROE) và nới lỏng Trailing SL ($8 - 15\text{ ATR}$) để gồng trọn siêu sóng. | `engine/discussions/mega_trend_and_dynamic_risk.md` |
| **`EX-08`** | **Khóa Cooldown Chống Lặp Lệnh (Micro-Churn Cooldown)** | 🟡 PROPOSED | Khóa không cho Playbook vừa đóng mở lại lệnh trong $N$ nến sau khi đóng tại Break-Even ($\text{PnL} \le 0.001$), gỡ khóa nếu có Volatility Breakout $\ge 1.5\text{ ATR}$. | `engine/discussions/mega_trend_and_dynamic_risk.md`, `engine/discussions/mega_trend_todo.md` |
| **`EX-09`** | **Realtime Mark Price Guard Độc Lập** | 🟡 PROPOSED | Luồng WS Mark Price mili-giây riêng biệt, kích hoạt Market Close khẩn cấp ngay giữa nến nếu chạm SL trước khi nến 1m đóng cửa. | `engine/discussions/mega_trend_and_dynamic_risk.md` |
| **`EX-10`** | **Gửi Lệnh Qua Pure WebSocket API thay vì REST API** | ⚠️ CONFLICT | Tài liệu PRD cũ đề xuất dùng WS API (`DerivativesTradingUsdsFuturesWsApi`) để giảm latency xuống 10-30ms. Hiện tại code đang dùng REST API qua `binance_sdk`. | `engine/specs/binance_live_execution.md` |
| **`EX-11`** | **Khóa Một Chiều Đơn Điệu (Monotonic Ratchet Guard)** | 🟢 ACTIVE | Đảm bảo Stop Loss chỉ được tịnh tiến theo chiều có lợi bảo vệ vốn (Long: $\max(SL_{old}, SL_{new})$, Short: $\min(SL_{old}, SL_{new})$), bất biến trước mọi chu kỳ Re-Opt. Đã đồng bộ 1:1 giữa `PaperTrader` và `OrderExecutor` (`evaluate_ratchet_sl`, chặn Naked Position và tiết kiệm API weight). | `engine/discussions/mega_trend_and_dynamic_risk.md` |
| **`EX-12`** | **Kích Hoạt Break-Even Động Theo Độ Nén (Squeeze-Aware BE)** | 🟡 PROPOSED | $\text{BE\_Distance} = \text{ATR} \times (\text{base\_sl\_atr} \times (1.0 + \text{squeeze\_multiplier}))$. Giãn rộng khi nén thanh khoản và thu hẹp khi bứt phá. | `engine/discussions/mega_trend_and_dynamic_risk.md` |
| **`EX-13`** | **Chi Tiết Trait `ExecutionEngine` & `ExecutionCore`** | 🟢 ACTIVE | Interface chuẩn hóa: `execute_action`, `on_candle`, `get_equity`, `get_position`, `on_account_event`. Struct `ExecutionCore` gom chung logic làm tròn và validate sàn. | `engine/specs/execution_engine_architecture_design.md` |
| **`EX-14`** | **Định Dạng Báo Cáo Giao Dịch Chi Tiết Trong `PaperTrader`** | 🟢 ACTIVE | Bổ sung các cột thông tin chuẩn mực vào bảng log `log_all_trades`: `OPEN TIME` (`MM-dd HH:mm`), `CLOSE TIME` (`MM-dd HH:mm`), `HELD` (số nến/phút thực tế `bars_held`). | `engine/discussions/mega_trend_todo.md` |
| **`EX-15`** | **Chiến Lược Kiểm Thử Đa Lớp (3-Tier Testing Architecture)** | 🟢 ACTIVE | Phân tầng kiểm thử: Lớp 1 Unit Tests (Math, Rounding), Lớp 2 Integration Tests (Mock signals, OrderExecutor), Lớp 3 Live Testnet Execution (`BINANCE_USE_TESTNET=true`). | `engine/specs/binance_live_execution.md` |

---

## 🔌 5. PHÂN HỆ CẦU NỐI DESKTOP CLIENT (TAURI V2 IPC & SYSTEM TRAY)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô TẢ & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`IPC-01`** | **Tích Hợp `bot_engine` Thành Thư Viện Crate Trong Tauri** | 🟢 ACTIVE | Đưa `bot_engine` vào `src-tauri/Cargo.toml`, lưu trữ `Arc<EngineController>` trong `tauri::State`. Zero Network Port. | `engine/README.md` |
| **`IPC-02`** | **Bộ Lệnh Điều Khiển Tauri Commands Chuẩn Hóa** | 🟢 ACTIVE | `cmd_start_asset`, `cmd_cancel_start`, `cmd_pause_asset`, `cmd_resume_asset`, `cmd_graceful_stop`, `cmd_cancel_graceful_stop`, `cmd_force_stop_asset`, `cmd_remove_asset`, `cmd_add_asset`, `cmd_rebalance_capital`, `cmd_get_full_snapshot`, `cmd_emergency_stop_all`, `cmd_unified_exit`. | `engine/TODO.md`, `engine/specs/engine_facade_and_lifecycle.md` |
| **`IPC-03`** | **Tauri Event Emitter (Delta Event Streaming)** | 🟢 ACTIVE | Phát các sự kiện thời gian thực (`event_asset_state_updated`, `event_candle_tick`, `event_position_updated`, `event_toast_notification`) về React UI. | `engine/TODO.md` |
| **`IPC-04`** | **Chạy Ẩn Khay Hệ Thống (System Tray 24/7)** | 🟢 ACTIVE | Bấm `[X]` cửa sổ tự ẩn xuống System Tray, bot tiếp tục chạy nền. Menu Tray 4 mục: 🟢 `Open Desk`, ⏸️ `Pause All`, 🚨 `Emergency Close All`, ❌ `Exit Application`. | `engine/specs/system_tray_and_exit_protocol.md` |
| **`IPC-05`** | **Giao Thức Thoát Tổng Hợp (Unified Exit Protocol)** | 🟢 ACTIVE | Chọn Exit $\rightarrow$ Phát event mở **Bảng Xác Nhận Thoát Tổng Hợp Duy Nhất** (`Flat All`, `Leave Cloud SL`, `Cancel`). | `engine/specs/system_tray_and_exit_protocol.md` |

---

## 🖥️ 6. PHÂN HỆ GIAO DIỆN NGƯỜI DÙNG (REACT 19 DESKTOP UI)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`UI-01`** | **App Shell, Slate Cyberpunk Tokens & Navigation** | 🟢 ACTIVE | Nền `#0F172A` (Slate 900), Card `#1E293B` (Slate 800); Topbar (Demo Badge, IPC Health Status); State-based Navigation `activePage: 'control' \| 'dashboard'`. | `UI/specs/app_layout_and_navigation.md`, `UI/specs/design_system_and_state.md` |
| **`UI-02`** | **Trang 2: Active Watchlist Table & Context Controls** | 🟡 PROPOSED | Quản lý từng cặp tiền, Badges trạng thái FSM đầy đủ, Champion DNA Metrics, Live Position, Trailing SL Target (Mark Price + 🛡️), Cột Allocated Capital kèm nút inline `[⚙️ Rebalance]`, Bộ nút thao tác theo ngữ cảnh. *(Tạm hoãn chi tiết để làm sau)*. | `UI/specs/bot_control_center.md` |
| **`UI-03`** | **Trang 2: Bảng Event Log Thu Nhỏ (`AssetLogPanel`)** | 🟢 ACTIVE | Hiển thị 5 log sự kiện gần nhất khi bấm chọn 1 cặp tiền cụ thể trên Watchlist. | `UI/specs/bot_control_center.md` |
| **`UI-04`** | **Trang 1: Thẻ KPI Tổng Quan (`KpiSummaryCards`)** | 🟢 ACTIVE | 4 thẻ thống kê: Wallet Balance, Available Balance, Total uPnL, Active Assets Count. | `UI/specs/executive_dashboard.md` |
| **`UI-05`** | **Trang 1: Thanh Sức Khỏe Hệ Thống (`SystemHealthBar`)** | 🟢 ACTIVE | 3 khối thời gian thực: REST API Weight (ProgressBar đổi màu Xanh $\le 50\%$, Vàng $>50\%$, Đỏ $>80\%$), WS Kline Latency (ms), SQLite Store Status. | `UI/specs/executive_dashboard.md` |
| **`UI-06`** | **Trang 1: Nút Dừng Khẩn Cấp Toàn Cục (`GlobalKillSwitch`)** | 🟢 ACTIVE | Nút lớn Emergency Stop All Bots kèm Modal xác nhận 2 bước chống bấm nhầm; tự động disabled khi không có asset nào active. | `UI/specs/executive_dashboard.md` |
| **`UI-07`** | **Hộp Thoại Điều Chuyển Vốn An Toàn (Rebalancing Modal)** | 🟢 ACTIVE | Modal popup khi giảm vốn cặp đang mở vị thế: `📌 Áp dụng từ lệnh kế tiếp` (khuyến nghị) vs `⚡ Đóng lệnh ngay lập tức`. | `engine/specs/capital_and_risk_compounding.md` |
| **`UI-08`** | **Bảng Xác Nhận Thoát Tổng Hợp (Unified Exit Modal)** | 🟢 ACTIVE | Modal hiển thị danh sách toàn bộ vị thế đang mở (6 cột: Symbol, Side, Size, Entry, Live uPnL, Binance SL Target) kèm 3 nút: `[⚡ ĐÓNG SẠCH TẤT CẢ MARKET]`, `[🛡️ GIỮ NGUYÊN STOP LOSS TRÊN SÀN]`, `[↩ HỦY BỎ]`. | `engine/specs/system_tray_and_exit_protocol.md` |
| **`UI-09`** | **Hộp Thoại Thêm Cặp Tiền Mới (`[➕ THÊM CẶP TIỀN]`)** | 🟢 ACTIVE | Modal tìm kiếm và thêm symbol mới vào Watchlist từ danh sách coin hỗ trợ của Binance Futures. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`UI-10`** | **Hệ Thống Toast Cảnh Báo Lỗi Tức Thời** | 🟢 ACTIVE | Toast thông báo lỗi cô lập từ backend (ví dụ thiếu ký quỹ) mà không chặn luồng thao tác của người dùng. | `engine/specs/engine_facade_and_lifecycle.md` |
| **`UI-11`** | **Nối Dây IPC Real-Time (`botService.ts`)** | 🟢 ACTIVE | Thay thế toàn bộ mock timer bằng gọi Tauri `invoke()` và `listen()` cập nhật state tức thì. | `UI/TODO.md` |
| **`UI-12`** | **Trang Trade Journal & Live Charts** | 🔴 DEPRECATED | Thiết kế cũ đề xuất vẽ biểu đồ nến và nhật ký trade phức tạp trên UI. Đã tạm hoãn để tối giản hóa giao diện thông tin sinh tồn cho bản Demo. | `engine/discussions/grill_me_questions_v1.md` |
| **`UI-13`** | **Hệ Thống Thanh Tab Lọc Nhanh (Fast Filter Tab System)** | 🟢 ACTIVE | Thanh tab 5 chế độ lọc phía trên Watchlist: `ALL`, `🟢 ACTIVE TRADING`, `⏸️ PAUSED`, `⚪ INACTIVE`, `💧 LIQUIDITY PASSED`. | `UI/specs/bot_control_center.md` |
| **`UI-14`** | **Console Log Stream Thu Nhỏ Trên Dashboard (Mini Console)** | 🟢 ACTIVE | Stream log tóm tắt các sự kiện toàn hệ thống từ Rust Backend hiển thị trên Trang 1 (Executive Dashboard). | `UI/TODO.md` |
| **`UI-15`** | **Bộ Lọc Thanh Khoản Tự Động (Liquidity Screener)** | 🟡 PROPOSED | Quét toàn bộ thị trường Binance Futures, tự động đề xuất và chấm điểm các cặp coin đạt tiêu chuẩn thanh khoản cao ($\text{Volume 24h} > \$50\text{M}$, $\text{Spread} < 0.02\%$). | `UI/specs/bot_control_center.md` |
| **`UI-16`** | **Báo Cáo Tài Chính & Kê Khai Thuế (Tax & Financial Reporting)** | 🟡 PROPOSED | Tự động tổng hợp lịch sử giao dịch, tính toán Realized PnL ròng sau khi trừ toàn bộ phí sàn Binance (Taker/Maker/Funding fee), xuất báo cáo CSV/Excel. | `UI/specs/bot_control_center.md` |
| **`UI-17`** | **Trang Cài Đặt Hệ Thống & Khóa API (Settings & API Key)** | 🔴 DEPRECATED / Post-Demo | Menu item `[🔒 Settings & API]` trong Sidebar ở trạng thái khóa (disabled), cấu hình API key qua biến môi trường `.env`. | `UI/specs/app_layout_and_navigation.md` |
| **`UI-18`** | **Thư Viện Định Dạng Số Chuẩn Hóa & Bộ Kiểm Thử Vitest** | 🟢 ACTIVE | Bộ hàm `formatters.ts` (tiền tệ `-$50.50`, phần trăm `+12.34%`, mapping Badge tiếng Việt) kèm bộ kiểm thử `formatters.test.ts` đảm bảo chính xác 100%. | `UI/TODO.md`, `bot_desktop_client/src/utils/formatters.ts` |

---

## ⚠️ 7. TỔNG HỢP CÁC ĐIỂM MÂU THUẪN & CẦN THẢO LUẬN (KEY TRIAGE TOPICS)

Dưới đây là **4 chủ đề trọng tâm** cần chúng ta thảo luận và chốt dứt điểm:

1. ⚠️ **Chủ đề 1: REST API vs Pure WebSocket API cho Đặt Lệnh (`EX-10`):**
   - *Tài liệu cũ (`binance_live_execution`):* Đề xuất dùng Pure WebSocket API (`DerivativesTradingUsdsFuturesWsApi`) để giảm độ trễ đặt lệnh xuống 10-30ms.
   - *Mã nguồn hiện tại:* Đang dùng REST API qua `binance_sdk::RestApi` (độ trễ 100-200ms). Với chiến lược nến 1m Helios V3, REST API đã đủ nhanh hay bắt buộc phải chuyển sang WS API?
2. 🟡 **Chủ đề 2: Cơ Chế Thả Diều Siêu Sóng Mega Trend (`EX-07`):**
   - Tài liệu `mega_trend_and_dynamic_risk` đề xuất cơ chế khóa Hard Profit Floor + nới lỏng Trailing SL khi PnL vượt ngưỡng. Tính năng này nên đưa vào Phase Live hiện tại hay để sau khi test xong bản cơ sở?
3. 🟡 **Chủ đề 3: Realtime Mark Price Guard Giữa Nến (`EX-09`):**
   - Hiện tại `OrderExecutor` đã treo lệnh Algo `STOP_MARKET` bằng Mark Price trên server Binance (sàn tự động kích hoạt dù bot rớt mạng). Có cần thiết phải duy trì thêm luồng WS Mark Price nội bộ trong bot để panic close giữa nến nữa không?
4. 🟡 **Chủ đề 4: Tần Suất & Cơ Chế Dọn Dẹp SQLite (`DE-09`):**
   - Thống nhất chính sách giữ rolling 33 ngày nến: Xóa tự động lúc khởi động bot hay chạy cron ngầm định kỳ?
