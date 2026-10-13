# DANH MỤC CÔNG VIỆC (TODO): TÁI CẤU TRÚC KIẾN TRÚC ĐỊNH LƯỢNG & ỔN ĐỊNH HÓA

> **Thư mục:** [`05_systemic_stability_and_quant_architecture_flaws/`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/)  
> **Tài liệu tham chiếu:**  
> - [`01_quant_architecture_flaws_and_stability_blueprint.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/01_quant_architecture_flaws_and_stability_blueprint.md)  
> - [`02_helios_and_trading_bot_architectural_flaws.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/02_helios_and_trading_bot_architectural_flaws.md)  
> **Mục tiêu:** Chuyển đổi hệ thống sang "Lỗ ít, lời ổn định, đủ nhiều, lọc sạch trạng thái nguy hiểm và tối đa hóa lợi nhuận".

---

## 📋 DANH SÁCH NHIỆM VỤ CHI TIẾT

### PHẦN 1: BẢO VỆ VỐN & KHÓA LỢI NHUẬN TỨC THÌ (RISK & EXECUTION)
- [ ] **TODO-STAB-01: Triển khai Tiered Profit Lock & Break-Even Ratchet trong Risk Manager**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs`, `trading_bot/bot_engine/src/execution/paper_trader.rs`
  - **Mô tả:** 
    - Nấc 1: Khi $\text{MFE} \ge +3.5\%$ ROE $\to$ dời SL lên tối thiểu $+1.60\%$ ROE (bảo hiểm đủ phí sàn Binance $20\text{x}$).
    - Nấc 2: Khi $\text{MFE} \ge +8.0\%$ ROE $\to$ khóa cứng mức lãi sàn $+4.00\%$ ROE.
  - **Unit Tests:**
    - [ ] `test_tiered_profit_lock_break_even_at_3_5_roe`
    - [ ] `test_tiered_profit_lock_locks_4_pct_at_8_pct_roe`
    - [ ] `test_anti_atr_expansion_tightening`

- [ ] **TODO-STAB-02: Khử triệt để dao rơi trong Absorption Scanner (`TODO-SCN-02`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs`
  - **Mô tả:** Bổ sung Macro Adverse Slope Veto: cấm Long khi `kama_slope_atr < -max_adverse_slope`, cấm Short khi `kama_slope_atr > max_adverse_slope`.
  - **Unit Tests:**
    - [ ] `test_absorption_rejects_falling_knife_adverse_kama`

---

### PHẦN 2: LỌC TRẠNG THÁI NGUY HIỂM & ĐỒNG THUẬN PHA (GATEKEEPER & SENSORS)
- [ ] **TODO-STAB-03: Triển khai Bộ Phân Loại Chế Độ Thị Trường Cấp Vĩ Mô (Macro Regime Classifier)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`, `matrix.rs`
  - **Mô tả:** Nhận diện pha Chop / Đi ngang không có trend (ADX < 20 hoặc KAMA đi ngang). Khóa hoàn toàn `Trend_PB` và `Momentum_IGN` trong pha này để hệ thống ngồi im (Sit on hands).
  - **Unit Tests:**
    - [ ] `test_regime_classifier_blocks_trend_momentum_in_chop`

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
- [ ] **TODO-STAB-06: Độc quyền hóa Thẩm quyền Risk về `helios::risk` (Khử Split-Brain)**
  - **File:** `trading_bot/bot_engine/src/execution/paper_trader.rs`, `order_executor.rs`, `helios_bridge.rs`
  - **Mô tả:** Đồng bộ 1:1 công thức và logic tính Stop Loss / Take Profit giữa Backtest và Bot Engine.
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

### PHẦN 6: GIẢI CỨU CÁC SCANNER BỊ TÊ LIỆT & ĐẤU THẦU CÔNG BẰNG (`helios`)
- [ ] **TODO-AUDIT-H-01: Hóa Giải Nghịch Lý Khối Lượng và Tái Sinh `reversal.rs`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`
  - **Mô tả:** Bỏ hàm `sat_proximity` ép vol về 0.50; chấp nhận bão khối lượng lớn ($vol\_ratio \ge 1.5$) là tín hiệu cạn kiệt; điều chỉnh tương thích giữa CLV và MFI.
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

