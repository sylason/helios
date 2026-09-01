# 🔬 Kiểm Chứng Chuyên Sâu: Lệch Pha Vòng Đời Kiểm Tra Stop Loss Giữa `AssetActor` và `Broker`

**Chủ đề:** Sự sai lệch thứ tự kiểm tra rủi ro (Risk Check Sequencing) giữa Backtest Engine và Live/Paper Execution  
**Vị trí liên quan:**  
- [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L342-L368)  
- [`ferro_trade/src/broker/broker.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/broker/broker.rs#L318-L347)  
- [`trading_bot/bot_engine/src/execution/paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)  

---

## 📌 1. Triệu Chứng & Hiện Tượng Thực Tế

Trong Forward Testing: Lệnh thường xuyên bị dính `HARD_SL` sớm tại râu nến trước khi Trailing Stop Loss kịp nâng đỡ đáy bảo vệ, hoặc lệnh mở mới bị trễ 1 nhịp nến so với thời điểm xuất hiện tín hiệu chuẩn trong Backtest.

---

## 🔬 2. Bằng Chứng Mã Nguồn & So Sánh Luồng Thực Thi

### 🔄 Luồng Chuẩn Trong `ferro_trade::Broker` (Backtest)
Trong [`ferro_trade/src/broker/broker.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/broker/broker.rs#L318-L347) và [`engine.rs`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/engine.rs#L250-L280):

1. Đầu nến $i$: `broker.next(i)` chạy `process_orders()`.
   - Kiểm tra xem giá `High/Low` của nến $i$ có chạm mức SL/TP của lệnh đang mở không.
   - Nếu có, lệnh được đóng tại giá khớp lệnh chính xác của nến $i$.
2. Sau đó: `strategy.next(&tick)` được gọi.
   - Nếu lệnh đã đóng ở bước 1, `tick.position().is_open()` trả về `false`.
   - Strategy đánh giá thị trường và có thể phát ra `Action::Buy` / `Action::Sell` mới.
3. Cuối nến: `broker.execute_actions()` thực thi các hành động từ Strategy. Nếu `trade_on_close = true`, lệnh mới được mở ngay tại giá `Close` của nến $i$.

---

### ⚠️ Luồng Bất Đối Xứng Trong `AssetActor` (Live / Paper Trading)
Trong [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L342-L368):

```rust
// BƯỚC 3.1: Quét nến real-time cho ExecutionEngine
let _ = self.execution_engine.on_candle_ctx(&candle, current_bar_idx).await;

// BƯỚC 3.2: Lấy số dư vốn thực tế và trạng thái vị thế mới nhất
let current_equity = self.execution_engine.get_equity();
let current_position = self.execution_engine.get_position();

// BƯỚC 3.3: Gọi HeliosBridge::evaluate_live_tick để đánh giá tín hiệu và dời Trailing SL
if let Some(ref mut strategy) = self.strategy {
    let actions = HeliosBridge::evaluate_live_tick(
        &self.symbol,
        rolling_buffer,
        slice_start,
        current_position.as_ref(),
        strategy.as_mut(),
        current_equity,
    );
    // ...
    for action in actions {
        self.execution_engine.execute_action_ctx(&action, ...).await;
    }
}
```

---

## 🧠 3. Cơ Chế Tạo Ra Sai Lệch (The Desynchronization Gap)

1. **Khoảng trễ 1 nến trong việc Dời Trailing Stop Loss:**
   - Tại nến $K$, giá bứt phá tạo đỉnh cao mới `peak_val`. Theo quy tắc, Trailing SL cần được dời lên mức bảo vệ cao hơn.
   - Nhưng `PaperTrader::on_candle` (Bước 3.1) lại thực thi **trước** khi Strategy chạy (Bước 3.3).
   - Nếu trong nến $K$, giá biến động mạnh quét râu xuống dưới mức SL cũ (của nến $K-1$), lệnh lập tức bị **quét Stop Loss cũ** và đóng vị thế.
   - Sang Bước 3.2, `current_position` đã trở thành `None`, dẫn đến việc Strategy ở Bước 3.3 không còn vị thế nào để dời Trailing SL nữa!
2. **Khắc phục:**
   - Cần đồng bộ cơ chế xử lý theo đúng vòng đời khép kín của `Broker`: Cho phép Strategy đánh giá và dời SL trên nến $K$ trước khi kiểm tra điều kiện thoát lệnh nếu nến đóng, hoặc mô phỏng đúng trật tự thời gian Tick.
