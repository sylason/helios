# 📋 MASTER ROADMAP: CHUẨN HÓA `live_demo_runner.rs` & ĐỒNG BỘ PAPER -> LIVE

Tài liệu quản lý danh mục công việc (TODO) nhằm đưa [`live_demo_runner.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/live_demo_runner.rs) đạt tính tương thích hành vi 1:1 (Behavioral Parity) với [`walk_forward_smart_seed.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/walk_forward_smart_seed.rs), đồng thời làm cầu nối kiểm chứng từ Paper Trade sang Testnet và Real Cash.

> [!NOTE]
> **Nguyên tắc quản trị:** Chúng ta vẫn đang trong quá trình debug song song Paper Trade, Helios V3 và Walk-Forward. Do đó, các tác vụ sau khi hoàn thành sẽ được phân loại rõ trạng thái:
> - `[x]` **COMPLETED**: Đã xong và đã kiểm chứng.
> - `[ ]` **IN PROGRESS**: Đang triển khai ngay.
> - `[PENDING / POST-DEBUG]`: Tạm hoãn, sẽ quay lại sau khi Paper Trade và Helios V3 hoàn toàn ổn định.

---

## 🧭 GIAI ĐOẠN 1: CHUẨN HÓA CẤU HÌNH & HÀNH VI 1:1 VỚI WALK-FORWARD
*Mục tiêu: Đưa toàn bộ thông số, logic khởi tạo, chu kỳ Re-Opt và quản lý rủi ro của `live_demo_runner.rs` khớp hoàn toàn với kết quả tốt nhất ở `walk_forward_smart_seed_30.log`.*

- [x] `[T1.1]` **Quy gom Bảng Điều khiển Trung tâm (Central Config Table)**:
  - Đưa toàn bộ hằng số cấu hình lên đầu file: `INITIAL_CASH = 60.0`, `LEVERAGE = 20.0`, `NUKE_LEVERAGE = 50.0`, `PERCENT_PER_TRADE = 0.20`, `MIN_ORDER_USD = 21.0`, `MAX_CAPITAL_FRACTION = 0.25`, `COMMISSION_RATE = 0.0004`, `MAX_LIVE_MEM_CANDLES = 43100` (đủ 50+ nến 4h), `BUFFER_PADDING_BARS = 200`.
- [x] `[T1.2]` **Cấu hình Rủi ro Chiến lược 1:1 (`HeliosStrategyV3`)**:
  - Gán chuẩn xác các giá trị `risk_base` (`leverage`, `nuke_leverage`, `percent_per_trade`, `min_order_usd`, `max_cap_fraction`) và bật `debug_mode = true` để ghi nhận đầy đủ Telemetry như `walk_forward`.
- [x] `[T1.3]` **Huấn luyện Initial GA Đồng Bộ trước khi nhận nến (Pre-Warmup)**:
  - Chạy `ProspectorRunner::run_optimization_cycle_range` trên 30 ngày nến lịch sử trước khi bật luồng nến WebSocket, ghi nhận Champion DNA ban đầu để bot sẵn sàng trade ngay từ nến 1m đầu tiên.
- [x] `[T1.4]` **Đồng bộ Chu kỳ Re-Opt & Smart Seeding 0.5% (`GA-01`, `GA-05`)**:
  - Thay đổi thời gian ngủ từ 5 phút sang 240 phút (4h).
  - Tích hợp hàm `run_optimization_cycle_range_smart_seed`, truyền `rank1_seed_dna` (0.5%) và xóa `prospector_db` trước mỗi chu kỳ để 99.5% quần thể được khám phá mới.
- [x] `[T1.5]` **Tích hợp Công tắc Đa Môi trường 3 Chế độ (`ExecutionMode`)**:
  - Cho phép người dùng cấu hình trực tiếp:
    1. `ExecutionMode::Paper` (Nhận nến WS Binance thật, khớp lệnh Paper giả lập $60 để test không rủi ro).
    2. `ExecutionMode::Testnet` (Khớp lệnh thật trên Binance Futures Testnet).
    3. `ExecutionMode::RealTrade` (Khớp lệnh tiền thật trên Binance Futures Live).

---

## 🛡️ GIAI ĐOẠN 2: ĐỒNG BỘ HÓA LOGIC THỰC THI (LIVE VS PAPER TRADE PARITY)
*Mục tiêu: Đảm bảo khi gửi lệnh thật qua `OrderExecutor`, hành vi khớp lệnh và phòng vệ sàn hoàn toàn tương đồng với `PaperTrader`.*

- [x] `[T2.1]` **Đồng bộ Khóa Một Chiều Đơn Điệu (`EX-11` Monotonic Ratchet Guard)**:
  - Kiểm tra và đảm bảo Stop Loss thật trên Binance chỉ tịnh tiến bảo vệ vốn (Long chỉ tăng, Short chỉ giảm), không bao giờ lùi.
  - Tích hợp `active_sl_prices: Arc<RwLock<HashMap<String, f64>>>` và hàm thuần túy `OrderExecutor::evaluate_ratchet_sl(is_long, old_sl, new_sl, has_position)` chống Naked Position và chặn spam lệnh trùng giá. Đã pass 6/6 unit tests.
- [ ] `[T2.2]` **Đồng bộ Quy trình Chống Xung Đột Mạng (`EX-03` Anti-Race Condition)**:
  - Kiểm tra quy trình: Hủy SL cũ $\rightarrow$ Nghỉ 150ms $\rightarrow$ Retry 3 lần đặt SL mới qua Algo `STOP_MARKET` Mark Price (`EX-02`).
- [ ] `[T2.3]` **Đồng bộ Công thức Tính Khối Lượng Lệnh (`EC-04`, `EX-04`)**:
  - Đảm bảo khối lượng lệnh Live tuân thủ vốn sàn lũy tiến và làm tròn chính xác theo `step_size`, `tick_size` và trần `min_notional >= 5.0 USDT`.
- [ ] `[T2.4]` **Đồng bộ Cơ Chế Khóa Cooldown (`EX-08` Micro-Churn Cooldown)**:
  - Đảm bảo khi vị thế đóng tại hòa vốn, bot kích hoạt Cooldown chống bào mòn vốn đúng như Paper Trade.
- [ ] `[T2.5]` **Xử lý Sự kiện Tài khoản Tức thì (`DE-10` User Data Stream)**:
  - Đảm bảo khi lệnh Stop Loss trên sàn cắn khớp, `AssetActor` nhận được sự kiện `ORDER_TRADE_UPDATE`, giải phóng vị thế và kích hoạt `Pending DNA` (`GA-07`) ngay lập tức.

---

## 📊 GIAI ĐOẠN 3: KIỂM CHỨNG, BÁO CÁO & CẬP NHẬT TÀI LIỆU
*Mục tiêu: Chạy thử nghiệm thực tế, rà soát log và cập nhật hồ sơ kỹ thuật.*

- [ ] `[T3.1]` **Chạy Kiểm chứng Chế độ `PaperLive` (WS Live + Paper Engine)**:
  - Chạy `live_demo_runner.rs` với WebSocket trực tiếp để xác nhận luồng nến 1m, chu kỳ 4h Re-Opt và nạp Pending DNA hoạt động chuẩn xác trong thời gian thực.
- [ ] `[T3.2]` **Chạy Kiểm chứng Chế độ `BinanceTestnet` (Micro-Order Verification)**:
  - Đặt lệnh thật trên Testnet để kiểm tra độ trễ mạng, hủy/đặt lại Trailing SL và phản hồi WebSocket.
- [ ] `[T3.3]` **Đồng bộ Tài liệu Hệ thống**:
  - Cập nhật các thay đổi vào [`FEATURE_INVENTORY.md`](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/FEATURE_INVENTORY.md) và Master [`trading_bot/TODO.md`](file:///E:/Projects/Automations/trading_backtest/trading_bot/TODO.md).

---

## ⏳ CÁC HẠNG MỤC TẠM HOÃN / QUAY LẠI SAU (`[PENDING / POST-DEBUG]`)
*Những tính năng này cần chờ giai đoạn debug Paper Trade & Helios V3 ổn định hoàn toàn mới kích hoạt:*

- `[PENDING]` **`EX-07` Mega Trend Hard Floor**: Cơ chế thả diều siêu sóng nới lỏng SL khi uPnL vượt ngưỡng (Cần thêm dữ liệu kiểm chứng Out-of-Sample).
- `[PENDING]` **`EX-12` Dynamic Squeeze-Aware BE**: Giãn/thu khoảng cách Break-Even theo độ nén Bollinger Bands.
- `[PENDING]` **`EX-10` Pure WebSocket API cho Đặt Lệnh**: Tạm thời giữ nguyên REST API vì nến 1m không yêu cầu độ trễ micro-giây.
- `[PENDING]` **`DE-09` Tự Động Dọn Nến SQLite Định Kỳ**: Tạm thời dọn dẹp thủ công hoặc nạp mới khi cần.
