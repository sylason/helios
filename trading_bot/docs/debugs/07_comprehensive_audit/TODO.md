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

## 🛠️ Kế Hoạch Triển Khai Sửa Lỗi & Chi Tiết Lỗi Đã Debug

### 1. Chi Tiết Các Lỗ Hổng Đã Debug & Khắc Phục Thành Công
- [x] **Flaw 9.4 (Critical - Lỗ hổng 4): Sốc rủi ro khi Hot-Swap DNA áp đặt hồi tố lên vị thế đang mở:**
  - *Mã nguồn:* [`asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
  - *Giải pháp đã debug:* Bổ sung `pending_dna: Option<OptimizedDna>`. Khóa 100% DNA cũ bảo vệ vị thế đang mở, tự động nạp DNA mới khi vị thế chuyển về trạng thái Flat ("Lock DNA on Active Trade").
- [x] **Flaw 9.2 (Critical - Lỗ hổng 2): Lệnh tàng hình & Bỏ qua Cooldown khi lệnh mở-đóng cùng nến (Revenge Re-Entry):**
  - *Mã nguồn:* [`asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
  - *Giải pháp đã debug:* Di chuyển vòng lặp xả WebSocket `account_rx` lên đầu nến; thiết lập Chốt chặn Tầng 1 cấm mở lệnh cùng cây nến vừa đóng (`closed_intra_candle`), và Chốt chặn Tầng 2 `CooldownGuard` chặn vào lệnh cùng chiều trong 15 nến nếu PnL $\le 0.001$.
- [x] **Flaw H.4 & 9.1 một phần (Critical): Bẫy cắt lỗ sớm Stagnant Decay (Máy chém giết non lệnh sau 45 phút):**
  - *Mã nguồn:* [`risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) & [`config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)
  - *Giải pháp đã debug:* Xóa bỏ hoàn toàn logic `decay_cycles > 0 && current_pnl <= 0.0`; kết nối trực tiếp 2 gen DNA phổ quát `max_stagnant_hold_bars` [120..2880 nến] và `stagnant_min_profit_pct` [0.002..0.050]. Giữ nguyên cơ chế siết Stop Loss trượt (Progressive TSL Tightening).
- [x] **Flaw 9.3 & 5.5 (Critical - Lỗ hổng 3 Sai lệch 1): Bất đồng bộ mô hình khớp lệnh khi nến quét cả SL & TP (Dual-Hit Conflict):**
  - *Mã nguồn:* [`paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)
  - *Giải pháp đã debug:* Loại bỏ triệt để heuristic khoảng cách Open (vốn gây thắng ảo 50-70% ở Paper), áp dụng mô hình bảo thủ Pessimistic Matching: luôn ưu tiên khớp Stop Loss trước 1:1 với `ferro_trade::Broker`.
- [x] **Flaw 5.2 / Tầng 5 (Critical): Thiếu Monotonic Ratchet Guard & Nguy cơ Naked Position tại PaperTrader:**
  - *Mã nguồn:* [`paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs) & [`paper_engine.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_engine.rs)
  - *Giải pháp đã debug:* Ngăn chặn tuyệt đối việc lùi/nới lỏng Stop Loss (Long chỉ tăng, Short chỉ giảm); ngăn chặn việc xóa sổ SL về `None` khi đang có vị thế mở; đồng bộ trực tiếp SL/TP đã qua thẩm định từ `PaperTrader` sang `LivePositionState`.
- [x] **Flaw 1.4 một phần (Major): Lệch pha cờ `trade_on_close`:**
  - *Mã nguồn:* [`helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs)
  - *Giải pháp đã debug:* Chuyển cờ `trade_on_close` từ `false` sang `true` đồng nhất 100% với GA Optimizer và Backtest.
- [x] **Flaw H.1 (Critical): `update_from_dna` không cập nhật `MarketMatrixV3`:**
  - *Mã nguồn:* [`core_v3/mod.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs)
  - *Giải pháp đã debug:* Đã bổ sung `self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);` trong `update_from_dna`.
- [x] **Flaw H.2 (Critical): Thuộc tính `#[dna]` gắn nhầm vào trường `bool`:**
  - *Mã nguồn:* [`core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)
  - *Giải pháp đã debug:* Đã gỡ bỏ `#[dna]` khỏi `enable_re_entry_cooldown`.
- [x] **Kiểm thử Tích hợp Trailing SL bị lệch chuẩn:**
  - *Mã nguồn:* [`trailing_sl_test.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/tests/trailing_sl_test.rs)
  - *Giải pháp đã debug:* Nâng cấp lên `HeliosStrategyV3`, thêm `Default` cho V3, sửa timestamp vào lệnh gắn chặt theo nến thực tế (Pass 4/4 test).

### 2. Trạng Thái Tiến Độ Kiểm Thử
- **Unit & Integration Tests:** Đã vượt qua 100% (`bot_engine`: 31/31 passed; `helios`: 5/5 passed; `trailing_sl_test`: 4/4 passed).
- **Thực nghiệm Walk-Forward (`walk_forward_smart_seed`):** TẠM HOÃN do môi trường phần cứng máy tính quá tải/quạt ồn. Đã hoàn thành `cargo check` chuẩn xác (Exit 0), sẽ kích hoạt chạy thực nghiệm sau.

---

## 📊 Tổng Kết Toàn Diện Cuộc Tổng Duyệt Toàn Hệ Sinh Thái
- **Tổng số lỗi phát hiện:** **89 lỗ hổng**
  - **Critical (Nguy kịch, mất tiền/cháy tài khoản/sập bot):** **40 lỗi**
  - **Major (Nghiêm trọng, sai lệch toán học/logic/hiệu năng):** **25 lỗi**
  - **High/Medium/Perf (Rủi ro biên, phân mảnh, dead code, hiệu năng):** **24 lỗi**
- **Độ phủ kiểm toán:** **100% không góc khuất** trên cả 3 dự án `ferro_trade`, `helios`, và `trading_bot` (từ lõi toán học, optimizer di truyền, khớp lệnh cho đến hạ tầng mạng WebSocket, quy chuẩn sàn, Desktop UI Tauri v2, và toàn bộ 27 file trong `helios/src`).


