# TODO VÒNG 2 — KIỂM TOÁN RỦI RO HELIOS V3.3
> **Ngày:** 02/10/2026  
> **Gốc phát hiện:** `risk_audit_round2.md`  
> **Nguyên tắc:** Fix theo thứ tự ưu tiên. Mỗi task là baby step độc lập. Test sau mỗi task.

---

## 🔥 CRITICAL — SỬA NGAY TRƯỚC KHI CHẠY TIẾP

- [x] **TASK-CORE-01: Cải Tổ Toàn Diện Cầu Dao Cooldown (Universal Pure TTL & True Break-Even)**
  - **Tài liệu đặc tả:** [`TASK_CORE_01_COOLDOWN_SPEC.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_risk/TASK_CORE_01_COOLDOWN_SPEC.md)
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:892-905` & `trading_bot/bot_engine/src/controller/asset_actor.rs:31-66, 174-248`
  - **Vấn đề:** `#[dna]` cho phép `re_entry_cooldown_bars = 0` làm tê liệt Cầu Dao tại Bot Engine, gây cào lệnh liên thanh 1 phút sau khi dính SL.
  - **Giải pháp:** Bỏ `#[dna]`, quy hoạch `EngineCooldownConfig` (Base 15 bars, Cap 45 bars), phân loại lệnh Hard SL vs True BE dung sai phí sàn.
  - **Trạng thái:** ✅ Đã hoàn thành 100% Bước 1, 2, 3, 4 (Code & Unit Tests Pass 100%).

- [x] **TODO-R2-01: Fix Flash Crash Blind Spot trong `is_be_triggered`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:291`
  - **Vấn đề:** `current_pnl > 0.0 &&` làm tắt Emergency BE Rejection khi nến flash crash đóng dưới Entry, dù `peak_roe` đã chạm ngưỡng an toàn từ trước.
  - **Fix:** Bỏ `current_pnl > 0.0 &&`, duy trì trạng thái kích hoạt BE khi râu nến đã đạt đỉnh an toàn (Ratchet Memory).
  - **Trạng thái:** ✅ Đã hoàn thành 100% & Pass 22/22 tests trong `risk::tests` (bao gồm `test_be_triggered_on_flash_crash`).

- [x] **TODO-WFE-01: Xóa Bỏ Cửa Sau Dung Thứ OOS Âm (Hard Zero-Tolerance WFE Gate)**
  - **File:** `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:405-420`
  - **Vấn đề:** `is_tolerant_pass` cho phép OOS Return âm tới -1.2% nếu In-Sample > 80%, đưa cá thể cào lệnh overfit của Run 2 vào thực tế.
  - **Fix:** Xóa bỏ hoàn toàn `is_tolerant_pass`. Bắt buộc 100%: `val_ret > 0.0%`, `val_trades > 0` và `val_dd <= 15.0%`.
  - **Trạng thái:** ✅ Đã hoàn thành 100% & Pass 7/7 tests trong `prospector_runner::tests` (bao gồm `test_wfe_gate_hard_zero_tolerance`).

- [ ] **TODO-R2-02: Giải quyết Giant Candle Trap cho Anomaly & Momentum**
  - **File:** `risk.rs:15-28` + `anomaly.rs:75,90` + `momentum_ignition.rs:144,151`
  - **Vấn đề:** Nến breakout to → SL xa Entry → ROE > 15% → `is_structural_sl_acceptable` VETO oan → mất tín hiệu tốt nhất.
  - **Phương án cần thảo luận:**
    - **A (De-leverage):** Risk Manager tự giảm `size_fraction` thay vì VETO, giữ ROE ≤ 15%
    - **B (Scanner Clamp):** Scanner kẹp SL: `sl = close - min(swing_dist, N * atr)` với N = DNA gen
  - **Hành động tiếp theo:** Thảo luận với người dùng trước khi triển khai.

- [ ] **TODO-R2-03: Thêm Chaos Filter cho Reversal Scanner**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs:59-82`
  - **Fix:**
    ```rust
    // Ngay đầu hàm scan:
    if i < matrix.chaos_index.len() && matrix.chaos_index[i] > 0.65 {
        return None; // Cấm bắt đáy khi bão chưa tan
    }
    ```
  - **Test:** Unit test với `chaos_index = 0.7` → Signal bị None.

---

## ⚡ HIGH — SỬA SỚM (Sprint tiếp theo)

- [ ] **TODO-R2-05: Fix KAMA Suction Trap trong Trend_PB Scanner**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:122,170`
  - **Vấn đề:** `low.min(kama_mid)` kéo SL sâu xuống KAMA khi KAMA trễ pha, gây VETO oan.
  - **Fix:** Bỏ `.min(kama_mid)`, chỉ dùng `low - (atr * buffer_mult)` cho Long.
  - **Lưu ý:** Kiểm tra `trend_pb.rs` xem có logic Short tương tự không.

- [ ] **TODO-R2-07: Fix State Desync — Cooldown Tracker PnL ảo**
  - **File:** `trading_bot/bot_engine/src/optimizer/helios_bridge.rs:271`
  - **Vấn đề:** `last_closed_trade: None` hardcode → CooldownTracker trong `mod.rs` đọc PnL sai.
  - **Fix:** Truyền `last_closed_trade` thực từ ExecutionEngine vào `evaluate_live_tick`.
  - **Phạm vi:** Cần thay đổi signature `evaluate_live_tick`, cập nhật call sites trong `asset_actor.rs`.

---

## ⚡ MEDIUM — SỬA Ở SPRINT SAU

- [ ] **TODO-R2-04: Fix Range Osc TP Inflation khi BB Squeeze**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs:146,181`
  - **Fix:** Nếu `target_price < close + min_viable_spread`, return `None` thay vì inflate TP.
  - **min_viable_spread:** Dùng `current_atr * 0.3` làm ngưỡng.

- [ ] **TODO-R2-06: Thêm Minimum Distance Guard vào `is_structural_sl_acceptable`**
  - **File:** `risk.rs:15-28`
  - **Fix:** Thêm `current_atr` parameter + kiểm tra `dist_atr < 0.5 ATR`.
  - **Lưu ý:** Phải cập nhật tất cả call sites (arbiter.rs, mod.rs) truyền thêm `current_atr`.

- [ ] **TODO-R2-08: Fix Dummy PnL=0.0 trong Live Bridge**
  - **File:** `helios_bridge.rs:252-258`
  - **Fix:** Tính Unrealized PnL tại Bridge từ `pos.entry_price` và `current_close`.

---

## 🔧 LOW — TECH DEBT (Khi có thời gian)

- [ ] **TODO-R2-09: Fix ClosePosition Reason Misclassification**
  - **File:** `paper_trader.rs:630-639`
  - **Fix:** Thêm `reason: Option<String>` vào `Action::ClosePosition`.
  - **Lưu ý:** Breaking change — cần cập nhật `ferro_trade` enum và tất cả match arms.

- [ ] **TODO-R2-10: Fix High Watermark Amnesia**
  - **File:** `helios_bridge.rs:222-229`, `risk.rs:127-150`
  - **Fix:** Lưu `peak_val` và `peak_bar` vào `TradeState`, truyền thẳng vào `risk.rs`.
  - **Lưu ý:** Thiết kế lại cấu trúc state, ảnh hưởng nhiều module.

- [ ] **TODO-R2-11: Dọn dẹp Bộ Tứ Gen Ma trong Config (`sl_min_dist_pct`, `base_lot_size`, `nuke_leverage`, `mega_trend_max_cycles`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:727,731,739,865`
  - **Vấn đề:** 4 field này không hề được logic nào tiêu thụ, gây nhiễu cấu hình và phình to struct.
  - **Fix:** Xóa bỏ hoặc wire vào nơi cần thiết (chú ý giữ nguyên các field DNA để không đổi `DNA_WIDTH`).

- [ ] **TODO-R2-12: Sửa Default `taker_fee_rate` từ `0.0004` thành `0.0005` (Binance Futures VIP0)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:766`
  - **Vấn đề:** Default 0.04% là Maker hoặc VIP cao, Binance Futures VIP0 chuẩn là 0.05% Taker. Làm `min_safe_be_roe` tính thiếu 16.7%.
  - **Fix:** Đổi `default_taker_fee_rate` trả về `0.0005`.

- [ ] **TODO-R2-13: Thêm Ordering Constraint TSL (`tight ≤ normal ≤ loose`)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:846-855`
  - **Vấn đề:** 3 gen TSL có cùng dải `[0.5, 4.5]` khiến GA Optimizer có thể chọn `tight = 4.3` và `loose = 0.6`, đảo ngược 180 độ logic nghiệp vụ.
  - **Fix:** Thêm sorting tự động trong `update_from_dna` đảm bảo `tsl_atr_tight <= tsl_atr_normal <= tsl_atr_loose`.

- [ ] **TODO-R2-14: Nâng Cận Dưới `ttp_activation_roe` từ 0.05 lên 0.10 (10% ROE)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/config.rs:910`
  - **Vấn đề:** Ở 20x, 5% ROE chỉ là biến động 0.25% giá, kích hoạt TTP quá sớm trong 1-2 nến, làm hỏng trend-following.
  - **Fix:** Điều chỉnh range `#[dna(0.10, 0.50, 0.01)]`.

- [ ] **TODO-R2-15: Bổ sung Guard `sl_price.is_nan()` trong `clamp_sl_price`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:57-92`
  - **Fix:** Thêm `if sl_price.is_nan() || sl_price <= 0.0 { return price; }`.

- [ ] **TODO-R2-16: Nghiên cứu Adaptive `GLOBAL_HARD_MAX_ROE` Cho Đòn Bẩy Cao (>50x)**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:10`
  - **Vấn đề:** 15% ROE tại 100x là $3.75 trên ETH, hẹp hơn cả spread và nến 1m, gây quét sạch SL.
  - **Fix:** Thảo luận công thức scale theo ATR và đòn bẩy.

---

## 🚨 PHÂN HỆ THỰC THI SÀN & CHỐNG SẬP (EXECUTION & CRASH RESILIENCE)

- [ ] **TODO-EXEC-01: Khắc phục Cửa Sổ Trơ Trọi (Naked SL Window 150ms)**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:192-200`
  - **Vấn đề:** Hủy SL cũ rồi mới sleep 150ms để đặt SL mới. Trong 150ms đó không có lệnh bảo vệ nào trên Binance.
  - **Fix:** Đặt SL mới trước, nhận xác nhận `NEW` từ Binance rồi mới hủy SL cũ; hoặc triển khai Emergency Panic Close nếu đặt fail.

- [ ] **TODO-EXEC-02: Khắc phục Silent Failure Desync Trap khi đặt SL thất bại**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:233-237`
  - **Vấn đề:** Đặt SL fail sau 3 lần retry thì log lỗi rồi trả về `Ok(())`. Bot tưởng còn SL cũ, sàn thì đã xóa sạch lệnh.
  - **Fix:** Nếu đặt SL fail sau 3 lần: Gửi ngay lệnh khẩn cấp Market Close toàn bộ vị thế (`reduce_only: true`) để bảo toàn tài khoản.

- [ ] **TODO-EXEC-03: Đổi làm tròn khối lượng từ `.round()` sang `FLOOR`**
  - **File:** `trading_bot/bot_engine/src/data_engine/market_info.rs:45`, `order_executor.rs:137`
  - **Vấn đề:** `.round()` làm tròn lên làm giá trị lệnh vượt quá số dư ký quỹ khả dụng $\to$ Binance reject lỗi `-2019 Margin Insufficient`.
  - **Fix:** Đổi sang `(qty / self.step_size).floor() * self.step_size`.

- [ ] **TODO-EXEC-04: Đặt lệnh Take Profit song song lên Binance Live**
  - **File:** `trading_bot/bot_engine/src/execution/order_executor.rs:242-260`
  - **Fix:** Khi mở vị thế, đặt song song cả Algo `STOP_MARKET` cho SL và Algo `TAKE_PROFIT_MARKET` cho TP (`closePosition: true`).

- [ ] **TODO-RECOV-01: Loại bỏ Clean Slate — Triển khai True "Adopt & Protect"**
  - **File:** `trading_bot/bot_engine/src/execution/account_manager.rs:175-178`
  - **Vấn đề:** Khởi động bot gọi `force_close_all_positions` và `cancel_all_orders`, bán sạch vị thế đang gồng lãi trên sàn!
  - **Fix:** Bỏ lệnh đóng vị thế cưỡng bức; đọc vị thế đang mở và đồng bộ các lệnh Algo SL đang nằm trên Binance Cloud.

- [ ] **TODO-RECOV-02: Ngăn chặn SL Retraction (Lùi SL về đáy sau Restart)**
  - **File:** `trading_bot/bot_engine/src/execution/live_engine.rs:125-135`, `order_executor.rs:122`
  - **Fix:** Trước khi cho phép đặt SL mới, query trực tiếp Algo SL đang có trên Binance gán vào `active_sl_prices` để Monotonic Ratchet Guard không bao giờ cho phép lùi SL.

- [ ] **TODO-RECOV-03: Tạo bảng `active_positions` & `account_capital_ledger` trong SQLite**
  - **File:** `trading_bot/bot_engine/src/data_engine/sqlite_impl.rs:30-78`
  - **Fix:** Lưu trữ `peak_val`, `peak_roe`, `real_entry_time`, `current_sl_price` và số dư lũy tiến vào SQLite.

---

## 🧮 PHÂN HỆ TOÁN HỌC CỰC ĐOAN & GA OPTIMIZER (EXTREME MATH & GA)

- [ ] **TODO-MATH-01: Vá Nghịch Lý Phạt Cháy Tài Khoản trong GA Optimizer**
  - **File:** `ferro_trade/src/backtest/engine.rs:249`, `trading_bot/bot_engine/src/optimizer/prospector_runner.rs:512`
  - **Vấn đề:** Cháy tài khoản chỉ bị phạt `-200` điểm, trong khi dừng lỗ an toàn 35% bị phạt `-5035` điểm! GA bị bẫy thích cá thể tự sát.
  - **Fix:** Khi `broker.next(i)` lỗi out of money, tạo ngay `EarlyTerminationInfo` với điểm phạt tối thiểu `-10,000.0`.

- [ ] **TODO-MATH-02: Bổ sung Mô Hình Thanh Lý (Liquidation & MMR) vào Backtest Broker**
  - **File:** `ferro_trade/src/broker/broker.rs:331-346`
  - **Fix:** Đưa công thức Binance Maintenance Margin vào kiểm tra trong `process_orders`. Nếu nến chạm giá thanh lý, đóng vị thế ngay với lỗ 100% margin.

- [ ] **TODO-MATH-03: Sửa Lỗ Hổng `NaN <= 0.0` trong `calculate_size`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:32-53`
  - **Fix:** Thêm `if equity.is_nan() || equity <= 0.0 || risk_pct.is_nan() { return 0.0; }`.

- [ ] **TODO-ARB-01: Khắc phục Độ Trễ MTF của `z_score` tại Boundary Guard**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/mod.rs:249`, `trend_pb.rs:96`
  - **Fix:** Thêm chốt chặn 1m: Cấm Long Trend_PB nếu $Close_{\text{1m}} \ge BB_{\text{upper, 1m}}$ hoặc $CLV_{\text{1m}} > 0.85$.

- [ ] **TODO-ARB-02: Sửa Lỗi Inverted SL trong `is_structural_sl_acceptable`**
  - **File:** `strategies_backtests/helios/src/strategy/core_v3/risk.rs:15-28`, `arbiter.rs:219`
  - **Fix:** Bổ sung `is_long: bool`. Long bắt buộc $SL < Price$, Short bắt buộc $SL > Price$.

---

## 📋 THỨ TỰ TRIỂN KHAI ĐỀ XUẤT (MASTER SPRINT ROADMAP)

```
Sprint 1 — Chốt Chặn Sinh Tử Tầng Chiến Lược & Toán Học (Core Strategy & Math):
  1. TODO-R2-01: Bỏ current_pnl > 0.0 trong is_be_triggered (risk.rs:291)
  2. TODO-MATH-03: Vá NaN check trong calculate_size (risk.rs:33)
  3. TODO-ARB-02: Sửa Inverted SL check (thêm is_long) (risk.rs:15)
  4. TODO-R2-03: Thêm chaos_index filter cho Reversal (reversal.rs:59)
  5. TODO-R2-12: Sửa default taker_fee_rate = 0.0005 (config.rs:766)
  6. TODO-R2-15: Thêm guard sl_price.is_nan() trong clamp_sl_price (risk.rs:57)

Sprint 2 — Chốt Chặn Sinh Tử Tầng Thực Thi & Sàn (Live Execution Defense):
  7. TODO-EXEC-03: Đổi round_qty sang .floor() để chống lỗi -2019
  8. TODO-EXEC-01 & 02: Atomic SL Replace & Panic Market Dump khi đặt SL fail
  9. TODO-RECOV-01: Xóa bỏ Clean Slate force_close trong account_manager.rs
  10. TODO-RECOV-02: Ngăn chặn SL Retraction sau khi restart bot

Sprint 3 — GA Optimizer & Khả Năng Thích Ứng Định Lượng (GA & Backtest Fidelity):
  11. TODO-MATH-01: Sửa Inverted Penalty Paradox (phạt cháy tài khoản -10000)
  12. TODO-MATH-02: Thêm Liquidation & MMR vào ferro_trade::Broker
  13. TODO-ARB-01: Chốt chặn Boundary 1m chống đu đỉnh KAMA MTF
  14. TODO-R2-02: Thảo luận & vá Giant Candle Trap (De-leverage động)
  15. TODO-R2-13: Ordering constraint cho TSL (tight <= normal <= loose)
  16. TODO-R2-14: Nâng min ttp_activation_roe lên 0.10

Sprint 4 — Kiến Trúc Bền Vững & Dọn Dẹp Mã Nguồn (Resilience & Cleanliness):
  17. TODO-RECOV-03: Thêm bảng active_positions & ledger vào SQLite
  18. TODO-EXEC-04: Đặt lệnh TP song song lên sàn Binance
  19. TODO-R2-05 & 04: KAMA Suction & Range Osc TP Squeeze
  20. TODO-R2-11: Dọn dẹp Bộ Tứ Gen Ma trong config.rs
```
