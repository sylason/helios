# 🟠 MASTER TODO: 25 LỖ HỔNG MAJOR (NGHIÊM TRỌNG) TOÀN HỆ SINH THÁI

> **Mục tiêu tài liệu:** Quản trị tập trung 25 lỗi mức độ **MAJOR / HIGH** (ảnh hưởng nghiêm trọng đến độ chính xác toán học, hiệu năng tối ưu di truyền, tính toàn vẹn dữ liệu, và xung đột hạ tầng).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/TODO_MAJOR.md`  
> **Trạng thái tổng quan:** **2/25 lỗi đã được xử lý**, **23/25 lỗi đang chờ xử lý**.

---

## 📊 Bảng Tiến Độ Tổng Quan Theo Cụm Chức Năng

| Cụm chức năng | Tổng số Major | Đã xử lý | Còn lại | Phạm vi ảnh hưởng chính |
| :--- | :---: | :---: | :---: | :--- |
| **Nhóm 1: Dữ Liệu Nến, Đồng Bộ REST & MTF Boundaries** | 3 | 0 | 3 | Lệch biên nến 4h, mất nến rào, nạp trùng nến |
| **Nhóm 2: Cấu Hình DNA, Dải Tham Số & Seeding** | 3 | 0 | 3 | Lệch dải volume Z-score, rớt seed âm thầm |
| **Nhóm 3: Cảm Biến, Bộ Nhớ Trượt & Cache Chỉ Báo** | 4 | 0 | 4 | Đè cache giữa các symbol, KAMA 4h trôi dạt |
| **Nhóm 4: Scanners, Arbiter & Logic Vòng Đời** | 3 | 1 | 2 | Phạt xung đột tín hiệu, bypass volume do NaN |
| **Nhóm 5: Động Cơ GA Optimizer & Fitness Cache** | 4 | 0 | 4 | Phạt oan 100% winrate, DashMap contention |
| **Nhóm 6: Sàn Binance, Kịch Bản Test & Hạ Tầng Mạng** | 5 | 0 | 5 | Combined WS stream, Cross Margin -4067, 2 WeightMonitor |
| **Nhóm 7: Kiến Trúc Mã Nguồn & Rác V2 Mồ Côi** | 3 | 1 | 2 | Re-export nhầm V2 config, cảm biến rỗng stoch_k |
| **TỔNG CỘNG** | **25** | **2** | **23** |  |

---

## 📅 NHÓM 1: DỮ LIỆU NẾN, ĐỒNG BỘ REST & MTF BOUNDARIES (3 LỖ HỔNG)

- [ ] **FLAW-1.3:** **Mất Nến Cột Rào 1000 Nến Do Điều Kiện `>` Thay Vì `>=`**
  - *Vị trí:* [`trading_bot/bot_engine/src/data_engine/binance_rest.rs:188`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/binance_rest.rs#L188)
  - *Bản chất:* Trong `calculate_sync_start`, điều kiện kiểm tra khoảng trống dùng `gap_candles > 1000`. Khi khoảng trống dữ liệu đúng bằng 1000 nến, hàm phân nhánh sai, làm mất nến rào ranh giới.
  - *Giải pháp đề xuất:* Sửa thành `gap_candles >= 1000`.

- [ ] **FLAW-1.4:** **Vỡ Ranh Giới Nến 4h Do Cắt Buffer Không Theo Bội Số 240**
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs:340`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L340)
  - *Bản chất:* Khi trượt bộ đệm, buffer bị cắt tỉa ở vị trí nến 1m bất kỳ không chia hết cho 240 (ví dụ 43,100 nến). Cây nến 4h đầu tiên trong khung MTF bị thiếu nến thành phần, làm méo mó các chỉ báo 4h (EMA, KAMA, Trend Strength).
  - *Giải pháp đề xuất:* Căn chỉnh `slice_start` lùi về mốc chia hết cho 240 phút (00:00, 04:00, 08:00, 12:00, 16:00, 20:00 UTC).

- [ ] **FLAW-1.5:** **Thiếu Warmup Nến 4h Khi Cắt Buffer 43,100 Nến**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs:92`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L92)
  - *Bản chất:* 43,100 nến 1m tương đương ~179 nến 4h. Nếu chỉ báo 4h yêu cầu chu kỳ dài (ví dụ EMA 200), dữ liệu sẽ không đủ warmup và trả về NaN.
  - *Giải pháp đề xuất:* Mở rộng buffer hoặc kẹp trần tối đa các chu kỳ chỉ báo 4h không vượt quá 100 nến.

---

## 🧬 NHÓM 2: CẤU HÌNH DNA, DẢI THAM SỐ & SEEDING (3 LỖ HỔNG)

- [ ] **FLAW-2.4:** **Silent Drop Hạt Giống Smart Seeding Khi Kích Thước Lệch**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:310`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L310)
  - *Bản chất:* Khi trích xuất Rank 1 Champion từ chu kỳ trước, nếu kích thước vector DNA trong SQLite khác với phiên bản hiện tại, code bỏ qua âm thầm mà không ghi log cảnh báo hay convert.
  - *Giải pháp đề xuất:* Thêm log `warn!` thông báo lý do drop seed và tự động padding/cắt tỉa vector nếu khả thi.

- [ ] **FLAW-2.5 & FLAW-4.6:** **Lệch Dải Tham Số `vol_z_threshold` Giữa DNA Và Code**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/config.rs:315`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L315) & [`scanners/absorption.rs:42`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L42)
  - *Bản chất:* DNA quy định dải `vol_z_threshold` là $[1.0 \dots 5.0]$, nhưng trong code scanner giá trị volume force lại được nén qua `tanh` về $[-1.0 \dots 1.0]$. Do đó điều kiện `volume_force > vol_z_threshold` hầu như không bao giờ thỏa mãn!
  - *Giải pháp đề xuất:* Chuẩn hóa dải DNA của `vol_z_threshold` về miền $[0.2 \dots 0.9]$ hoặc giữ volume force dạng Z-Score chưa nén.

- [ ] **FLAW-7.8:** **Trùng Lặp Nến Ranh Giới Giữa In-Sample Và Out-Of-Sample**
  - *Vị trí:* [`trading_bot/bot_engine/examples/walk_forward_smart_seed.rs:215`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/walk_forward_smart_seed.rs#L215)
  - *Bản chất:* Do câu lệnh truy vấn SQLite dùng `>= train_end_ts`, nến tại mốc ranh giới bị nạp 2 lần: lần 1 trong buffer lịch sử, lần 2 trong stream Out-Of-Sample, kích hoạt tick kép tại nến giao thoa.
  - *Giải pháp đề xuất:* Đổi điều kiện stream OOS thành `> train_end_ts`.

---

## ⚡ NHÓM 3: CẢM BIẾN, BỘ NHỚ TRƯỢT & CACHE CHỈ BÁO (4 LỖ HỔNG)

- [ ] **FLAW-3.5:** **`CacheKey` Thiếu Trường `symbol` Gây Đè Dữ Liệu Đa Cặp Tiền**
  - *Vị trí:* [`ferro_trade/src/indicators/identifier.rs:152-157`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/identifier.rs#L152-L157) & [`global_cache.rs:77`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/data/global_cache.rs#L77)
  - *Bản chất:* Struct `CacheKey` chỉ chứa loại chỉ báo, tham số và khung thời gian, hoàn toàn thiếu mã cặp tiền (`symbol`). Nếu dùng chung cache cho nhiều bot (BTC, ETH), các bot sẽ lấy nhầm chỉ báo của nhau!
  - *Giải pháp đề xuất:* Bổ sung `symbol: String` vào `CacheKey`.

- [x] **FLAW-3.6:** **`add_raw_cached` Gán Cứng Khung 1m Đè Cache Đa Khung Thời Gian**
  - *Vị trí:* [`ferro_trade/src/backtest/manager.rs:70-76`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/manager.rs#L70-L76)
  - *Bản chất:* Hàm cache chỉ báo thô tự động gán `Timeframe::T1m`. Hai chỉ báo cùng loại khác khung (ví dụ EMA 1h và EMA 4h) sẽ ghi đè lên nhau.
  - *Giải pháp đề xuất:* Truyền `Timeframe` thực tế của chỉ báo vào hàm cache.

- [ ] **FLAW-3.7:** **Bypass Bộ Lọc Volume Khi Gặp `NaN` (Chuẩn IEEE-754)**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs:23-33`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L23-L33)
  - *Bản chất:* Theo IEEE-754, `NaN > threshold` trả về `false`. Nếu volume force bị NaN, bộ lọc chống bão volume bị bypass, mở lệnh sideway ngay giữa lúc bão giá!
  - *Giải pháp đề xuất:* Kiểm tra `v.is_nan()` và fallback về giá trị phòng thủ.

- [ ] **FLAW-3.8:** **KAMA Memory Drift Khung 4h Làm Lật Dấu Xu Hướng**
  - *Vị trí:* [`ferro_trade/src/indicators/calculate/bulk.rs:1561`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1561) & [`scanners/trend_pb.rs:37`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L37)
  - *Bản chất:* KAMA 4h lưu giữ ký ức khởi tạo rất lâu do số nến ít. Sai lệch 0.1% - 0.5% giữa Backtest 90 ngày và Live 30 ngày làm đổi dấu độ dốc `kama_slope_adj`, khiến Live mở lệnh ngược Backtest.
  - *Giải pháp đề xuất:* Nâng số nến buffer hoặc dùng EMA làm bộ lọc thay thế khi nến 4h chưa đủ độ sâu.

---

## ⚔️ NHÓM 4: SCANNERS, ARBITER & LOGIC VÒNG ĐỜI (3 LỖ HỔNG)

- [ ] **FLAW-4.7:** **Naive Winner-Takes-All Thiếu Conflict Penalty Trong Arbiter**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:145-180`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L145-L180)
  - *Bản chất:* Khi cả 2 scanner Long và Short đều phát tín hiệu mạnh (thị trường giằng co, cào 2 đầu), Arbiter chỉ chọn bên có điểm nhỉnh hơn 0.01 mà không áp dụng hình phạt xung đột (Conflict Penalty), dẫn đến mở lệnh rủi ro cao.
  - *Giải pháp đề xuất:* Áp dụng công thức trừ điểm tự tin: $Confidence_{final} = Confidence_{win} - (Conflict\_Factor \times Confidence_{lose})$.

- [x] **FLAW-4.8:** **Lệnh Cắn Stop Loss Cùng Nến Bị Tàng Hình Trước Strategy** `[ĐÃ SỬA ✅]`
  - *Vị trí:* [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
  - *Đã xử lý:* Đã bắt cờ `closed_intra_candle` và chặn đứng lệnh mở mới trên cùng cây nến ở Bước 2.

- [ ] **FLAW-H.3 một phần:** **Cấu Trúc Tỷ Lệ Vị Thế Bị Kẹp Trần Nhỏ Hơn Khối Lượng Tối Thiểu Sàn**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:20`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L20)
  - *Bản chất:* Khi vốn tài khoản nhỏ ($50 - $100), tỷ lệ 10-20% vốn chỉ tương đương $5 - $10, nhỏ hơn quy định `min_notional = $20` của Binance Futures khiến lệnh bị sàn từ chối liên tục.
  - *Giải pháp đề xuất:* Điều chỉnh logic sizing: Nếu vốn $\le \$200$, tự động làm tròn lên mức min sàn quy định.

---

## 🧬 NHÓM 5: ĐỘNG CƠ GA OPTIMIZER & FITNESS CACHE (4 LỖ HỔNG)

- [ ] **FLAW-6.5:** **Silent Drop Hạt Giống Khi Lệch Chiều Trong `ProspectorRunner`**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:240`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L240)
  - *Bản chất:* Tương tự Flaw 2.4, khi truyền hạt giống vào thế hệ 0 của GA, nếu độ dài chromosome không khớp 100%, hạt giống bị loại bỏ âm thầm khiến Smart Seeding mất tác dụng.
  - *Giải pháp đề xuất:* Bổ sung log cảnh báo rõ ràng khi hạt giống bị drop.

- [ ] **FLAW-6.6:** **Phạt Oan 100% Winrate Khi Profit Factor Bị `NaN`**
  - *Vị trí:* [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:225`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L225)
  - *Bản chất:* Khi cá thể toàn thắng 100% (0 lệnh thua), tổng lỗ $= 0$, phép chia $Profit / 0 = NaN$. Code kiểm tra `pf.is_nan()` phạt cá thể về điểm âm cực nặng, vô tình tiêu diệt cá thể hoàn hảo nhất!
  - *Giải pháp đề xuất:* Nếu `gross_loss == 0.0 && gross_profit > 0.0`, gán `Profit Factor = 100.0` (hoặc mức trần cực đại).

- [ ] **FLAW-6.7:** **Flush-All Cache Stampede Xóa Trắng 50.000 Bản Ghi Khi Đầy**
  - *Vị trí:* [`ferro_trade/src/optimizer/fitness_cache.rs:45`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/fitness_cache.rs#L45)
  - *Bản chất:* Khi bộ nhớ đệm đánh giá cá thể đạt 50,000 entries, hàm cache gọi `clear()` xóa trắng 100%, gây hiện tượng Cache Stampede khiến CPU vọt lên 100% tính lại từ đầu.
  - *Giải pháp đề xuất:* Chuyển sang chiến lược loại bỏ LRU hoặc chỉ xóa 20% cá thể cũ nhất.

- [ ] **FLAW-6.8:** **Lock Contention Của `DashMap::len()` Trong Vòng Lặp GA Đa Luồng**
  - *Vị trí:* [`ferro_trade/src/optimizer/fitness_cache.rs:52`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/optimizer/fitness_cache.rs#L52)
  - *Bản chất:* Mỗi lần một luồng Rayon insert kết quả, nó gọi `self.map.len()` để kiểm tra dung lượng. `DashMap::len()` phải khóa toàn bộ 64 shards, gây thắt cổ chai nghẽn đa luồng.
  - *Giải pháp đề xuất:* Dùng `AtomicUsize` đếm riêng số phần tử thay vì gọi `len()`.

---

## 🌐 NHÓM 6: SÀN BINANCE, KỊCH BẢN TEST & HẠ TẦNG MẠNG (5 LỖ HỔNG)

- [ ] **FLAW-7.9:** **Lỗi Cross Margin API `-4067 / -4068` Bị Bỏ Qua Âm Thầm**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/account_manager.rs:85-90`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/account_manager.rs#L85-L90)
  - *Bản chất:* Khi thiết lập Isolated Margin 20x, nếu đang có vị thế mở hoặc sàn trả lỗi, code nuốt lỗi `Err` và tiếp tục chạy. Bot giao dịch với rủi ro Cross Margin toàn bộ số dư ví Futures.
  - *Giải pháp đề xuất:* Bắt buộc kiểm tra Margin Type thành công, nếu thất bại phải báo lỗi dừng khởi động bot.

- [ ] **FLAW-8.5:** **Vi Phạm Thiết Kế Combined Streams (Mở Quá Nhiều WebSocket Riêng)**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/ws_manager.rs:74`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs#L74)
  - *Bản chất:* Thiết kế gốc yêu cầu 1 kết nối Combined Streams duy nhất cho mọi symbol. Hiện tại mỗi symbol lại tự mở 1 kết nối WebSocket riêng, gây quá tải kết nối và dễ bị Binance ngắt kết nối.
  - *Giải pháp đề xuất:* Gom tất cả symbol vào 1 URL combined stream dạng `stream?streams=btcusdt@kline_1m/ethusdt@kline_1m`.

- [ ] **FLAW-8.6:** **Trade API WebSocket Chỉ Là Dummy Stub `sleep(60)` Rỗng**
  - *Vị trí:* [`trading_bot/bot_engine/src/execution/ws_manager.rs:248`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs#L248)
  - *Bản chất:* Hàm lắng nghe lệnh giao dịch qua WebSocket chỉ chứa `sleep(60)`, toàn bộ lệnh và trạng thái khớp lệnh vẫn phải phụ thuộc vào REST API có độ trễ cao.
  - *Giải pháp đề xuất:* Triển khai đúng Binance User Data Stream WebSocket để nhận `ORDER_TRADE_UPDATE` tức thời.

- [ ] **FLAW-8.7:** **Xung Đột & Trùng Lặp Giữa 2 Bộ `WeightMonitor` Độc Lập**
  - *Vị trí:* [`trading_bot/bot_engine/src/models/weight_monitor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/weight_monitor.rs) & [`execution/rate_limit_monitor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/rate_limit_monitor.rs)
  - *Bản chất:* Tồn tại 2 struct quản lý giới hạn API 2400 Weight độc lập với ngưỡng cảnh báo và chu kỳ reset lệch nhau, gây xung đột trạng thái phòng thủ sàn.
  - *Giải pháp đề xuất:* Xóa bỏ bộ cũ, hợp nhất thành 1 struct `WeightMonitor` duy nhất dùng chung qua Arc.

---

## 🏗️ NHÓM 7: KIẾN TRÚC MÃ NGUỒN & RÁC V2 MỒ CÔI (3 LỖ HỔNG)

- [ ] **FLAW-H.5:** **`lib.rs` Re-Export Nhầm Config V2 (`pub use strategy::config::*;`)**
  - *Vị trí:* [`strategies_backtests/helios/src/lib.rs:11`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/lib.rs#L11)
  - *Bản chất:* `lib.rs` export struct `HeliosConfig` (cấu hình V2 cũ 89 gen) ra ngoài root, khiến các module bên ngoài dễ nạp nhầm struct cũ thay vì `HeliosConfigV3` (110 gen).
  - *Giải pháp đề xuất:* Sửa thành `pub use strategy::core_v3::config::*;` và ẩn hoàn toàn cấu hình V2.

- [ ] **FLAW-H.6:** **Cảm Biến Treo `stoch_k` Bị Rỗng & 7 Mảng Chỉ Báo Tính Xong Bị Bỏ Hoang**
  - *Vị trí:* [`strategies_backtests/helios/src/strategy/sensors.rs:56`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/sensors.rs#L56)
  - *Bản chất:* Trường `stoch_k` trong `HeliosSensors` được khởi tạo rỗng nhưng không được gán dữ liệu; 7 mảng chỉ báo khác được tính toán trong `init()` tốn tài nguyên nhưng không có scanner nào sử dụng.
  - *Giải pháp đề xuất:* Điền đúng dữ liệu cho `stoch_k` và dọn bỏ các mảng chỉ báo thừa để tăng tốc độ backtest.

- [ ] **FLAW-H.7:** **Hai Kiến Trúc V2 Và V3 Nằm Lẫn Lộn (>50% Mã Nguồn Là Rác V2 Mồ Côi)**
  - *Vị trí:* [`strategies_backtests/helios/src/`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/)
  - *Bản chất:* Thư mục `core_v2/`, `types.rs`, `memory.rs` không còn được sử dụng trong phiên bản V3 nhưng vẫn nằm chung trong crate, gây nhầm lẫn khi import và bảo trì. *(Gần đây đã gây ra lỗi import nhầm trong `trailing_sl_test.rs`).*
  - *Giải pháp đề xuất:* Đóng gói toàn bộ code V2 vào folder `legacy/` hoặc dọn dẹp sạch sẽ để tinh gọn codebase.
