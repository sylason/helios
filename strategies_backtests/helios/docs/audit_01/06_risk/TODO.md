# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ: MÔ-ĐUN QUẢN TRỊ RỦI RO (RISK MANAGEMENT)

> **Mô-đun:** `Helios Core V3.3 - RiskManager & Stop Management`  
> **Tài liệu kiểm toán gốc:** [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_risk/technical_audit.md)  
> **Cập nhật lần cuối:** 19/09/2026  

---

## 1. 🔴 MỨC ĐỘ CRITICAL (BẮT BUỘC XỬ LÝ TRƯỚC KHI CHẠY LIVE)

- [ ] **TODO-RISK-01: Triển khai gửi lệnh Algo Take Profit lên Binance Futures (`OrderExecutor`)**
  - **Tệp liên đới:** [`trading_bot/bot_engine/src/execution/order_executor.rs:241-260`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L241-L260), [`live_engine.rs:92-95`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L92-L95)
  - **Vấn đề:** Hiện tại `OrderExecutor` chỉ gửi lệnh Algo `STOP_MARKET` cho SL, hoàn toàn bỏ qua TP (`warn!` rồi bỏ qua). Điều này khiến các lệnh có TP cố định (`RANGE_OSC`, `REVERSAL`) không bao giờ được chốt lời trên sàn Live!
  - **Giải pháp:** Khởi tạo thêm lệnh Algo `TAKE_PROFIT_MARKET` hoặc Limit Order Reduce-Only với mức giá TP chỉ định.

- [ ] **TODO-RISK-02: Khống chế dải Stop Loss của Absorption để chặn đứng Drawdown -26.6%**
  - **Tệp liên đới:** [`config.rs:554`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L554), [`absorption.rs:98-108`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L98-L108)
  - **Vấn đề:** `struct_sl_buffer_atr` có dải `[0.25, 10.00]`. Khi GA chọn 10 ATR cộng thêm khoảng cách 2-3 ATR tới `bb_lower`, SL cách tới 12-13 ATR (~7.5% giá), đòn bẩy 20x làm bay 30% tài khoản khi cắn Hard SL!
  - **Giải pháp:** Hạ trần `struct_sl_buffer_atr` từ 10.0 về tối đa `2.0 - 2.5 ATR`, kẹp cứng `clamp_sl_price` theo `max_initial_sl_pct` ngay tại thời điểm tạo proposal.

- [ ] **TODO-RISK-03: Sửa lỗi Flash Stop-Out của Momentum Ignition (Nới rộng SL tối thiểu)**
  - **Tệp liên đới:** [`momentum_ignition.rs:65-74`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L65-L74)
  - **Vấn đề:** SL đặt tại `Low/High +/- 0.10*ATR` quá sát (~0.4% giá), bị râu nến vi mô 1m quét dính Hard SL trong 2-3 nến.
  - **Giải pháp:** Tăng SL buffer tối thiểu lên `0.50 - 1.00 ATR` và tích hợp `Wick Rejection Filter` (chống mua đỉnh râu nến Shooting Star).

- [ ] **TODO-RISK-04: Vá lỗ hổng IEEE 754 NaN Bypass trong `calculate_size`**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:11-26`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L11-L26)
  - **Vấn đề:** Phép so sánh `<= 0.0` trả về `false` khi `equity.is_nan()`. Hàm tiếp tục tính toán và trả về `NaN.min(0.25) = 0.25`, cấp margin 25% cho tài khoản có vốn bị lỗi!
  - **Giải pháp:** Bổ sung kiểm tra `.is_nan()` tường minh ngay đầu hàm cho tất cả tham số float.

- [x] **TODO-RISK-09: Thiết kế Cầu Dao Arbiter Gatekeeper Đa Tầng Đối Phó Chaos Whipsaw & Volatility Flatline (`gatekeeper.rs`, `arbiter.rs`)**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs), [`arbiter.rs:160-205`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L160-L205), [`config.rs:303-360`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L303-L360)
  - **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH 100% (TẦNG 1)**.
  - **Chi tiết đã triển khai:**
    1. Tạo module `gatekeeper.rs` độc lập chuẩn SOLID, thực thi $O(1)$ Early Exit trước khi quét Scanners.
    2. Chốt chặn Chaos Whipsaw với Stateless Window Lookback ($K = 5$ bars) và cơ chế Hysteresis Schmitt Trigger delta 0.10.
    3. Chốt chặn Volatility Flatline bù phí sàn với $NATR < \text{min\_natr\_threshold}$ (DNA `[0.0008, 0.0030]`).
    4. Tinh giản chấm điểm tự tin sang Pure Confidence ($Raw = Conf$), loại bỏ hoàn toàn phạt mềm hình thức cũ.
    5. Đã verify 100% với 6 unit tests riêng cho Gatekeeper, 8 tests cho Arbiter, và toàn bộ 77 tests của Helios PASS.

- [x] **TODO-RISK-10: Kẹp Cứng Trần Stop Loss $\le 15\%$ ROE Toàn Cục & Structural Risk Veto (`risk.rs`, `config.rs`, `arbiter.rs`)**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs), [`config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs), [`arbiter.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs)
  - **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH 100% (TẦNG 3)**.
  - **Chi tiết đã triển khai:**
    1. Thu hẹp dải tìm kiếm DNA của 7 gen `max_initial_sl_pct` (6 Playbooks + 1 Untagged) từ `[0.04, 0.40, step 0.015]` xuống `[0.04, 0.15, step 0.01]` (12 nấc mịn), giữ nguyên `DNA_WIDTH = 102`.
    2. Thiết lập trần bất biến toàn cục `GLOBAL_HARD_MAX_ROE = 0.15` (15% ROE) trong `RiskManager::clamp_sl_price`, bảo vệ tuyệt đối không cho phép bất kỳ lệnh nào thua lỗ $> 15\%$ ROE.
    3. Tích hợp cơ chế Sniper Entry Guard (`StructuralRiskVeto`) tại Arbiter: Nếu khoảng cách từ Entry đến SL cấu trúc vượt quá 15% ROE, proposal bị VETO ngay từ cửa, ghi nhận minh bạch vào Auction Ledger Telemetry.
    4. Đã verify 100% với 5 unit tests mới kiểm chứng: kẹp trần SL, toán học chấp nhận SL cấu trúc, và 2 kịch bản Arbiter VETO / APPROVE.

- [x] **TODO-RISK-11: Thêm Boundary Guard Z-Score Chống Đu Đỉnh & Chop Zone Cho TrendPB (`trend_pb.rs`, `config.rs`)**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs), [`config.rs:449-460`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L449-L460)
  - **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH 100% (TẦNG 2)**.
  - **Chi tiết đã triển khai:**
    1. Bổ sung 2 gen DNA mới vào `TacticalPullbackParams`:
       - `max_boundary_z`: `#[dna(min = 0.20, max = 0.80, step = 0.05)]`, default `0.50` (13 nấc).
       - `max_squeeze_intensity`: `#[dna(min = 0.50, max = 0.90, step = 0.05)]`, default `0.75` (9 nấc).
    2. Chiều dài DNA của Helios mở rộng từ 100 lên **102 chiều**, 100% dynamic qua GA Optimizer, 0% hardcode cảm tính.
    3. Triển khai Boundary Guard trong `trend_pb.rs`:
       - Chiều Long: Cấm khi $Z_{\text{score}} > +max\_boundary\_z$ (chặn đứng mua đu đỉnh phân phối).
       - Chiều Short: Cấm khi $Z_{\text{score}} < -max\_boundary\_z$ (chặn đứng bán đuổi đáy hoảng loạn).
    4. Triển khai Chop Zone Defense: Cấm khi $Squeeze > max\_squeeze\_intensity$ để nhường sân cho `range_osc`.
    5. Đã verify: Cập nhật `test_dna_width_and_parameter_defs_integrity` (102 chiều) và viết 3 unit tests mới độc lập kiểm chứng cả 3 chốt chặn.

- [x] **TODO-RISK-13: Kiến Trúc Telemetry Module & Ma Trận 3 Tầng Phòng Thủ Vốn (`telemetry/`, `cards.rs`, `defense_ledger.rs`, `asset_actor.rs`)**
  - **Tệp liên đới:** [`core_v3/telemetry/`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/telemetry/), [`cards.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/telemetry/cards.rs), [`defense_ledger.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/telemetry/defense_ledger.rs), [`core_v3/mod.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs), [`risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs), [`asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
  - **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH 100%**.
  - **Chi tiết đã triển khai:**
    1. Tách biệt 100% Tầng Trình Diễn (Presentation Layer) khỏi Tầng Nghiệp Vụ (Domain Core): Tạo thư mục mới `core_v3/telemetry/` gồm `cards.rs`, `defense_ledger.rs`, `mod.rs`. Di dời 150 dòng vẽ thẻ ASCII ra khỏi `core_v3/mod.rs`.
    2. Sổ Cái Phòng Thủ Vốn `DefenseLedger`: Theo dõi và ghi nhận toàn diện 3 Tầng:
       - Tầng 1: Đếm số nến né bão Chaos / Flatline, phát hiện cạnh State Transition (Vào bão / Ra bão) kèm thời lượng né bão.
       - Tầng 2: Đếm số lần chặn mua đu đỉnh, bán đuổi đáy, và vùng co thắt Chop Zone của TrendPB.
       - Tầng 3: Đếm số lần bắn hạ lệnh có SL > 15% ROE (Sniper Veto) và số lần kẹp SL thô về trần.
    3. Thẻ Đỏ VETO thời gian thực `format_veto_card`: In trực quan biên bản từ chối lệnh khi vi phạm Gatekeeper hoặc Sniper Guard.
    4. Bảng Sổ Cái Tổng Kết `format_summary_table`: Xuất tự động bảng tổng kết `FINAL DEFENSE LEDGER` cuối phiên chạy qua `AssetActor::log_final_summary`.
    5. Vá dứt điểm lỗ hổng rò rỉ log TTP trong `risk.rs:260, 271` bằng `if _debug_mode`, triệt tiêu nguy cơ nổ log 174 MB trong GA Optimizer.
    6. Đã verify 100%: Toàn bộ 87 tests của `helios` PASS, 39 tests của `bot_engine` PASS, 0 warning, 0 error.

- [ ] **TODO-RISK-12: Bơm Đa Dạng Di Truyền Phá Vỡ Bẫy Hội Tụ Smart Seeding (`prospector_runner.rs`)**
  - **Tệp liên đới:** [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs:320-370`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs#L320-L370)
  - **Vấn đề:** Smart Seeding 0.5% khiến quần thể GA bị giam giữ quanh cực trị địa phương của dữ liệu cũ, In-Sample đạt 232% nhưng OOS âm -21%, dẫn đến 74.5% chu kỳ bị WFE Gate REJECT, bot bị kẹt dùng DNA lỗi thời khi thị trường đổi pha.
  - **Giải pháp:** Khi bị REJECT liên tiếp $\ge 2$ lần, tự động nâng tỷ lệ khám phá (Exploration Mutation Rate) hoặc tạm ngừng nạp hạt giống cũ để giải phóng quần thể tìm kiếm vùng cực trị mới thích nghi với pha thị trường hiện tại.

---

## 2. 🟠 MỨC ĐỘ HIGH (NGUY CƠ TOÁN HỌC & TỶ LỆ R:R THẤP)

- [ ] **TODO-RISK-05: Chốt chặn tỷ lệ R:R tối thiểu cho Reversal Scanner**
  - **Tệp liên đới:** [`reversal.rs:66, 75, 85`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L66)
  - **Vấn đề:** Khi râu nến dài và giá đã hồi phục sát `kama_mid`, $\Delta_{SL}$ rất lớn trong khi $\Delta_{TP}$ rất nhỏ, sinh ra proposal có R:R < 1:0.3.
  - **Giải pháp:** Thêm điều kiện cứng: $\frac{\Delta_{TP}}{\Delta_{SL}} \ge \text{min\_rr\_ratio}$ (ví dụ $\ge 1.0$ hoặc $1.2$). Nếu không thỏa mãn, hủy proposal.

- [ ] **TODO-RISK-06: Xử lý mâu thuẫn giữa `min_order_usd` ($21) và `max_cap_fraction` (0.25)**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:18-26`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L18-L26)
  - **Vấn đề:** Khi vốn quá nhỏ ($2 - $4), lệnh sau khi nâng lên $21 sẽ bị kẹp bởi trần 25% vốn, làm quy mô thực tế tụt xuống $10 < $21, dẫn đến việc sàn Binance từ chối lệnh (`REJECTED: MIN_NOTIONAL`).
  - **Giải pháp:** Nếu sau khi kẹp trần `cap` mà `final_margin * leverage < min_order_usd`, hàm phải từ chối mở lệnh (trả về `0.0`).

---

## 3. 🟡 MỨC ĐỘ MEDIUM (CẢI THIỆN ĐỘ CHÍNH XÁC KẾ TOÁN & MA SÁT THỊ TRƯỜNG)

- [ ] **TODO-RISK-04: Nâng cấp và cấu hình hóa bộ đệm phí sàn `fee_buffer_pct` trong Break-Even**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:327`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L327), [`config.rs:634-771`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L634-L771)
  - **Vấn đề:** Mức đệm `0.0008` (0.08%) đang bị hardcode và không đủ bù đắp phí Taker 2 chiều của sàn Binance Futures (0.10%), chưa kể trượt giá (slippage 0.03% - 0.05%).
  - **Giải pháp:**
    - Đưa trường `pub be_fee_buffer_pct: f64` vào struct `RiskStopParams` trong `config.rs` (giá trị mặc định khuyến nghị: `0.0015` tức 0.15%).
    - Thay thế hardcode `let fee_buffer_pct = 0.0008;` bằng `risk_cfg.be_fee_buffer_pct.max(0.0012)`.

- [ ] **TODO-RISK-05: Nhận thức Market Filter (Step Size, Tick Size, Min Qty) ở tầng định cỡ**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:11`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L11), [`mod.rs:374`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L374)
  - **Vấn đề:** `calculate_size` chỉ tính tỷ lệ liên tục `f64`. Khi xuống sàn, lệnh bị `.floor()` theo `step_size` làm lệch tỷ lệ rủi ro so với backtest.
  - **Giải pháp:** Bổ sung cơ chế kiểm tra bước khối lượng tối thiểu và cảnh báo nếu tỷ lệ cắt giảm do làm tròn vượt quá 10%.

---

## 4. 🟢 MỨC ĐỘ LOW / TECH DEBT (TỐI ƯU HÓA MÃ NGUỒN & KIỂM THỬ)

- [ ] **TODO-RISK-06: Đổi tên và nâng cấp hàm thành `compute_order_size_with_risk_mode`**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:11`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L11)
  - **Mục tiêu:** Hỗ trợ nhiều chế độ định cỡ vốn: `FixedFraction`, `VolatilityAdjusted` (chia theo ATR), hoặc `PlaybookScaled` (quy mô co giãn theo điểm tự tin Arbiter Confidence Score).

- [ ] **TODO-RISK-07: Bổ sung Unit Test cho Edge Case `f64::NAN`**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:558`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L558)
  - **Mục tiêu:** Viết test kiểm tra `calculate_size(f64::NAN, 0.2, 21.0, 20.0, 0.25)` phải trả về `0.0`.

- [ ] **TODO-RISK-08: Telemetry log khi Trailing Take Profit (TTP) bị chặn do Gap Down**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:234, 246`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L234)
  - **Mục tiêu:** Thêm log cảnh báo khi giá quay đầu xuyên thủng `ttp_trigger_price` nhưng lại rơi xuống dưới `min_exit_price` để theo dõi độ trượt giá trong các cú flash crash.
