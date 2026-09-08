# 🟡 MASTER TODO: 24 LỖ HỔNG HIGH, MEDIUM & HIỆU NĂNG (PERFORMANCE)

> **Mục tiêu tài liệu:** Quản trị tập trung 24 lỗ hổng mức độ **HIGH**, **MEDIUM** và **PERFORMANCE** (tối ưu hóa tài nguyên CPU/RAM, hạn chế rủi ro biên, hoàn thiện kiến trúc và tăng tốc độ xử lý).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/TODO_HIGH_MEDIUM_PERF.md`  
> **Trạng thái tổng quan:** **3/24 lỗi đã được xử lý gián tiếp qua các bước trước**, **21/24 lỗi đang chờ xử lý**.

---

## 📊 Bảng Tiến Độ Tổng Quan Theo Nhóm Vấn Đề

| Nhóm vấn đề | Số lượng | Đã xử lý | Còn lại | Trọng tâm cần tối ưu |
| :--- | :---: | :---: | :---: | :--- |
| **Nhóm 1: Rủi Ro Biên & Đo Đạc Quản Trị Vốn (Risk & Metrics)** | 6 | 1 | 5 | Max Drawdown Paper, Sizing khi vốn âm, BE Buffer |
| **Nhóm 2: Toán Học Sensors & Chỉ Báo (Math & Indicators)** | 3 | 0 | 3 | Hazen Dedup, Bão hòa sớm ADX, Lệch biên DataView |
| **Nhóm 3: Hiệu Năng Đa Luồng & Phân Bổ Bộ Nhớ (Perf & Allocations)** | 4 | 0 | 4 | 250k Heap allocations/backtest, Rayon vs Tokio |
| **Nhóm 4: Ổn Định Vòng Đời, FSM & Phục Hồi (State & Lifecycle)** | 5 | 1 | 4 | Bù nến khi Broadcast Lagged, Unified Exit Ctrl+C |
| **Nhóm 5: Tương Thích Cấu Hình & Mã Nguồn Rác (Code Hygiene)** | 6 | 1 | 5 | Dọn rác duckdb, parse LinReg/Vidya, Dummy Champions |
| **TỔNG CỘNG** | **24** | **3** | **21** |  |

---

## 💰 NHÓM 1: RỦI RO BIÊN & ĐO ĐẠC QUẢN TRỊ VỐN (6 LỖ HỔNG)

- [ ] **FLAW-5.6 (High):** **Nguy Cơ `NaN` Trong `calculate_size` Khi Vốn $\le 0$**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:13-24`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L13-L24)
  - *Bản chất:* Phép chia `0.0 / 0.0` sinh ra `NaN`. Biểu thức `NaN.min(0.20)` trong Rust trả về `0.20`, tiếp tục cấp margin 20% cho tài khoản đã cháy vốn.
  - *Giải pháp:* Thêm guard `if equity <= 0.0 { return 0.0; }`.

- [ ] **FLAW-5.7 (Medium):** **Break-Even Buffer (0.08%) Không Đủ Bù Đắp Trượt Giá Sàn**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:242`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L242)
  - *Bản chất:* Mức đệm 0.08% chỉ vừa đủ bù phí danh nghĩa Binance Futures (0.04% Taker x 2). Khi gặp trượt giá (Slippage) lúc khớp lệnh Market, lệnh dời hòa vốn vẫn bị lỗ ròng.
  - *Giải pháp:* Nâng buffer lên $0.12\% - 0.15\%$ hoặc tham số hóa động theo ATR.

- [x] **FLAW-5.8 (Medium):** **Pessimistic Bias Tuyệt Đối Ở Nến Quét 2 Đầu** `[ĐÃ XỬ LÝ ✅]`
  - *Vị trí:* [`ferro_trade/src/broker/broker.rs:137-142`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/broker/broker.rs#L137-L142)
  - *Đã xử lý:* Đã đồng bộ hóa PaperTrader sang Pessimistic Matching ở Bước 4 để đảm bảo thống nhất 100% giữa Backtest và Paper.

- [x] **FLAW-7.10 (High):** **Bẫy Đóng Lệnh Cưỡng Bức Do `decay_cycles` Jump Khi Hot-Swap DNA** `[ĐÃ XỬ LÝ ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)
  - *Đã xử lý:* Triệt tiêu hoàn toàn nhờ cơ chế "Lock DNA on Active Trade" (Bước 1) và tái thiết kế Stagnant Decay (Bước 3).

- [ ] **FLAW-7.11 (High):** **Thiếu Đo Đạc Max Drawdown Trong PaperTrader**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/paper_trader.rs:60-80`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L60-L80)
  - *Bản chất:* PaperTrader chỉ theo dõi `equity` hiện tại mà không lưu trữ `peak_equity` và `max_drawdown_pct`, khiến người dùng mù về mức sụt giảm vốn lớn nhất trong quá trình Forward Test.
  - *Giải pháp:* Thêm `peak_equity: f64` và `max_drawdown_pct: f64` cập nhật theo từng nến.

- [ ] **FLAW-7.12 (High):** **Ngộ Nhận "Diagnostic Backtest" Là Out-Of-Sample**
  - *Vị trí:* [`trading_bot/bot_engine/examples/walk_forward_smart_seed.rs:241`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/walk_forward_smart_seed.rs#L241)
  - *Bản chất:* Chạy hàm `diagnostic_backtest` trên cửa sổ 30 ngày thực chất là đo lại In-Sample, không phản ánh đúng hiệu suất Out-Of-Sample của Paper Trader.
  - *Giải pháp:* Đổi tên rõ ràng thành In-Sample Diagnostic và bổ sung bảng đo lường PnL thực tế của PaperTrader.

---

## 📐 NHÓM 2: TOÁN HỌC SENSORS & CHỈ BÁO (3 LỖ HỔNG)

- [ ] **FLAW-1.6 (Medium):** **Lệch Biên `DataView::close_at` Ở Nến Cuối Cùng**
  - *Vị trí:* [`ferro_trade/src/data/data_view.rs:58`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/data/data_view.rs#L58)
  - *Bản chất:* Khi truy cập nến cuối cùng, việc ép kiểu `u32` có thể làm lệch biên giá đóng cửa nếu mảng nến chưa hoàn tất đồng bộ.
  - *Giải pháp:* Kiểm tra cận an toàn `i.min(self.total_len() - 1)`.

- [ ] **FLAW-3.9 (Medium):** **Float Epsilon Dedup Mismatch Trong `rolling_percentile`**
  - *Vị trí:* [`strategies_backtests/helios/src/utils/normalization.rs:77, 88`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/utils/normalization.rs#L77)
  - *Bản chất:* `dedup_by` dùng epsilon làm mốc phân biệt, nhưng `binary_search` lại dùng `total_cmp`. Các số thực lệch nhau $< \text{EPSILON}$ bị mất dấu, làm `binary_search` trả về Err và biến nến hợp lệ thành `NaN`.
  - *Giải pháp:* Đồng nhất logic so sánh số thực giữa dedup và search.

- [ ] **FLAW-3.10 (Medium):** **Bão Hòa Nhị Phân Sớm Của `trend_strength` Trong `tanh_scale`**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/matrix.rs:81`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L81)
  - *Bản chất:* Dùng nhầm `z_score_sensitivity` ($1 \to 10$) cho chỉ báo ADX ($0 \to 100$). Chỉ cần ADX $\ge 30$, hàm nén đã ép chặt về $0.997$, mất tính phân cấp liên tục của xu hướng.
  - *Giải pháp:* Chia tỉ lệ chuẩn hóa ADX về $[0.0 \dots 1.0]$ trước khi đưa vào hàm `tanh`.

---

## ⚡ NHÓM 3: HIỆU NĂNG ĐA LUỒNG & BỘ NHỚ (4 LỖ HỔNG)

- [ ] **FLAW-H.8 (Perf):** **Cấp Phát Heap Liên Tục (6 Lần `Box::new` / Nến) Trong `next()`**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/mod.rs:209`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L209)
  - *Bản chất:* Mỗi cây nến đều khởi tạo lại 6 Scanners bằng `Box::new`, gây ra $>250,000$ heap allocations cho mỗi backtest 43k nến, làm chậm GA Optimizer 30-40%.
  - *Giải pháp:* Lưu trữ cố định 6 Scanners trong struct `HeliosStrategyV3` hoặc dùng mảng tĩnh trên Stack.

- [ ] **FLAW-6.9 (High):** **Bẫy Overfitting Do Drawdown Diminishing Sensitivity**
  - *Vị trí:* [`ferro_trade/src/optimizer/prospector_runner.rs:180`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L180)
  - *Bản chất:* Công thức phạt Max DD bị giảm độ nhạy khi DD vượt quá 30%, khiến cá thể sụt giảm 35% và 50% bị phạt điểm số gần như tương đương nhau, làm GA ưu tiên cá thể liều lĩnh.
  - *Giải pháp:* Áp dụng hàm phạt lũy tiến bậc hai (Quadratic Penalty) cho Max Drawdown: $\text{Penalty} \propto (\text{MaxDD} - 0.25)^2$.

- [ ] **FLAW-6.11 (Medium):** **Rayon Thread Oversubscription Trong Tokio Worker Threads**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:115`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L115)
  - *Bản chất:* Chạy trực tiếp `rayon::par_iter()` bên trong async task của Tokio mà không dùng `spawn_blocking`, gây tranh chấp luồng CPU và làm giật lag vòng lặp sự kiện Tokio.
  - *Giải pháp:* Đóng gói toàn bộ tiến trình tối ưu GA vào `tokio::task::spawn_blocking`.

- [ ] **FLAW-7.14 (Medium):** **Nghẽn Async Event Loop Do `diagnostic_backtest` Chạy Đồng Bộ**
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs:173-181`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L173-L181)
  - *Bản chất:* Chạy backtest 43k nến đồng bộ ngay trong `tokio::select!` của Actor làm nghẽn luồng xử lý, dẫn đến rớt gói tin nến WebSocket (`data_rx LAGGED`).
  - *Giải pháp:* Chuyển diagnostic backtest sang luồng nền hoặc tách riêng khỏi luồng nhận nến.

---

## 🔄 NHÓM 4: ỔN ĐỊNH VÒNG ĐỜI, FSM & PHỤC HỒI (5 LỖ HỔNG)

- [ ] **FLAW-7.13 (High):** **High-Watermark & Age Reset Khi Bot Khởi Động Lại**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/live_engine.rs:130`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L130)
  - *Bản chất:* Khi tiếp quản vị thế sau restart, `entry_timestamp` bị gán `chrono::Utc::now()`, reset tuổi nến về 0 và làm mất toàn bộ đỉnh lợi nhuận lịch sử của vị thế.
  - *Giải pháp:* Truy vấn thời gian mở lệnh thực tế từ Binance User Trade History REST API.

- [ ] **FLAW-7.16 (Medium):** **Lệch Chuẩn Máy Trạng Thái (FSM) Giữa Thiết Kế Và Mã Nguồn**
  - *Vị trí:* [`trading_bot/bot_engine/src/models/models.rs:37-43`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/models.rs#L37-L43)
  - *Bản chất:* Enum trạng thái thiếu `INACTIVE`, `PAUSED`, `GRACEFUL_STOPPING`, khiến bot không có cơ chế dừng an toàn có kiểm soát.
  - *Giải pháp:* Hoàn thiện đầy đủ các trạng thái của vòng đời Actor theo tài liệu kiến trúc.

- [ ] **FLAW-8.8 (High):** **Kênh Broadcast Bị Lagged Làm Rớt Nến Vĩnh Viễn Không Bù Đắp**
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs:211`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L211)
  - *Bản chất:* Khi kênh broadcast của Tokio bị đầy, gói tin nến mới bị drop với lỗi `RecvError::Lagged`, Actor bỏ qua luôn mà không kích hoạt Gap Filler để kéo lại nến bị mất.
  - *Giải pháp:* Khi bắt gặp `Lagged(n)`, tự động kích hoạt `InitialSync` kéo bù $n$ cây nến bị mất từ SQLite.

- [ ] **FLAW-8.9 (High):** **Thoát Bàn Phím (Ctrl+C) Tự Bán Tháo Market Bỏ Qua Hộp Thoát Tổng Hợp**
  - *Vị trí:* [`trading_bot/bot_engine/src/main.rs:219`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/main.rs#L219)
  - *Bản chất:* Khi người dùng nhấn Ctrl+C, hệ thống kích hoạt `PanicCloseAll` bán tháo toàn bộ vị thế ở giá Market, vi phạm thiết kế 3 chế độ thoát (Flat All, Leave Cloud SL, Cancel).
  - *Giải pháp:* Kết nối tín hiệu Ctrl+C với menu điều khiển thoát tổng hợp hoặc chỉ hủy bot và giữ nguyên Trailing SL trên sàn.

- [ ] **FLAW-7.15 (Medium):** **Bài Test Phục Hồi Giả Tạo (`resilience_recovery_test.rs`)**
  - *Vị trí:* [`trading_bot/bot_engine/examples/resilience_recovery_test.rs:54-76`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/resilience_recovery_test.rs#L54-L76)
  - *Bản chất:* Testcase tự gán biến struct trong bộ nhớ rồi assert chính biến đó, không hề kiểm tra tính bền vững của SQLite hay kịch bản crash thực tế.
  - *Giải pháp:* Viết lại testcase ghi dữ liệu vào file SQLite tạm, crash actor và khôi phục từ file DB.

---

## 🧹 NHÓM 5: TƯƠNG THÍCH CẤU HÌNH & MÃ NGUỒN RÁC (5 LỖ HỔNG)

- [x] **FLAW-2.6 (Medium):** **Thuộc Tính Treo `#[dna]` Trên Trường Bool `enable_re_entry_cooldown`** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)
  - *Đã xử lý:* Đã gỡ bỏ thuộc tính `#[dna]` ở Bước 3.

- [ ] **FLAW-2.7 (Medium):** **Lỗi Parse MaType `LinReg` & `Vidya` Không Tương Thích JSON**
  - *Vị trí:* [`ferro_trade/src/optimizer/params.rs:85`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/params.rs#L85)
  - *Bản chất:* Enum `MaType` parse chuỗi không tương thích với tên định nghĩa trong file cấu hình JSON, gây lỗi fallback về EMA mặc định.
  - *Giải pháp:* Đồng bộ hóa chuỗi định danh `MaType` trong serde và macro derive.

- [ ] **FLAW-2.8 (Medium):** **Trùng Lặp Cache Key Khi Cấu Hình Có Tham Số Phụ Khác Nhau**
  - *Vị trí:* [`ferro_trade/src/data/global_cache.rs:110`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/data/global_cache.rs#L110)
  - *Bản chất:* Khóa băm cache của một số chỉ báo bỏ qua tham số làm mượt phụ, làm 2 chỉ báo khác tham số bị trùng cache.
  - *Giải pháp:* Đưa toàn bộ tham số vào hàm băm `CacheKey`.

- [ ] **FLAW-6.10 (Medium):** **Tiêm Dummy Candidate `0.0` Vào Cuối Danh Sách Champions**
  - *Vị trí:* [`ferro_trade/src/optimizer/prospector.rs:480`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/prospector.rs#L480)
  - *Bản chất:* Khi quần thể không đủ cá thể hợp lệ, code tự điền các chromosome toàn số 0.0 vào danh sách kết quả, có thể gây lỗi chia cho 0 khi nạp.
  - *Giải pháp:* Chỉ trả về các cá thể hợp lệ thực tế và cảnh báo nếu số lượng ít hơn yêu cầu.

- [ ] **FLAW-8.10 (Medium):** **Mã Nguồn Rác & File Bỏ Hoang (`duckdb_impl.rs`, `binance_stream.rs`)**
  - *Vị trí:* [`trading_bot/bot_engine/src/data_engine/duckdb_impl.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/duckdb_impl.rs)
  - *Bản chất:* 308 dòng code DuckDB không sử dụng, file `interface/mod.rs` (0 bytes) và `binance_stream.rs` bị bỏ hoang, làm rối rắm codebase.
  - *Giải pháp:* Xóa bỏ các file mồ côi để làm sạch dự án.
