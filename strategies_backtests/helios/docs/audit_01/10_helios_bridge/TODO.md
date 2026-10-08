# DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO): HELIOS_BRIDGE & HỆ THỐNG LIÊN ĐỚI

> **Mục tiêu:** Khắc phục triệt để các lỗ hổng chí tử về phân kỳ vị thế, rò rỉ bộ đệm High Watermark, mù trạng thái đóng lệnh và tối ưu hóa hiệu năng cấp phát bộ nhớ trong `helios_bridge.rs`.  
> **Module phụ trách:** [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs)  
> **Ngày lập kế hoạch:** 19/09/2026  
> **Phân loại độ ưu tiên:** P0 (Khẩn cấp / Nguy hiểm tài chính) > P1 (Quan trọng / Đồng bộ hóa) > P2 (Hiệu năng / Bộ nhớ) > P3 (Dọn dẹp / Kiểm thử)

---

## 🚨 P0: CÁC VẤN ĐỀ CHÍ TỬ (CRITICAL DEFECTS - CẦN VÁ NGAY)

- [ ] **TODO-BRG-01: Khắc phục triệt để thảm họa `High Watermark Buffer Bleed` khi `mapped_entry_bar == 0`**
  - **Mô tả:** Khi vị thế được giữ lâu và nến vào lệnh trôi ra ngoài biên trái của bộ đệm (`pos.entry_timestamp < candles[0].timestamp`), `mapped_entry_bar` bị gán về 0. `RiskManager::manage_trailing_stop` quét toàn bộ nến từ index 1 (quá khứ không thuộc lệnh), gán nhầm đỉnh giá cũ thành đỉnh lợi nhuận `peak_val`, đẩy Stop Loss vọt lên trên giá hiện tại và cắt lỗ lệnh oan.
  - **Giải pháp:**
    1. Bổ sung trường `highest_favorable_price: f64` vào cấu trúc [`LivePositionState`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L20-L29) và [`ferro_trade::broker::trade::Trade`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/broker/trade.rs).
    2. Cập nhật đỉnh lợi nhuận đơn điệu tại mỗi cây nến mới trong `PaperTrader` và `BinanceLiveExecutionEngine`:
       $$\text{Long: } H_{t} = \max(H_{t-1}, \text{candle.high}); \quad \text{Short: } L_{t} = \min(L_{t-1}, \text{candle.low})$$
    3. Trong `RiskManager::manage_trailing_stop`, sử dụng trực tiếp giá trị `trade.highest_favorable_price` thay vì duyệt mảng `highs[entry_bar + 1..=i]`.
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs:220–236`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L220-L236), [`risk.rs:130–150`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L130-L150), [`asset_actor.rs:20–29`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L20-L29).

- [x] **TODO-BRG-02: Vá lỗ hổng mù trạng thái lệnh vừa đóng (`last_closed_trade: None`) trong `evaluate_live_tick`** - ✅ **ĐÃ HOÀN THÀNH**
  - **Mô tả:** [`helios_bridge.rs:271`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L271) từng truyền cứng `last_closed_trade: None` vào `TickContext`. Khi vị thế vừa đóng ngoài đời (do TP/SL/thanh lý), chiến lược mù hoàn toàn PnL thực tế, rơi vào nhánh fallback tính sai PnL từ giá đóng cửa, dẫn đến việc kích hoạt Cooldown phạt sai lệch hoặc mất dấu chuỗi thua lỗ.
  - **Giải pháp đã thực hiện:**
    1. Mở rộng tham số hàm `evaluate_live_tick` nhận thêm `last_closed_trade: Option<&crate::execution::paper_trader::PaperTradeRecord>`.
    2. Trong `AssetActor::sync_and_handle_closed_positions`: trích xuất `engine_closed_trade` từ `ExecutionEngine::take_last_closed_trade()`, lưu vào `self.last_closed_trade_for_bridge`.
    3. Chuyển đổi sang `ferro_trade::broker::trade::Trade` và nạp vào `TickContext { ..., last_closed_trade: closed_trade_snapshot.as_ref(), ... }`.
    4. Reset `self.last_closed_trade_for_bridge = None` ngay sau khi nến đã đánh giá xong (đảm bảo tính nhất quán 1 nến duy nhất).
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs), [`asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs).

- [ ] **TODO-BRG-03: Sửa chữa đứt gãy trạng thái vị thế trong `BinanceLiveExecutionEngine`**
  - **Mô tả:** Trong [`live_engine.rs:120–139`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L120-L139), mỗi khi nhận sự kiện vị thế từ Binance WebSocket, `current_sl_price` và `opening_playbook_tag` bị bỏ mặc bằng `None`, còn `entry_timestamp` bị ghi đè bằng `Utc::now().timestamp_millis()`. Dẫn đến việc chiến lược không nhận diện được chiến thuật mở lệnh, tính sai thời gian giữ lệnh và mất mức SL bảo vệ trên sàn.
  - **Giải pháp:**
    1. Lưu vết `opening_playbook_tag` và `entry_timestamp` gốc từ lúc gửi lệnh thành công trong `OrderExecutor`.
    2. Đọc mức Stop Loss đang treo trên sàn từ `OrderExecutor.active_sl_prices` gán vào `LivePositionState.current_sl_price`.
    3. Khi cập nhật `PositionUpdate`, bảo tồn `entry_timestamp` gốc, không ghi đè thời gian nhận gói tin WS.
  - **Tập tin ảnh hưởng:** [`live_engine.rs:120–139`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs#L120-L139), [`order_executor.rs:57–70`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L57-L70).

---

## ⚡ P1: ĐỒNG BỘ HÓA & LOGIC HỆ THỐNG (HIGH PRIORITY)

- [x] **TODO-BRG-04: Thống nhất hai bộ lọc Cooldown độc lập (Dual Sentinels Unification)** - ✅ **ĐÃ HÓA GIẢI & ĐỒNG BỘ QUA SSOT**
  - **Mô tả:** Từng tồn tại phân kỳ giữa `strategy.cooldown_tracker` (Core V3) và `actor.cooldown_guard` (AssetActor) do chiến lược bị mù trạng thái lệnh vừa đóng trên Live.
  - **Kết luận kiến trúc & Giải pháp:**
    1. Khi `TODO-BRG-02` được giải quyết, `strategy` nhận được đúng bản ghi lệnh vừa đóng thực tế (`TickContext.last_closed_trade`). Chiến lược tự động kích hoạt Cooldown phạt nội tại và tự chặn phát tín hiệu từ bên trong, đồng bộ 100% với `EngineCooldownGuard`.
    2. Cơ chế 2 tầng phòng ngự (Defense-in-Depth) được giữ nguyên mà không cần thêm kênh "Execution Feedback" phức tạp gây over-engineering: Tầng 1 (Strategy) chặn từ gốc; Tầng 2 (Engine) đóng vai trò chốt chặn cuối cùng bảo vệ tài khoản.
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs), [`asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs).

- [ ] **TODO-BRG-05: Cập nhật chính xác PnL cho `Position` snapshot trong `evaluate_live_tick`**
  - **Mô tả:** Tại [`helios_bridge.rs:252–258`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L252-L258), `pnl` và `pnl_pct` được gán cứng bằng 0.0. Bất kỳ chỉ báo hoặc Playbook nào kiểm tra lợi nhuận vị thế qua `tick.position().pnl()` đều bị mù.
  - **Giải pháp:**
    Tính toán PnL chưa thực hiện (Unrealized PnL) dựa trên giá đóng cửa nến hiện tại `candles.last().unwrap().close` và giá vào lệnh `pos.entry_price`:
    $$\text{pnl} = (P_{\text{curr}} - P_{\text{entry}}) \times \text{size}; \quad \text{pnl\_pct} = \frac{P_{\text{curr}} - P_{\text{entry}}}{P_{\text{entry}}} \times \text{leverage} \times 100$$
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs:252–258`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L252-L258).

---

## 🚀 P2: TỐI ƯU HÓA HIỆU NĂNG & BỘ NHỚ (PERFORMANCE & MEMORY OPTIMIZATION)

- [ ] **TODO-BRG-06: Chuyển đổi Resampling sang thuần Rust không qua Polars trong `evaluate_live_tick`**
  - **Mô tả:** `candles_to_df` tạo mới DataFrame, Dynamic Grouping, Hash Aggregations tiêu tốn ~11.2 MB RAM và 50-150ms CPU mỗi phút cho mỗi Symbol chỉ để lấy 1 nến cuối cùng.
  - **Giải pháp:**
    1. Tận dụng trực tiếp hàm [`fast_resample_ohlcv`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/utils/resample.rs#L152-L230) đã có sẵn trong `ferro_trade`, thực hiện gom nến $O(N)$ trực tiếp trên mảng nến thô mà không cần cấp phát Polars DataFrame.
    2. Loại bỏ bước trung gian chuyển đổi sang DataFrame và `drop(df)`.
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs:165–190`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L165-L190), [`resample.rs:152–230`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/utils/resample.rs#L152-L230).

- [ ] **TODO-BRG-07: Tái sử dụng IndicatorManager & Bộ đệm Chỉ báo (Incremental Indicator Caching)**
  - **Mô tả:** Mỗi phút gọi `strategy.init(&mut init_context)` tính lại từ đầu toàn bộ 20+ chỉ báo cho 15,000 cây nến, lãng phí 99.99% năng lực CPU.
  - **Giải pháp:**
    1. Duy trì `IndicatorManager` và `GlobalCache` sống xuyên suốt vòng đời của `AssetActor`.
    2. Thiết kế hàm cập nhật gia tăng (Incremental Update): Chỉ tính toán giá trị nến mới nhất $O(1)$ cho KAMA, BBands, RSI, ATR và append vào bộ đệm thay vì tính lại toàn bộ mảng.
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs:197–205`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L197-L205), [`asset_actor.rs:625–635`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L625-L635).

---

## 🧹 P3: DỌN DẸP MÃ NGUỒN & KIỂM THỬ TỰ ĐỘNG (CLEANUP & TESTING)

- [ ] **TODO-BRG-08: Xóa bỏ mã chết (Dead Code) `HeliosBridge::analyze`**
  - **Mô tả:** Hàm `HeliosBridge::analyze` ([`helios_bridge.rs:25–150`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L25-L150)) không còn được sử dụng ở bất kỳ đâu trong toàn bộ codebase. Logic chạy broker cục bộ độc lập gây nhầm lẫn về kiến trúc.
  - **Giải pháp:** Xóa bỏ toàn bộ hàm `analyze` và các đoạn mã phụ thuộc không sử dụng.
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs:25–150`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L25-L150).

- [ ] **TODO-BRG-09: Bổ sung bộ Unit Tests chuyên sâu cho `evaluate_live_tick`**
  - **Mô tả:** Chưa có test tự động kiểm tra các trường hợp biên của cơ chế tiêm vị thế.
  - **Kịch bản test cần viết:**
    1. `test_evaluate_live_tick_buffer_bleed_protection`: Vị thế có timestamp cũ hơn nến đầu tiên trong buffer $\implies$ Không bị nhảy Stop Loss ảo.
    2. `test_evaluate_live_tick_with_last_closed_trade`: Truyền lệnh đóng có lợi nhuận $\implies$ Cooldown reset về 0; truyền lệnh đóng lỗ $\implies$ Cooldown tính đúng số nến phạt lũy tiến.
    3. `test_evaluate_live_tick_ragged_4h_boundary`: Kiểm tra tính hội tụ của chỉ báo 4h khi cửa sổ nến 1m bắt đầu tại thời điểm bất kỳ giữa chu kỳ 4h.
  - **Tập tin ảnh hưởng:** [`helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs).
