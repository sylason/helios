# 📚 TỪ ĐIỂN TÍNH NĂNG TOÀN DIỆN (FEATURE INVENTORY & SPEC AUDIT)

Tài liệu này là **Bản ghi Tính năng Độc bản (Single Source of Truth)** tổng hợp, bóc tách và phân loại $100\%$ các tính năng, cơ chế và yêu cầu kỹ thuật đã được ghi nhận trong toàn bộ tài liệu của `trading_bot/docs/` từ trước đến nay.

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
| **`DE-02`** | **Động cơ Phân tích DuckDB** | 🔴 DEPRECATED | Thiết kế ban đầu dùng DuckDB lưu nến lịch sử. Đã được thay thế hoàn toàn bằng SQLite để tối ưu hóa tính nhỏ gọn Local-First và tránh lock file. | `blueprints/02_System_Architecture/database_schema.md` |
| **`DE-03`** | **Single Shared WsManager (Combined Stream)** | 🟢 ACTIVE | 1 kết nối WebSocket duy nhất tới `wss://fstream.binancefuture.com/stream?streams=...` gom chung toàn bộ các cặp tiền, tránh bị sàn chặn IP. | `engine/architecture/shared_ws_and_routing.md` |
| **`DE-04`** | **Bảng Điều Hướng Gói Tin $O(1)$ (`routing_table`)** | 🟢 ACTIVE | `Arc<RwLock<HashMap<String, Sender<Candle>>>>` trong `WsManager` để định tuyến nến trực tiếp cho đúng `AssetActor` mà không làm phiền actor khác. | `engine/architecture/shared_ws_and_routing.md` |
| **`DE-05`** | **Đăng ký / Hủy Stream Động (Dynamic WS)** | 🟢 ACTIVE | Gửi payload JSON `SUBSCRIBE` / `UNSUBSCRIBE` trực tiếp trên kết nối WS hiện hữu khi thêm/bớt cặp tiền mà không cần ngắt kết nối. | `engine/architecture/shared_ws_and_routing.md` |
| **`DE-06`** | **Tự Tính Nến 1m từ WebSocket `aggTrade`** | 🔴 DEPRECATED | Tự tổng hợp nến từ tick `aggTrade`. Đã bỏ vì tốn CPU/RAM và tăng độ trễ không cần thiết; chuyển sang subscribe thẳng kênh Kline 1m. | `blueprints/05_Discussions_and_Decisions/grill_me_questions.md` |
| **`DE-07`** | **Tự Động Bù Khoảng Trống Nến (REST Gap Filler)** | 🟢 ACTIVE | Khi khởi động hoặc sau khi mất mạng, tự động gọi REST API kéo nến lịch sử để lấp đầy các khoảng trống dữ liệu trong SQLite. | `blueprints/03_Feature_Specifications/data_engine/spec.md` |
| **`DE-08`** | **Giám Sát Trọng Số API (`WeightMonitor`)** | 🟢 ACTIVE | Đọc header `X-MBX-USED-WEIGHT-1M`, kiểm soát giới hạn 2400 weight/phút. Cảnh báo ở 1920 và tạm hoãn gọi REST ở 2200. | `features/binance_live_execution/TECHNICAL_DESIGN.md` |
| **`DE-09`** | **Tự Động Dọn Dẹp Nến Cũ Hàng Ngày** | 🟡 PROPOSED | Tự động xóa dữ liệu nến cũ hơn 33 ngày khi mở ứng dụng để tiết kiệm dung lượng ổ cứng. | `blueprints/05_Discussions_and_Decisions/grill_me_questions.md` |

---

## 🧬 2. PHÂN HỆ TỐI ƯU HÓA DI TRUYỀN & THÍCH ỨNG (GA OPTIMIZER & DNA GUARD)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`GA-01`** | **Tối Ưu Hóa Cửa Sổ Trượt (Rolling Window 30D)** | 🟢 ACTIVE | Chạy định kỳ thuật toán di truyền (`Prospector GA`) trên 30 ngày nến gần nhất ($43.158$ nến 1m) để liên tục thích nghi với thị trường. | `GEMINI.md`, `tests/paper_forward_test/16_walk_forward_smart_seed_analysis.md` |
| **`GA-02`** | **Bộ Điều Phối JIT Optimizer (`Semaphore(1)`)** | 🟢 ACTIVE | Hàng đợi tuần tự dùng `tokio::sync::Semaphore(1)` để chỉ cho phép 1 task GA chạy tại 1 thời điểm, triệt tiêu nguy cơ nghẽn CPU đa luồng. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`GA-03`** | **Nguyên Tắc Nạp Nến JIT (Just-In-Time Query)** | 🟢 ACTIVE | Chỉ truy vấn 30 ngày nến mới nhất từ SQLite ngay tại thời điểm giành được Semaphore lock trước khi bắt đầu tính toán GA. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`GA-04`** | **Kiểm Tra Hạn Sử Dụng DNA (`DNA Freshness Guard`)** | 🟢 ACTIVE | Khi bấm Start: Nếu $\Delta t < 15$p $\rightarrow$ Nạp ngay (<1s) và trade lập tức; Nếu $\Delta t \ge 15$p $\rightarrow$ Xếp hàng chạy GA JIT rồi mới kích hoạt trade. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`GA-05`** | **Smart Seeding 0.5% Quần Thể** | 🟡 PROPOSED | Tiêm cá thể Rank 1 Champion từ chu kỳ trước làm hạt giống mồi vào thế hệ 0 (chiếm 0.5%), 99.5% là đột biến/LHS mới. *Cần kiểm soát nguy cơ Overfitting*. | `tests/paper_forward_test/16_walk_forward_smart_seed_analysis.md` |
| **`GA-06`** | **Hot-Swapping DNA Không Cần Khởi Động Lại** | 🟢 ACTIVE | Khi Re-Optimizer tìm ra DNA mới, truyền qua kênh `tokio::sync::watch` cập nhật trực tiếp vào `AssetActor` mà không làm gián đoạn bot. | `controller/asset_actor.rs`, `GEMINI.md` |
| **`GA-07`** | **Bảo Lưu Tham Số Rủi Ro Khi Hot-Swap (Anti-Decay Jump)** | 🟢 ACTIVE | Gắn cố định cấu hình rủi ro vào vị thế khi mở lệnh, ngăn chặn hiện tượng nhảy vọt `decay_cycles` làm đóng lệnh đột ngột khi nạp DNA mới. | `debugs/06_mtf_streaming_and_bridge_desync/03_dna_hotswap_decay_jump.md` |

---

## 👑 3. PHÂN HỆ LÕI ĐIỀU HÀNH ĐA TÀI SẢN & FSM (ENGINECONTROLLER)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`EC-01`** | **Facade API `EngineController`** | 🟢 ACTIVE | Cấu trúc trung tâm quản lý danh bạ `AssetActorRegistry`, điều phối kết nối WS, JIT Scheduler và lưu trữ SQLite. | `engine/README.md`, `engine/TODO.md` |
| **`EC-02`** | **Máy Trạng Thái Đối Xứng (Symmetric FSM)** | 🟢 ACTIVE | Vòng đời 8 trạng thái: `INACTIVE`, `STARTING`, `RUNNING_ACTIVE`, `PAUSED`, `GRACEFUL_STOPPING`, `STOPPED`, `ERROR`. Mọi lệnh Bật đều có Tắt/Hủy. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`EC-03`** | **Bộ Lệnh Điều Khiển Vòng Đời Đầy Đủ** | 🟢 ACTIVE | 8 lệnh: `start_asset`, `cancel_start`, `pause_trading`, `resume_trading`, `graceful_stop`, `cancel_graceful_stop`, `force_stop_immediate`, `remove_asset`. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`EC-04`** | **Quản Trị Vốn Lũy Tiến Kèm Vốn Sàn (Compounding Floor)** | 🟢 ACTIVE | $\text{Capital} = \max(\text{INITIAL\_BASE},\ \text{INITIAL\_BASE} + \sum \text{Realized PnL})$. Tự động tăng volume khi có lãi và bảo vệ không tụt dưới vốn cơ sở ban đầu. | `engine/architecture/capital_and_risk_compounding.md` |
| **`EC-05`** | **Luồng Điều Chuyển Vốn An Toàn (Capital Rebalancing)** | 🟢 ACTIVE | Cho phép điều chuyển lãi thực giữa các cặp tiền. Nếu cặp đang mở vị thế $\rightarrow$ Phân nhánh lựa chọn an toàn (áp dụng lệnh sau vs đóng lệnh ngay). | `engine/architecture/capital_and_risk_compounding.md` |
| **`EC-06`** | **Tiếp Quản Vị Thế Có Sẵn (`Adopt & Protect`)** | 🟢 ACTIVE | Khi kích hoạt bot, tự động kiểm tra vị thế mở sẵn trên sàn Binance, nạp vào state và lập tức treo lệnh Trailing SL bảo vệ. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`EC-07`** | **Tự Động Cấu Hình `ISOLATED Margin 20x`** | 🟢 ACTIVE | Tự động gọi API đặt `marginType = ISOLATED` và `leverage = 20x` khi khởi động cặp tiền để cách ly rủi ro tuyệt đối. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`EC-08`** | **Nguyên Tắc Cô Lập Lỗi (Fault Isolation)** | 🟢 ACTIVE | Lỗi ở 1 cặp tiền (ví dụ thiếu ký quỹ) chỉ chuyển cặp đó sang `ERROR`/`PAUSED`, tuyệt đối không làm dừng hay ảnh hưởng đến các cặp tiền khác. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`EC-09`** | **Xóa Cặp Tiền Khỏi Watchlist (`remove_asset`)** | 🟢 ACTIVE | Ngắt stream WS, gỡ khỏi `routing_table`, giải phóng RAM nhưng giữ nguyên vẹn dữ liệu nến và lịch sử trade trong SQLite. | `engine/architecture/engine_facade_and_lifecycle.md` |

---

## 🛡️ 4. PHÂN HỆ THỰC THI & PHÒNG NGỰ SÀN (EXECUTION ENGINE & RISK DEFENSE)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`EX-01`** | **Động Cơ Khớp Lệnh Kép (`ExecutionEngineFactory`)** | 🟢 ACTIVE | Đa hình hóa giữa `PaperExecutionEngine` (giả lập PnL, phí sàn, trượt giá) và `BinanceLiveExecutionEngine` (gửi lệnh thực tế tới sàn). | `execution/engine.rs` |
| **`EX-02`** | **Mark Price Trailing SL Guard** | 🟢 ACTIVE | Toàn bộ lệnh Algo `STOP_MARKET` (`closePosition: true`) đều dùng `working_type: WorkingTypeEnum::MarkPrice` để chống râu nến ảo quét thanh khoản trong 0.1s. | `engine/architecture/trailing_sl_execution.md` |
| **`EX-03`** | **Quy Trình Chống Xung Đột Mạng (Anti-Race Condition)** | 🟢 ACTIVE | Hủy SL cũ $\rightarrow$ Chờ an toàn 150ms $\rightarrow$ Vòng lặp Retry 3 lần đặt SL mới để tránh xung đột sổ lệnh Binance. | `engine/architecture/trailing_sl_execution.md` |
| **`EX-04`** | **Chuẩn Hóa Số Thập Phân (`MarketFilter`)** | 🟢 ACTIVE | Tự động làm tròn giá theo `tick_size` và làm tròn khối lượng theo `step_size` từ `exchangeInfo` trước khi gửi lệnh. | `engine/architecture/trailing_sl_execution.md` |
| **`EX-05`** | **Bảo Toàn Initial SL Khi PnL Âm (SL Invariance)** | 🟢 ACTIVE | Ràng buộc `current_pnl > 0.0` trước khi cho phép Trailing SL dời bám đỉnh, ngăn chặn bóp nghẹt Stop Loss khi vị thế đang âm danh nghĩa. | `debugs/05_position_sync_and_sl_invariance/02_initial_sl_invariance_proof.md` |
| **`EX-06`** | **Sửa Lỗi Phân Loại `ExitReason` trong PaperTrader** | 🟢 ACTIVE | So sánh mức giá SL kích hoạt với `initial_sl_price` thay vì `entry_price` để ghi nhận chính xác các lệnh thoát bằng Trailing SL. | `debugs/06_mtf_streaming_and_bridge_desync/04_paper_trader_exit_reason_misclassification.md` |
| **`EX-07`** | **Cơ Chế Thả Diều Siêu Sóng (Mega Trend Hard Floor)** | 🟡 PROPOSED | Khi $\text{uPnL} \ge \text{trigger}$: Khóa mức sàn bảo hiểm cứng ($\ge 50\%$ Peak ROE) và nới lỏng Trailing SL ($8 - 15\text{ ATR}$) để gồng trọn siêu sóng. | `features/dynamic_adaptive_risk_and_mega_trend/ARCHITECTURE.md` |
| **`EX-08`** | **Khóa Cooldown Chống Lặp Lệnh (Micro-Churn Cooldown)** | 🟡 PROPOSED | Khóa không cho Playbook vừa đóng mở lại lệnh trong $N$ nến sau khi đóng tại Break-Even (trừ khi có Volatility Breakout $>1.0\text{ ATR}$). | `features/dynamic_adaptive_risk_and_mega_trend/ARCHITECTURE.md` |
| **`EX-09`** | **Realtime Mark Price Guard Độc Lập** | 🟡 PROPOSED | Luồng WS Mark Price mili-giây riêng biệt, kích hoạt Market Close khẩn cấp ngay giữa nến nếu chạm SL trước khi nến 1m đóng cửa. | `features/dynamic_adaptive_risk_and_mega_trend/ARCHITECTURE.md` |
| **`EX-10`** | **Gửi Lệnh Qua Pure WebSocket API thay vì REST API** | ⚠️ CONFLICT | Tài liệu PRD cũ đề xuất dùng WS API (`DerivativesTradingUsdsFuturesWsApi`) để giảm latency xuống 10-30ms. Hiện tại code đang dùng REST API qua `binance_sdk`. | `features/binance_live_execution/TECHNICAL_DESIGN.md` |

---

## 🔌 5. PHÂN HỆ CẦU NỐI DESKTOP CLIENT (TAURI V2 IPC & SYSTEM TRAY)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`IPC-01`** | **Tích Hợp `bot_engine` Thành Thư Viện Crate Trong Tauri** | 🟢 ACTIVE | Đưa `bot_engine` vào `src-tauri/Cargo.toml`, lưu trữ `Arc<EngineController>` trong `tauri::State`. Zero Network Port. | `engine/README.md` |
| **`IPC-02`** | **Bộ Lệnh Điều Khiển Tauri Commands** | 🟢 ACTIVE | `cmd_start_asset`, `cmd_cancel_start`, `cmd_pause_asset`, `cmd_resume_asset`, `cmd_graceful_stop`, `cmd_force_stop_asset`, `cmd_remove_asset`, `cmd_rebalance_capital`, `cmd_get_full_snapshot`, `cmd_emergency_stop_all`. | `engine/TODO.md` |
| **`IPC-03`** | **Tauri Event Emitter (Delta Event Streaming)** | 🟢 ACTIVE | Phát các sự kiện thời gian thực (`event_asset_state_updated`, `event_candle_tick`, `event_position_updated`, `event_toast_notification`) về React UI. | `engine/TODO.md` |
| **`IPC-04`** | **Chạy Ẩn Khay Hệ Thống (System Tray 24/7)** | 🟢 ACTIVE | Bấm `[X]` cửa sổ tự ẩn xuống System Tray, bot tiếp tục chạy nền. Menu Tray: `Open Desk`, `Pause All`, `Emergency Close All`, `Exit Application`. | `engine/architecture/system_tray_and_exit_protocol.md` |
| **`IPC-05`** | **Giao Thức Thoát Tổng Hợp (Unified Exit Protocol)** | 🟢 ACTIVE | Chọn Exit $\rightarrow$ Phát event mở **Bảng Xác Nhận Thoát Tổng Hợp Duy Nhất** (`Flat All`, `Leave Cloud SL`, `Cancel`). | `engine/architecture/system_tray_and_exit_protocol.md` |

---

## 🖥️ 6. PHÂN HỆ GIAO DIỆN NGƯỜI DÙNG (REACT 19 DESKTOP UI)

| Mã ID | Tên Tính Năng | Trạng Thái | Mô Tả & Bản Chất Kỹ Thuật | Tài Liệu Gốc |
| :--- | :--- | :---: | :--- | :--- |
| **`UI-01`** | **App Shell & State-Based Navigation** | 🟢 ACTIVE | Layout Dark Theme tối giản, Topbar (Demo Badge, IPC Status), Sidebar chuyển đổi giữa Dashboard & Bot Control Center. | `UI/architecture/app_layout_and_navigation.md` |
| **`UI-02`** | **Trang 2: Active Watchlist Table** | 🟢 ACTIVE | Quản lý từng cặp tiền (`ETHUSDT`, `BTCUSDT`, `SOLUSDT`), hiển thị trạng thái Badges, Champion DNA Metrics, Live Position, Trailing SL Target. | `UI/pages/bot_control_center.md` |
| **`UI-03`** | **Trang 2: Bảng Event Log Thu Nhỏ (`AssetLogPanel`)** | 🟢 ACTIVE | Hiển thị 5 log sự kiện gần nhất khi bấm chọn 1 cặp tiền cụ thể trên Watchlist. | `UI/pages/bot_control_center.md` |
| **`UI-04`** | **Trang 1: Thẻ KPI Tổng Quan (`KpiSummaryCards`)** | 🟢 ACTIVE | 4 thẻ thống kê: Wallet Balance, Available Balance, Total uPnL, Active Assets Count. | `UI/pages/executive_dashboard.md` |
| **`UI-05`** | **Trang 1: Thanh Sức Khỏe Hệ Thống (`SystemHealthBar`)** | 🟢 ACTIVE | Theo dõi API Weight Limit (2400 limit), trạng thái WebSocket connection, ping latency. | `UI/pages/executive_dashboard.md` |
| **`UI-06`** | **Trang 1: Nút Dừng Khẩn Cấp Toàn Cục (`GlobalKillSwitch`)** | 🟢 ACTIVE | Nút lớn Emergency Stop All Bots, đóng sạch toàn bộ vị thế trên sàn khi gặp sự cố. | `UI/pages/executive_dashboard.md` |
| **`UI-07`** | **Hộp Thoại Điều Chuyển Vốn An Toàn (Rebalancing Modal)** | 🟢 ACTIVE | Modal popup khi giảm vốn cặp đang mở vị thế: `📌 Áp dụng từ lệnh kế tiếp` (khuyến nghị) vs `⚡ Đóng lệnh ngay lập tức`. | `engine/architecture/capital_and_risk_compounding.md` |
| **`UI-08`** | **Bảng Xác Nhận Thoát Tổng Hợp (Unified Exit Modal)** | 🟢 ACTIVE | Modal hiển thị danh sách toàn bộ vị thế đang mở kèm 3 nút: `[⚡ ĐÓNG SẠCH TẤT CẢ MARKET]`, `[🛡️ GIỮ NGUYÊN STOP LOSS TRÊN SÀN]`, `[↩ HỦY BỎ]`. | `engine/architecture/system_tray_and_exit_protocol.md` |
| **`UI-09`** | **Hộp Thoại Thêm Cặp Tiền Mới (`[➕ THÊM CẶP TIỀN]`)** | 🟢 ACTIVE | Modal tìm kiếm và thêm symbol mới vào Watchlist từ danh sách coin hỗ trợ của Binance Futures. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`UI-10`** | **Hệ Thống Toast Cảnh Báo Lỗi Tức Thời** | 🟢 ACTIVE | Toast thông báo lỗi cô lập từ backend (ví dụ thiếu ký quỹ) mà không chặn luồng thao tác của người dùng. | `engine/architecture/engine_facade_and_lifecycle.md` |
| **`UI-11`** | **Nối Dây IPC Real-Time (`botService.ts`)** | 🟢 ACTIVE | Thay thế toàn bộ mock timer bằng gọi Tauri `invoke()` và `listen()` cập nhật state tức thì. | `UI/TODO.md` |
| **`UI-12`** | **Trang Trade Journal & Live Charts** | 🔴 DEPRECATED | Thiết kế cũ đề xuất vẽ biểu đồ nến và nhật ký trade phức tạp trên UI. Đã tạm hoãn để tối giản hóa giao diện thông tin sinh tồn cho bản Demo. | `blueprints/05_Discussions_and_Decisions/grill_me_questions.md` |

---

## ⚠️ 7. TỔNG HỢP CÁC ĐIỂM MÂU THUẪN & CẦN THẢO LUẬN (KEY TRIAGE TOPICS)

Dưới đây là **4 chủ đề trọng tâm** cần chúng ta thảo luận và chốt dứt điểm trong Bước 3:

1. ⚠️ **Chủ đề 1: REST API vs Pure WebSocket API cho Đặt Lệnh (`EX-10`):**
   - *Tài liệu cũ (`binance_live_execution`):* Đề xuất dùng Pure WebSocket API (`DerivativesTradingUsdsFuturesWsApi`) để giảm độ trễ đặt lệnh xuống 10-30ms.
   - *Mã nguồn hiện tại:* Đang dùng REST API qua `binance_sdk::RestApi` (độ trễ 100-200ms). Với chiến lược nến 1m Helios V3, REST API đã đủ nhanh hay bắt buộc phải chuyển sang WS API?
2. 🟡 **Chủ đề 2: Cơ Chế Thả Diều Siêu Sóng Mega Trend (`EX-07`):**
   - Tài liệu `features/dynamic_adaptive_risk_and_mega_trend` đề xuất cơ chế khóa Hard Profit Floor + nới lỏng Trailing SL khi PnL vượt ngưỡng. Tính năng này nên đưa vào Phase Live hiện tại hay để sau khi test xong bản cơ sở?
3. 🟡 **Chủ đề 3: Realtime Mark Price Guard Giữa Nến (`EX-09`):**
   - Hiện tại `OrderExecutor` đã treo lệnh Algo `STOP_MARKET` bằng Mark Price trên server Binance (sàn tự động kích hoạt dù bot rớt mạng). Có cần thiết phải duy trì thêm luồng WS Mark Price nội bộ trong bot để panic close giữa nến nữa không?
4. 🟡 **Chủ đề 4: Tần Suất & Cơ Chế Dọn Dẹp SQLite (`DE-09`):**
   - Thống nhất chính sách giữ rolling 33 ngày nến: Xóa tự động lúc khởi động bot hay chạy cron ngầm định kỳ?
