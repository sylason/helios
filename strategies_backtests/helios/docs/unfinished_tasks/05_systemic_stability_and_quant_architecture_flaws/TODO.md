# DANH MỤC CÔNG VIỆC (TODO): TÁI CẤU TRÚC KIẾN TRÚC ĐỊNH LƯỢNG & ỔN ĐỊNH HÓA

> **Thư mục:** [`05_systemic_stability_and_quant_architecture_flaws/`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/)  
> **Tài liệu tham chiếu:** Toàn bộ chuỗi 55 Báo cáo Khoa học & Kiểm toán Hạ tầng Hệ thống trong [`notes/`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/) (từ Note 01 đến Note 55).  
> **Mục tiêu:** Chuyển đổi hệ thống sang "Lỗ ít, lời ổn định, đủ nhiều, lọc sạch trạng thái nguy hiểm và tối đa hóa lợi nhuận".







---

## 📋 DANH SÁCH NHIỆM VỤ CHI TIẾT

### PHẦN 1: BẢO VỆ VỐN & KHÓA LỢI NHUẬN TỨC THÌ (RISK & EXECUTION)
- [x] **TODO-STAB-01: Triển khai Tiered Profit Lock & Break-Even Ratchet trong Risk Manager**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `trading_bot/bot_engine/src/execution/paper_trader.rs`
  - **Mô tả:** 
    - Nấc 1: Khi $\text{MFE} \ge +3.5\%$ ROE $\to$ dời SL lên tối thiểu $+1.60\%$ ROE (bảo hiểm đủ phí sàn Binance $20\text{x}$).
    - Nấc 2: Khi $\text{MFE} \ge +8.0\%$ ROE $\to$ khóa cứng mức lãi sàn $+4.00\%$ ROE.
  - **Unit Tests:**
    - [x] `test_tiered_profit_lock_break_even_at_3_5_roe`
    - [x] `test_tiered_profit_lock_locks_4_pct_at_8_pct_roe`
    - [x] `test_anti_atr_expansion_tightening`

- [x] **TODO-STAB-02: Khử triệt để dao rơi trong Absorption Scanner (`TODO-SCN-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:123-140`, `config.rs:1027-1031`
  - **Mô tả:** [ĐÃ HOÀN THÀNH] Bổ sung Macro Adverse Slope Veto (`FLAW-REG.9 TẦNG 2`): cấm Long khi `kama_slope_atr < -max_adverse_slope`, cấm Short khi `kama_slope_atr > max_adverse_slope` (tham số hóa DNA `max_adverse_slope` $\in [0.02, 0.25]$, mặc định `0.15`), kết hợp loại bỏ bộ lọc trễ pha `vwma < bb_middle` và neo SL trực tiếp theo đáy/đỉnh nến Spring/Upthrust vi mô.
  - **Unit Tests:**
    - [x] `test_absorption_falling_knife_blocked_by_macro_trend`
    - [x] `test_absorption_sl_spring_geometry`

---

### PHẦN 2: LỌC TRẠNG THÁI NGUY HIỂM & ĐỒNG THUẬN PHA (GATEKEEPER & SENSORS)
- [x] **TODO-STAB-03: Triển khai Bộ Phân Loại Chế Độ Thị Trường Cấp Vĩ Mô (Macro Regime Classifier)**
  - **Tài liệu thiết kế:** [`03_technical_spec_macro_regime_classifier.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/03_technical_spec_macro_regime_classifier.md)
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`, `gatekeeper.rs`, `config.rs`
  - **Mô tả:** Triển khai enum `MarketRegime` và chốt chặn `is_scanner_eligible_for_regime`. Nhận diện pha Chop / Đi ngang (RangeBound), Trending và Breakout. Khóa hoàn toàn `Trend_PB` và `Momentum_IGN` trong pha RangeBound và cấm `ClimaxReversal`/`AnomalyFade` trong Trending để bảo toàn vốn.
  - **Unit Tests:**
    - [x] `test_regime_classification_trending`
    - [x] `test_regime_classification_breakout`
    - [x] `test_regime_classification_range_bound`
    - [x] `test_selective_routing_matrix_contract`
    - [x] `test_arbiter_process_tick_selective_routing_ledger`

- [ ] **TODO-STAB-04: Bổ sung Chốt Chặn Đồng Thuận Pha Đa Khung Thời Gian (MTF Phase Coherence Gate)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`, `momentum_ignition.rs`
  - **Mô tả:** Chỉ cho phép vào lệnh 1m khi xu hướng khung 15m/1h đồng thuận pha (KAMA 15m cùng chiều, không đối nghịch).
  - **Unit Tests:**
    - [ ] `test_mtf_phase_coherence_blocks_counter_trend_micro_noise`

---

### PHẦN 3: TÁI CẤU TRÚC HÀM MỤC TIÊU GA (OPTIMIZER ALIGNMENT)
- [~] **TODO-STAB-05: Tái Cấu Trúc Hàm Mục Tiêu Fitness GA Theo Chuẩn t-Statistic EV_Net (TRẠNG THÁI: HOÀN THIỆN MỘT PHẦN — PHASE 1 DONE)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs`, `ferro_trade/src/stats/core.rs`
  - **Đã Hoàn Thành (Giai Đoạn 1 — Quick Win):**
    - [x] Triển khai hàm `calculate_enhanced_fitness_with_leverage` và struct `FitnessConfig`.
    - [x] Khấu trừ Trượt giá Động theo đòn bẩy tài khoản: $\text{SlippageDrag} = 2 \times \text{slippage\_rate} \times L$.
    - [x] Cổng chặn mẫu tự co giãn theo thời gian: $N_{\min}, N_{\max}$ theo $T_{\text{days}}$.
    - [x] Đồng bộ Cổng WFE Validation Gate: $\text{val\_net\_ret} > 0.0$ mới duyệt DNA.
    - [x] Unit Tests $7/7$ PASSED: `cargo test --lib optimizer::prospector_runner`.
  - **Kế Hoạch Dài Hạn (Giai Đoạn 2 — Hoàn Thiện Thể Chế, Sẽ Thực Hiện Khi App Ổn Định):**
    - [ ] Mở rộng trường dự phòng `custom_1` trong struct 256-byte aligned `PackedStats` (`ferro_trade`) để tính toán Downside Deviation và EV trực tiếp từ từng giao dịch riêng lẻ (`closed_trades`).


---

### PHẦN 4: HÓA GIẢI CÁC ĐIỂM YẾU GIAO THOA GIỮA HELIOS VÀ TRADING BOT
- [x] **TODO-STAB-06: Độc quyền hóa Thẩm quyền Risk về `helios::risk` (Khử Split-Brain) & Pure WebSocket Trade API**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs`, `live_engine.rs`, `ws_manager.rs`
  - **Mô tả:** 
    - Độc quyền hóa thẩm quyền Risk về `helios::risk`, đồng bộ 1:1 Stop Loss và Playbook Tag giữa Helios và Bot Engine.
    - Chuyển đổi 100% việc gửi lệnh Market (`order.place`), hủy Algo SL (`algoOrder.cancel`) và đặt Algo SL (`algoOrder.place`) sang Pure WebSocket của Binance Connector Rust, bảo toàn 2400 API Weight/phút và triệt tiêu độ trễ HTTP.
    - Cơ chế Fast Reconnect và theo dõi `active_algo_ids` chuẩn xác theo quy tắc vi mô sàn Binance (Hủy SL cũ -> Đặt SL mới trong 5-15ms).
  - **Unit Tests:**
    - [x] `test_order_executor_wait_ws_ready_timeout`
    - [x] `test_order_executor_algo_id_tracking`
    - [x] `test_live_engine_syncs_current_sl_price`
    - [x] `test_live_engine_preserves_playbook_tag`
    - [x] `test_live_engine_adopt_position_with_active_sl`
- [ ] **TODO-STAB-07: Cơ chế Tick-Level Emergency Circuit Breaker (Khử Intra-Candle Blindness)**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs`
  - **Mô tả:** Lắng nghe WebSocket MarkPrice để cắt lỗ khẩn cấp ngay trong nến khi xảy ra Flash Crash hoặc gãy cấu trúc nghiêm trọng.
- [ ] **TODO-STAB-08: Xây dựng Portfolio Risk Guardian (Khống chế Đòn bẩy Danh mục)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs` (mới)
  - **Mô tả:** Kiểm soát tổng đòn bẩy danh mục không vượt quá giới hạn an toàn khi nhiều cặp tiền cùng mở lệnh.
- [ ] **TODO-STAB-09: Atomic SL Replacement (Khử Naked SL Gap 150ms)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs`
  - **Mô tả:** Đặt SL mới thành công trước khi hủy SL cũ trên Binance Futures.
- [ ] **TODO-STAB-10: True Adopt & Protect khi Khởi động lại Bot (Chống SL Retraction)**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs`, `live_engine.rs`
  - **Mô tả:** Query trực tiếp lệnh Algo SL đang sống trên sàn để kế thừa mức giá dừng lỗ đã khóa.

---

### PHẦN 5: KHẮC PHỤC CÁC TỬ HUYỆN TOÁN HỌC & ĐỘNG CƠ BACKTEST (`ferro_trade`)
- [ ] **TODO-AUDIT-FT-01: Triệt tiêu Lookahead Bias trong Sizing Margin tại Nến Khớp Lệnh**
  - **File:** `ferro_trade/src/broker/broker.rs`, `ferro_trade/src/backtest/engine.rs`
  - **Mô tả:** Định cỡ và tính toán ký quỹ tại đầu nến $i$ bằng giá $Open[i]$ hoặc giá đóng nến trước đó $Close[i-1]$, tuyệt đối không dùng $Close[i]$.
- [ ] **TODO-AUDIT-FT-02: Khử Backfill (`bfill`) trong `atr_sma` và Bơm Số 0 trong `chaos_ratio`**
  - **File:** `ferro_trade/src/indicators/bulk.rs`
  - **Mô tả:** Trả về $NaN$ trong giai đoạn Warmup của ATR; ngăn chặn thổi phồng `chaos_ratio` và bảo đảm tính toàn vẹn của `actual_warmup`.
- [ ] **TODO-AUDIT-FT-03: Sửa Công Thức Geometric Mean ($N$ thay vì $N+1$) trong Thống Kê**
  - **File:** `ferro_trade/src/backtest/stats/core.rs`
  - **Mô tả:** Chuẩn hóa công thức $gmean$ theo đúng số bước thời gian $N$.
- [ ] **TODO-AUDIT-FT-04: Sửa Logic Pessimistic Execution cho Stop Entry & Bảo Toàn Vốn Hedging**
  - **File:** `ferro_trade/src/broker/broker.rs`
  - **Mô tả:** Khớp SL bảo thủ ngay trong nến nếu vi phạm mức dừng lỗ; tính đúng Unrealized PnL khi vị thế ròng về 0.

---

- [x] **TODO-AUDIT-H-01: Hóa Giải Nghịch Lý Khối Lượng và Tái Sinh `reversal.rs`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`, `config.rs`, `core_v3/mod.rs`
  - **Mô tả:** Thay thế trường DNA `exhaustion_vol_drop` bằng `climax_vol_threshold` (1.00 -> 3.50, default 1.50) dùng hàm `sat_unbounded(vol_ratio, vol_thresh)`. Chấp nhận bão khối lượng lớn ($vol\_ratio \ge 1.5$) là tín hiệu Selling/Buying Climax; bảo toàn ngân sách 107 chiều DNA.
  - **Unit Tests:**
    - [x] `test_reversal_volume_exhaustion_behavior`
    - [x] `test_dna_width_and_parameter_defs_integrity`
- [ ] **TODO-AUDIT-H-02: Phá Bẫy Catch-22 Giữa Stop Loss Cấu Trúc và VETO ROE**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`, `anomaly.rs`, `arbiter.rs`
  - **Mô tả:** Cho phép Scanner tự động co hẹp SL về trần an toàn $15\%$ ROE thay vì bị Arbiter giết chết toàn bộ tín hiệu biên độ lớn.
- [ ] **TODO-AUDIT-H-03: Xử Lý Mã Nguồn Ma `range_osc.rs` & Chuẩn Hóa Điểm Số Arbiter**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs`, `arbiter.rs`
  - **Mô tả:** Tích hợp hoặc loại bỏ an toàn `range_osc`; chuẩn hóa phân phối điểm số (Z-Score scaling) giữa các Scanner trong sàn đấu Arbiter.
- [ ] **TODO-AUDIT-H-04: Mở Khóa Hyperparameters Trong DNA & Tinh Chỉnh KAMA Slope**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs`, `dna_mapping.rs`, `matrix.rs`
  - **Mô tả:** Đưa `min_activation_score` vào DNA; điều chỉnh hệ số chia `atr * 0.20` thay vì `0.05` để khử bão hòa nhị phân trong KAMA slope.

---

### PHẦN 7: GIA CỐ NỀN TẢNG THỰC THI & CONCURRENCY (`trading_bot`)
- [ ] **TODO-AUDIT-TB-01: Tách Biệt Kết Nối SQLite & Khử Nghẽn Luồng Worker Tokio**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs`
  - **Mô tả:** Chuyển sang mô hình Read/Write connection pool hoặc tách riêng Connection cho InitialSync/Optimizer bằng `spawn_blocking`.
- [ ] **TODO-AUDIT-TB-02: WebSocket Watchdog & Chống Zombie Connection**
  - **File:** `trading_bot/bot_engine/src/data_engine/ws_manager.rs`, `binance_stream.rs`
  - **Mô tả:** Bổ sung Heartbeat / Ping-Pong và kiểm tra độ tươi nến (Candle Freshness Guard). Tự động reconnect nếu không nhận được dữ liệu quá 60s.
- [ ] **TODO-AUDIT-TB-03: Spawn Blocking Cho Diagnostic Backtest & Khử Deadlock Cờ Lệnh**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs`, `execution/live_engine.rs`
  - **Mô tả:** Đưa tác vụ backtest nặng sang `spawn_blocking`; bổ sung timeout tự động reset cờ `is_submitting_order` sau 5 giây nếu không nhận phản hồi sàn.
- [ ] **TODO-AUDIT-TB-04: Khắc Phục Lỗ Hổng Panic Close All & Cô Lập Vốn Ví**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs`, `execution/order_executor.rs`
  - **Mô tả:** Gửi lệnh Market Close thật lên sàn Binance khi Panic Close; tách kênh dữ liệu số dư cho từng Actor tương ứng với vốn phân bổ.

---

### PHẦN 8: NHIỆM VỤ CẤP BÁCH TỪ KIỂM TOÁN RỦI RO HỆ THỐNG MỚI (NOTES 12, 13, 14)
- [ ] **TODO-AUDIT-NEW-01: Vá Lỗ Hổng Phình To Vốn Sàn Trên Live Engine**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:133`
  - **Mô tả:** Đổi `usable_capital = self.account_state.available_balance.min(self.core.allocated_capital)`. Tuyệt đối không để một cặp tiền dùng toàn bộ số dư ví Binance (\$10,000) thay vì ngân sách được cấp (\$60).
- [ ] **TODO-AUDIT-NEW-02: Khử Bẫy Silent Failure & Naked SL Gap 150ms**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:190-236`
  - **Mô tả:** Nếu sau 3 lần retry đặt SL mới thất bại $\to$ kích hoạt Emergency Market Close toàn bộ vị thế (`reduce_only: true`), không bao giờ để vị thế mồ côi không SL ngoài thị trường.
- [ ] **TODO-AUDIT-NEW-03: Xóa Bỏ Bẫy Clean Slate Bán Tháo Sạch Vị Thế Khi Khởi Động**
  - **File:** `trading_bot/bot_engine/src/execution/account_manager.rs:175-178`
  - **Mô tả:** Xóa bỏ `cancel_all_orders` và `force_close_all_positions`. Triển khai True Adopt & Protect query Open Algo Orders để bảo vệ vị thế đang sống.
- [x] **TODO-AUDIT-NEW-04: Phá Bẫy Catch-22 VETO Triệt Tiêu Nến Đẹp của Arbiter**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:254-280`
  - **Mô tả:** Xóa bỏ Binary Veto khi SL $> 15\%$ ROE. Thay bằng tự động gọi `clamp_sl_price` kẹp SL về trần an toàn và co giãn khối lượng (Dynamic Sizing: $\text{scale} = 15\% / \text{suggested\_roe}$).
- [x] **TODO-AUDIT-NEW-05: Vá Lỗ Hổng Inverted SL Bằng Tham Số `is_long` & Chuẩn Hóa Stop Loss Toàn Hệ Thống (`TODO-ARB-02-R2`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `arbiter.rs`, `scanners/*.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH 100% QUA 3 BƯỚC] 
    1. Bổ sung `is_long: bool` và variant `InvertedPrice` vào `validate_structural_sl`; Arbiter VETO tức thì proposal dị tật.
    2. Kiến tạo hàm tĩnh chuẩn hóa `RiskManager::sanitize_structural_sl` nắn hình học nến vi mô cho Stop Loss (Long $SL < Entry$, Short $SL > Entry$, tự động fallback an toàn nếu Doji/lỗi số học).
    3. Tích hợp `sanitize_structural_sl` vào toàn bộ 6 Scanners (`reversal`, `anomaly`, `trend_pb`, `absorption`, `momentum_ignition`, `range_osc`), xóa bỏ 100% code kẹp thủ công phân mảnh vi phạm DRY.
  - **Unit Tests:**
    - [x] `test_validate_structural_sl_directional_and_inverted`
    - [x] `test_arbiter_vetos_inverted_sl_proposal`
    - [x] `test_sanitize_structural_sl_exhaustive`
    - [x] `test_absorption_sl_spring_geometry`
- [x] **TODO-AUDIT-NEW-06: Khử Lỗi Mù SL Của Strategy (Phantom SL Desync)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:117-160`
  - **Mô tả:** Cập nhật ngay `live_position.current_sl_price = Some(new_sl)` khi dời SL để Strategy không bị mù trạng thái sàn và ngừng spam lệnh dời SL mỗi nến. (Đã hoàn thành và kiểm thử trong `test_live_engine_syncs_current_sl_price`).
- [ ] **TODO-AUDIT-NEW-07: Khai Thông `Trend_PB` & Sửa Hàm Volume Bán Tháo `Reversal`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:64, 98`, `reversal.rs:105-107`
  - **Mô tả:** Gỡ bỏ Squeeze ngáng chân trong TrendPB (vì RangeOsc đã tắt); sửa hàm volume Reversal chấp nhận cả bão khối lượng bán tháo ($vol\_ratio > 1.5$) là tín hiệu đảo chiều.
- [ ] **TODO-AUDIT-NEW-08: Triển Khai Staleness Watchdog Chống Zombie DNA Lock-in**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:423-440`
  - **Mô tả:** Đếm số lần Reject liên tiếp: sau 3 lần giảm $50\%$ size/leverage; sau 6 lần kích hoạt Defensive Hold (đứng ngoài thị trường bảo toàn vốn).
- [ ] **TODO-AUDIT-NEW-09: Thống Nhất Mô Hình Cooldown Giữa Backtest và Live**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:272`, `ferro_trade/src/broker/broker.rs`
  - **Mô tả:** Đưa logic Cooldown vào Backtest Broker để GA tối ưu hóa và đánh giá đường cong vốn khớp 1:1 với thực tế.

---

### PHẦN 9: NHIỆM VỤ CỐT TỬ TỪ KIỂM TOÁN GIAO TIẾP BOT-HELIOS & CONCURRENCY (NOTES 16, 17, 18)
- [x] **TODO-AUDIT-NEW-10: Vá Khuyết Tật Bỏ Quên Hard SL Ban Đầu Khi Mở Lệnh Live (`BUG-EXEC-NEW-01`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs`, `trading_bot/bot_engine/src/execution/live_engine.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH] Bóc tách `sl_price` từ `policy: Some(RiskPolicy::FixedPrice { sl_price, .. })` thay vì chỉ đọc `stop` (vốn là None); bổ sung `pending_initial_sl` trên `LiveEngine` chống race condition với `PositionUpdate`; nâng cấp 7 lần retry qua WebSocket API với exponential backoff; xử lý lỗi trùng lệnh `-4061` và kích hoạt 2-tier Emergency Panic Close (WS -> REST fallback) triệt tiêu hoàn toàn Naked Position.
  - **Unit Tests:**
    - [x] `test_extract_sl_price_from_action`
    - [x] `test_live_engine_initial_sl_from_policy`
- [x] **TODO-AUDIT-NEW-11: Sửa Lỗ Hổng Tê Liệt GA Optimizer Từ Chu Kỳ 2 Trở Đi (`NEW-OPT-01`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:360-385`, `trading_bot/bot_engine/examples/walk_forward_smart_seed.rs:320-330`
  - **Mô tả:** [ĐÃ HOÀN THÀNH] Khởi tạo Job mới vô điều kiện mỗi chu kỳ Walk-Forward Rolling (`add_initial_job`), loại bỏ hoàn toàn bẫy kiểm tra Resume trên đĩa (`if !db_exists || get_max_job_id() == 0`) gây tê liệt GA; tự động dọn sạch tệp/thư mục đệm cũ trước khi chạy; loại bỏ dòng code xóa nhầm SQLite database chính (`walk_forward_smart_seed_db`) trong example.
  - **Unit Tests:**
    - [x] `test_prospector_runner_unconditional_job_seeding`
- [x] **TODO-AUDIT-NEW-12: Khắc Phục Thảm Họa High Watermark Buffer Bleed Khi Lệnh Trôi Khỏi Buffer (`TODO-BRG-01`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:223-238`, `strategies_backtests/helios/src/strategy/core_v3/risk.rs:310-330`
  - **Mô tả:** Bổ sung `highest_favorable_price` vào `LivePositionState` và `Trade`, cập nhật đơn điệu qua từng nến, xóa bỏ hoàn toàn vòng lặp quét lại toàn bộ nến quá khứ từ index 1 khi `mapped_entry_bar == 0`.
- [ ] **TODO-AUDIT-NEW-13: Sửa Lỗ Hổng Same-Bar Re-entry Guard Bị Bypassed Khi Đóng Lệnh Giữa Nến (`BUG-ACT-NEW-02`)**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs:373-378, 637-644`
  - **Mô tả:** Bổ sung `last_exit_bar_idx: Option<usize>` vào `AssetActor`, khóa cứng cấm mở lệnh lại trên cùng cây nến vừa thoát lệnh, bất kể lệnh đóng giữa nến qua WS hay tại nến đóng.
- [x] **TODO-AUDIT-NEW-14: Khôi Phục Tag Playbook & Đồng Bộ SL Sàn Vào Live Position (`TODO-BRG-03`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:197-207`
  - **Mô tả:** Lưu và chuyển tiếp `opening_playbook_tag` sang vị thế để kích hoạt Dynamic TP của Range/Reversal; đồng bộ `current_sl_price` từ `OrderExecutor` để Strategy không bị mù trạng thái sàn. (Đã kiểm thử trong `test_live_engine_preserves_playbook_tag` & `test_live_engine_adopt_position_with_active_sl`).
- [x] **TODO-AUDIT-NEW-15: Đồng Bộ Hóa Cửa Sổ Bộ Nhớ Theo Ngày & Xóa Bỏ Cắt Lát Nến Cụt (`TODO-ACT-04`)**
  - **File:** `trading_bot/bot_engine/src/models/config.rs:24-95`, `models/models.rs:29-76`, `controller/asset_actor.rs:148, 196, 575`, `examples/walk_forward_smart_seed.rs:63, 192`
  - **Mô tả:** [ĐÃ HOÀN THÀNH: Bác bỏ phương án hardcode 4h UTC vi phạm tính độc lập khung thời gian]. Triển khai kiến trúc Single Source of Truth dẫn xuất động theo số ngày (`EngineConfig::for_days(days)`): Động hóa `Asset.max_capacity`, đồng bộ `sync_history` với `historical_sync_days`. Trong `walk_forward_smart_seed.rs`, đồng bộ RAM của Actor bằng đúng `TRAIN_WINDOW_DAYS` (40 ngày = 57,600 nến 1m) khiến `slice_start = 0`, triệt tiêu hoàn toàn nến cụt và sai số khởi tạo MTF, đồng bộ 1:1 dữ liệu giữa GA Optimizer và Actor.
- [ ] **TODO-AUDIT-NEW-16: Chuẩn Hóa `apply_dna` Cho Toàn Bộ Điểm Cập Nhật DNA Nóng (`TODO-ACT-05`)**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs:338, 515`
  - **Mô tả:** Thay thế các lệnh gọi trực tiếp `update_from_dna` bằng `self.apply_dna(...)` để kiểm tra kích thước DNA (103 chiều) và chặn triệt để `NaN`/`Inf`.
- [ ] **TODO-AUDIT-NEW-17: Khử Bom Nổ Chậm Panic Crash Trong `RangeOsc` & Dọn Cảm Biến Rỗng (`TODO-SEN-08`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs:77-78`, `core_v3/mod.rs:148-173`
  - **Mô tả:** Thêm guard kiểm tra độ dài vector trước khi index `sensors.stoch_range_k[i]`, tránh crash ứng dụng khi tái kích hoạt Scanner.
- [ ] **TODO-AUDIT-NEW-18: Thẩm Định Ngoài Mẫu OOS Thuần Khiết Cắt Bỏ Nhiễm Bẩn Warmup (`TODO-PROS-02`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:410-435`
  - **Mô tả:** Chỉ tính toán thống kê WFE Gate trên các lệnh có `entry_bar >= warmup_offset`, loại bỏ hoàn toàn 5,760 nến In-Sample gối đầu.
---

### PHẦN 10: NHIỆM VỤ CỐT TỬ TỪ KIỂM TOÁN MARKET MATRIX & REGIME GATEKEEPER (NOTES 19, 20)
- [x] **TODO-AUDIT-NEW-19: Khử Triệt Để Bẫy Tụt Về 0.50 Của Squeeze Intensity (`TODO-MAT-01`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/matrix.rs:55-63, 81`
  - **Mô tả:** Loại bỏ hoàn toàn bước bọc qua `rolling_percentile`. Gán trực tiếp giá trị thực phi thứ nguyên $\text{sqz\_raw} = \frac{1}{1 + (BB/KC)^2} \in [0.0, 1.0]$, bảo toàn độ nén thực khi thị trường tích lũy kéo dài.
- [x] **TODO-AUDIT-NEW-20: Triển Khai Tanh Scaling Cho KAMA Slope Chuẩn Hóa Theo ATR (`TODO-MAT-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/matrix.rs:81-84`, `config.rs:416-422`
  - **Mô tả:** [ĐÃ HOÀN THÀNH]: Thay thế phép kẹp cứng `.clamp(-1.0, 1.0)` bằng hàm hyperbol trơn $\tanh(s / \text{scale\_atr})$, mở rộng dải DNA `kama_slope_scale_atr` `[0.02, 1.00, step = 0.02]` (50 nấc giá trị). Khôi phục gradient trơn mượt phi tuyến tính, triệt tiêu hoàn toàn hiện tượng kẹt trần và mù giảm tốc khi sóng hạ nhiệt. Đã kiểm chứng qua unit tests `test_kama_slope_atr_scale_asset_agnostic` và `test_kama_slope_atr_tanh_scaling_preserves_gradient_under_extreme_trend`.
- [-] **TODO-AUDIT-NEW-21: Đồng Bộ Float Comparator Trong Chuẩn Hóa Phân Vị (`TODO-SEN-11`)**
  - **File:** `strategies_backtests/helios/src/utils/normalization.rs:68-115`
  - **Trạng thái:** `[-] (OBSOLETE / ARCHIVED)`
  - **Mô tả:** Đã gắn cờ `#[deprecated]` cho `rolling_percentile`. Hàm này là Dead Code trong Helios V3 (đã được thay thế hoàn toàn bởi các bộ chuẩn hóa vật lý liên tục $O(1)$ như Tanh scaling và direct bounds theo Note 08 & Note 19 để triệt tiêu lỗi forward-fill MTF). Được lưu giữ tạm thời phục vụ Core V2 legacy và sẽ được dọn dẹp cùng Core V2.
- [ ] **TODO-AUDIT-NEW-22: Sửa Fallback Z-Score Thành 0.0 Khi Thị Trường Đứng Im (`TODO-FT-02`)**
  - **File:** `ferro_trade/src/indicators/calculate/bulk.rs:128`, `helios/src/utils/normalization.rs:236`
  - **Mô tả:** Khi $\sigma \le 10^{-9}$ và $|X - \mu| \le 10^{-9}$, trả về $Z = 0.0$ thay vì `NaN`, ngăn ngừa lây nhiễm `NaN` toàn hệ thống khi thị trường flat.
- [x] **TODO-AUDIT-NEW-23: Chuyển Đổi Veto Structural SL Sang Cơ Chế Auto-Clamping (`CATCH-22-STRUCTURAL-SL`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:254-280`
  - **Mô tả:** Xóa bỏ lệnh Hard Veto hủy tín hiệu khi SL vượt trần; tự động kẹp SL về `max_allowed_roe` và bảo toàn rủi ro bằng cách giảm quy mô vị thế, kích hoạt lại khối mã `clamp_sl_price`.
- [x] **TODO-AUDIT-NEW-24: Tách Bạch Micro NATR vs Macro NATR Cho Cầu Dao Flatline (`NEW-NATR-4H-BLIND`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs:24-32`
  - **Mô tả:** Sử dụng nến vi mô 1m/5m cho chốt chặn Flatline thay vì dùng NATR khung 4h, khắc phục triệt để bẫy mù thanh khoản khi thị trường đứng im.
- [ ] **TODO-AUDIT-NEW-25: Gỡ Bỏ Lệnh Đánh Giá Lặp Lại Gatekeeper Trong Arbiter (`NEW-DOUBLE-EVAL`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:168-173`
  - **Mô tả:** Loại bỏ lệnh gọi `gatekeeper.evaluate` lần 2 trong `process_tick()`, thống nhất quyền kiểm soát cửa ngõ duy nhất tại `core_v3/mod.rs:241`.
- [ ] **TODO-AUDIT-NEW-26: Dọn Cảm Biến Mồ Côi & Gỡ Ô Nhiễm Tham Số ATR Khỏi Độ Dốc KAMA (`ORPHAN-SENS`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:114-120`
  - **Mô tả:** Xóa bỏ `sensors.kama_slope` không sử dụng; thay `ind.atr.period` bằng `ind.kama.short_period` trong hàm gọi `ctx.kama_slope` để tránh ô nhiễm tham số SL vào KAMA.

---

### PHẦN 11: NHIỆM VỤ CỐT TỬ TỪ KIỂM TOÁN ARBITER & RÀO CẢN TIỀN KHỚP LỆNH (NOTES 21, 22)
- [x] **TODO-AUDIT-NEW-27: Xóa Bỏ Nhánh Đối Kháng Nến Đơn Là Mã Chết 100% Trong Arbiter (`TODO-ARB-01`) (BÁO ĐỘNG GIẢ: ĐÃ BÁC BỎ)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:521-538`
  - **Mô tả:** [BÁO ĐỘNG GIẢ / ĐÃ BÁC BỎ: Tiền đề note 21 sai lầm do AnomalyFade và RangeOscillator không dùng CLV, cặp Long/Short nến đơn hoàn toàn xảy ra trong thực tế. Nhánh resolve_conflicts là chốt chặn phòng vệ xung đột tối quan trọng, giữ nguyên 100% mã nguồn theo nguyên lý First-Principles.] Do $CLV$ chỉ có 1 dấu trên mỗi cây nến $i$, nhánh đối đầu liên chiều Long vs Short không bao giờ xuất hiện. Đơn giản hóa `resolve_conflicts` về một chuỗi tìm kiếm cực đại đơn giản `proposals.into_iter().max_by(...)`.
- [x] **TODO-AUDIT-NEW-28: Triển Khai Inter-Bar Flip-Flop Guard Chống Bẫy Vả 2 Đầu (`TODO-ARB-02`) (BÁO ĐỘNG GIẢ: ĐÃ BÁC BỎ)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:111-158`, `core_v3/mod.rs`
  - **Mô tả:** [BÁO ĐỘNG GIẢ / ĐÃ BÁC BỎ: Hiện tượng bẫy vả 2 đầu đã được ngăn chặn triệt để ở tầng Bot Engine thông qua Dual Sentinels: No Same-Bar Re-entry + EngineCooldownGuard (khóa 15-45 nến khi dính SL). Đề xuất nhét logic này vào Arbiter là sai lệch phân tầng kiến trúc theo nguyên tắc ARCH-COOLDOWN-SEPARATION tại mod.rs:273.] Bổ sung bộ nhớ liên nến: Cấm mở lệnh đảo chiều ngược lại trong vòng $K=2$ nến ngay sau khi vừa bị cắt lỗ Stop Loss, trừ khi có nến xung lực bứt phá cực đại $> 2.0\text{ ATR}$.
- [-] **TODO-AUDIT-NEW-29: Chuyển Đổi Sniper Veto Sang Dynamic De-leverage & Size Scaling (`TODO-ARB-03`) (PENDING: Chờ Cụm F)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:223-289`, `risk.rs:84-138`
  - **Mô tả:** [PENDING: Phần Arbiter gỡ bỏ Hard Veto đã hoàn thành tại TODO-AUDIT-NEW-04 & NEW-23. Phần co size $Fixed Risk USD thuộc thẩm quyền tầng Quản trị Rủi ro tại Cụm F (TODO-AUDIT-NEW-55 & TODO-AUDIT-NEW-118).] Khi SL cấu trúc $> 15\%$ ROE (nhưng $\le 35\%$ ROE), tự động co giảm kích thước vị thế tỷ lệ nghịch với khoảng cách SL để bảo toàn \$ USD Risk cố định thay vì VETO hủy lệnh, tái kích hoạt khối code `clamp_sl_price`.
- [x] **TODO-AUDIT-NEW-30: Bổ Sung EV Hurdle Gate Nhận Thức Chi Phí Ma Sát Sàn 2.6% ROE (`TODO-ARB-05`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:221-470`, `config.rs:517`, `telemetry/cards.rs:122, 260`
  - **Mô tả:** [ĐÃ HOÀN THÀNH] Triển khai cơ chế Dual-Track EV Hurdle Gate (`validate_ev_hurdle`) $O(1)$ phi thứ nguyên trong `HeliosArbiter`: (1) Track 1 (Explicit TP cho Reversal/RangeOsc) kiểm tra tính nhất quán hướng và biên độ $|TP - P|/P \ge (2 \times \text{fee} + \text{slippage}) \times \text{min\_ev\_hurdle\_ratio}$; (2) Track 2 (Dynamic Trend cho TrendPB/Momentum/Absorption) kiểm tra không gian biến động vi mô $\text{ATR}_{\text{micro}}/P \ge \text{Required Hurdle}$. Bổ sung `AuctionStatus::FrictionalHurdleVeto` và `min_ev_hurdle_ratio = 2.50` (`#[serde(default)]`, bảo toàn `DNA_WIDTH = 105`), tương thích đa tài sản từ Forex đến Memecoin.
  - **Unit Tests:**
    - [x] `test_ev_hurdle_explicit_tp_rejection`
    - [x] `test_ev_hurdle_explicit_tp_direction_contradiction`
    - [x] `test_ev_hurdle_dynamic_trend_rejection`
    - [x] `test_ev_hurdle_pass_and_forex_multi_asset_compatibility`
- [ ] **TODO-AUDIT-NEW-31: Dọn Dẹp Triệt Để 8 Biến Ma & Trường Rác Khỏi `ArbiterConfig` (`ZOMBIE-ARBITER-CFG`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:450-478`
  - **Mô tả:** Xóa bỏ 6 dead fields cũ và 2 trường bù nhìn (`max_chaos_penalty`, `conflict_confidence_margin`), tách các trường thuộc Cầu Dao sang `GatekeeperConfig`.
- [x] **TODO-AUDIT-NEW-32: Vá Lỗ Hổng Inverted SL & Động Hóa MMR Theo Tier Sàn (`TODO-ARB-02-R2`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:32-138`, `trading_bot/bot_engine/src/models/market_info.rs`, `trading_bot/bot_engine/src/controller/asset_actor.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH TOÀN DIỆN]
    1. *Vế 1 (Inverted SL):* Đã được bảo vệ 100% trong `TODO-AUDIT-NEW-05` với `SlValidationResult::InvertedSlVeto`.
    2. *Vế 2 (Động hóa MMR - Phòng vệ 2 tầng Hybrid Defense-in-Depth):*
       - **Pha 1 (`helios` Core):** Thêm `RiskManager::resolve_effective_mmr(configured_mmr, leverage)` tự động nội suy Tier 1 chuẩn sàn Binance (0.40% cho >100x, 0.50% cho >50x, 0.65% cho >20x, 1.00% cho <=20x) khi `configured_mmr <= 0.0 || NaN` (Backtest offline), trong khi vẫn tôn trọng 100% MMR cấu hình tường minh của Altcoin.
       - **Pha 2 (`trading_bot` Engine):** Bổ sung `BracketTier` và `resolve_mmr_for_notional` trong `MarketFilter`; thêm `fetch_leverage_brackets` vào `AccountManager`; nạp brackets từ Binance REST API lúc startup và cập nhật tự động `sync_dynamic_mmr()` trong `AssetActor` theo quy mô vị thế `Notional = Allocated Capital * Leverage`.
  - **Unit Tests:**
    - [x] `test_resolve_effective_mmr_tier_mapping` (Helios)
    - [x] `test_resolve_effective_mmr_preserves_explicit_custom_mmr` (Helios)
    - [x] `test_validate_structural_sl_approves_high_leverage_btc_scalp` (Helios)
    - [x] `test_resolve_mmr_for_notional` (Bot Engine)
- [-] **TODO-AUDIT-NEW-33: Triển Khai Post-Storm Ramp-up Filter Sau Bão (`POST-STORM-BLINDNESS`) (PENDING: TẠM DỪNG CHỜ THỰC NGHIỆM)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`, `gatekeeper.rs:177-206`
  - **Mô tả:** [PENDING: ĐỐI CHẤT PHƯƠNG ÁN A & B TRƯỚC KHI QUYẾT ĐỊNH] Đánh giá nhu cầu kiểm soát rủi ro sau khi thoát bão biến động cực hạn (`CircuitBreaker` mở lại) để phòng ngừa trượt giá và râu nến quét SL khi sổ lệnh còn mỏng.
    - **Bản chất vấn đề:** Note 22 lo ngại hệ thống mở khóa quá sớm ngay sau bão lớn, dẫn đến việc vào lệnh trong vùng dư chấn biến động cao khi thị trường chưa ổn định.
    - **Đối chất Phương án A (Hệ thống phòng vệ 4 tầng hiện tại đã tự giải quyết):**
      1. *Dải trễ Hysteresis 5 bars (`gatekeeper.rs:273-330`):* Cầu dao không mở lại vội vã mà duy trì bộ đệm trễ $[t_{\text{exit}}, t_{\text{enter}}]$. Chỉ khi biến động thực sự hạ nhiệt bền vững mới mở khóa.
      2. *Micro Liquidity Floor (`gatekeeper.rs:211-230`):* Chặn tuyệt đối khi `micro_natr` nhỏ hơn phí ma sát sàn Taker, ngăn vào lệnh khi nến tê liệt / sổ lệnh mất thanh khoản.
      3. *Risk Parity Sizing tự nhiên (`risk.rs:291-295`):* Quy mô vị thế tính theo $\text{Position\_USD} = \text{Risk\_USD} / \text{sl\_dist\_pct}$. Sau bão biến động cao, râu nến rộng khiến khoảng cách SL lớn $\implies$ Hệ thống toán học đã **tự động co nhỏ vị thế** để bảo toàn mức lỗ hằng số (\$ USD Risk). Việc ép giảm cứng 50% size là bẫy phạt đúp (double-penalty) thiếu cơ sở.
      4. *Bảo toàn vị thế loại trừ tuyệt đối (`core_v3/mod.rs:232-238`):* Khi lệnh đang mở (`is_pos_open = true`), bot cấm hoàn toàn lệnh mới. Lệnh cũ phải đóng toàn phần $100\%$ về $0.0$ trước khi lệnh mới được xem xét.
    - **Đối chất Phương án B (Thêm bộ lọc định lượng: Kaufman Efficiency Ratio KER / Chaos Index / Thanh khoản sau bão):**
      1. *Ưu điểm lý thuyết:* Thay vì đếm mù quáng $M=3$ nến hay giảm 50% size cảm tính, ta có thể dùng $\text{KER} \ge \text{threshold}$ hoặc `chaos_index < 0.60` để xác nhận thị trường không còn hỗn loạn dao động do dự.
      2. *Nhược điểm & Bẫy triệt tiêu cơ hội (Blanket Veto):* Ép KER cao ngay sau bão sẽ vô tình **triệt tiêu hoàn toàn các cơ hội gom hàng tạo đáy** kinh điển của `AbsorptionScout` (vốn xuất hiện trong vùng nén tích lũy đáy biên độ hẹp với KER thấp) hoặc bắt dao cạn kiệt của `RangeOscillator`.
    - **Kết luận:** Tạm hoãn (PENDING) không đưa thêm rào cản đếm nến hardcoded hay ép chỉ số làm biến dạng các playbook. Bảo lưu theo dõi trên dữ liệu Telemetry thực tế từ Walk-Forward Out-Of-Sample dài hạn. Chỉ can thiệp nếu nhật ký thực nghiệm phát hiện tổn thất tập trung đặc thù sau nến thoát bão.

---

### PHẦN 12: KIỂM TOÁN TỪNG CHIẾN ĐỘI SCANNER (SCANNER AUDIT & QUANT METHODOLOGY)

- [x] **TODO-AUDIT-NEW-34: Tháo Gỡ Nghịch Lý Hình Học Nến (%B vs CLV) Trong Absorption Scanner (`ABS-GEOM-PARADOX`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:79-105`
  - **Mô tả:** [ĐÃ HOÀN THÀNH BƯỚC 1] Tách bạch điều kiện quét râu (`pct_b_low <= max_bound`) với điều kiện rút chân (`CLV >= min_clv`), loại bỏ việc ép `Close` phải đóng sát đáy `%B <= max_bound` đang tự triệt tiêu các nến Spring kinh điển. Đảm bảo nến đóng kiểm soát ở nửa dưới dải nén (`pct_b_close <= max_bound + spring_tol && close <= bb_middle`).
  - **Unit Tests:**
    - [x] `test_absorption_spring_geometry_wick_deep_close_controlled`
    - [x] `test_absorption_blocks_chasing_when_close_crosses_middle`
    - [x] `test_absorption_blocks_marubozu_knife_at_lower_band`
    - [x] `test_absorption_upthrust_short_symmetric`
    - [x] `test_absorption_blocks_deep_underground_breakdown`
    - [x] `test_absorption_blocks_runaway_breakout`

- [x] **TODO-AUDIT-NEW-35: Khắc Phục Lệch Pha MTF Giữa Nến Vi Mô 1m & Dải Bollinger MTF 15m (`ABS-MTF-SCALE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:79-105`
  - **Mô tả:** [ĐÃ HOÀN THÀNH BƯỚC 1] Thay vì so sánh giá đóng cửa nến 1m với dải Bollinger 15m khổng lồ, chuyển sang đo lường tọa độ râu nến nhúng sâu vào vùng đáy tích lũy `pct_b_low <= max_bound` (khi Squeeze dải BB 15m co thắt biên độ tương thích), kết hợp loại bỏ hoàn toàn bộ lọc trễ pha `vwma < bb_middle` theo `TODO-SCN-02`.
  - **Unit Tests:**
    - [x] `test_absorption_valid_accumulation_in_neutral_range`
    - [x] `test_absorption_spring_geometry_wick_deep_close_controlled`

- [x] **TODO-AUDIT-NEW-36: Hiệu Chuẩn Điểm Tự Tin Anti-Inflation & Loại Bỏ VWMA Trễ Pha (`ABS-SCORE-INFLATION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`, `normalization.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH: 12/12 Audit PASS, 5 tests PASS]. 
    1. Hiệu chuẩn lại hàm `sat_ratio_damped` bằng mẫu số điều hòa nến chuẩn 1.0 ATR: $\text{den} = 1.0 + \text{norm\_spread}$, triệt tiêu 100% điểm kỳ dị chia cho 0 và số ma thuật 0.10, dập tắt hiện tượng thổi phồng điểm $raw\_score \approx 0.98$ khi nến hẹp.
    2. Bổ sung chốt chặn sàn cứng bất biến $Z_{\text{vol}} \le 0.0$ tại đầu hàm scan để từ chối ngay lập tức các nến chợ chiều vắng khách.
    3. Khử số ma thuật `.max(0.10)` tại `norm_spread` và dọn dẹp sạch sẽ tàn dư docstring về `vwma`.
  - **Unit Tests:**
    - [x] `test_sat_ratio_damped_prevents_explosion`
    - [x] `test_sat_ratio_damped_rejects_sub_normal_volume`
    - [x] `test_sat_ratio_damped_rewards_genuine_wyckoff_spring`
    - [x] `test_absorption_early_exit_on_sub_zero_volume`
    - [x] `test_absorption_raw_score_not_inflated_on_normal_volume`

- [x] **TODO-AUDIT-NEW-37: Nâng Cấp Kiến Trúc Tách Bạch 3 Tầng & Bộ Nhớ Sự Kiện Vi Mô (`ARCH-DECOUPLED-TIERS`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/` (`matrix.rs`, `gatekeeper.rs`, `arbiter.rs`, `risk.rs`, `scanners/`)
  - **Mô tả:** Tách bạch 3 ranh giới trách nhiệm: (1) Context Tracker theo dõi bối cảnh và bộ nhớ sự kiện vi mô (Spring tại bar $i$, xác nhận tại bar $i+1$); (2) Scanners chỉ phát hiện xác suất hiện tượng và điểm neo cấu trúc; (3) Arbiter & Risk độc quyền phân bổ vốn và co size động theo SL (`Dynamic De-leverage`) thay vì VETO hủy lệnh.

- [x] **TODO-AUDIT-NEW-38: Khôi Phục Neo Khoảng Cách KAMA Trong Trend_PB (`GHOST-PULLBACK`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:47-198`
  - **Mô tả:** [ĐÃ HOÀN THÀNH - KHÔI PHỤC NEO KAMA & SCALE-INVARIANT MTF EQUIV ATR].
    1. Khôi phục biến `delta_kama = (close - kama_mid) / kama_equiv_atr` và hộp khoảng cách an toàn:
       - Long: `delta_kama < -inval_floor || delta_kama > max_dist => None`
       - Short: `delta_kama > inval_floor || delta_kama < -max_dist => None`
    2. Khử triệt để bẫy lệch thang đo đa khung thời gian MTF (`TODO-SEM-02`) bằng công thức tỷ lệ chuẩn:
       $\text{kama\_equiv\_atr} = \text{current\_atr} \times \sqrt{\frac{T_{\text{kama}}}{T_{\text{atr}}}}$
    3. Hồi sinh 2 gen DNA mồ côi `max_pullback_dist_atr` và `invalidation_buffer_atr` trong `TacticalPullbackParams` kết hợp với `f_dist = sat_proximity(delta_kama.abs(), max_dist)` vào hàm tổ hợp lồi `convex_combine(&[f_slope, f_dist, f_stoch, f_clv], &[0.35, 0.25, 0.20, 0.20])`.
    4. Bảo toàn 100% kích thước không gian tìm kiếm `DNA_WIDTH = 110`.
    5. Unit tests: 9/9 tests trong `trend_pb.rs` (bao gồm `test_trend_pb_kama_distance_box_and_mtf_scaling`) và 8/8 tests trong `mod.rs` (`test_trend_pb_*`) PASSED; toàn bộ 145/145 tests thư viện Helios PASSED.

- [x] **TODO-AUDIT-NEW-39: Khắc Phục Lỗi Tê Liệt Stochastic Hook Guard Do Forward-Fill 15m (`STOCH-15M-BLIND`, `TODO-PB-04`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:86, 110-120, 162-172`
  - **Mô tả:** [ĐÃ HOÀN THÀNH - TWO-TIER HOOK GUARD]. Triệt tiêu 100% Vùng Chết [climax, thresh] và Sàn Bão Hòa $K=D=0.0 / 100.0$:
    1. Cưỡng chế chặn đứng khi bão hòa cực đoan ($K \le 1.0 \land D \le 1.0$ cho Long; $K \ge 99.0 \land D \ge 99.0$ cho Short).
    2. Vùng Climax ($K < climax\_thresh$): Đòi hỏi xác nhận kép (không rơi $K \ge prev\_K$ VÀ không nằm dưới $D$: $K \ge D$), khử nến phẳng chúi xuống và chặn 100% Bull Trap.
    3. Vùng Pullback thường ($climax \le K \le stoch\_thresh$): Triệt tiêu hoàn toàn Vùng Chết, chặn đứng khi nến đang rơi dốc ($K < prev$) hoặc nến phẳng đang chúi xuống ($K == prev \land K < D$).
    4. Sanitize `stoch_trend_d` chống `NaN`. Vượt qua 17/17 tests trong `cargo test --lib trend_pb`.

- [ ] **TODO-AUDIT-NEW-40: Ngăn Chặn Đu Đỉnh Kiệt Sức Buying Climax Trong Momentum Ignition (`MOM-CLIMAX-SPAM`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:53-57, 108-126`
  - **Mô tả:** Bắt buộc có giai đoạn nén biến động trước đó (`prior_squeeze`), khử bẫy thưởng điểm cao nhất tại nến kiệt sức ngoài dải BB.

- [x] **TODO-AUDIT-NEW-41: Sửa Nghịch Lý Volume Trong Reversal Scanner (`REVERSAL-VOL-PARADOX`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs:101-107`, `config.rs`, `core_v3/mod.rs`
  - **Mô tả:** Thay thế hàm `sat_proximity` bằng `sat_unbounded(vol_ratio, climax_vol_threshold)`. Loại bỏ nghịch lý phạt 0 điểm khi volume bùng nổ hoảng loạn; chấp nhận volume cao trào là đặc trưng của Selling/Buying Climax; bảo toàn 107 chiều DNA.

- [ ] **TODO-AUDIT-NEW-42: Khắc Phục Các Tử Huyệt Trên Live Engine (`LIVE-SAFETY-PACK`)**
  - **File:** `trading_bot/bot_engine/src/execution/` (`live_engine.rs:133, 206`, `order_executor.rs:249`, `account_manager.rs:175`)
  - **Mô tả:** Kẹp `usable_capital = balance.min(allocated_capital)`, bóc tách `policy.sl_price` để đặt ngay Algo SL khi mở lệnh, xóa bỏ lệnh bán tháo vị thế khi khởi động bot, và đồng bộ `opening_playbook_tag` tránh rỗng tag.

- [ ] **TODO-AUDIT-NEW-43: Bổ Sung Tiền Đề Prior Squeeze & Khử Bẫy Buying Climax Trong Momentum Ignition (`MOM-PRIOR-SQUEEZE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:40-60`
  - **Mô tả:** Bắt buộc có giai đoạn nén biến động `squeeze_intensity >= 0.50` trong $5 \sim 10$ nến trước đó mới cho phép kích hỏa; khử $100\%$ các lệnh mua đuổi khi thị trường đã tăng dốc từ lâu.

- [ ] **TODO-AUDIT-NEW-44: Nhận Diện Cú Bung Đầu Tiên (Breakout Cross-over) & Khử Spam Nến Đa Thanh (`MOM-FIRST-IGNITION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:53-57`
  - **Mô tả:** Chuyển từ kiểm tra tĩnh `close > bb_u` sang phát hiện nến giao cắt bứt phá đầu tiên `close[i] > bb_u[i] && close[i-1] <= bb_u[i-1]`, cấm spam mở lệnh khi giá đã trôi ngoài dải quá 3 nến.

- [ ] **TODO-AUDIT-NEW-45: Hiệu Chuẩn Thang Điểm Gia Tốc & Khử Mẫu Số Cố Định RSI 15.0 (`MOM-SCORE-CALIBRATION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:107-126`
  - **Mô tả:** Thay thế hàm `sat_penetration` (càng bay xa điểm càng cao) bằng hàm đo gia tốc bứt phá ban đầu; hiệu chuẩn lại hàm RSI để không dìm điểm ở chân sóng và không thưởng điểm tối đa ở đỉnh kiệt sức.

- [-] **TODO-AUDIT-NEW-46: Bổ Sung Tầng Đạo Hàm Động Lực Học (Velocity & Acceleration) Trong Matrix (`MAT-KINEMATICS`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/matrix.rs`, `sensors.rs`
  - **Trạng thái:** `[-] (OBSOLETE / REDUNDANT)`
  - **Mô tả:** [ĐÃ ĐÓNG / BỎ QUA - TRÁNH BẪY OVER-ENGINEERING]: Sau khi kiểm định toàn diện 11 tài liệu Bản thể học Hành vi Thị trường (Notes 31-46), xác nhận hệ thống không cần thêm 3 mảng đạo hàm toàn cục vào `MarketMatrixV3`. Mọi hành vi động lực học đã được giải quyết trọn vẹn: Vĩ mô đã có `matrix.kama_slope_atr`, `matrix.bb_middle_slope_adj` (với bước nhảy `k_bars` chống forward-fill) và `matrix.squeeze_intensity`; Vi mô đã được giải quyết trực diện $O(1)$ tại từng Scanner (nến giao cắt đầu tiên `close[i] > bb_u[i] && close[i-1] <= bb_u[i-1]` trong `NEW-44`, `delta_kama` trong `NEW-38`, và `CLV` trong `NEW-34, 51`). Bảo toàn nguyên vẹn kiến trúc 6 trục trực giao tinh gọn của `MarketMatrixV3`.

- [ ] **TODO-AUDIT-NEW-47: Triển Khai Cơ Chế Thoát Lệnh Khi Bứt Phá Xịt (Impulse Stagnation Exit & Decaying TTL) Trong Risk Manager (`MOM-STAGNATION-EXIT`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `mod.rs`
  - **Mô tả:** Nếu sau $N \le 4$ nến kể từ khi kích hoạt Momentum Ignition mà giá không tiếp tục tạo đỉnh cao hơn (Higher High), tự động kích hoạt Stagnation Exit để cắt lỗ vi mô ($1\% \sim 2\%$ ROE) thay vì thả trôi cho dính full Stop Loss cấu trúc.

- [ ] **TODO-AUDIT-NEW-48: Thiết Lập Máy Trạng Thái Giám Sát Sau Kích Hỏa (Post-Trigger State Machine) (`POST-TRIGGER-FSM`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`, `mod.rs`, `risk.rs`
  - **Mô tả:** Chuyển hệ thống sang chế độ giám sát nến xác nhận $t+1$ ngay sau khi kích hỏa: Nếu có Follow-through bar thì kéo ngay SL về hòa vốn; nếu bị từ chối đóng ngược vào dải BB thì kích hoạt Emergency Fade Exit.

- [ ] **TODO-AUDIT-NEW-49: Triển Khai Bộ Nhớ Sự Kiện Chuỗi Hấp Thụ Đa Nến (Multi-bar Absorption Sequence) (`ABS-SEQUENCE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`, `arbiter.rs`
  - **Mô tả:** Tách bạch tiến trình hấp thụ thành chuỗi 3 nến: Nến $t-1$ quét râu (Sweep), Nến $t$ hấp thụ rút chân (Rejection), và Nến $t+1$ kiểm định cạn cung (No-Supply Test). Chấm dứt việc ép toàn bộ chu trình Wyckoff vào đúng 1 cây nến đơn lẻ.

- [ ] **TODO-AUDIT-NEW-50: Khắc Phục Bẫy Vỡ Tường Limit & Nhận Diện Ngưỡng Cản Cấu Trúc Thực Sự (`ABS-STRUCTURAL-WALL`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`, `sensors.rs`
  - **Mô tả:** Nhận diện vùng hấp thụ tại các ngưỡng hỗ trợ/kháng cự cấu trúc thực sự (Prior Swing Lows / Value Area) thay vì dải Bollinger trôi nổi; bổ sung chốt chặn vỡ tường Limit khi lực xả phá vỡ đáy hấp thụ.

- [x] **TODO-AUDIT-NEW-51: Triển Khai Chuỗi Xác Nhận Đảo Chiều Đa Nến & Cổng Kiểm Tra R:R Tối Thiểu (`REV-SEQUENCE-RR`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`, `config.rs`, `core_v3/mod.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH]. Kết nối `matrix` để đọc độ dốc KAMA và Chaos Index (chặn bắt dao rơi khi KAMA dốc $<-0.40$ hoặc bão Whipsaw $>0.65$); bổ sung bộ xác nhận đa nến Wyckoff Spring ($i$ vs $i-1$) trên nến 1m nguyên bản (nến xanh hấp thụ và giữ vững đáy); áp dụng cổng kiểm tra tỷ lệ kỳ vọng toán học $\frac{\text{TP Dist}}{\text{SL Dist}} \ge 1.20$; bảo toàn 107 chiều DNA; bộ test 11/11 tests pass 100%.

- [ ] **TODO-AUDIT-NEW-52: Triển Khai Bộ Nhận Diện Cấu Trúc Đỉnh/Đáy Bậc Thang Dow & Bảo Vệ Đáy Sóng Cũ (`TREND-DOW-STRUCTURE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`, `sensors.rs`
  - **Mô tả:** Bổ sung cơ chế theo dõi cấu trúc đỉnh/đáy bậc thang (HH/HL cho Uptrend, LH/LL cho Downtrend); cấm mở lệnh nếu nhịp hồi đã đâm thủng đáy Swing Low trước đó; bổ sung nến xác nhận tiếp diễn $t+1$.

---

### PHẦN 13: KIỂM TOÁN HẠ TẦNG CỐT LÕI (CORE_V3/MOD, CONFIG & DNA MAPPING - NOTES 47, 48)

- [x] **TODO-AUDIT-NEW-53: Đồng Bộ Tái Tính Toán Sensors Khi Nhận DNA Hot-Swap (`H-MOD-01`, `H-CFG-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:67-79`, `config.rs:24-70`
  - **Mô tả:** [ĐÃ HOÀN THÀNH]. Đã xác minh bản chất: `HeliosBridge::evaluate_live_tick` gọi `init()` trên mỗi nến 1m mới nên `sensors` luôn được tính lại theo DNA mới trong thực chiến. Đã sửa lỗi gọi `MarketMatrixV3::from_sensors` vô điều kiện khi `sensors` rỗng lúc Strategy mới khởi tạo (trong `ferro_trade::run_with_dna`) bằng guard `if !self.sensors.closes.is_empty()`. Việc tinh giản các gen indicator cố định thuộc scope của `TODO-AUDIT-NEW-63`.
  - **Unit Tests:**
    - [x] `test_update_from_dna_safe_on_empty_sensors`
    - [x] `test_update_from_dna_updates_matrix_when_sensors_present`

- [ ] **TODO-AUDIT-NEW-54: Loại Bỏ Lệnh Gọi Cầu Dao Đánh Giá Lặp Lại Tại `mod.rs` (`H-MOD-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:241`, `arbiter.rs:168-173`
  - **Mô tả:** Xóa bỏ lệnh gọi `RegimeGatekeeper::evaluate` thừa thãi tại `mod.rs:241` khi có defense_ledger, thống nhất quyền đánh giá duy nhất bên trong `arbiter.process_tick()`.

- [x] **TODO-AUDIT-NEW-55: Chuyển Đổi Veto Structural SL Sang Dynamic De-leverage Co Size Động (`H-MOD-03`, `H-MOD-04`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:277-283, 332-370`, `risk.rs:156-189`
  - **Mô tả:** Thay thế công thức tính size cố định theo vốn $\text{Size} = \frac{\text{Margin} \times L}{P}$ bằng công thức chuẩn quant $\text{Size} = \frac{\text{Risk USD}}{\text{Distance SL}}$; loại bỏ mã chết kẹp SL 15% ROE và kích hoạt lại khối lệnh an toàn.

- [x] **TODO-AUDIT-NEW-56: Chuẩn Hóa Lát Cắt Tính Slope BB Middle Trong `init()` (`H-MOD-05`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:181-200`
  - **Mô tả:** Chuyển phép tính `bb_middle_slope_adj` sang sử dụng ATR cùng khung thời gian MTF của dải Bollinger (15m/1h) thay vì dùng 1m ATR tạo bậc thang giả.

- [ ] **TODO-AUDIT-NEW-57: Dọn Dẹp Biến Ma & Khử Lỗi Silent Abort Trong `mod.rs` (`H-MOD-06`, `H-MOD-07`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:59, 87-94, 226, 232`
  - **Mô tả:** Xóa biến `prev_open_trade` ghi không đọc; chuyển `println!` silent abort khi period = 0 thành `panic!` hoặc gán fallback an toàn tránh bot chạy 0 lệnh trong im lặng.

- [ ] **TODO-AUDIT-NEW-58: Sửa Lỗi Toán Học Discrete Binning Lệch Xác Suất Trong `derive_dna_mappable` (`H-CFG-03`)**
  - **File:** `ferro_trade/ferro_trade_derive/src/lib.rs:172, 181, 187`
  - **Mô tả:** Sửa phép tính `.round() as usize` đang làm hai lựa chọn đầu mút chỉ có 50% xác suất so với phần tử ở giữa; chuyển sang phân phối rời rạc đều `Uniform Discrete Binning`.

- [ ] **TODO-AUDIT-NEW-59: Bổ Sung `#[serde(default)]` Cấp Struct Con Cho Toàn Bộ Config (`H-CFG-04`, `TODO-CFG-08`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:23, 392, 537, 612, 750, 852, 924, 1011`
  - **Mô tả:** Bổ sung `#[serde(default)]` cho tất cả struct con để chống lỗi parse JSON hoặc panic crash khi nhận cấu hình thiếu trường từ UI client.

- [x] **TODO-AUDIT-NEW-60: Thiết Lập Dải Phân Tầng Trailing Stop Thứ Bậc Bất Biến (`H-CFG-05`, `TODO-CFG-03`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:1223-1234, 1326-1338`, `mod.rs:75-85`, `trading_bot/bot_engine/src/controller/asset_actor.rs:485`
  - **Mô tả:** Mở rộng dải tìm kiếm DNA toàn năng $[0.40 \to 6.00\ \text{ATR}]$ (step 0.10, 57 nấc đều) cho cả 3 gen `tsl_atr_tight`, `tsl_atr_normal`, `tsl_atr_loose`; thiết lập phương thức khóa bất biến thứ bậc `enforce_tsl_hierarchy()` sắp xếp $O(1)$ (`tight <= normal <= loose`) tại các cổng nạp DNA và khởi tạo, triệt tiêu hoàn toàn nghịch lý GA siết chặt siêu sóng và thả rông thị trường range.
  - **Unit Tests:**
    - [x] `test_dna_width_and_parameter_defs_integrity` (khóa dải $[0.40, 6.00]$ và kiểm tra chia hết step cho cả 3 gen)
    - [x] `test_tsl_hierarchy_enforcement_prevents_inversion` (kiểm tra hoán vị nghịch lý tại struct và qua `update_from_dna`)

- [x] **TODO-AUDIT-NEW-61: Mở Rộng Dải Sàn Biến Động `min_natr_threshold` Cho Đa Tài Sản (`H-CFG-06`, `TODO-CFG-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:497-503`
  - **Mô tả:** Mở rộng dải DNA $[0.0001, 0.0100]$ (step 0.0002) để không VETO oan các cặp Forex biến động thấp và lọc sạch nhiễu của Altcoins/Memecoins biến động cao.
  - **✅ Đã chốt & triển khai:** `#[dna(min = 0.0002, max = 0.0300, step = 0.0002)]` — 150 nấc đều ($0.02\% \to 3.00\%$). Note sơ bộ $[0.0001, 0.0100]$ step 0.0002 bị loại vì $0.0099/0.0002 = 49.5$ bước (lệch biên xác suất) và trần 1.00% chưa phủ Memecoin. Giữ nguyên default `0.0012`. `gatekeeper.rs` không cần sửa vì sàn phí `roundtrip_fee_pct` là chốt chặn độc lập.
  - **Unit Tests:**
    - [x] `test_gatekeeper_permits_forex_low_volatility` (kèm đối chứng dải cũ 0.0008 → MacroFlatline)
    - [x] `test_gatekeeper_filters_memecoin_high_noise` (kèm đối chứng NATR 2.00% → PASS)
    - [x] `test_dna_width_and_parameter_defs_integrity` (khóa dải + kiểm tra chia hết step)

- [ ] **TODO-AUDIT-NEW-62: Dọn Dẹp Dead Fields, Biến Bù Nhìn & Struct Mồ Côi Trong `config.rs` (`H-CFG-07`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:461-477, 1090, 1338-1347`
  - **Mô tả:** Xóa 7 dead fields trong `ArbiterConfig`, xóa struct mồ côi `WarmupConfig`, và nối dây logic thực thi cho `mega_trend_max_cycles` trong `risk.rs`.

- [ ] **TODO-AUDIT-NEW-63: Tinh Giản Không Gian DNA Về 2 Scanners Cốt Lõi (~36 Chiều) (`H-CFG-01`, `H-CFG-09`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs`, `core_v3/mod.rs`
  - **Mô tả:** Tắt gen DNA của Momentum và Reversal (chỉ để lại Trend_PB và Absorption); loại bỏ các gen cảm biến dư thừa, hạ số chiều DNA từ 102 về ~36 chiều để GA hội tụ siêu tốc và chống overfitting.

---

### PHẦN 14: KIỂM TOÁN ĐỘNG CƠ BACKTEST FERRO_TRADE (BROKER & ENGINE - NOTE 49)

- [ ] **TODO-AUDIT-NEW-64: Triển Khai Type-Safe `OrderSize` Enum Thay Thế Ép Kiểu Ngầm `(-1.0..1.0)` (`FT-BRK-01`, `TODO-MOD-04`)**
  - **File:** `ferro_trade/src/broker/broker.rs:477-494`, `order.rs`, `actions.rs`
  - **Mô tả:** Thay thế số `f64` trần trụi và bẫy khoảng ma thuật `size \in (-1.0, 1.0)` bằng enum `OrderSize { MarginFraction(f64), BaseUnits(f64), NotionalUsd(f64) }`, triệt tiêu hoàn toàn lỗi nhầm lẫn giữa % vốn và khối lượng coin khi giao dịch Bitcoin/Altcoin.

- [ ] **TODO-AUDIT-NEW-65: Sửa Lỗi Đóng Lệnh Cuối Kỳ Finalize Trades Tại Giá `Close` Thay Vì `Open` (`FT-ENG-01`)**
  - **File:** `ferro_trade/src/backtest/engine.rs:378-386`
  - **Mô tả:** Đóng trực tiếp các vị thế mở cuối kỳ tại giá `close_at(final_bar_index)` thay vì gọi lại `broker.next()` khớp tại `Open`, bảo toàn trọn vẹn PnL của cây nến kết thúc và không ghi đè méo mó `equity_curve`.

- [ ] **TODO-AUDIT-NEW-66: Dự Phòng Phí Sàn 2 Chiều Trong Tính Toán Ký Quỹ Khả Dụng (`FT-BRK-02`)**
  - **File:** `ferro_trade/src/broker/broker.rs:474-477, 517-523`
  - **Mô tả:** Trích lập dự phòng cả phí mở lệnh (Entry) và phí đóng lệnh tiềm năng (Exit Taker Fee) khi kiểm tra `required_margin <= available_margin`, ngăn chặn tài khoản bị âm vốn ký quỹ tức thì sau khi mở vị thế ở đòn bẩy cao.

- [ ] **TODO-AUDIT-NEW-67: Chuẩn Hóa Khớp Lệnh Đảo Chiều Vị Thế (Flip Orders Dưới `hedging = false`) (`FT-BRK-04`)**
  - **File:** `ferro_trade/src/broker/broker.rs:497-515`
  - **Mô tả:** Hợp nhất mức giá thực thi và spread cho cả phần đóng vị thế cũ và phần mở vị thế mới ngược chiều, đảm bảo tính nhất quán $1:1$ với hành vi khớp lệnh Market Flip trên sàn Binance Futures.

- [ ] **TODO-AUDIT-NEW-68: Tích Hợp Early Termination Penalty Vào Báo Cáo Hiệu Suất `Stats` (`FT-ENG-03`)**
  - **File:** `ferro_trade/src/backtest/engine.rs:275-279, 345-356`
  - **Mô tả:** Tự động trừ điểm phạt rủi ro vào `stats` khi vòng lặp bị ngắt sớm (Early Termination), giúp backtest đơn lẻ phản ánh trung thực mức độ vi phạm sụt giảm vốn thay vì chỉ có tác dụng trong GA Optimizer.

- [ ] **TODO-AUDIT-NEW-69: Bổ Sung Guard `margin_rate <= 0.0` Chống Tràn Vô Hạn Leverage (`FT-BRK-05`)**
  - **File:** `ferro_trade/src/backtest/engine.rs:216`, `broker.rs:66`
  - **Mô tả:** Thêm guard kẹp sàn an toàn nếu `margin_rate <= 0.0` thì fallback về `margin_rate = 1.0` (tài khoản Spot không đòn bẩy), ngăn ngừa `leverage = f64::INFINITY` làm tê liệt logic kiểm tra ký quỹ.

- [ ] **TODO-AUDIT-NEW-70: Kẹp Trần `size_to_close` Chống Panic Crash Trong `close_trade` (`FT-BRK-06`)**
  - **File:** `ferro_trade/src/broker/broker.rs:236-240, 674`
  - **Mô tả:** Thêm `.min(self.trades[idx].size.abs())` khi tính `size_to_close` để ngăn ngừa trường hợp `portion_size.max(min_unit)` vượt quá khối lượng vị thế thực tế, gây `assert!` fail và panic crash toàn bộ tiến trình tối ưu hóa GA.

- [ ] **TODO-AUDIT-NEW-71: Chuyển Tiếp Tham Số `portion` Trong `Action::CloseLongs` và `CloseShorts` (`FT-BRK-07`)**
  - **File:** `ferro_trade/src/broker/broker.rs:771-782`
  - **Mô tả:** Sửa lỗi bỏ qua `portion` (luôn đóng 1.0), cho phép chiến lược thực hiện chốt lời từng phần (Partial Take Profit / Scale-out) chính xác theo tỷ lệ phần trăm được cấu hình.

---

### PHẦN 15: KIỂM TOÁN TẦNG THỰC THI SÀN THẬT (LIVE ENGINE & ORDER EXECUTOR - NOTE 50)

- [ ] **TODO-AUDIT-NEW-72: Khắc Phục Bẫy Tràn Vốn Thảm Họa `usable_capital` Lấy Nhầm `available_balance` (`TB-LIVE-01`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:133-143`, `order_executor.rs:284-285`
  - **Mô tả:** Khóa cứng trần vốn `usable_capital = self.core.allocated_capital.min(self.account_state.available_balance)`, chấm dứt việc nuốt trọn 100% số dư toàn bộ tài khoản ví Binance khi mở lệnh trên 1 cặp tiền duy nhất.

- [ ] **TODO-AUDIT-NEW-73: Đồng Bộ Trạng Thái SL/TP Lên `LivePositionState` Chống Spam Lệnh Dời SL (`TB-LIVE-02`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:198-207`, `order_executor.rs:58-65`
  - **Mô tả:** Nối dây đồng bộ `current_sl_price` từ `OrderExecutor::active_sl_prices` và `OrderTradeUpdate` vào `live_position`, ngăn ngừa Strategy bị mù SL sàn và phát tín hiệu dời SL liên tục mỗi phút.

- [ ] **TODO-AUDIT-NEW-74: Bảo Toàn Tag Playbook Khi Khởi Tạo Vị Thế Sàn (`TB-LIVE-03`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:206`, `order_executor.rs:340-350`
  - **Mô tả:** Lưu trữ và phục hồi `opening_playbook_tag` (TrendPB vs Absorption) vào SQLite khi mở lệnh, đảm bảo RiskManager áp dụng đúng quy tắc Trailing SL riêng của từng playbook.

- [ ] **TODO-AUDIT-NEW-75: Đồng Bộ `live_entry_bar` Khớp Đúng Bar Index Của Nến Mở Lệnh (`TB-LIVE-04`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:203`, `controller/asset_actor.rs:280-310`
  - **Mô tả:** Ghi nhận chính xác `live_entry_bar` theo `current_bar_idx` thực tế thay vì gán cứng 0, triệt tiêu lỗi `bars_held = 43,100` kích hoạt Time-based Exit sớm vô lý.

- [ ] **TODO-AUDIT-NEW-76: Timeout Watchdog Cho Cờ Lệnh `is_submitting_order` Chống Deadlock Khi WS Trễ (`TB-LIVE-05`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:140-160`
  - **Mô tả:** Bổ sung cơ chế Timeout Watchdog 5,000ms: Tự động giải phóng cờ `is_submitting_order = false` và thực hiện REST reconciliation nếu kết nối WebSocket bị rớt hoặc chậm trễ bản tin.

- [ ] **TODO-AUDIT-NEW-77: Triển Khai Atomic SL Replacement Triệt Tiêu Naked SL Gap 150ms (`TB-ORD-01`, `TODO-RSK-05`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:190-235`
  - **Mô tả:** Chuyển sang cơ chế đặt lệnh SL mới thành công trước, sau đó mới hủy SL cũ theo `algo_id` cụ thể, loại bỏ hoàn toàn khoảng trống 150-250ms vị thế bị mồ côi không có SL trên sàn Binance.

- [ ] **TODO-AUDIT-NEW-78: Tự Động Đặt Lệnh Algo Take Profit Lên Binance Futures (`TB-ORD-02`, `TODO-EXEC-04`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:243-260`
  - **Mô tả:** Hiện thực hóa việc gửi lệnh Algo `TAKE_PROFIT_MARKET` (`close_position: true`) lên sàn Binance Futures ngay khi vào lệnh hoặc khi Strategy cập nhật TP, chốt lời kịp thời các cú giật râu nến chớp nhoáng.

- [ ] **TODO-AUDIT-NEW-79: Hủy Algo Order Theo ID Cụ Thể Thay Vì Hủy Toàn Bộ Symbol (`TB-ORD-03`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:192, 398`
  - **Mô tả:** Thay thế `cancel_all_algo_open_orders` bằng `cancel_algo_order` theo `algo_id` cụ thể, tránh xóa nhầm các lệnh chờ độc lập khác trên cùng cặp tiền.

- [ ] **TODO-AUDIT-NEW-80: Đọc Min Notional Động Từ `MarketFilter` Thay Vì Gán Cứng 5.0 USDT (`TB-ORD-04`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:141`
  - **Mô tả:** Sử dụng `self.market_filter.min_notional` thay cho con số ma thuật 5.0 USDT, đảm bảo thích ứng với mọi quy định ký quỹ tối thiểu của từng cặp tiền.

- [ ] **TODO-AUDIT-NEW-81: Chống Nhân Đôi Khối Lượng Khi Đảo Chiều Vị Thế (Flip Order Under Stale State) (`TB-ORD-05`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:288-305`
  - **Mô tả:** Kiểm tra đối soát lại vị thế thực tế trước khi gửi lệnh Flip tổng hợp, ngăn ngừa việc gửi lệnh gấp đôi khối lượng nếu vị thế cũ đã bị thanh lý/cắt lỗ ngoài sàn.

- [ ] **TODO-AUDIT-NEW-82: Lưu Trữ `active_sl_prices` Xuống SQLite Chống Mất Dữ Liệu Khi Restart Bot (`TB-ORD-06`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:31, 58-70`, `data_engine/sqlite_impl.rs`
  - **Mô tả:** Đồng bộ `active_sl_prices` vào bảng SQLite để khi bot restart, Monotonic Ratchet Guard có thể khôi phục ngay mức SL đang chạy trên sàn, chống việc kéo lùi SL sau khởi động lại.

- [ ] **TODO-AUDIT-NEW-83: Cô Lập Đường Cong Vốn Đa Cặp Tiền Trong `get_equity()` & Validate Float (`TB-LIVE-07`, `TB-ORD-07`)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:169-173`, `order_executor.rs:149-160`
  - **Mô tả:** Tách bạch vốn `equity` theo từng cặp tiền thay vì lấy chung `wallet_balance` toàn ví; xử lý an toàn `price.is_finite()` thay vì gọi `.unwrap()` gây panic crash trong `format_price_decimal`.

---

### PHẦN 16: KIỂM TOÁN TẦNG CƠ SỞ DỮ LIỆU SQLITE & DATA ENGINE (NOTE 51)

- [ ] **TODO-AUDIT-NEW-84: Tái Cấu Trúc `SqliteStore` Sang Asynchronous Connection Pool (`TB-DB-01`, `TODO-AUDIT-TB-01`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:10-28`
  - **Mô tả:** Thay thế `Arc<Mutex<Connection>>` đơn bằng mô hình Connection Pool (1 Writer Connection + N Reader Connections), giải phóng hoàn toàn tiềm năng đọc đồng thời của SQLite WAL Mode.

- [ ] **TODO-AUDIT-NEW-85: Bọc Toàn Bộ Thao Tác Disk I/O Trong `spawn_blocking` Chống Treo Worker Tokio (`TB-DB-02`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:94-238`
  - **Mô tả:** Chuyển toàn bộ các lệnh gọi blocking I/O (query nến, commit transaction) vào `tokio::task::spawn_blocking`, bảo vệ luồng worker của Tokio không bị nghẽn làm rớt mạng WebSocket.

- [ ] **TODO-AUDIT-NEW-86: Bổ Sung B-Tree Index Trên `open_time` và `trade_history` Triệt Tiêu Full Table Scan (`TB-DB-03`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:31-43, 62-71`
  - **Mô tả:** Thêm index `idx_candles_open_time` trên bảng `candles_1m` để tối ưu hóa `cleanup_old_data` từ $O(N)$ về $O(\log N)$; thêm index `idx_trade_history_symbol_time` cho bảng lịch sử lệnh.

- [ ] **TODO-AUDIT-NEW-87: Xóa Bỏ `unwrap_or_default()` Trong `get_best_dna`, Chống Trả Về DNA Rỗng (`TB-DB-04`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:278-286`
  - **Mô tả:** Thay thế `.unwrap_or_default()` bằng việc trả về `Err(BotError::DatabaseError)` rõ ràng khi parse JSON thất bại, ngăn chặn việc nạp vector DNA rỗng `[]` làm sập chiến lược Helios.

- [ ] **TODO-AUDIT-NEW-88: Triển Khai Bảng `schema_version` & Migration Tự Động (`TB-DB-05`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:30-71`
  - **Mô tả:** Bổ sung cơ chế kiểm soát phiên bản schema và chạy script migration tự động khi nâng cấp cấu trúc bảng CSDL, chống lỗi runtime `no such column`.

- [ ] **TODO-AUDIT-NEW-89: Dọn Dẹp Mã Mồ Côi `duckdb_impl.rs` Khỏi Codebase (`TB-DB-06`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/duckdb_impl.rs`
  - **Mô tả:** Xóa bỏ file `duckdb_impl.rs` không sử dụng và loại bỏ dependency `duckdb` khỏi `Cargo.toml` để tối ưu hóa kích thước binary và giảm thời gian biên dịch.

- [ ] **TODO-AUDIT-NEW-90: Tách CSDL Theo Cặp Tiền Hoặc Kênh MPSC Writer Riêng Biệt (`TB-DB-07`)**
  - **File:** `trading_bot/bot_engine/src/main.rs:55`, `sqlite_impl.rs`
  - **Mô tả:** Tách file lưu trữ nến theo từng symbol (`storage/candles_{symbol}.db`) hoặc thiết lập kênh MPSC chuyên dụng để ghi nến bất đồng bộ, triệt tiêu tranh chấp khóa giữa các Actor.

- [ ] **TODO-AUDIT-NEW-91: Mở Rộng Bảng `trade_history` Đầy Đủ PnL, Phí Sàn, Playbook Tag và Exit Reason (`TB-DB-08`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:62-71`
  - **Mô tả:** Bổ sung các cột `realized_pnl`, `commission`, `opening_playbook_tag`, `exit_reason` vào bảng `trade_history` để cung cấp dữ liệu toàn diện cho giao diện Desktop UI.

- [ ] **TODO-AUDIT-NEW-92: Nâng Cấp Bộ Kiểm Tra Tính Toàn Vẹn Chuỗi Thời Gian Chống Khoảng Trống Nến (`TB-DB-09`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:212-227`, `initial_sync.rs`
  - **Mô tả:** Thay thế phép đếm thô `SELECT COUNT(*)` bằng thuật toán quét khoảng trống liên tục (Continuity Gap Check), ngăn ngừa bot giao dịch trên chuỗi nến bị đứt gãy.

- [ ] **TODO-AUDIT-NEW-93: Tự Động Tạo Thư Mục Cha Trong `SqliteStore::new` Chống Lỗi CantOpen (`TB-DB-10`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:16-28`
  - **Mô tả:** Tự động gọi `std::fs::create_dir_all(parent)` trước khi mở file CSDL SQLite, tránh lỗi IO crash khi đường dẫn thư mục tùy biến chưa được tạo trước.

---

## 🏛️ PHẦN 17: KIỂM TOÁN HẠ TẦNG THỐNG ĐỐC QUẢN TRỊ RỦI RO DANH MỤC (`portfolio_guardian.rs`)
> *Nguồn tài liệu:* [`52_missing_portfolio_guardian_and_multi_asset_risk_flaws.md`](notes/52_missing_portfolio_guardian_and_multi_asset_risk_flaws.md)

- [ ] **TODO-AUDIT-NEW-94: Kiến Thiết Module `portfolio_guardian.rs` Và Giao Thức Handshake Xin Phép Mở Lệnh (`TB-PORT-01`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs` (Tạo mới), `asset_actor.rs:630-672`
  - **Mô tả:** Xây dựng module Thống Đốc Quản Trị Danh Mục trung tâm (`PortfolioGuardian`) với giao thức Two-Way Handshake (`request_order_permit`), khống chế tổng đòn bẩy danh mục không vượt quá ngưỡng an toàn (ví dụ tối đa $3.0\text{x}$ vốn tổng thay vì bùng nổ $80\text{x} - 200\text{x}$).

- [ ] **TODO-AUDIT-NEW-95: Khống Chế Giới Hạn Số Vị Thế Mở Đồng Thời Toàn Hệ Thống (`TB-PORT-02`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`, `asset_actor.rs`
  - **Mô tả:** Thiết lập chốt chặn `max_concurrent_positions` (ví dụ tối đa 2 hoặc 3 vị thế mở song song), từ chối cấp phép khi các Actor khác đã chiếm dụng hết hạn ngạch vị thế, tránh dàn mỏng vốn trên nhiều cặp Altcoins.

- [ ] **TODO-AUDIT-NEW-96: Tích Hợp Bộ Lọc Tương Quan Xu Hướng Vĩ Mô BTC (`TB-PORT-03`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`
  - **Mô tả:** Giám sát trạng thái xu hướng và biến động vĩ mô của Bitcoin (`BTCUSDT` 4h/1D); tự động VETO các yêu cầu mở vị thế Long trên Altcoins khi Bitcoin đang sập mạnh hoặc bước vào vùng hỗn loạn cực đoan.

- [ ] **TODO-AUDIT-NEW-97: Triển Khai Cầu Dao Ngắt Mạch Danh Mục Toàn Cục Hàng Ngày (`TB-PORT-04`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`
  - **Mô tả:** Theo dõi tỷ lệ sụt giảm vốn tổng thể tài khoản trong ngày (`Daily Portfolio Drawdown`); tự động kích hoạt `EmergencyCmd::PanicCloseAll` và phong tỏa mở lệnh mới 24h nếu tổng tài khoản bị lỗ vượt trần cho phép (ví dụ $-5.0\%$).

- [ ] **TODO-AUDIT-NEW-98: Hàng Đợi Ký Quỹ & Điều Phối Lệnh Chống Tranh Chấp Ví Sàn (`TB-PORT-05`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`
  - **Mô tả:** Quản lý hạn ngạch ký quỹ khả dụng tập trung, xếp hàng điều phối tuần tự các yêu cầu mở lệnh vào giây $00$ của nến đóng, triệt tiêu lỗi sàn Binance `-2019 Margin Insufficient`.

- [ ] **TODO-AUDIT-NEW-99: Cơ Chế Cấp Vốn & Tái Cân Bằng Động Dựa Trên Hiệu Suất (`TB-PORT-06`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`, `asset_actor.rs`
  - **Mô tả:** Thay thế mức vốn phân bổ tĩnh ($60) bằng mô hình phân bổ vốn động: Tăng tỷ trọng vốn cho các cặp tiền đang có Profit Factor cao / xu hướng mạnh và giảm thiểu vốn ở các cặp tiền đang chịu chuỗi thua liên tiếp.

- [ ] **TODO-AUDIT-NEW-100: Bộ Giám Sát Sức Khỏe Toàn Diện & Phản Xạ Thiên Nga Đen Tự Động (`TB-PORT-07`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`
  - **Mô tả:** Tự động giám sát độ trễ mạng API, trạng thái tài khoản sàn và độ biến động đột biến toàn thị trường; tự động đưa hệ thống về trạng thái phòng thủ (Flat All & Exit) khi xảy ra sự cố thiên nga đen hoặc sàn ngắt kết nối.

- [ ] **TODO-AUDIT-NEW-101: Lưu Trữ Bền Vững Trạng Thái PnL Trong Ngày Tránh Mất Bộ Nhớ Sau Restart (`TB-PORT-08`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`, `sqlite_impl.rs`
  - **Mô tả:** Lưu trạng thái vốn đầu ngày (`day_start_equity`) và khoản lỗ đã chốt (`daily_realized_loss`) theo ngày UTC vào bảng CSDL `portfolio_daily_stats`, chống hiện tượng reset hạn mức lỗ sau khi bot restart.

- [ ] **TODO-AUDIT-NEW-102: Cơ Chế Check-And-Reserve Ký Quỹ Nguyên Tử Chống TOC/TOU (`TB-PORT-09`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`
  - **Mô tả:** Triển khai cơ chế khóa nguyên tử (Atomic Check-and-Reserve) khi cấp hạn mức ký quỹ, đảm bảo số dư khả dụng bị trừ ngay khi cấp phép, ngăn chặn các Actor cùng lúc chiếm dụng vượt trần đòn bẩy.

- [ ] **TODO-AUDIT-NEW-103: Chốt Chặn Khống Chế Lệch Pha Định Hướng Toàn Danh Mục (`TB-PORT-10`)**
  - **File:** `trading_bot/bot_engine/src/controller/portfolio_guardian.rs`
  - **Mô tả:** Theo dõi Net Notional Delta danh mục; tự động từ chối các lệnh cùng chiều khi tỷ lệ thiên lệch Long/Short vượt quá ngưỡng an toàn (ví dụ: $|\text{Net Delta}| > 60\%$), chống biến danh mục thành 1 vị thế cược thị trường khổng lồ.

---

## 🏛️ PHẦN 18: KIỂM TOÁN HẠ TẦNG ĐỘNG CƠ KHỚP LỆNH GIẢ LẬP (`paper_engine.rs` & `paper_trader.rs`)
> *Nguồn tài liệu:* [`53_paper_execution_engine_and_paper_trader_flaws.md`](notes/53_paper_execution_engine_and_paper_trader_flaws.md)

- [x] **TODO-AUDIT-NEW-104: Đồng Bộ Hóa PnL Thả Nổi Vào `PaperExecutionEngine::get_equity()` (`TB-PPR-01`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_engine.rs:14-38, 56-61, 115-136, 185-275`
  - **Mô tả:** Lưu trữ `last_close` trong `execute_action_ctx` và `on_candle_ctx`, đồng bộ hóa vốn Marked-to-Market (`get_total_equity(last_close)`) có kẹp sàn Isolated Margin (`.max(0.0)`) và cơ chế fallback an toàn khi giá không hợp lệ (NaN, <= 0), kèm unit test suite 100% pass.

- [ ] **TODO-AUDIT-NEW-105: Triển Khai Liquidation Watchdog Mô Phỏng Cháy Tài Khoản (`TB-PPR-02`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:477-575`
  - **Mô tả:** Tự động kích hoạt cơ chế cưỡng bức thanh lý khi khoản lỗ thả nổi chạm trần $\ge 90\%$ tiền ký quỹ, loại bỏ ảo tưởng số dư vốn âm không thực tế trong Paper Trading.

- [ ] **TODO-AUDIT-NEW-106: Tích Hợp `MarketFilter` Làm Tròn Step Size & Kiểm Tra Min Notional (`TB-PPR-03`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:440-450`, `paper_engine.rs:42-60`
  - **Mô tả:** Làm tròn khối lượng vị thế theo `step_size` chuẩn sàn và từ chối các lệnh có giá trị danh nghĩa dưới \$21 (`min_notional`), đảm bảo 100% lệnh Paper tương thích hoàn hảo với Binance API.

- [~] **TODO-AUDIT-NEW-107: Bổ Sung Mô Hình Trượt Giá & Chênh Lệch Spread Cho Lệnh Market (`TB-PPR-04`) [PENDING - TẠM GÁC]**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:450`
  - **Mô tả:** Trừ hao trượt giá thực tế ($0.05\% - 0.10\%$ giá vào lệnh) đối với lệnh Market. *(Hiện trạng: Tạm hoãn đánh giá để tránh áp đặt số phạt tĩnh chủ quan làm lệch pha 1:1 với backtest khi chưa có luồng dữ liệu WebSocket Orderbook Depth L2, tập trung chẩn đoán sự ổn định hệ thống trước).*

- [ ] **TODO-AUDIT-NEW-108: Sửa Sai Lệch Telemetry EventBus: Cập Nhật `available_balance` & `unrealized_profit` (`TB-PPR-05`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:179-188`
  - **Mô tả:** Khấu trừ tiền ký quỹ chiếm dụng khỏi `available_balance` và cập nhật đúng `unrealized_profit` khi phát `AccountEvent`, cung cấp dữ liệu trung thực cho Desktop UI Dashboard.

- [ ] **TODO-AUDIT-NEW-109: Mô Phỏng Khấu Trừ Phí Funding Rate Chu Kỳ 8 Giờ UTC (`TB-PPR-06`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:477-575`
  - **Mô tả:** Khấu trừ phí Funding Rate danh nghĩa ($0.01\%$) khi vị thế giữ lệnh bước qua các mốc 00:00, 08:00, 16:00 UTC, phản ánh chính xác chi phí giữ lệnh dài hạn của Perpetual Futures.

- [ ] **TODO-AUDIT-NEW-110: Khắc Phục Lỗi Gán Cứng `bar_idx = 0` Trong `execute_action()` (`TB-PPR-07`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_engine.rs:94-97`
  - **Mô tả:** Truyền đúng chỉ số nến hiện tại hoặc đồng bộ từ `last_candle_idx` thay vì gán cứng 0, bảo đảm tính toán chính xác chỉ số số nến giữ lệnh (`bars_held`).

- [ ] **TODO-AUDIT-NEW-111: Giới Hạn Bộ Nhớ Đệm Lịch Sử Lệnh Qua RingBuffer Chống Rò Rỉ Heap (`TB-PPR-08`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:85`, `paper_engine.rs:152-154`
  - **Mô tả:** Giới hạn `closed_trades` trong RAM (ví dụ tối đa 500 lệnh gần nhất) và tránh clone toàn bộ vector trên heap khi truy vấn, bảo vệ bot vận hành bền bỉ 24/7.

- [ ] **TODO-AUDIT-NEW-112: Cơ Chế Khống Chế Trần Lãi Kép Bảo Vệ Vốn Sàn (`TB-PPR-09`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs:441`
  - **Mô tả:** Thiết lập trần lãi kép (Compounding Ceiling Cap) để bảo vệ lợi nhuận tích lũy của các chuỗi thắng, ngăn chặn việc 2 lệnh thua cuốn bay sạch toàn bộ thành quả.

- [ ] **TODO-AUDIT-NEW-113: Tương Thích Hóa `on_account_event` Tiếp Nhận Lệnh Điều Phối Danh Mục (`TB-PPR-10`)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_engine.rs:132-134`
  - **Mô tả:** Cho phép PaperEngine tiếp nhận các tín hiệu tái cân bằng vốn và lệnh ngắt mạch khẩn cấp từ Portfolio Guardian thay vì cô lập hoàn toàn.

---

## 🏛️ PHẦN 19: KIỂM TOÁN HẠ TẦNG CỔNG VÀO GATEKEEPER & QUẢN TRỊ RỦI RO CHIẾN THUẬT (`gatekeeper.rs` & `risk.rs`)
> *Nguồn tài liệu:* [`54_risk_and_gatekeeper_architectural_flaws.md`](notes/54_risk_and_gatekeeper_architectural_flaws.md)

- [ ] **TODO-AUDIT-NEW-114: Nâng Cấp Phân Luồng Gatekeeper Cho Phép RangeOsc/Absorption Hoạt Động Khi Nén Biến Động (`H-GATE-01`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs:140-152`
  - **Mô tả:** Thay thế cơ chế VETO cào bằng toàn bộ Playbook khi `VolatilityFlatline` bằng cơ chế phân luồng chọn lọc: Chỉ tạm khóa các Playbook theo đà (`Trend_PB`, `Momentum_Ign`), đồng thời giải phóng quyền giao dịch cho `Range_Osc` và `Absorption` trong vùng tích lũy.

- [x] **TODO-AUDIT-NEW-115: Kiến Thiết Bộ Phân Loại Chế Độ Thị Trường Vĩ Mô Macro Regime Classifier (`H-GATE-02`)**
  - **Tài liệu thiết kế:** [`03_technical_spec_macro_regime_classifier.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/03_technical_spec_macro_regime_classifier.md)
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs:48-67`, `arbiter.rs`, `config.rs`, `cards.rs`
  - **Mô tả:** Nâng cấp `GateEvaluation` trả về enum `MarketRegime` 6 trạng thái khép kín (`VolatilityFlatline`, `TurbulentChaos`, `DataAnomaly`, `Trending`, `RangeBound`, `Breakout`) dựa trên mô hình trực giao Kaufman Efficiency Ratio ($KER$), Welles Wilder $ADX$, và $SqueezeIntensity$. Tích hợp chốt chặn phân luồng thẩm quyền chọn lọc playbook `is_scanner_eligible_for_regime` tại Arbiter, minh bạch sổ cái đấu thầu `AuctionStatus::RegimeIneligible`. Bảo toàn tuyệt đối 104 chiều DNA.
  - **Unit Tests:**
    - [x] `test_regime_classification_trending`
    - [x] `test_regime_classification_breakout`
    - [x] `test_regime_classification_range_bound`
    - [x] `test_regime_data_anomaly_on_nan_inputs`
    - [x] `test_selective_routing_matrix_contract`
    - [x] `test_arbiter_process_tick_selective_routing_ledger`

- [ ] **TODO-AUDIT-NEW-116: Triệt Tiêu Đánh Giá Lặp Lại `RegimeGatekeeper::evaluate` Tại Arbiter (`H-GATE-03`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:169`, `mod.rs:163`
  - **Mô tả:** Tái cấu trúc truyền kết quả `gate_eval` từ `mod.rs` vào `arbiter.process_tick()`, xóa bỏ lệnh gọi lặp lại Gatekeeper trong Arbiter, tối ưu hóa tốc độ tick.

- [ ] **TODO-AUDIT-NEW-117: Chuyển Đổi Sniper VETO Sang Dynamic De-leverage Trong Thẩm Định SL Cấu Trúc (`H-RSK-01`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:123-138`, `arbiter.rs`
  - **Mô tả:** Thay vì VETO nhị phân hủy bỏ setup đẹp khi SL cấu trúc vượt nhẹ 15% ROE, tự động kẹp SL về trần an toàn và co tỷ lệ ký quỹ vị thế tương ứng để bảo toàn $Risk USD mục tiêu.

- [ ] **TODO-AUDIT-NEW-118: Tái Cấu Trúc Định Cỡ Vị Thế Cố Định Rủi Ro Dollar ($Fixed Risk USD Sizing) (`H-RSK-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:158-189`
  - **Mô tả:** Thay thế công thức tính Notional cố định theo `equity * risk_pct * leverage` bằng mô hình định cỡ Quant chuẩn mực $\text{Size} = \frac{\text{Target Risk USD}}{|\text{Entry} - \text{SL}|}$, đảm bảo số tiền mất USD đồng nhất giữa các lệnh SL hẹp và SL rộng.

- [ ] **TODO-AUDIT-NEW-119: Tách Biệt Nhóm Kích Hoạt Break-Even Cho Absorption Khỏi TrendPB (`H-RSK-03`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:407-411`
  - **Mô tả:** Tách `ABSORPTION` thành nhóm tham số Break-Even độc lập với ngưỡng kích hoạt $1.2 - 1.5\ \text{ATR}$ thay vì ép chung mốc $2.5\ \text{ATR}$ của TrendPB, bảo toàn lợi nhuận săn nén vi mô.

- [ ] **TODO-AUDIT-NEW-120: Đồng Bộ Khép Kín Trạng Thái Stop Loss Sàn Giữa RAM Và Cloud Binance (`H-RSK-04`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:650-653`, `trading_bot/bot_engine/src/execution/order_executor.rs`
  - **Mô tả:** Thiết lập cơ chế Closed-Loop Sync xác nhận mức SL sàn đã khớp lệnh trước khi Strategy dời SL tiếp theo, xóa bỏ hoàn toàn hiện tượng spam lệnh và Naked SL Gap 150ms.

- [ ] **TODO-AUDIT-NEW-121: Thích Ứng Hóa Thời Gian Giải Phóng Vốn Ngâm Theo Độ Nén Biến Động Squeeze (`H-RSK-05`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:358-371`
  - **Mô tả:** Kết hợp chỉ số `squeeze_intensity` vào logic `Stagnant Capital Release`, tự động gia hạn thời gian giữ lệnh khi thị trường đang tích lũy nén cực đại, tránh cắt lỗ non trước sóng bùng nổ.

---

## 🏛️ PHẦN 20: KIỂM TOÁN HẠ TẦNG TẦNG NẠP DỮ LIỆU MẠNG SÀN BINANCE (`binance_rest.rs`, `ws_manager.rs`, `weight_monitor.rs`)
> *Nguồn tài liệu:* [`55_binance_network_data_engine_and_ws_manager_flaws.md`](notes/55_binance_network_data_engine_and_ws_manager_flaws.md)

- [ ] **TODO-AUDIT-NEW-122: Tối Ưu Hóa Backward Sync Đa Luồng Với Rate Throttling Tránh Nghẽn Weight (`TB-NET-01`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/binance_rest.rs:61-145, 178-261`
  - **Mô tả:** Thiết kế hàng đợi fetch nến có điều tiết hạn mức (Rate Throttler), khống chế tổng weight nạp lịch sử không vượt quá 1,200/phút khi khởi chạy đồng thời nhiều cặp tiền, chống bão hòa weight sàn.

- [ ] **TODO-AUDIT-NEW-123: Xóa Bỏ Bẫy Break Sớm Khi Gặp Khoảng Trống Dữ Liệu Nến (Gap Skip Trap) (`TB-NET-02`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/binance_rest.rs:85-87, 209-210`
  - **Mô tả:** Thay thế lệnh `break` khi `data.is_empty()` bằng thuật toán nhảy cóc thời gian (`current_start += 1000 * 60000`), bảo đảm quá trình đồng bộ lấp đầy khoảng trống nến tiếp tục chạy tới mốc thời gian hiện tại.

- [ ] **TODO-AUDIT-NEW-124: Cơ Chế Cầu Dao Khẩn Cấp Emergency Backoff Khi Gặp HTTP 429/418 (`TB-NET-03`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/binance_rest.rs:73, 198`, `models/weight_monitor.rs`
  - **Mô tả:** Tự động phát hiện mã lỗi 429 (Too Many Requests) hoặc 418 (IP Ban) từ Binance REST API, phong tỏa toàn bộ request mạng trong 5 phút để bảo vệ địa chỉ IP không bị đưa vào danh sách đen vĩnh viễn.

- [ ] **TODO-AUDIT-NEW-125: Chuyển Đổi Kênh Phân Phối Nến Sang Bảng Băm O(1) MPSC Độc Lập Chống Mất Nến (`TB-NET-04`)**
  - **File:** `trading_bot/bot_engine/src/execution/ws_manager.rs:48, 134`, `controller/asset_actor.rs`
  - **Mô tả:** Xóa bỏ kênh `broadcast::Sender` dùng chung (vốn gây mất nến âm thầm khi Actor lag CPU dính lỗi `Lagged`), chuyển sang bảng băm `Arc<RwLock<HashMap<String, mpsc::Sender<Candle>>>>` phân luồng độc lập từng cặp tiền.

- [ ] **TODO-AUDIT-NEW-126: Triển Khai Heartbeat Liveness Watchdog Chống Chết Lâm Sàng WebSocket (`TB-NET-05`)**
  - **File:** `trading_bot/bot_engine/src/execution/ws_manager.rs:139-142`
  - **Mô tả:** Giám sát thời gian trôi qua kể từ nến cuối cùng; nếu quá 120 giây không nhận được nến mới từ WebSocket (Half-open TCP silent disconnect), tự động hủy kết nối cũ và tái kết nối lập tức.

- [ ] **TODO-AUDIT-NEW-127: Tự Động Reconnect Stream User Data Khi Tái Cấp listenKey Sau 24 Giờ (`TB-NET-06`)**
  - **File:** `trading_bot/bot_engine/src/execution/ws_manager.rs:207-217`
  - **Mô tả:** Khi vòng lặp keepalive phát hiện listenKey cũ hết hạn hoặc bị hủy, tự động ngắt kết nối WebSocket User Data cũ và mở kết nối mới với listenKey vừa cấp, duy trì thông suốt kênh sự kiện vị thế.

- [ ] **TODO-AUDIT-NEW-128: Dọn Dẹp Mã Mồ Côi `binance_stream.rs` Khỏi Codebase (`TB-NET-07`)**
  - **File:** `trading_bot/bot_engine/src/data_engine/binance_stream.rs`
  - **Mô tả:** Xóa bỏ hoàn toàn file mã cũ `binance_stream.rs` 74 dòng, hợp nhất $100\%$ logic luồng dữ liệu WebSocket về `ws_manager.rs`.

- [ ] **TODO-AUDIT-NEW-129: Chuẩn Hóa Bộ Nhớ Đa Luồng SeqCst & Đệm Thời Gian Trừ Hao Trong WeightMonitor (`TB-NET-08`)**
  - **File:** `trading_bot/bot_engine/src/models/weight_monitor.rs:53-63`
  - **Mô tả:** Chuyển `Ordering::Relaxed` sang `Ordering::SeqCst` và thêm đệm an toàn 500ms khi bước sang phút mới UTC, chống hiện tượng đồng hồ máy tính chạy nhanh hơn Binance làm bùng nổ request.



---

## 🏛️ PHẦN 21: KIỂM TOÁN & GIA CỐ ĐỘNG CƠ BACKTEST & GA OPTIMIZER (`ferro_trade` - NOTES 03, 04, 10, 11, 48, 49)

- [ ] **TODO-AUDIT-NEW-130: Khử Lỗi Trôi Dạt Sai Số Float Trong Cửa Sổ Trượt `efficiency_ratio` (`FT-FLOAT-DRIFT-ER`)**
  - **File:** `ferro_trade/src/indicators/bulk.rs:157-175`
  - **Mô tả:** Thuật toán rolling sum `sum += diff - old_diff` bị trôi dạt dấu phẩy động tích lũy (Catastrophic cancellation). Khi thị trường đi ngang, mẫu số có thể bị âm nhẹ hoặc bằng 0.0, thiếu phép kẹp an toàn `.clamp(0.0, 1.0)`. Hệ quả là $ER$ nổ tung $> 1.0$ hoặc rơi về số âm, làm biến dạng hoàn toàn KAMA và chỉ số nhiễu động.
  - **Unit Tests:**
    - [ ] `test_efficiency_ratio_float_drift_resistance`

- [ ] **TODO-AUDIT-NEW-131: Khử Race Condition Trong `GlobalCache` Bằng Upgradable Lock (`FT-CACHE-THUNDERING`)**
  - **File:** `ferro_trade/src/data/global_cache.rs:164-182`
  - **Mô tả:** Hàm `get_or_compute` nhả `read_lock` rồi mới xin `write_lock`. Khi hàng chục luồng Rayon cùng chạy GA song song, hiện tượng Thundering Herd xảy ra khiến cùng một chỉ báo kỹ thuật bị tính toán trùng lặp hàng chục lần, gây nghẽn CPU. Chuyển sang `parking_lot::RwLock` với cơ chế upgradable lock.
  - **Unit Tests:**
    - [ ] `test_global_cache_concurrent_compute_no_duplicate`

- [ ] **TODO-AUDIT-NEW-132: Triển Khai Cơ Chế Đào Thải LRU Cho Fitness Cache Chống Cache Thrashing (`FT-CACHE-THRASHING`)**
  - **File:** `ferro_trade/src/optimizer/fitness_cache.rs:102-115`
  - **Mô tả:** Khi cache chạm ngưỡng `MAX_CACHE_ENTRIES` (50,000 entries), code gọi thẳng `self.cache.clear()`, xóa sạch 100% dữ liệu đã tính, làm tỷ lệ Hit-Rate rớt về 0%, gây sốc tải CPU đột ngột giữa các thế hệ GA. Thay thế bằng cấu trúc `lru::LruCache` hoặc đào thải 25% cá thể cũ nhất.
  - **Unit Tests:**
    - [ ] `test_fitness_cache_lru_eviction`

- [x] **TODO-AUDIT-NEW-133: Định Danh Cache Key Theo Khung Thời Gian Trong `IndicatorManager` (`FT-TIMEFRAME-POLLUTE`)**
  - **File:** `ferro_trade/src/backtest/manager.rs:106-115`
  - **Mô tả:** `add_raw_cached` hardcode cứng chuỗi `"T1m"` và cột `"Close"` trong cache key. Khi nạp dữ liệu MTF (5m, 15m, 1h) vào cùng cache, các mảng nến khung khác nhau sẽ ghi đè lên nhau, gây sai lệch nghiêm trọng chỉ báo MTF. Bổ sung `timeframe.as_str()` và `column_name` vào cache key.
  - **Unit Tests:**
    - [x] `test_manager_cache_isolation_across_timeframes`

- [ ] **TODO-AUDIT-NEW-134: Khắc Phục Over-subscription CPU Bằng Nested Rayon ThreadPool Guard (`FT-CPU-OVERSUBSCRIBE`)**
  - **File:** `ferro_trade/src/optimizer/prospector.rs:898-903`
  - **Mô tả:** Worker GA bên trong lại tiếp tục gọi song song Rayon ngầm, sinh ra $N \times M$ luồng tranh chấp nhân CPU, gây quá tải Context Switching làm giảm 40% tốc độ tiến hóa. Khởi tạo ThreadPool đơn tập trung ở cấp `Prospector` và chia sẻ `&ThreadPool` cho các worker.
  - **Unit Tests:**
    - [ ] `test_prospector_cpu_threads_bounded`

- [ ] **TODO-AUDIT-NEW-135: Thay Thế `isize::MIN` Bằng Điểm Phạt Hữu Hạn Trong Đánh Giá GA (`FT-VAR-EXPLOSION-MIN`)**
  - **File:** `ferro_trade/src/optimizer/ga_worker.rs:345-350`
  - **Mô tả:** Khi cá thể vi phạm điều kiện hoặc bị panic, fitness được gán `isize::MIN` ($\approx -9.22 \times 10^{18}$). Khi tính phương sai quần thể để kiểm tra hội tụ, số này bị bình phương làm phương sai nổ tung lên $10^{36}$, vô hiệu hóa hoàn toàn cơ chế Stagnation Pruning. Thay bằng giá trị sàn hữu hạn `-100_000.0`.
  - **Unit Tests:**
    - [ ] `test_population_variance_stability_with_failed_individuals`

- [ ] **TODO-AUDIT-NEW-136: Khử Deadlock Đệ Quy Trong `IndicatorManager::get_indicator_slice` (`FT-RECURSIVE-DEADLOCK`)**
  - **File:** `ferro_trade/src/backtest/manager.rs:271-285`
  - **Mô tả:** `get_indicator_slice` nắm giữ `RwLockReadGuard` trên toàn bộ bảng băm HashMap. Nếu một chỉ báo phụ thuộc gọi đệ quy một chỉ báo khác chưa tính toán và yêu cầu `write_lock`, hệ thống rơi vào deadlock vĩnh viễn. Sao chép `Arc<Vec<f64>>` và nhả ngay `read_lock` trước khi tính toán phụ thuộc.
  - **Unit Tests:**
    - [ ] `test_manager_recursive_indicator_resolution`

- [ ] **TODO-AUDIT-NEW-137: Chia Sẻ Dữ Liệu `Arc<DataView>` Cho GA Workers Thay Vì Deep Clone (`FT-DEEP-CLONE-DATAVIEW`)**
  - **File:** `ferro_trade/src/optimizer/ga_worker.rs:194-205`
  - **Mô tả:** Mỗi cá thể GA khi gọi `evaluate_dna` đều nhân bản (deep clone) cấu trúc `DataView` chứa hàng trăm nghìn nến, ngốn RAM và tốn thời gian cấp phát bộ nhớ heap. Bọc mảng nến trong `Arc<RawDataCache>` hoặc `Arc<DataView>`, truyền tham chiếu read-only cho các worker (Zero-copy).
  - **Unit Tests:**
    - [ ] `test_ga_worker_zero_copy_dataview`

- [ ] **TODO-AUDIT-NEW-138: Thêm Chốt Chặn `max_position_size` Khi Nhồi Vị Thế Pyramiding (`FT-PYRAMID-UNCHECKED`)**
  - **File:** `ferro_trade/src/broker/broker.rs:324-335`
  - **Mô tả:** Khi lệnh Buy/Sell được khớp để nhồi thêm khối lượng vào vị thế đang mở, broker chỉ kiểm tra số dư ký quỹ tức thời mà không kiểm tra giới hạn `max_position_size`, cho phép tích lũy vị thế khổng lồ phá vỡ quy chuẩn rủi ro. Bổ sung kiểm tra trần khối lượng tối đa.
  - **Unit Tests:**
    - [ ] `test_broker_rejects_pyramiding_exceeding_max_size`

- [ ] **TODO-AUDIT-NEW-139: Bảo Toàn Nến Khung Lớn Dở Dang Ở Cuối Dữ Liệu Resample (`FT-MTF-CUTOFF-LOSS`)**
  - **File:** `ferro_trade/src/utils/resample.rs:112-125`
  - **Mô tả:** Khi chuỗi nến 1m không tròn khung 4h (ví dụ dư 140 nến 1m ở cuối), hàm resample cắt bỏ hoàn toàn các nến dư thừa này thay vì tổng hợp thành một nến 4h dở dang, gây mất mát dữ liệu những giờ giao dịch quan trọng nhất ngay sát thời điểm hiện tại.
  - **Unit Tests:**
    - [ ] `test_resample_preserves_incomplete_trailing_candles`

- [ ] **TODO-AUDIT-NEW-140: Chuẩn Hóa Timestamp Milliseconds & Guard Assertion (`FT-TIMESTAMP-MISMATCH`)**
  - **File:** `ferro_trade/src/data/raw_data.rs:88-100`
  - **Mô tả:** Trường `timestamp` nhận số `i64` trần trụi, trộn lẫn giữa giây ($10^9$) và mili-giây ($10^{12}$) khi import dữ liệu từ các sàn khác nhau mà không có assertion kiểm tra. Tự động phát hiện nếu `timestamp < 10_000_000_000` $\implies$ nhân 1,000 quy đổi về milliseconds và kiểm tra bước nhảy nến 60,000ms.
  - **Unit Tests:**
    - [ ] `test_raw_data_auto_converts_seconds_to_milliseconds`

- [ ] **TODO-AUDIT-NEW-141: Khắc Phục Thiên Kiến Khớp Lệnh Khi Nến Chạm Cả TP và SL (Dual Trigger Priority Tới Open) (`FT-BRK-03`)**
  - **File:** `ferro_trade/src/broker/broker.rs:137-142`
  - **Mô tả:** Broker luôn đưa Stop Loss vào đầu hàng đợi và Take Profit vào cuối. Khi gặp nến quét 2 đầu bao trùm cả TP và SL, Broker luôn ép dính SL làm thổi phồng Drawdown giả tạo. Cải tiến mô hình khớp lệnh: So sánh khoảng cách từ giá `Open` của nến tới TP và SL để xác định thứ tự chạm giá hợp lý hơn.
  - **Unit Tests:**
    - [ ] `test_broker_dual_hit_order_priority_proximity_to_open`

- [ ] **TODO-AUDIT-NEW-142: Đồng Bộ Hóa Equity Curve Khi Khớp Lệnh `trade_on_close = true` (`FT-ENG-02`)**
  - **File:** `ferro_trade/src/broker/broker.rs:413-417`, `engine.rs:298-329`
  - **Mô tả:** Khi bật `trade_on_close = true`, lệnh được khớp tại giá Close nến trước nhưng `equity()` nến trước đã bị ghi đè vào `equity_curve` từ vòng lặp trước. Cần đồng bộ thời gian vào lệnh và đường cong vốn để tránh lookahead bias.
  - **Unit Tests:**
    - [ ] `test_broker_trade_on_close_equity_curve_alignment`

- [ ] **TODO-AUDIT-NEW-143: Nâng Cấp Macro `dna_signature()` Băm Cả Dải Min/Max/Step & Triệt Tiêu Lỗi XOR Collision (`H-CFG-10`)**
  - **File:** `ferro_trade/ferro_trade_derive/src/lib.rs:100-132, 260-273`
  - **Mô tả:** Macro hiện tại chỉ băm tên trường và kiểu dữ liệu, mù hoàn toàn trước thay đổi `min`, `max`, `step`, `options`, khiến SQLite Storage nhầm tưởng cấu trúc cũ vẫn hợp lệ và đọc lại các cá thể Champion cũ không tương thích. Thay thế phép toán XOR (`^=`) tự triệt tiêu bằng thuật toán băm tuần tự an toàn (Hasher / FNV-1a).
  - **Unit Tests:**
    - [ ] `test_dna_signature_changes_when_min_max_step_modified`
    - [ ] `test_dna_signature_no_xor_cancellation_collision`

- [ ] **TODO-AUDIT-NEW-144: Bật Mô Hình Trượt Giá & Spread Tối Thiểu Trong `ferro_trade::Broker` Khi Chạy GA (`FT-GA-SPREAD-SIMULATION`)**
  - **File:** `ferro_trade/src/broker/broker.rs:80`, `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:524`
  - **Mô tả:** `BacktestBuilder` khi chạy GA đang để `spread = 0.0`. GA trong phòng thí nghiệm hoàn toàn bị "mù" trượt giá, sinh ra các cá thể cào lệnh vi mô tưởng như có lãi nhưng vừa bước ra cửa WFE Gate ngoài OOS là bị reject $46.7\%$. Nạp tham số `slippage` tối thiểu $0.02\%$ vào `BacktestBuilder` trong mỗi vòng đánh giá cá thể GA.
  - **Unit Tests:**
    - [ ] `test_ga_evaluation_penalizes_zero_spread_scalping`

---

## 🏛️ PHẦN 22: KIỂM TOÁN & GIA CỐ NỀN TẢNG THỰC THI & CONCURRENCY (`trading_bot` - NOTES 03, 14, 16, 17, 18, 29, 30, 40)

- [ ] **TODO-AUDIT-NEW-145: Khử Deadlock Bounded Channel Trên `status_tx` Bằng Non-blocking Try-Send / Watch Channel (`TB-UI-BACKPRESSURE-LOCK`)**
  - **File:** `trading_bot/bot_engine/src/bus.rs:28`, `src/controller/asset_actor.rs:484`
  - **Mô tả:** Kênh `status_tx` đẩy trạng thái sang giao diện Desktop UI bị giới hạn dung lượng cứng là 100 gói tin và gọi lệnh `send().await`. Khi UI bị đơ hoặc lag không đọc kịp, lệnh này chặn đứng hoàn toàn Actor loop, khiến bot ngừng quét Trailing Stop và rớt mạng WebSocket. Chuyển sang `try_send()` hoặc kênh `tokio::sync::watch`.
  - **Unit Tests:**
    - [ ] `test_asset_actor_loop_survives_ui_channel_overflow`

- [ ] **TODO-AUDIT-NEW-146: Triển Khai Atomic Quota Pre-Reservation Trong `WeightMonitor` Chống Ban IP (`TB-WEIGHT-BURST-BAN`)**
  - **File:** `trading_bot/bot_engine/src/models/weight_monitor.rs:41-65`
  - **Mô tả:** Cơ chế kiểm tra trọng số API bị lỗi Check-then-act: Nhiều Actor cùng kiểm tra thấy trọng số khả dụng, sau đó đồng loạt bắn request REST API lên Binance, vượt ngưỡng 2400 weight/phút và bị phạt HTTP 429/418. Triển khai hàm `reserve_weight(estimated_cost) -> Result<ReservationTicket, BotError>` trừ hạn mức ngay lập tức trước khi gửi HTTP request.
  - **Unit Tests:**
    - [ ] `test_weight_monitor_atomic_reservation_blocks_concurrent_burst`

- [ ] **TODO-AUDIT-NEW-147: Sửa Lỗi Triệt Tiêu Nến Bù Gap Filler Trong `AssetActor::push_candle` (`TB-GAP-FILLER-BUFFER-MERGE`)**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs:409`, `src/models/candle.rs:92`
  - **Mô tả:** Khi kết nối bị trễ, gói tin nến bị mất. Khi Gap Filler kéo nến bù về, hàm `push_candle` lại thẳng tay vứt bỏ với điều kiện `if candle.timestamp <= last.timestamp { return; }`. Hậu quả là bộ đệm nến trượt bị thủng lỗ vĩnh viễn, làm sai lệch chỉ báo MTF. Triển khai phương thức `insert_or_replace_candle` vào đúng vị trí timestamp trong Rolling Buffer.
  - **Unit Tests:**
    - [ ] `test_rolling_buffer_gap_fill_recovers_missing_candle`

- [ ] **TODO-AUDIT-NEW-148: Chuẩn Hóa State Machine & Ngưỡng Co Size Trong Staleness Watchdog (`TB-ZOMBIE-DNA-STALENESS-WATCHDOG`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:412-440`
  - **Mô tả:** Bổ sung trường `consecutive_rejects: u32` vào `ProspectorRunnerState` trong SQLite. Triển khai enum `SystemDefenseMode { Normal, HalfSize, DefensiveHold }`: Giảm 50% size sau 3 lần reject liên tiếp và kích hoạt DefensiveHold (đứng ngoài thị trường bảo toàn vốn) sau 6 lần reject liên tiếp.
  - **Unit Tests:**
    - [ ] `test_staleness_watchdog_triggers_half_size_at_3_rejects`
    - [ ] `test_staleness_watchdog_triggers_defensive_hold_at_6_rejects`

- [ ] **TODO-AUDIT-NEW-149: Chuyển Đổi MTF Resampling Sang Rust Thuần (`fast_resample_ohlcv`), Triệt Tiêu Polars DataFrame Trong Live Tick Engine (`TODO-BRG-06`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:105-180`, `ferro_trade/src/utils/resample.rs`
  - **Mô tả:** Mỗi phút trôi qua `evaluate_live_tick` khởi tạo mới một Polars DataFrame từ bộ đệm nến 1m để chạy resampling đa khung (15m, 1h, 4h), ngốn 11.2MB RAM/phút/cặp tiền và mất 80–120ms CPU blocking. Thay thế bằng hàm gom nến $O(N)$ single-pass iterator viết bằng Rust thuần (`fast_resample_ohlcv`), triệt tiêu hoàn toàn runtime Polars trên đường thực thi live.
  - **Unit Tests:**
    - [ ] `test_fast_resample_ohlcv_matches_polars_resample_15m_1h_4h`
    - [ ] `test_fast_resample_handles_empty_and_single_candle`
    - [ ] `test_fast_resample_benchmark_sub_millisecond_latency`

- [ ] **TODO-AUDIT-NEW-150: Bổ Sung Snapshot PnL Thả Nổi (`pnl`, `pnl_pct`) Vào `TickContext` Của `HeliosBridge` (`TODO-BRG-05`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:240-255`
  - **Mô tả:** Khi tạo `TickContext` để truyền vào `HeliosStrategyV3::next()`, trường `pnl` và `pnl_pct` của `Position` bị gán cứng `0.0`. Hệ quả là các logic tính toán trong Strategy dựa trên vị thế mở (đòn bẩy động, drawdown thả nổi nội nến, bảo hiểm lợi nhuận MFE) bị mù thông tin. Bóc tách `unrealized_pnl` từ `LivePositionState` hoặc tính toán trực tiếp và truyền vào `TickContext`.
  - **Unit Tests:**
    - [ ] `test_helios_bridge_transfers_real_unrealized_pnl_to_tick_context`
    - [ ] `test_tick_context_pnl_pct_matches_direction_and_leverage`

- [ ] **TODO-AUDIT-NEW-151: Khắc Phục Lỗi Gán Cứng Size Vị Thế `1.0/-1.0` Trong `last_closed_trade` Của `HeliosBridge` (`TODO-BRG-02`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:258-270`
  - **Mô tả:** `HeliosBridge` khi dựng `last_closed_trade` để Strategy đánh giá chuỗi lệnh thắng/thua đã gán cứng `size = 1.0` (Long) hoặc `-1.0` (Short). Nạp đúng khối lượng thực tế `trade.size` từ `ClosedTradeState` hoặc bảng CSDL `trade_history`.
  - **Unit Tests:**
    - [ ] `test_bridge_populates_exact_closed_trade_size`

- [ ] **TODO-AUDIT-NEW-152: Triển Khai Baseline Fallback Seed Chống Bẫy Khởi Động Lạnh (Cold Start Starvation) Cho `ProspectorRunner` (`TODO-PROS-01`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:450-475`
  - **Mô tả:** Khi bot mới triển khai trên một cặp tiền mới với database rỗng, chu kỳ GA đầu tiên nếu bị WFE Gate từ chối (`REJECT`), Runner thoát ra và không nạp bất kỳ DNA nào vào SQLite, khiến bot rơi vào bẫy "đói hạt giống" suốt 20–40 giờ. Tự động nạp cá thể chuẩn từ file `helios_baseline_dna.json` vào bảng `champion_dnas` làm hạt giống an toàn cấp cứu.
  - **Unit Tests:**
    - [ ] `test_cold_start_fallback_to_baseline_seed_on_wfe_reject`
    - [ ] `test_existing_champion_not_overwritten_by_baseline`

- [ ] **TODO-AUDIT-NEW-153: Triển Khai Bộ Đệm Top-K Candidates Waterfall ($K=5$) Vượt Cửa Ải WFE Gate Trong `ProspectorRunner` (`TODO-PROS-08`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:395-408`
  - **Mô tả:** Runner hiện chỉ trích xuất duy nhất 1 cá thể Rank 1 In-Sample đi thẩm định OOS. Trích xuất Top 5 cá thể In-Sample tốt nhất từ quần thể GA và chạy thẩm định OOS tuần tự theo cơ chế Waterfall: Cá thể đầu tiên vượt qua tiêu chuẩn Do-No-Harm sẽ được chọn làm Champion triển khai ra Live; nếu cả 5 đều rớt mới kích hoạt Defensive Hold.
  - **Unit Tests:**
    - [ ] `test_top_k_waterfall_picks_first_passing_candidate`
    - [ ] `test_top_k_waterfall_all_reject_triggers_defensive_hold`

- [ ] **TODO-AUDIT-NEW-154: Giải Phóng Khóa Tĩnh Toàn Cục `DIAGNOSTIC_LOCK` Cho Phép Đa Cặp Tiền Backtest Đồng Thời (`TODO-TB-05`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:335`, `controller/asset_actor.rs:360`
  - **Mô tả:** Biến `static DIAGNOSTIC_LOCK: Lazy<Mutex<()>>` khóa tuần tự toàn bộ các symbol khi nhận DNA mới, làm các Actor phải xếp hàng chờ nhau mất tới 50 giây, gây lag nghẽn kênh WebSocket và rớt nến. Xóa bỏ hoàn toàn khóa tĩnh này và bọc `diagnostic_backtest` trong `tokio::task::spawn_blocking`.
  - **Unit Tests:**
    - [ ] `test_concurrent_diagnostic_backtests_no_deadlock`

- [ ] **TODO-AUDIT-NEW-155: Đổi Làm Tròn Khối Lượng Vị Thế Từ `.round()` Sang `Floor` Trong `OrderExecutor` Và `LiveEngine` (`TODO-EXEC-03`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:145-165`, `live_engine.rs`
  - **Mô tả:** Hàm làm tròn khối lượng theo `step_size` sàn Binance sử dụng phép làm tròn thông thường (`.round()`). Khi số dư ký quỹ vừa khít, việc làm tròn lên sẽ khiến tổng giá trị ký quỹ vượt quá số dư khả dụng, sàn Binance trả về mã lỗi `-2019 Margin Insufficient`. Thay thế toàn bộ bằng phép làm tròn xuống (`floor`).
  - **Unit Tests:**
    - [ ] `test_order_executor_qty_always_floored_to_step_size`
    - [ ] `test_order_executor_never_exceeds_available_margin_due_to_rounding`

- [ ] **TODO-AUDIT-NEW-156: Chốt Chặn Kiểm Soát Độ Giãn Spread Thời Gian Thực Trước Khi Gửi Lệnh (`EXEC-BID-ASK-SPREAD-GATE`)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs`, `live_engine.rs`, `models/market_filter.rs`
  - **Mô tả:** Khi nến bùng nổ kích hoạt, các Market Maker rút lệnh (Quote Pulling) làm chênh lệch Bid-Ask (Spread) dãn to đột ngột từ 0.01% lên 0.20%–0.50%, nuốt chửng 4%–10% ROE ngay khi vào lệnh. Đọc BookTicker thời gian thực từ Binance WebSocket: Nếu $\text{Spread} = \frac{Ask - Bid}{Mid} > \text{max\_allowed\_spread}$ (mặc định 0.08%) $\implies$ Hoãn gửi lệnh hoặc từ chối thực thi.
  - **Unit Tests:**
    - [ ] `test_order_executor_blocks_market_order_when_spread_blows_out`
    - [ ] `test_order_executor_allows_execution_when_spread_is_tight`

---

## 🏛️ PHẦN 23: TÁI CẤU TRÚC HẠ TẦNG MA TRẬN & CỔNG VÀO GATEWAY (NOTES 08, 09, 19, 20, 23, 35, 36, 40)

- [x] **TODO-AUDIT-NEW-157: Sửa Dị Tật Forward-Fill Percentile Của `chaos_ratio` & `entropy` Trong `matrix.rs` (`H-FORWARD-FILL-MTF-PERCENTILE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/matrix.rs:82-85`
  - **Mô tả:** `sensors.chaos_ratio` và `sensors.efficiency_ratio` được tính từ khung 15m/1h và đã bị rải phẳng thành mảng 1m (15 giá trị giống hệt nhau). `matrix.rs` lại gọi `rolling_percentile` chạy trên mảng 1m này, khiến CDF bị méo mó và điểm số giật cục từng nấc thang 15 phút. Đã chuyển hoàn toàn sang hàm chuẩn hóa liên tục O(1) qua $\tanh((\text{chaos} - 1.0) / 1.0)$ và $\text{noise} = 1.0 - \text{ER}$. Xóa bỏ 2 gen ma `trend_lookback` và `chaos_lookback` khỏi DNA.
  - **Unit Tests:**
    - [x] `test_matrix_chaos_index_smooth_no_staircase_artifacts`

- [x] **TODO-AUDIT-NEW-158: Khử Nhiễu Sàn 0.25 Của Chaos Index & Ràng Buộc Đồng Bộ Timeframe Giữa NATR Và KAMA ER (`TODO-MAT-04`, `FLAW-MTF-FF`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/matrix.rs:98-104`, `config.rs`
  - **Mô tả:** Công thức $\text{chaos\_index} = c\_r \times (1.0 - \text{entropy})$ bị bẫy nhiễu sàn 0.25 khi thị trường phẳng lì (do rolling percentile của chuỗi flatline bằng 0.50). Đã khử triệt để bẫy sàn 0.25 bằng công thức chuẩn hóa liên tục $O(1)$: khi market flat ($\text{chaos\_ratio} \le 1.0 \implies c_r = 0.0$), $\text{chaos\_index}$ trả về đúng $0.0$ tuyệt đối, bảo toàn tính độc lập Data Lineage không gây rủi ro trễ pha đa khung thời gian.
  - **Unit Tests:**
    - [x] `test_chaos_index_zero_when_market_flat`
    - [x] `test_matrix_chaos_index_smooth_no_staircase_artifacts`

- [ ] **TODO-AUDIT-NEW-159: Thiết Lập Kiến Trúc Độc Lập Khung Thời Gian Động ($T_{micro}, T_{meso}, T_{macro}$) Và Ràng Buộc Phổ Biến Động $\ge 3.0$ (`H-TIMEFRAME-AGNOSTIC-SPECTRUM`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs`, `mod.rs`
  - **Mô tả:** Code hiện tại hardcode cứng các mốc 15m, 1h, 4h trong logic chiến lược. Toàn bộ các mốc tham chiếu phải được cấu hình động qua $T_{micro}, T_{meso}, T_{macro}$ và bắt buộc thỏa mãn tỷ số phổ biến động $\ge 3.0$ để tránh tự tương quan âm và nhiễu vi cấu trúc.
  - **Unit Tests:**
    - [ ] `test_spectral_separation_constraint_enforcement`

- [ ] **TODO-AUDIT-NEW-160: Chính Thức Khai Tử Vai Trò Tactical Scanner Của `anomaly.rs` Khỏi Arbiter Loop (`ANO-DECOMMISSION-SCANNER`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/anomaly.rs`, `core_v3/mod.rs`, `arbiter.rs`, `config.rs`
  - **Mô tả:** Xóa bỏ hoàn toàn việc đăng ký `anomaly.rs` trong sàn đấu Arbiter, xóa các gen DNA liên quan, giải phóng tài nguyên CPU và làm sạch kiến trúc. Chuyển hóa toàn bộ giá trị phòng vệ của Anomaly sang Cầu Dao Ngắt Mạch Khẩn Cấp tại Gateway.
  - **Unit Tests:**
    - [ ] `test_arbiter_runs_cleanly_without_anomaly_scanner_proposals`

- [x] **TODO-AUDIT-NEW-161: Kiến Thiết Cầu Dao Khẩn Cấp Flash Crash & Liquidity Shock Circuit Breaker Tại Gateway (`GATE-FLASH-CRASH-BREAKER`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs`, `config.rs`, `telemetry/defense_ledger.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH: 12/12 Audit PASS, 21 Gatekeeper Tests PASS, DNA 106 -> 110 chiều]. Triển khai cơ chế nhận diện 2 tầng nến sốc (Bão thanh lý xả lũ: `span >= 3.5 ATR` & `Z_vol >= 4.0` hoặc Rỗng sổ lệnh `span >= 4.5 ATR`). Kích hoạt phong tỏa mở vị thế mới `GateVetoReason::FlashCrashFreeze` trong cửa sổ trượt $K$ nến phi trạng thái $O(K)$, bảo vệ an toàn 100% vốn trước râu nến quét thanh khoản tử thần.
  - **Unit Tests:**
    - [x] `test_gatekeeper_triggers_emergency_freeze_on_flash_crash_candle`
    - [x] `test_gatekeeper_triggers_on_liquidity_vacuum_without_volume`
    - [x] `test_gatekeeper_cools_down_and_resumes_cleanly_after_shock`
    - [x] `test_gatekeeper_ignores_normal_breakout_candle`
    - [x] `test_gatekeeper_flash_crash_freeze_edge_detection`
    - [x] `test_dna_width_and_parameter_defs_integrity` (110 chiều)

- [ ] **TODO-AUDIT-NEW-162: Bộ Lọc Khử Ô Nhiễm Phương Sai Z-Score Bằng Robust Dispersion / Winsorization / MAD (`SENS-ROBUST-ZSCORE`)**
  - **File:** `strategies_backtests/helios/src/utils/normalization.rs`, `sensors.rs`, `matrix.rs`
  - **Mô tả:** Khi xuất hiện 1 cây nến dị thường khổng lồ (Flash Crash / Anomaly), phương sai mẫu $\sigma$ bị thổi phồng lên gấp 5–8 lần, làm đè bẹp Z-Score của 30 cây nến tiếp theo tiệm cận 0 và gây mù hệ thống. Áp dụng kỹ thuật Robust Statistics: Kẹp ngoại lai Winsorization tại $\pm 3.5\sigma$ hoặc sử dụng Median Absolute Deviation (MAD): $\sigma_{\text{robust}} = 1.4826 \times \text{MAD}(X)$.
  - **Unit Tests:**
    - [ ] `test_robust_zscore_resists_single_extreme_outlier_candle`
    - [ ] `test_robust_zscore_matches_standard_zscore_in_normal_distribution`

- [ ] **TODO-AUDIT-NEW-163: Bộ Lọc Cửa Sổ Giờ Chốt Phí Funding Rate (Funding Settlement Spike Blackout Filter) Tại Gateway (`GATE-FUNDING-BLACKOUT`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs`, `scanners/`
  - **Mô tả:** Tại các mốc 00:00, 08:00, 16:00 UTC ($\pm 2$ phút), các quỹ Basis Arbitrage và Market Maker đóng/mở vị thế hedging làm giật râu volume khổng lồ ảo. Gatekeeper tự động phát hiện cửa sổ Funding Settlement: $t \pmod{8\text{h}} \in [00:00 - 2\text{m}, 00:00 + 1\text{m}]$ và phong tỏa mở lệnh mới trong 3 phút nhạy cảm này.
  - **Unit Tests:**
    - [ ] `test_funding_settlement_blackout_blocks_entries`

- [ ] **TODO-AUDIT-NEW-164: Bộ Lọc Cú Sốc Mở Phiên Lớn London / New York Cho Hệ Thống Giao Dịch (`GATE-SESSION-OPEN-SHOCK`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs`, `arbiter.rs`
  - **Mô tả:** Tại thời điểm giao phiên London (14:00 VN) và New York (20:30 VN), thanh khoản bùng nổ quét râu 2 đầu làm gãy các mô hình vi mô. Tạm ngừng mở lệnh mới hoặc nới rộng đệm SL trong 5–15 phút đầu mở phiên lớn.
  - **Unit Tests:**
    - [ ] `test_session_open_volatility_buffer_cooldown`

---

## 🏛️ PHẦN 24: TÁI CẤU TRÚC SÀN ĐẤU THẦU ARBITER & RỦI RO CHIẾN THUẬT (NOTES 05, 12, 14, 15, 21, 22)

- [-] **TODO-AUDIT-NEW-165: Xây Dựng Module Quantile Normalization & Đấu Thầu EV Kỳ Vọng Toán Học Thuần Khiết Trong Arbiter (`H-QUANTILE-FAIR-ARBITER`) (ĐÃ TỐI GIẢN HÓA & QUY HOẠCH CỤM E)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:150-185`
  - **Mô tả:** [PENDING: Chờ bằng chứng thực nghiệm OOS. Unit Test `test_intra_regime_fair_auction_pattern_quality_wins` đã pass nhưng dữ liệu 29 chu kỳ Walk-Forward OOS chưa có trường hợp nào xung đột 2 proposal cùng regime để kiểm chứng tác động PnL.] Chuẩn hóa điểm số thô của từng scanner theo phân phối lịch sử của chính nó thành điểm phân vị $Z$-Score $\in [0.0, 1.0]$. Đấu thầu dựa trên điểm kỳ vọng toán học chuẩn hóa: $EV = \text{Score}_{\text{norm}} \times \frac{\text{TP Dist}}{\text{SL Dist}} - \text{Friction}$, triệt tiêu hoàn toàn tình trạng Absorption cướp quyền vô lý của Momentum và Reversal.
  - **Unit Tests:**
    - [x] `test_intra_regime_fair_auction_pattern_quality_wins`

- [ ] **TODO-AUDIT-NEW-166: Điều Chỉnh `min_lock_roe` Khử Vùng Chân Không Trailing SL Từ $+6\%$ Đến $+20\%$ ROE (`H-TSL-VACUUM-RECONCILIATION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:1187`, `risk.rs:431-445`
  - **Mô tả:** `min_lock_roe` đang bị hardcode cứng ở mức $+20.0\%$ ROE, tạo ra vùng chân không bảo vệ khiến lệnh lãi $+16.5\%$ ROE rơi thẳng về mức hòa vốn $+2.6\%$ ROE khi đảo chiều. Cài đặt 3 nấc khóa lãi bậc thang trong `risk.rs`: $+6\% \to +2.5\%$, $+10\% \to +5.0\%$, $+15\% \to +9.0\%$ ROE.
  - **Unit Tests:**
    - [ ] `test_tiered_profit_lock_locks_9_pct_at_15_pct_roe`

- [ ] **TODO-AUDIT-NEW-167: Thêm Chốt Chặn Lợi Nhuận Bỏ Qua Stagnant Hold Exit Khi Vị Thế Đang Có Lãi Lớn (`H-RSK-STAGNANT-PROFIT-BYPASS`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:358-364`
  - **Mô tả:** Logic giải phóng vốn ngâm cưỡng bức đóng lệnh Market sau $N$ nến mà không kiểm tra vị thế đang lãi hay lỗ. Bổ sung điều kiện: Nếu `unrealized_roe >= min_profitable_roe` (ví dụ $+5.0\%$ ROE), **CẤM KÍCH HOẠT STAGNANT EXIT**, tiếp tục để Trailing SL gồng lãi theo xu hướng.
  - **Unit Tests:**
    - [ ] `test_stagnant_exit_bypassed_when_position_in_strong_profit`

- [ ] **TODO-AUDIT-NEW-168: Cơ Chế Đệm Chuyển Tiếp Trạng Thái 1-Bar Tránh Sốc Tín Hiệu Sau Khi DNA Hot-Swap (`HOTSWAP-STATE-INVALIDATION`)**
  - **File:** `trading_bot/bot_engine/src/controller/asset_actor.rs:338-345`, `strategies_backtests/helios/src/strategy/core_v3/mod.rs`
  - **Mô tả:** Khi nhận DNA mới ở trạng thái Flat, các ngưỡng kích hoạt của Scanner và Gatekeeper thay đổi tức thì, trong khi bộ đệm trượt lịch sử chưa có giai đoạn chuyển tiếp, dễ tạo ra các cú sốc tín hiệu ảo (Spurious Signal Spikes). Áp dụng 1-bar Transition Guard: Sau khi nạp DNA mới, bỏ qua việc mở lệnh tại đúng cây nến hiện tại, chỉ bắt đầu quét mở lệnh từ cây nến đóng tiếp theo.
  - **Unit Tests:**
    - [ ] `test_dna_hotswap_transition_guard_prevents_immediate_spike`

---

## 🏛️ PHẦN 25: GIA CỐ TOÀN DIỆN CHIẾN ĐỘI MOMENTUM IGNITION (NOTES 28, 29, 30, 31)

- [ ] **TODO-AUDIT-NEW-169: Khắc Phục Lệch Pha Thang Đo MTF Giữa Nến Vi Mô 1m & Dải Bollinger 15m (`MOM-MTF-SCALE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs:51-57`, `matrix.rs`
  - **Mô tả:** Scanner so sánh `close` 1m với dải Bollinger 15m (`bb_upper`, `bb_lower`). Khi nến 1m đầu tiên bùng nổ thoát hộp nén vi mô, giá vẫn nằm trọn trong dải BB 15m $\implies$ Bỏ lỡ 100% điểm vào chân sóng, chỉ phát hiện khi nến đã chạy đến cây thứ 8–10 (sát đỉnh Climax). Chuyển sang so sánh với dải bứt phá vi mô tương thích (1m/5m Bollinger/Keltner Bands hoặc Micro Donchian Channel).
  - **Unit Tests:**
    - [ ] `test_momentum_ignition_detects_first_micro_breakout_before_15m_expansion`
    - [ ] `test_momentum_ignition_ignores_subthreshold_micro_noise`

- [ ] **TODO-AUDIT-NEW-170: Chốt Chặn Trần Quá Độ Biến Động & Bão Thanh Lý Dây Chuyền $|Z| \le 3.0$ (`MOM-MAX-PRICE-Z-CAP`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs`, `config.rs`
  - **Mô tả:** Khi bão thanh lý Short xảy ra, giá bị giật bay 5–10 ATR vào "túi khí" sổ lệnh mỏng. Mua tại đây là mua ngay trước cú sụp đổ hồi quy. Thiết lập trần quá độ $|Price\_Z| \le \text{max\_price\_z}$ (mặc định $3.0\sigma$) và cự ly bứt phá không vượt quá $1.5\text{ ATR}$ tính từ đường trung bình, tự động VETO các cú kích hỏa đã bị kéo dãn quá đà.
  - **Unit Tests:**
    - [ ] `test_momentum_ignition_vetoes_blown_out_liquidation_cascade`
    - [ ] `test_momentum_ignition_accepts_healthy_early_breakout`

- [ ] **TODO-AUDIT-NEW-171: Chốt Chặn Kiểm Tra Khoảng Trống Cản Vĩ Mô MTF Headroom $\ge 1.5\text{ ATR}$ (`MOM-MTF-HEADROOM`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs`, `arbiter.rs`
  - **Mô tả:** Khi nến 1m bùng nổ nhưng ngay phía trên đầu ($< 0.5\text{ ATR}$) là đỉnh Swing High 4h hoặc dải BB Upper 1h/4h, giá sẽ đâm sầm vào tường cản cứng và dội ngược lại tạo Bull Trap. Bắt buộc kiểm tra khoảng trống dư địa (Headroom): $\text{Distance to MTF Resistance} \ge 1.5\text{ ATR}$; nếu không thỏa mãn tự động hủy Proposal.
  - **Unit Tests:**
    - [ ] `test_momentum_ignition_rejects_breakout_directly_into_4h_resistance`
    - [ ] `test_momentum_ignition_allows_breakout_with_wide_headroom`

- [ ] **TODO-AUDIT-NEW-172: Chốt Chặn Lọc Vi Bứt Phá Giả Trong Dải Nén Quá Hẹp (`MOM-MIN-EXPANSION-SPAN`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs`, `matrix.rs`
  - **Mô tả:** Khi dải BB co thắt cực hẹp ($< 0.10\%$), chỉ vài bước giá nhảy ngẫu nhiên của 1 lệnh nhỏ cũng làm nến đóng ngoài dải BB Upper dù không có dòng tiền thật. Yêu cầu nến kích hỏa phải có chiều dài thực tế tối thiểu $\text{Range} \ge 0.70 \times ATR$ và độ mở dải biến động $\Delta(\text{Band Width}) > 0$.
  - **Unit Tests:**
    - [ ] `test_momentum_ignition_rejects_micro_burst_in_dead_band`
    - [ ] `test_momentum_ignition_fires_on_genuine_range_expansion`

- [ ] **TODO-AUDIT-NEW-173: Cơ Chế Chốt Lời Co Giãn Xung Lực Nhanh (Impulse Profit Expansion Lock) Và Partial TP Cho Momentum (`MOM-IMPULSE-PROFIT-LOCK`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `config.rs`
  - **Mô tả:** Momentum Ignition là chiến thuật xung lực có thời gian suy hao năng lượng cực nhanh. Ép lệnh Momentum dùng chung logic Trailing Stop chậm của TrendPB khiến 80% lệnh đang lãi $+20\% \sim +30\%$ ROE bị tuột về hòa vốn. Triển khai nhánh riêng cho `Playbook::MomentumIgnition`: Khi đạt $\text{MFE} \ge +1.5\text{ ATR}$ $\to$ dời ngay SL lên $+1.0\text{ ATR}$; khi đạt $+2.5\text{ ATR}$ $\to$ chốt lời chủ động 50% vị thế (Partial Take Profit).
  - **Unit Tests:**
    - [ ] `test_momentum_impulse_locks_profit_at_1_5_atr`
    - [ ] `test_momentum_partial_tp_at_2_5_atr`

---

## 🏛️ PHẦN 26: GIA CỐ TOÀN DIỆN CHIẾN ĐỘI ABSORPTION (NOTES 24, 25, 32)

- [ ] **TODO-AUDIT-NEW-174: Chốt Chặn Sàn Khối Lượng Tuyệt Đối Chống Khối Lượng Ma Giờ Thanh Khoản Mỏng Trong Absorption (`ABS-PHANTOM-VOL-FLOOR`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:60-70`, `config.rs`
  - **Mô tả:** Vào đêm khuya hoặc cuối tuần, thanh khoản mỏng, chỉ cần một vài lệnh nhỏ khớp giá đã đẩy $Z_{\text{vol}}$ vọt lên $+4.0\sigma$ do khối lượng trung bình $\mu_{\text{vol}}$ quá nhỏ, bẫy scanner vào lệnh cản tàu. Bổ sung ngưỡng sàn khối lượng danh nghĩa tối thiểu: `quote_volume >= min_quote_volume_usd` và từ chối phát tín hiệu nếu không đạt sàn.
  - **Unit Tests:**
    - [ ] `test_absorption_rejects_phantom_volume_in_illiquid_periods`

- [ ] **TODO-AUDIT-NEW-175: Chốt Chặn Vùng Nén Bị "Hóa Đá" Kéo Dài (Frozen Squeeze Stagnation Watchdog) Trong Absorption (`ABS-FROZEN-SQUEEZE-WATCHDOG`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`, `matrix.rs`
  - **Mô tả:** Khi dải Bollinger Bands nằm lọt thỏm trong Keltner Channel suốt hàng trăm nến liên tiếp, `squeeze_intensity` bão hòa ở mức 1.0 nhưng thị trường hoàn toàn cạn kiệt động năng (Zero Kinetic Energy). Theo dõi biến đếm `continuous_squeeze_bars`: Nếu độ nén kéo dài quá 80 nến 1m mà không có đột phá volume, hạ điểm tự tin hoặc tạm khóa mở lệnh mới.
  - **Unit Tests:**
    - [ ] `test_absorption_rejects_frozen_squeeze_prolonged`

- [x] **TODO-AUDIT-NEW-176: Chốt Chặn Biên Độ Nén Nhỏ Hơn Chi Phí Sàn 2 Chiều (Sub-Frictional Compression Filter) Trong Absorption (`ABS-SUB-FRICTIONAL-FILTER`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs:79-90`
  - **Mô tả:** Khi dải nén quá hẹp, độ rộng dải Bollinger $\text{BB}_{\text{Width}} < 0.20\%$ giá. Trong khi đó, phí sàn 2 chiều + trượt giá ngốn mất $0.13\%$ giá ($2.60\%$ ROE tại 20x). Tiềm năng hồi quy về trục giữa KAMA chỉ đạt $0.10\%$ giá $\implies$ Lệnh chắc chắn bị lỗ ròng sau phí. Kiểm tra điều kiện biên độ tối thiểu: $\text{BB}_{\text{Width}} \ge 2.5 \times \text{Roundtrip Friction} \approx 0.35\%$.
  - **Unit Tests:**
    - [x] `test_absorption_rejects_sub_frictional_bandwidth`

- [ ] **TODO-AUDIT-NEW-177: Chốt Chặn Xung Đột Đa Khung Thời Gian (15m Squeeze vs 1h/4h Macro Expansion Contradiction) Trong Absorption (`ABS-MACRO-EXPANSION-VETO`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`, `matrix.rs`
  - **Mô tả:** Khung 15m đang nén Squeeze nhưng khung 1h hoặc 4h đang trong pha Mở rộng Bùng nổ (Macro Expansion) theo chiều đối nghịch. Cú nén vi mô chỉ là nhịp dừng tạm thời trước khi sóng thần vĩ mô cuốn phăng toàn bộ vùng hỗ trợ. Nếu khung 1h/4h đang có $KER > 0.60$ và độ dốc KAMA mạnh ngược chiều đề xuất $\implies$ VETO tín hiệu Absorption.
  - **Unit Tests:**
    - [ ] `test_absorption_blocks_during_macro_expansion_counter_trend`

---

## 🏛️ PHẦN 27: GIA CỐ TOÀN DIỆN CHIẾN ĐỘI TREND PULLBACK (NOTES 34, 39, 43, 44)

- [x] **TODO-AUDIT-NEW-178: Thay Thế Boundary Guard 5m Bằng Dải Vi Mô 1m Trong `Trend_PB` (`H-TREND-PB-BOUNDARY-5M`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:117-120, 174-177`, `config.rs:690-695, 1448-1450`
  - **Mô tả:** [ĐÃ HOÀN THÀNH] Hiệu chuẩn chốt chặn biên độ thống kê `max_boundary_z` từ `[0.20, 0.80]` (mặc định 0.50) sang `[0.50, 3.00]`, step `0.10` (26 nấc), mặc định `1.60` (tiệm cận trần xu hướng tuyến tính $\sqrt{3}\sigma \approx 1.732\sigma$). Giải phóng hoàn toàn các điểm vào lệnh thuận xu hướng mạnh trong vùng độ trôi tự nhiên $Z \in [+0.60\sigma, +1.50\sigma]$, đồng thời chặn đứng rủi ro mua đu đỉnh Climax phân phối ($Z > 1.60\sigma$) và bán đuổi đáy hoảng loạn ($Z < -1.60\sigma$).
  - **Unit Tests:**
    - [x] `test_trend_pb_allows_entry_in_strong_macro_uptrend`
    - [x] `test_trend_pb_allows_short_entry_in_strong_macro_downtrend`

- [ ] **TODO-AUDIT-NEW-179: Xác Thực Bất Đối Xứng Khối Lượng & Chốt Chặn Cạn Cung No-Supply Trong Nhịp Hồi (`TREND-VOL-ASYMMETRY`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:60-80`
  - **Mô tả:** Bản chất của một nhịp hồi lành mạnh là khối lượng phải cạn kiệt ($Vol_{\text{pullback}} \le 0.80 \times SMA_{\text{vol}}$) để chứng minh phe bán không có nguồn cung để xả lũ (No-Supply). Nếu giá rơi về KAMA với volume bùng nổ ($Vol \ge 1.5 \times SMA_{\text{vol}}$) $\implies$ VETO hủy lệnh Long ngay lập tức để phòng ngừa phân phối đỉnh và gãy xu hướng.
  - **Unit Tests:**
    - [ ] `test_trend_pb_rejects_high_volume_distribution_retrace`
    - [ ] `test_trend_pb_accepts_volume_dry_up`

- [ ] **TODO-AUDIT-NEW-180: Chốt Chặn Giới Hạn Biên Độ Thoái Lui ($L_{retrace} \le 0.618 \times L_{impulse}$) & So Sánh Vận Tốc ($v_{retrace} < v_{impulse}$) (`TREND-MAX-RETRACEMENT-RATIO`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`
  - **Mô tả:** Sóng hồi không được xâm thực quá 61.8% chiều dài bước sóng đẩy trước đó ($L_{\text{retrace}} \le 0.618 \times L_{\text{impulse}}$). Đồng thời, vận tốc rơi của nhịp hồi ($\frac{\Delta P}{\Delta t}$) bắt buộc phải nhỏ hơn vận tốc của sóng đẩy ($v_{\text{retrace}} < v_{\text{impulse}}$). Nếu giá sập hình chữ V với vận tốc cực lớn $\implies$ Hủy bỏ trạng thái Pullback.
  - **Unit Tests:**
    - [ ] `test_trend_pb_rejects_deep_retracement_exceeding_618`
    - [ ] `test_trend_pb_rejects_flash_velocity_inversion`

- [ ] **TODO-AUDIT-NEW-181: Độ Sâu Nhịp Hồi Thích Ứng Cho Supertrend Dốc Đứng (`TREND-ADAPTIVE-DEPTH`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`, `matrix.rs`
  - **Mô tả:** Khi thị trường bước vào Supertrend dốc đứng ($|KAMA\_Slope| \ge 0.60$, $ADX > 35$), giá chỉ thoái lui nông (20%–38.2% Fibo, bám dải ngoài BB/EMA9) mà KHÔNG BAO GIỜ chạm tới KAMA Midline. Bổ sung cơ chế Adaptive Retracement: Khi Trend cực mạnh, cho phép kích hoạt điểm vào tại mốc thoái lui nông ($EMA_{\text{fast}}$ / Swing Support vi mô) với đệm SL co hẹp.
  - **Unit Tests:**
    - [ ] `test_trend_pb_adapts_to_shallow_supertrend_pullback`
    - [ ] `test_trend_pb_requires_deep_kama_touch_in_moderate_trend`

- [ ] **TODO-AUDIT-NEW-182: Chốt Chặn Đâm Thủng KAMA $> -0.5\text{ ATR}$ & Phòng Thủ Breakeven Khi Gặp Từ Chối Tại Đỉnh Cũ (`TREND-PIERCING-AND-OVERHEAD-DEFENSE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`, `risk.rs`
  - **Mô tả:** Nếu nhịp hồi đóng cửa bên dưới đường KAMA Midline quá $-0.5\text{ ATR}$, hủy bỏ trạng thái Pullback. Khi giá sau nhịp hồi tăng trở lại tiếp cận đỉnh cũ mà xuất hiện nến bị từ chối mạnh ($CLV \le -0.40$), tự động kích hoạt dời Stop Loss về hòa vốn ngay lập tức (Breakeven Defense).
  - **Unit Tests:**
    - [ ] `test_trend_pb_rejects_kama_piercing_below_half_atr`
    - [ ] `test_trend_pb_be_defense_on_overhead_rejection`

- [ ] **TODO-AUDIT-NEW-183: Giới Hạn Thời Gian Nhịp Hồi ($T_{retrace} \le 3 \times T_{impulse}$) & TrendPB Stagnation Exit TTL 6 Nến (`TREND-TEMPORAL-STAGNATION-EXIT`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`, `risk.rs`
  - **Mô tả:** Số lượng nến của nhịp hồi không được vượt quá gấp 3 lần số lượng nến của bước sóng đẩy ($T_{\text{retrace}} \le 3 \times T_{\text{impulse}}$). Nếu sau $N \le 6$ nến kể từ khi khớp lệnh mà giá không tạo được đỉnh cao hơn (Higher High), tự động kích hoạt Stagnation Exit đóng lệnh tại hòa vốn.
  - **Unit Tests:**
    - [ ] `test_trend_pb_temporal_ratio_guard_rejects_dragged_pullback`
    - [ ] `test_trend_pb_stagnation_exit_6_bars`

- [ ] **TODO-AUDIT-NEW-184: Cơ Chế Cắt Lỗ Cấu Trúc Khi Gãy Cấu Trúc Thị Trường (Market Structure Break - MSB Exit Thủng Higher Low) (`TREND-MSB-EXIT`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `arbiter.rs`
  - **Mô tả:** Khi đang nắm giữ vị thế thuận xu hướng, nếu nến đóng cửa xuyên thủng đáy sóng hồi gần nhất ($Higher\ Low$ - MSB / CHoCH), xu hướng đã chính thức tử trận. Thay vì thụ động chờ giá rơi tiếp để quét dính Trailing Stop ATR cách xa, Risk Manager kích hoạt Structural Invalidation Exit đóng vị thế ngay lập tức tại giá đóng nến $Close[i]$ để bảo toàn phần lớn vốn.
  - **Unit Tests:**
    - [ ] `test_risk_manager_triggers_msb_exit_on_higher_low_breach`
    - [ ] `test_risk_manager_holds_trend_position_while_higher_low_intact`

- [ ] **TODO-AUDIT-NEW-185: Chốt Chặn R:R Kỳ Vọng TrendPB $\ge 1.5$ & Ngoại Lệ Squeeze Cho Mô Hình Cờ Tăng Bull Flag (`TREND-RR-HURDLE-AND-FLAG-EXCEPTION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`
  - **Mô tả:** Bắt buộc tỷ lệ kỳ vọng $\frac{\text{Distance to Prior High}}{\text{Distance to SL}} \ge 1.5$. Đồng thời, mở khóa ngoại lệ cho phép mở lệnh trong vùng Squeeze nếu KAMA slope vẫn dương mạnh và giá nằm ở nửa trên dải nén (Mô hình cờ tiếp diễn Bull Flag).
  - **Unit Tests:**
    - [ ] `test_trend_pb_objective_rr_hurdle_ge_1_5`
    - [ ] `test_trend_pb_allows_bull_flag_continuation_squeeze`

- [ ] **TODO-AUDIT-NEW-186: Triển Khai Kiểm Tra Nhịp Hồi Phức Tạp Đa Sóng ABC (Two-Leg Retracement Structure) Cho TrendPB (`TREND-ABC-STRUCTURE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`
  - **Mô tả:** Nhịp hiệu chỉnh thường có cấu trúc $A-B-C$. Sóng $B$ tạo bẫy tăng giả (Headfake). Nhận diện cấu trúc 2 nhịp thoái lui để chỉ kích hoạt lệnh khi sóng $C$ đã hoàn tất kiểm định hoặc có nến bứt phá qua đỉnh sóng $B$.
  - **Unit Tests:**
    - [ ] `test_trend_pb_two_leg_retracement_abc_awareness`

- [ ] **TODO-AUDIT-NEW-187: Thiết Kế Playbook Tiếp Diễn Xu Hướng Động Năng Runaway Supertrend Continuation (`TREND-RUNAWAY-CONTINUATION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/`, `arbiter.rs`
  - **Mô tả:** Lấp đầy khoảng trống lớn nhất của hệ thống: Khi giá bơm/xả 10–20 ATR dốc đứng mà không hồi về KAMA và không có Squeeze trước đó. Nhận diện trạng thái Runaway qua Kaufman $ER \ge 0.75$, $ADX \ge 35$ và $Close$ bám dải ngoài; kích hoạt lệnh tiếp diễn vi mô khi có nến tích lũy cạn volume 1m (Micro Consolidation Breakout) với Trailing SL siết chặt 1.0 ATR.
  - **Unit Tests:**
    - [ ] `test_runaway_continuation_captures_parabolic_surge`
    - [ ] `test_runaway_continuation_refuses_choppy_market`

---

## 🏛️ PHẦN 28: GIA CỐ TOÀN DIỆN CHIẾN ĐỘI REVERSAL (NOTES 33, 41, 42)

- [x] **TODO-AUDIT-NEW-188: Triển Khai Kiến Trúc Dual-Layer TP (Static Limit Anchor $TP_0$ + Unified Adaptive TTP) Cho Reversal Chống KAMA Trôi Dốc Đè Bẹp TP (`REV-MONOTONIC-TP-FLOOR`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `scanners/reversal.rs`
  - **Mô tả:** [ĐÃ HOÀN THÀNH] Triệt tiêu tận gốc hiện tượng KAMA Midline trôi dốc đè bẹp Take Profit của Reversal bằng kiến trúc 2 tầng chuẩn mực:
    1. *Tầng Sàn (Binance Orderbook):* Giữ nguyên $TP_0 = \text{kama\_mid}_{t_0}$ làm **Static Limit Anchor** bất biến trên sàn (bảo toàn tỷ lệ $R:R \ge 1.2$ đã kiểm định ở `scan()`, tối ưu Maker fee, 0 trượt giá khi có râu nến quét).
    2. *Tầng RAM (Helios Engine):* Mở khóa `is_ttp_eligible = true` trong `risk.rs` cho toàn bộ Playbook; kích hoạt Trailing Take Profit bám đỉnh (Peak-Callback $0.8\text{ ATR}$) và Tiered Profit Ratchet khi vị thế có lãi lớn ($\ge 15\%$ ROE) rồi quay đầu hụt hơi trước khi chạm $TP_0$.
  - **Unit Tests:**
    - [x] `test_reversal_static_tp_anchor_immune_to_kama_downward_drift`
    - [x] `test_reversal_ttp_locks_profit_before_hitting_tp0`

- [ ] **TODO-AUDIT-NEW-189: Triển Khai Chốt Chặn Phá Vỡ Cấu Trúc Secondary Test (Structural Invalidation Guard Cắt Lỗ Khi Thủng $Low_{SC}$) (`REV-STRUCTURAL-INVALIDATION`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`, `risk.rs`
  - **Mô tả:** Trong cấu trúc Wyckoff, đáy thấp nhất của nhịp cao trào bán tháo ($Low_{SC}$) là mốc bất khả xâm phạm. Nếu nến sau đóng cửa đâm thủng mốc này ($Close < Low_{SC}$), toàn bộ tiền đề đảo chiều bị hủy bỏ hoàn toàn; hệ thống kích hoạt Emergency Exit cắt lỗ ngay lập tức, cấm tuyệt đối việc gồng lỗ hoặc nhồi lệnh bắt đáy trung bình giá.
  - **Unit Tests:**
    - [ ] `test_reversal_structural_invalidation_on_sc_breakdown`
    - [ ] `test_reversal_aborts_on_failed_secondary_test`

- [x] **TODO-AUDIT-NEW-190: Bổ Sung Tiêu Chuẩn Râu Nến Từ Chối Giá Cực Trị (Excess Wick Requirement $CLV \ge 0.50$, Cấm Bắt Đáy Poor Low) (`REV-EXCESS-WICK-GATE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`
  - **Mô tả:** Cấm bắt đáy khi nhịp chạm biên cực trị tạo thành một đáy phẳng lì không có râu nến từ chối ($Poor\ Low / Zero\ Excess$). Bắt buộc nến chạm biên phải có râu dưới rút chân tối thiểu $CLV \ge 0.50$ và chiều dài râu dưới $\ge 0.25 \times \text{Body}$ để chứng minh phiên đấu giá tại đáy đã hoàn tất.
  - **Unit Tests:**
    - [x] `test_reversal_rejects_poor_low_without_excess_wick`
    - [x] `test_reversal_approves_when_clv_ge_0_50_with_strong_wick`

- [x] **TODO-AUDIT-NEW-191: Cổng Chặn Sức Mạnh Xu Hướng Vĩ Mô (Macro Trend Invalidation Gate Cho Reversal: Cấm Long Khi KAMA 1h Dốc Mạnh Và ADX 1h > 35) (`REV-MACRO-GATE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`, `config.rs`, `core_v3/mod.rs`
  - **Mô tả:** Đã kích hoạt `matrix` và thiết lập Cổng Chặn Phòng Vệ Kép (Dual Macro Trend Hurdle - Logic OR). Bổ sung 2 gen DNA toàn năng `max_adverse_kama_slope` (0.02..0.80, step 0.02) và `max_adverse_adx` (15.0..75.0, step 5.0) vào `TacticalReversalParams`, nâng `DNA_WIDTH` lên 112 chiều. VETO Long khi KAMA dốc cắm đứng hoặc KAMA âm kèm ADX bão xả lũ; VETO Short khi KAMA phóng đứng hoặc KAMA dương kèm ADX bão tăng giá.
  - **Unit Tests:**
    - [x] `test_reversal_kama_slope_adverse_rejection`
    - [x] `test_reversal_adx_hurricane_selling_rejection`
    - [x] `test_reversal_adx_hurricane_buying_rejection`

- [ ] **TODO-AUDIT-NEW-192: Cơ Chế Thoát Lệnh Khi Hồi Quy Đàn Hồi Thất Bại (Elastic Snap-back Stagnation Exit TTL 5 Nến) & Cầu Dao Cooldown 2 Lệnh Thua Liên Tiếp (`REV-ELASTIC-STAGNATION-AND-COOLDOWN`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `reversal.rs`, `arbiter.rs`
  - **Mô tả:** Bản chất của Reversal là cú giật đàn hồi nhanh như sợi dây chun về trục KAMA Midline. Nếu sau khi mở vị thế mà 5 cây nến tiếp theo giá không giật mạnh về phía KAMA mà nằm bẹp đi ngang tại đáy (Stagnation Drift), kích hoạt Elastic Stagnation Exit đóng lệnh ở mức lỗ vi mô ($0.2R$). Sau 2 lệnh Reversal thua liên tiếp, tự động đóng băng mở lệnh Reversal trong 15 nến tiếp theo để né bão phân kỳ kéo dài.
  - **Unit Tests:**
    - [ ] `test_reversal_stagnation_exit_ttl_5_bars`
    - [ ] `test_reversal_consecutive_losses_circuit_breaker`

- [ ] **TODO-AUDIT-NEW-193: Triệt Tiêu Bộ Lọc MFI Nhị Phân Trễ Pha, Chuyển Sang Hàm Làm Mềm Điểm Tự Tin Liên Tục (`REV-SOFT-MFI-WEIGHT`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs:60-65`
  - **Mô tả:** Điều kiện nhị phân `mfi > mfi_thresh` loại bỏ oan uổng nhiều nến rút chân hoàn hảo chỉ vì MFI trễ pha 1–2 nến. Chuyển MFI từ chốt chặn nhị phân sang trọng số làm mềm điểm tự tin liên tục.
  - **Unit Tests:**
    - [ ] `test_reversal_softens_mfi_threshold_instead_of_binary_veto`

---

## 🏛️ PHẦN 29: TÁI THIẾT KẾ THỊ TRƯỜNG ĐI NGANG RANGE-BOUND (NOTES 37, 38, 45, 46)

- [ ] **TODO-AUDIT-NEW-194: Tái Thiết Kế `range_bound.rs` Thay Thế `range_osc.rs` Dựa Trên Hộp Cấu Trúc Cố Định ($VAH/VAL/POC$) (`RNG-ARCH-STRUCTURAL-BOX`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_bound.rs`, `arbiter.rs`
  - **Mô tả:** Loại bỏ hoàn toàn tư duy bám theo dải Bollinger Bands động 20 nến co giãn méo mó. Xây dựng Hộp Cấu Trúc Cố Định (Fixed Structural Box) xác lập biên trên $Range_{\text{High}}$ ($VAH$) và biên dưới $Range_{\text{Low}}$ ($VAL$) cùng trục giá trị công bằng $POC$ từ các cực trị đã được kiểm định tối thiểu 2 lần (KDE / Donchian phẳng).
  - **Unit Tests:**
    - [ ] `test_range_bound_box_construction_from_extrema`
    - [ ] `test_range_bound_identifies_fixed_support_resistance`

- [ ] **TODO-AUDIT-NEW-195: Tháo Gỡ Bẫy "Kẹp Trong Dải" Bằng Cơ Chế Liquidity Sweep Re-entry ($CLV \ge 0.50$ Sau Spring/Upthrust) (`RNG-SPRING-LIQUIDITY-SWEEP`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_bound.rs`
  - **Mô tả:** Xóa bỏ điều kiện `close <= bb_m && close >= bb_l` (vốn ép điểm = 0 khi nến quét râu ngoài biên); cho phép và ưu tiên bắt đảo chiều khi giá quét thủng biên hộp để nuốt thanh khoản nhưng nến đóng cửa rút chân trở lại vào bên trong hộp với $|CLV| \ge 0.50$ (Mô hình Wyckoff Spring & Upthrust).
  - **Unit Tests:**
    - [ ] `test_range_bound_spring_liquidity_sweep_reentry`
    - [ ] `test_range_bound_upthrust_reentry`

- [ ] **TODO-AUDIT-NEW-196: Chốt Lời Sang Biên Đối Diện Hộp (Opposite Boundary TP, $R:R \ge 1.5$) & Khóa Lãi Hòa Vốn Tại $POC$ (`RNG-OPPOSITE-BOUNDARY-TP`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_bound.rs`, `risk.rs`
  - **Mô tả:** Khắc phục tử huyệt R:R âm do chốt lời non tại trục giữa ($bb\_middle$). Take Profit thực tế bắt buộc phải hướng sang biên đối diện của hộp ($VAL \to VAH$ hoặc $VAH \to VAL$) để bảo đảm $R:R \ge 1.5$. Trục giữa $POC$ chỉ đóng vai trò là điểm kích hoạt dời Stop Loss về hòa vốn (Break-Even Lock).
  - **Ghi chú Kiến trúc & Định hướng Refactor (Kế thừa Thảo luận Task 188):** Đánh giá chuyển đổi cơ chế TP của `RANGE_OSC` từ Dynamic BB sang **Static Limit Anchor $TP_0$ (tại biên đối diện) + Unified Adaptive TTP trong RAM** (đồng bộ với Reversal `TODO-AUDIT-NEW-188`). Khi hoàn tất, sẽ xóa bỏ hoàn toàn khối Dynamic TP còn lại tại `risk.rs:718-768`, vừa chống co thắt dải làm co hẹp TP, vừa tối ưu Maker fee và bảo toàn $R:R$ trên sổ lệnh sàn.
  - **Unit Tests:**
    - [ ] `test_range_bound_targets_opposite_boundary_rr_ge_1_5`
    - [ ] `test_range_bound_breakeven_lock_at_poc`

- [ ] **TODO-AUDIT-NEW-197: Bộ Đếm Chu Kỳ Sống Của Hộp (Boundary Touch Counter: Chỉ Đánh Lần 2, 3, 4; Cấm Lần 5+) & Anti-Breakout Veto (`RNG-TOUCH-COUNTER-AND-ANTI-BREAKOUT`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_bound.rs`
  - **Mô tả:** Đếm số lần kiểm định biên của hộp: Chỉ cho phép mở lệnh đảo chiều ở lần chạm thứ 2, 3, 4; khóa hoàn toàn giao dịch nảy biên từ lần chạm thứ 5 trở đi vì xác suất vỡ cản $> 80\%$. Nếu nến đóng ngoài biên hộp kèm Volume $\ge 2.5 \times SMA_{\text{vol}}$ và $|CLV| \ge 0.70 \implies$ Hủy bỏ ngay lập tức trạng thái Range để chống cản đầu xe tăng.
  - **Unit Tests:**
    - [ ] `test_range_bound_touch_counter_locks_after_4_touches`
    - [ ] `test_range_bound_breakout_volume_invalidates_range`

- [ ] **TODO-AUDIT-NEW-198: Sàn Chi Phí Sinh Tồn Của Hộp (Minimum Economic Height $\ge 1.2\%$) & Vùng Cấm Trade $40\%$ Quanh POC (`RNG-ECONOMIC-HEIGHT-AND-DEADZONE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_bound.rs`
  - **Mô tả:** Bắt buộc chiều cao hộp $Height_{\text{box}} \ge 3.0 \times (2 \times \text{Fee} + \text{Slippage})$ (tối thiểu $> 1.2\%$ tại 20x), cấm tiệt giao dịch trong các hộp teo tóp dưới sàn ma sát. Cấm mở lệnh nếu vị trí giá nằm trong vùng chết 40% quanh trục giữa $POC$ (Dead-Zone Exclusion Box).
  - **Unit Tests:**
    - [ ] `test_range_bound_rejects_sub_frictional_micro_range`
    - [ ] `test_range_bound_dead_zone_exclusion_around_poc`

- [ ] **TODO-AUDIT-NEW-199: Giao Thức Đón Sóng Sau Đột Phá Giả UTAD (UTAD Re-entry Protocol) (`RNG-UTAD-REENTRY`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_bound.rs`
  - **Mô tả:** Nhận diện cây nến đỏ quay ngược trở lại vào trong Range sau cú rướn bứt phá giả $> 1.5\text{ ATR}$ ngoài Range High (Upthrust After Distribution), kích hoạt lệnh Short đón sóng sập về đáy hộp với Stop Loss đặt ngay trên đỉnh râu nến UTAD.
  - **Unit Tests:**
    - [ ] `test_range_bound_utad_reentry_protocol`

---

## 🏛️ PHẦN 30: DỌN DẸP MÃ NGUỒN, NỢ KỸ THUẬT & TỐI ƯU HÓA HIỆU NĂNG (NOTES 06, 16, 27, 48)

- [ ] **TODO-AUDIT-NEW-200: Đưa `VolMaParams` Vào Cấu Hình DNA Có Thể Tối Ưu Hóa (`H-CFG-FROZEN-VOLMA`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:216-225`
  - **Mô tả:** Struct `VolMaParams` hardcode `period = 20`, `timeframe = 1m`, làm đóng băng thước đo khối lượng baseline, gây méo mó tỷ lệ kiệt sức/bùng nổ volume trên các tài sản có thanh khoản và độ biến động khác nhau. Bổ sung macro `#[derive(DnaMappable)]` cho `VolMaParams`, mở dải period $[10, 50]$ cho GA tối ưu.
  - **Unit Tests:**
    - [ ] `test_vol_ma_params_dna_mapping`

- [ ] **TODO-AUDIT-NEW-201: Chuẩn Hóa Lưới Độ Mịn Bước Nhảy DNA Cho Các Tham Số Vi Cấu Trúc Nến (`DNA-STEP-QUANT-MISMATCH`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs`
  - **Mô tả:** Một số tham số vi cấu trúc nến cực kỳ nhạy cảm (`min_clv`, `spring_tolerance`, `struct_sl_buffer_atr`) hiện đang bị phân bổ bước nhảy quá thô (ví dụ: `min_clv` step 0.05 từ 0.05 đến 0.50 chỉ có 10 bins), khiến GA dễ nhảy vọt qua điểm tối ưu toàn cục. Nâng độ mịn bước nhảy cho các tham số nhạy cảm: `min_clv` (step 0.01), `spring_tolerance` (step 0.01), `struct_sl_buffer_atr` (step 0.02).
  - **Unit Tests:**
    - [ ] `test_dna_mapping_step_granularity_for_microstructural_params`

- [ ] **TODO-AUDIT-NEW-202: Đồng Bộ Hóa Default Timeframe Của Keltner (15m) Khớp Với BBands Trong `config.rs` (`H-CFG-11`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:148, 172, 192`
  - **Mô tả:** `BbandsParams::default()` chọn 15m nhưng `default_timeframe()` của KeltnerParams lại trả về 5m, phụ thuộc vào lệnh ép đồng bộ thủ công trong `mod.rs`. Đồng bộ hàm fallback của Keltner về 15m để cấu hình độc lập luôn an toàn.
  - **Unit Tests:**
    - [ ] `test_keltner_and_bbands_default_timeframe_coherence`

- [ ] **TODO-AUDIT-NEW-203: Dọn Dẹp Mã Chết `HeliosBridge::analyze` 125 Dòng Khỏi Codebase (`TODO-BRG-08`)**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:360-485`
  - **Mô tả:** Hàm `HeliosBridge::analyze` dài 125 dòng mã là tàn dư cũ của bản thiết kế ban đầu, hoàn toàn không được gọi ở bất kỳ luồng nào trong dự án, gây ô nhiễm mã nguồn. Xóa bỏ hoàn toàn hàm này.
  - **Unit Tests:**
    - [ ] `test_helios_bridge_compilation_without_analyze`

- [ ] **TODO-AUDIT-NEW-204: Đóng Gói `core_v2` Cũ Vào `legacy/` & Bóc Tách 1,500 Dòng Test Khỏi `core_v3/mod.rs` (`CLEANUP-V2-AND-MOD-TESTS`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v2/`, `strategies_backtests/helios/src/strategy/core_v3/mod.rs`
  - **Mô tả:** Thư mục `core_v2` cũ không còn sử dụng gây nhiễu code; file `core_v3/mod.rs` dài hơn 1,800 dòng do nhét toàn bộ test cases vào cuối file, làm giảm tính công thái học và cản trở bảo trì. Di chuyển `core_v2` vào `legacy/core_v2` và bóc tách toàn bộ `mod tests` trong `core_v3/mod.rs` sang file `strategies_backtests/helios/tests/core_v3_integration_tests.rs`.
  - **Unit Tests:**
    - [ ] `cargo test -p helios` xác nhận 100% tests pass nguyên vẹn sau khi di chuyển.

- [ ] **TODO-AUDIT-NEW-205: Thiết Kế Máy Trạng Thái Vòng Đời Vị Thế 3 Pha & Động Cơ Thăng Cấp Siêu Sóng (`RISK-3STAGE-LIFECYCLE`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `config.rs`, `mod.rs`
  - **Mô tả:** [PHƯƠNG ÁN D / NOTE 56] Thiết kế kiến trúc quản trị thoát lệnh và chốt lời đa tầng dựa trên vòng đời vị thế 3 pha (3-Stage Lifecycle Exit Engine): (1) Pha 1 - Thoát Ma Sát (Friction Escape & Initial SL đến khi đạt EV Hurdle); (2) Pha 2 - Đấu Thầu Giá Trị Công Bằng (Fair Value Limit Anchor tại KAMA/POC/BBands + Dynamic BE Ratchet); (3) Pha 3 - Thăng Cấp Thả Diều Siêu Sóng (Momentum Runner Promotion: khi nến bứt phá qua vạch đích với Volume/ADX bùng nổ, hủy trần TP tĩnh và chuyển sang Trailing Take Profit bám đỉnh). Tham số hóa các ngưỡng chuyển pha vào DNA Mappable của Risk Manager.
  - **Unit Tests:**
    - [ ] `test_lifecycle_friction_escape_to_break_even`
    - [ ] `test_lifecycle_fair_value_limit_anchor_execution`
    - [ ] `test_lifecycle_momentum_runner_promotion_on_climax_breakout`
