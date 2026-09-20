# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO): ĐỘNG CƠ KHỚP LỆNH MÔ PHỎNG (PAPER MATCHING ENGINE)

> **Mô-đun:** `PaperExecutionEngine` & `PaperTrader` (Helios V3.3)  
> **Thư mục tài liệu:** `strategies_backtests/helios/docs/audit_01/13_paper_matching_engine/`  
> **Mã nguồn liên quan:**
> - [`trading_bot/bot_engine/src/execution/paper_trader.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)
> - [`trading_bot/bot_engine/src/execution/paper_engine.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_engine.rs)
> - [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
> - [`ferro_trade/src/broker/broker.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/broker/broker.rs)

---

## 📌 MỨC ĐỘ ƯU TIÊN P0: KHẨN CẤP (ẢNH HƯỞNG TRỰC TIẾP ĐẾN LOGIC TÍNH TOÁN VỐN)

- [ ] **P0.1: Đồng bộ hóa Floating PnL vào `PaperExecutionEngine::get_equity()`**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_engine.rs:113-115`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_engine.rs#L113-L115)
  - **Mô tả:** Hiện tại hàm `get_equity(&self)` chỉ trả về `self.paper_trader.equity` (số dư ví tiền mặt, không tính PnL thả nổi của vị thế đang mở). Điều này khiến Strategy và AssetActor bị "mù" PnL khi đánh giá drawdown hay quy mô vốn.
  - **Giải pháp:**
    1. Bổ sung trường `pub last_close: f64` trong struct `PaperExecutionEngine`.
    2. Cập nhật `self.last_close = candle.close` trong hàm `on_candle_ctx(&mut self, candle: &Candle, ...)`.
    3. Cập nhật hàm `get_equity(&self)`:
       ```rust
       fn get_equity(&self) -> f64 {
           if self.paper_trader.position_type != PositionType::None && self.last_close > 0.0 {
               self.paper_trader.get_total_equity(self.last_close)
           } else {
               self.paper_trader.equity
           }
       }
       ```
  - **Tiêu chí nghiệm thu:** `AssetActor` nhận được giá trị vốn Marked-to-Market chính xác 100% khi vị thế đang mở.

- [ ] **P0.2: Tích hợp `MarketFilter` (Làm tròn Step Size & Kiểm tra Min Notional) trước khi mở lệnh**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_engine.rs:42-60`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_engine.rs#L42-L60) & [`paper_trader.rs:339-348`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L339-L348)
  - **Mô tả:** `PaperTrader` tính `position_size = trade_value / current_price` là số thực float 64-bit tự do, không kiểm tra giới hạn kích thước tối thiểu (`min_order_usd` / `min_notional`) và không làm tròn theo `stepSize` của sàn.
  - **Giải pháp:**
    1. Trong `PaperExecutionEngine::execute_action_ctx`, trước khi gọi `process_action`:
       - Kiểm tra `trade_value >= self.core.market_filter.min_notional`. Nếu không đạt, từ chối mở lệnh và ghi log cảnh báo `warn!`.
       - Làm tròn khối lượng dự kiến qua `self.core.market_filter.round_qty(...)`.
    2. Hoặc bổ sung tham số `fractional_unit: Option<f64>` vào `PaperTrader` tương tự như `ferro_trade::Broker`.
  - **Tiêu chí nghiệm thu:** Không phát sinh bất kỳ lệnh Paper Trading nào có kích thước danh nghĩa dưới ngưỡng sàn quy định ($5 hoặc $21) hoặc có số lẻ vượt quá quy cách hợp đồng.

---

## 📌 MỨC ĐỘ ƯU TIÊN P1: QUAN TRỌNG (NÂNG CAO ĐỘ TIN CẬY & PHÒNG THỦ)

- [ ] **P1.1: Bổ sung Cơ chế Giám sát Cháy Tài khoản (Liquidation Watchdog) trong `PaperTrader`**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs:374-465`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L374-L465)
  - **Mô tả:** Với đòn bẩy 20x, khi giá đi ngược $\ge 4.5\% - 5.0\%$, tài khoản thực tế trên Binance sẽ bị kích hoạt Margin Call / Liquidation. `PaperTrader` hiện tại không kiểm tra ngưỡng phá sản, có nguy cơ giữ lệnh đến âm vốn nếu nến quét vượt xa SL.
  - **Giải pháp:**
    Trong `on_candle`:
    ```rust
    let worst_price = match self.position_type {
        PositionType::Long => candle.low,
        PositionType::Short => candle.high,
        PositionType::None => candle.close,
    };
    let worst_pnl = self.get_unrealized_pnl(worst_price);
    let margin_used = (self.entry_price * self.position_size) / self.leverage;
    if margin_used > 0.0 && worst_pnl <= -margin_used * 0.90 {
        // Cưỡng bức thanh lý mô phỏng
        error!("💥 [SIMULATED LIQUIDATION] Symbol: {} | Position {:?} liquidated at worst price {:.2}!", self.symbol, self.position_type, worst_price);
        self.close_position(worst_price, ExitReason::HardStopLoss, candle.timestamp, bar_index, account_tx);
        return true;
    }
    ```
  - **Tiêu chí nghiệm thu:** Ngăn chặn tuyệt đối tình trạng vốn tài khoản bị rơi xuống số âm trong Paper Trading.

- [ ] **P1.2: Cập nhật `available_balance` và `unrealized_profit` trong `broadcast_account_events`**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs:161-182`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L161-L182)
  - **Mô tả:** Hàm `broadcast_account_events` hiện hardcode `unrealized_profit: 0.0` và `available_balance: self.equity` mà không trừ đi ký quỹ chiếm dụng (`margin_used`).
  - **Giải pháp:**
    1. Tính `margin_used = (self.entry_price * self.position_size) / self.leverage`.
    2. Gán `available_balance = (self.equity - margin_used).max(0.0)`.
    3. Gán `unrealized_profit` bằng `self.get_unrealized_pnl(self.entry_price)` hoặc truyền `current_price` vào hàm.
  - **Tiêu chí nghiệm thu:** UI Dashboard hoặc Client nhận được số dư khả dụng thực tế phản ánh đúng lượng ký quỹ đang bị khóa.

- [ ] **P1.3: Thêm Mô hình Trượt Giá Giả Lập (Simulated Slippage & Spread)**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs:348`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L348)
  - **Mô tả:** Lệnh Market đang được khớp tại chính xác $Close[i]$ với 0 slippage. Cần đưa vào tham số cấu hình trượt giá (ví dụ 0.5 tick hoặc $0.01\%$) để kết quả Paper Trading thận trọng hơn, phản ánh đúng ma sát thị trường thực tế.
  - **Giải pháp:** Bổ sung trường `spread_pct: f64` vào `PaperTrader` (tương tự `spread` trong `ferro_trade::Broker`), điều chỉnh giá vào lệnh:
    $$P_{\text{entry\_long}} = Close[i] \times (1 + \text{spread\_pct})$$
    $$P_{\text{entry\_short}} = Close[i] \times (1 - \text{spread\_pct})$$

---

## 📌 MỨC ĐỘ ƯU TIÊN P2: NÂNG CAO (HOÀN THIỆN TOÀN DIỆN HỆ THỐNG)

- [ ] **P2.1: Mô phỏng Phí Funding Fee Chu kỳ 8 giờ (Perpetual Futures Funding Rate)**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs:374-380`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L374-L380)
  - **Mô tả:** Khi một vị thế ngâm qua mốc thời gian 00:00, 08:00, 16:00 UTC, tính toán và khấu trừ phí Funding Rate danh nghĩa:
    $$\text{FundingFee} = \text{PositionSize} \times \text{ClosePrice} \times \text{FundingRate}$$
    (Mặc định sử dụng tỷ lệ chuẩn $0.01\%$ nếu không có dữ liệu stream funding từ sàn).

- [ ] **P2.2: Tách Biệt Phân Loại Lý Do Đóng Lệnh `ExitReason::ZombieTimeout`**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs:507-534`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L507-L534)
  - **Mô tả:** Hiện tại khi Strategy phát tín hiệu `Action::ClosePosition`, `PaperTrader` chỉ phân chia thành `TrailingTakeProfit` (khi PnL > 0) hoặc `StagnantDecay` (khi PnL $\le$ 0). Cần phân biệt rõ khi nào là lệnh bị Zombie Timeout purge (ngâm quá 2880 bars) bằng cách kiểm tra số bars held hoặc truyền reason chi tiết từ Strategy.

- [ ] **P2.3: Bổ sung Bộ Test Tự Động Toàn Diện (Automated Regression Tests)**
  - **Tệp liên quan:** [`trading_bot/bot_engine/src/execution/paper_trader.rs:631-849`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L631-L849)
  - **Mục tiêu:**
    1. `test_floating_equity_reflection`: Xác nhận `get_total_equity()` phản ánh đúng floating PnL khi nến biến động.
    2. `test_gap_slippage_execution`: Xác nhận cả Long Gap Down và Short Gap Up đều khớp tại giá Open thay vì SL price.
    3. `test_pessimistic_dual_hit_sl_priority`: Đảm bảo khi High >= TP và Low <= SL, luôn ưu tiên khớp Stop Loss.
    4. `test_fee_accounting_and_be_buffer`: Đảm bảo tổng phí 2 chiều đạt 0.08% và Break-Even buffer bù đắp chính xác 0.08% phí.
