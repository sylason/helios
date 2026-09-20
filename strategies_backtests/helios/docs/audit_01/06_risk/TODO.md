# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ: MÔ-ĐUN QUẢN TRỊ RỦI RO (RISK MANAGEMENT)

> **Mô-đun:** `Helios Core V3.3 - RiskManager & Stop Management`  
> **Tài liệu kiểm toán gốc:** [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_risk/technical_audit.md)  
> **Cập nhật lần cuối:** 19/09/2026  

---

## 1. 🔴 MỨC ĐỘ CRITICAL (BẮT BUỘC XỬ LÝ TRƯỚC KHI CHẠY LIVE)

- [ ] **TODO-RISK-01: Triển khai gửi lệnh Algo Take Profit lên Binance Futures (`OrderExecutor`)**
  - **Tệp liên đới:** [`trading_bot/bot_engine/src/execution/order_executor.rs:241-260`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L241-L260), [`live_engine.rs:92-95`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L92-L95)
  - **Vấn đề:** Hiện tại `OrderExecutor` chỉ gửi lệnh Algo `STOP_MARKET` cho SL, hoàn toàn bỏ qua TP (`warn!` rồi bỏ qua). Điều này khiến các lệnh có TP cố định (`RANGE_OSC`, `REVERSAL`) không bao giờ được chốt lời trên sàn Live!
  - **Giải pháp:**
    - Khởi tạo thêm lệnh Algo `TAKE_PROFIT_MARKET` hoặc Limit Order Reduce-Only với mức giá TP chỉ định.
    - Trong trường hợp Binance OCO Futures yêu cầu, sử dụng Dual Algo Orders (1 Stop Market + 1 Take Profit Market) và đồng bộ hủy lệnh đối ứng qua WebSocket `OrderTradeUpdate` khi một trong hai lệnh khớp.

---

## 2. 🟠 MỨC ĐỘ HIGH (NGUY CƠ TOÁN HỌC & LỖI BẢO MẬT DỮ LIỆU)

- [ ] **TODO-RISK-02: Vá lỗ hổng IEEE 754 NaN Bypass trong `calculate_size`**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:11-26`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L11-L26)
  - **Vấn đề:** Phép so sánh `<= 0.0` trả về `false` khi `equity.is_nan()`. Hàm tiếp tục tính toán và trả về `NaN.min(0.25) = 0.25`, cấp margin 25% cho tài khoản có vốn bị lỗi!
  - **Giải pháp:** Bổ sung kiểm tra `.is_nan()` tường minh ngay đầu hàm:
    ```rust
    if equity.is_nan() || equity <= 0.0 
        || risk_pct.is_nan() || risk_pct <= 0.0 
        || leverage.is_nan() || leverage <= 0.0 
        || min_order_usd.is_nan() || max_cap_fraction.is_nan() {
        return 0.0;
    }
    ```

- [ ] **TODO-RISK-03: Xử lý mâu thuẫn giữa `min_order_usd` ($21) và `max_cap_fraction` (0.25)**
  - **Tệp liên đới:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs:18-26`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L18-L26)
  - **Vấn đề:** Khi vốn quá nhỏ ($2 - $4), lệnh sau khi nâng lên $21 sẽ bị kẹp bởi trần 25% vốn, làm quy mô thực tế tụt xuống $10 < $21, dẫn đến việc sàn Binance từ chối lệnh (`REJECTED: MIN_NOTIONAL`).
  - **Giải pháp:** Nếu sau khi kẹp trần `cap` mà `final_margin * leverage < min_order_usd`, hàm phải từ chối mở lệnh (trả về `0.0`):
    ```rust
    let final_fraction = size_fraction.min(cap);
    if final_fraction * equity * leverage < min_order_usd {
        return 0.0; // Vốn không đủ an toàn để đáp ứng Min Notional của sàn
    }
    final_fraction
    ```

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
