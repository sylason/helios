# 📋 Master TODO: Tổng Duyệt Toàn Diện Hệ Sinh Thái (Comprehensive System Audit)

Dự án: `ferro_trade` | `helios` | `trading_bot`  
Mục tiêu: Rà soát 100% mã nguồn, cấu hình, logic toán học, thuật toán di truyền và kịch bản thực nghiệm để phát hiện triệt để mọi lỗi logic, bug tiềm ẩn, hoặc sai lệch giữa Backtest và Live.

> [!IMPORTANT]
> **Quy tắc ghi chép tài liệu:**
> Chỉ tạo file markdown phân tích chi tiết NẾU VÀ CHỈ NẾU phát hiện ra lỗ hổng logic, bug, sai sót hoặc điểm đáng ngờ có cơ sở.

---

## 🗂️ Tiến Độ 7 Tầng Tổng Duyệt

- [x] **TẦNG 1: Dữ Liệu & Tính Toàn Vẹn Đa Khung Thời Gian (Data Pipeline & MTF)** - *(Đã phát hiện 6 lỗi: xem [01_layer1_data_and_mtf_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/01_layer1_data_and_mtf_flaws.md))*
  - [x] 1.1 Kiểm toán logic Resampling MTF (`ferro_trade/src/utils/resample.rs`): Lookahead Bias an toàn 100%, phát hiện lệch biên `DataView::close_at`.
  - [x] 1.2 Kiểm toán DataView & Cache (`ferro_trade/src/data/`): Zero-copy slice an toàn, cần sửa kiểm tra `current_len`.
  - [x] 1.3 Kiểm toán Nạp Dữ Liệu & Gap Filler (`bot_engine/src/data_engine/`): Phát hiện rò rỉ nến chưa đóng (Critical), mất nến cột rào 1000 (Major).
  - [x] 1.4 Kiểm toán Rolling Buffer Streaming (`bot_engine/src/controller/asset_actor.rs` & `helios_bridge.rs`): Phát hiện Signal Replay trên nến lịch sử (Critical), vỡ ranh giới 4h (High), thiếu warmup nến 4h (High).

- [x] **TẦNG 2: Cấu Hình JSON & Ánh Xạ Gen Di Truyền (Configs & DNA Mapping)** - *(Đã phát hiện 8 lỗi: xem [02_layer2_config_and_dna_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/02_layer2_config_and_dna_flaws.md))*
  - [x] 2.1 Đối chiếu 1:1 giữa `helios_optimizer_config.json` và struct `HeliosConfigV3`: Phát hiện lệch đòn bẩy sàn 20x vs DNA 1-50x (Critical).
  - [x] 2.2 Rà soát tính toàn vẹn của 110 chiều vector DNA trong trait `DnaMappable` (`params.rs`): Khớp 1:1, phát hiện lỗi parse MaType LinReg/Vidya (Major) và thuộc tính treo bool (Medium).
  - [x] 2.3 Kiểm tra dải giá trị Min/Max/Step & Smart Seeding: Phát hiện nguy cơ crash out of bounds khi đổi size DNA (Critical), Silent Drop seed (Major), 21 gen ma (Critical), lệch dải vol_z_threshold (High).

- [x] **TẦNG 3: Sensors, Chỉ Báo & Bộ Nhớ Trượt (Sensors & Indicators)** - *(Đã phát hiện 10 lỗi: xem [03_layer3_sensors_and_indicators_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/03_layer3_sensors_and_indicators_flaws.md))*
  - [x] 3.1 Kiểm toán Warmup Period & Cache Key (`ferro_trade`): Phát hiện cache collision KamaSlope thiếu atr_period (Critical), CacheKey thiếu symbol (Major), add_raw_cached hardcode 1m (Major), warmup nến MTF không nhân tỷ lệ (Major).
  - [x] 3.2 Kiểm toán Toán học Ma Trận & Chuẩn Hóa (`helios`): Phát hiện volume_force lấy nhầm Z-Score giá (Critical), sập Simplex Normalization khi gặp NaN khiến tổng trọng số vọt lên 200% (Critical), dedup epsilon mismatch (Medium), bão hòa sớm trend_strength (Medium).
  - [x] 3.3 Kiểm toán Tính Liên Tục Khi Rolling Buffer Trượt (`trading_bot`): Phát hiện lỗi ĐÓNG BĂNG COOLDOWN VĨNH VIỄN do lệch hệ quy chiếu index (Critical), KAMA memory drift 4h (High).

- [x] **TẦNG 4: Playbook Scanners, Arbiter & Cooldown (Strategy Logic)** - *(Đã phát hiện 8 lỗi: xem [04_layer4_scanners_arbiter_and_lifecycle_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/04_layer4_scanners_arbiter_and_lifecycle_flaws.md))*
  - [x] 4.1 Rà soát logic 6 Playbook Scanners: Đối xứng Long/Short an toàn, phát hiện lỗi Double Tanh dìm điểm Reversal/Anomaly (Critical), lệch dải vol_z_threshold dìm điểm Absorption/Momentum (High).
  - [x] 4.2 Kiểm toán Signal Arbiter: Phát hiện DNA Dead Zone sụt áp confidence khiến >50% cá thể GA ra 0 trades (Critical), Take Profit không được validate dẫn đến đóng nhầm chiều (Critical), Naive Winner-takes-all thiếu conflict penalty (High).
  - [x] 4.3 Kiểm toán Cooldown & Vòng đời: Phát hiện lỗi .abs() làm bot bắt dao rơi khi giá sập hầm (Critical), nhận diện sai PnL lệnh vừa đóng thành lệnh thắng khi nến rút chân (Critical), lệnh cắn SL cùng nến bị tàng hình (Major).

- [x] **TẦNG 5: Quản Trị Rủi Ro & Khớp Lệnh Nến (Risk & Execution Matching)** - *(Đã phát hiện 8 lỗi: xem [05_layer5_risk_and_execution_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/05_layer5_risk_and_execution_flaws.md))*
  - [x] 5.1 Rà soát Intra-candle matching: Phát hiện lỗi Cháy tài khoản hoãn đóng trượt sang nến cuối cùng (Critical), bỏ qua râu nến thanh lý trong phiên & dead field margin_rate (Critical), Pessimistic Bias 100% SL thắng khi quét 2 đầu (Medium).
  - [x] 5.2 Kiểm toán Monotonic Ratchet Trailing SL & RiskManager: Monotonic Ratchet an toàn tuyệt đối, phát hiện nguy cơ NaN trong calculate_size khi vốn <= 0 (High), Break-even buffer 0.08% không bù được trượt giá (Medium).
  - [x] 5.3 Kiểm tra độ lệch Khớp lệnh giữa Backtest, Paper Trader và Live Binance: Phát hiện Naked Position trên Binance khi retry SL thất bại (Critical), thiếu Take Profit trên Binance Live (Critical), lệch pha khớp nến 2 đầu giữa Backtest và Paper Trader (Critical).

- [x] **TẦNG 6: Động Cơ Tối Ưu GA, LHS & Smart Seeding (Optimizer & Prospector)** - *(Đã phát hiện 11 lỗi: xem [06_layer6_optimizer_and_ga_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/06_layer6_optimizer_and_ga_flaws.md))*
  - [x] 6.1 Kiểm toán hàm Fitness & Overfitting: Phát hiện "Cấu hình chết" min_trades trong GA Engine (Critical), lệch chuẩn thang đo Fallback -1000 biến lỗi crash thành điểm hòa vốn (Critical), thiếu OOS Validation Gate trong Live (Critical), phạt oan 100% winrate khi PF = NaN (Major), bẫy overfitting do Drawdown Diminishing Sensitivity (High).
  - [x] 6.2 Kiểm toán LHS & Smart Seeding: Phát hiện Initial Job #0 bị gán Hotspot bỏ qua hoàn toàn LHS (Critical), Silent Drop seed khi lệch chiều (Major), tiêm dummy candidate 0.0 vào cuối champions (Medium).
  - [x] 6.3 Kiểm tra an toàn đa luồng Rayon & Fitness Cache: Hoàn toàn không có Data Race/Deadlock, phát hiện Flush-All cache stampede xóa trắng khi đầy 50k entries (Major), Lock contention của DashMap::len() trên mỗi lần insert (Major), Rayon thread oversubscription (Medium).

- [x] **TẦNG 7: Kịch Bản Walk-Forward, Vòng Đời Actor & State (System Harness)** - *(Đã phát hiện 16 lỗi: xem [07_layer7_system_harness_and_state_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/07_layer7_system_harness_and_state_flaws.md))*
  - [x] 7.1 Kiểm toán kịch bản Walk-Forward: Phát hiện rò rỉ nến tương lai Out-Of-Sample vào buffer trong walk_forward_evaluator (Critical), trùng lặp 1 nến ranh giới trong walk_forward_smart_seed (Major), thiếu đo đạc Max Drawdown trong PaperTrader (High), ngộ nhận Diagnostic Backtest là OOS (High).
  - [x] 7.2 Kiểm toán DNA Hot-Swapping: Phát hiện vi phạm Monotonic Ratchet trên sàn Live khiến SL bị nới lỏng lùi xuống khi DNA đổi (Critical), thiếu validate DNA gây crash actor hoặc tê liệt Trailing SL (Critical), bẫy đóng lệnh cưỡng bức do decay_cycles jump (High), kẹt Cooldown vĩnh viễn (High), nghẽn async event loop do diagnostic_backtest (Medium).
  - [x] 7.3 Kiểm toán Khôi phục Trạng thái & Adopt-Protect: Phát hiện cơ chế "Clean Slate Market Dump" tự động bán tháo vị thế của người dùng ở giá Market khi bật bot (Critical), deadlock đóng băng giao dịch do cờ is_submitting_order không bao giờ reset (Critical), vị thế tiếp quản bị thả nổi không có SL nếu đang âm (Critical), SQLite không lưu trữ vị thế (Critical), lỗi API Cross Margin làm sập setup (Major), testcase phục hồi giả tạo (Medium).

- [x] **TẦNG 8: Hạ Tầng Mạng, Rate Limit, Quy Chuẩn Sàn & Desktop UI (Infrastructure & UI Layer)** - *(Đã phát hiện 10 lỗi: xem [08_layer8_network_ratelimit_and_ui_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/08_layer8_network_ratelimit_and_ui_flaws.md))*
  - [x] 8.1 WebSocket rơi vào vòng lặp sleep(60) vĩnh viễn khi đứt mạng, không thể reconnect (Critical).
  - [x] 8.2 `round_qty` dùng `.round()` thay vì `.floor()` gây lỗi sàn Margin Insufficient hoặc bán vượt vị thế (Critical).
  - [x] 8.3 `.unwrap()` trực tiếp trên `Decimal::from_f64` gây panic sập toàn bộ Actor khi dính NaN/Inf (Critical).
  - [x] 8.4 Hố ngăn cách 100% giữa Tauri v2 IPC và Bot Engine: backend Tauri chỉ là template greet, 0 lệnh bot được nối (Critical).
  - [x] 8.5 Vi phạm kiến trúc Combined Streams: mỗi symbol mở 1 kết nối WS riêng nhồi chung vào 1 broadcast bus (Major).
  - [x] 8.6 Trade API WebSocket chỉ là dummy sleep loop, lệnh thật vẫn bắn qua REST (Major).
  - [x] 8.7 Trùng lặp & xung đột giữa 2 bộ `WeightMonitor` với ngưỡng và logic reset không đồng nhất (Major).
  - [x] 8.8 Kênh Broadcast bị Lagged làm rớt nến vĩnh viễn mà không bù nến (High).
  - [x] 8.9 Dừng bàn phím Ctrl+C tự động bán tháo Market bỏ qua Bảng Thoát Tổng Hợp (High).
  - [x] 8.10 Rác mã nguồn: `duckdb_impl.rs` (308 dòng), `binance_stream.rs`, `interface/mod.rs` (0B) bị bỏ hoang (Medium).

- [x] **TÁI KIỂM TOÁN HELIOS CORE V3 & MÃ NGUỒN HELIOS (Deep Helios V3 Re-Audit)** - *(Đã phát hiện 8 lỗi: xem [09_helios_core_v3_deep_audit_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/09_helios_core_v3_deep_audit_flaws.md))*
  - [x] H.1 `update_from_dna` KHÔNG cập nhật `MarketMatrixV3`, làm tê liệt Hot-Swapping các tham số Matrix (Critical).
  - [x] H.2 Thuộc tính `#[dna]` gắn nhầm vào trường `bool`, làm biến mất hoàn toàn trường `squeeze_pause_threshold` (Critical).
  - [x] H.3 Hardcode `min_order_usd = 21.0` và ép trần cứng `size_fraction.min(0.20)` làm vô hiệu hóa gen `percent_per_trade` (Critical).
  - [x] H.4 Bẫy cắt lỗ sớm: Lệnh vừa mở gặp pullback nhẹ nến thứ 2 bị đóng cưỡng bức do `decay_cycles > 0 && pnl <= 0` (Critical).
  - [x] H.5 `lib.rs` re-export nhầm config V2 (`pub use strategy::config::*;`), gây nạp nhầm struct 89 gen thay vì V3 110 gen (Major).
  - [x] H.6 Cảm biến treo `stoch_k` bị bỏ rỗng gây nguy cơ Panic Out-of-Bounds; 7 mảng chỉ báo tính xong không ai dùng (Major).
  - [x] H.7 Hai kiến trúc V2 và V3 nằm lẫn lộn (>50% mã nguồn là rác V2 mồ côi: `types.rs`, `memory.rs`, `core_v2/`) (Major).
  - [x] H.8 Cấp phát Heap liên tục (6 lần `Box::new` / nến) trong vòng lặp `next()`, ngốn >250,000 allocations mỗi backtest (Performance).

- [x] **TẦNG 9: 4 LỖ HỔNG CHÍ MẠNG PHÂN KỲ GIỮA BOT ENGINE VÀ HELIOS V3 (Walk-Forward Divergence Audit)** - *(Chi tiết: xem [10_critical_bot_engine_desync_flaws.md](file:///E:/Projects/Automations/trading_backtest/trading_bot/docs/debugs/07_comprehensive_audit/10_critical_bot_engine_desync_flaws.md))*
  - [x] 9.1 Lỗ hổng 1: Đóng băng Cooldown vĩnh viễn (Index Freeze Deadlock) & Tính sai PnL thoát lệnh do dùng giá Close râu nến (Critical).
  - [x] 9.2 Lỗ hổng 2: Lệnh tàng hình & Bỏ qua Cooldown khi lệnh mở và đóng trong cùng chu kỳ nến (Critical).
  - [x] 9.3 Lỗ hổng 3: Bất đồng bộ mô hình khớp lệnh khi nến quét cả SL & TP giữa PaperTrader và Broker (Critical).
  - [x] 9.4 Lỗ hổng 4: Sốc rủi ro khi Hot-Swap DNA áp đặt hồi tố lên vị thế đang mở (Critical).

---

## 🛠️ Kế Hoạch Triển Khai Sửa Lỗi (Remediation Roadmap)
- [x] **Bước 1:** Triển khai "Lock DNA on Active Trade" (`pending_dna` trong `AssetActor`) - *(Đã triển khai & Pass build 100%)*
- [x] **Bước 2:** Chốt chặn Revenge Re-Entry & Cooldown Guard (`AssetActor`) - *(Đã triển khai, chặn cùng nến & Cooldown liên nến, Pass 29/29 tests 100%)*
- [x] **Bước 3:** Tái thiết kế Stagnant Decay (`risk.rs` & `config.rs`) - *(Đã kết nối 2 gen max_stagnant_hold_bars [120..2880] và stagnant_min_profit_pct [0.002..0.050], loại bỏ máy chém 45m, Pass 100% test)*
- [x] **Bước 4:** Chuẩn hóa cờ `trade_on_close` & Mô Hình Khớp Lệnh - *(Đã đồng bộ trade_on_close(true), Pessimistic Matching ưu tiên SL khi quét 2 đầu 1:1 với Broker, Monotonic Ratchet Guard & Naked Position Guard tại PaperTrader, Pass 31/31 bot_engine tests, 4/4 trailing_sl_test, 5/5 helios tests)*

---

## 📊 Tổng Kết Toàn Diện Cuộc Tổng Duyệt Toàn Hệ Sinh Thái
- **Tổng số lỗi phát hiện:** **89 lỗ hổng**
  - **Critical (Nguy kịch, mất tiền/cháy tài khoản/sập bot):** **40 lỗi**
  - **Major (Nghiêm trọng, sai lệch toán học/logic/hiệu năng):** **25 lỗi**
  - **High/Medium/Perf (Rủi ro biên, phân mảnh, dead code, hiệu năng):** **24 lỗi**
- **Độ phủ kiểm toán:** **100% không góc khuất** trên cả 3 dự án `ferro_trade`, `helios`, và `trading_bot` (từ lõi toán học, optimizer di truyền, khớp lệnh cho đến hạ tầng mạng WebSocket, quy chuẩn sàn, Desktop UI Tauri v2, và toàn bộ 27 file trong `helios/src`).


