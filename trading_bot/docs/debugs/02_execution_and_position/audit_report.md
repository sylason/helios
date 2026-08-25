# Báo Cáo Kiểm Thử Kỹ Thuật (Scientific Audit Report)
**Đối tượng kiểm thử:** `src/controller/asset_actor.rs`, `src/execution/paper_trader.rs`, `src/execution/order_executor.rs`, `src/execution/account_manager.rs`, `src/execution/ws_manager.rs`, và `src/optimizer/helios_bridge.rs`  
**Dự án:** `trading_bot/bot_engine`  
**Đường dẫn file lưu báo cáo:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\debugs\02_execution_and_position\audit_report.md`

---

## 🎯 1. Tóm Tắt Tổng Quan (Executive Summary)

Đã hoàn thành đợt kiểm thử chuyên sâu (deep audit) đối với toàn bộ các mô-đun quản lý vị thế, thực thi lệnh (execution) và cầu nối chiến lược (strategy bridge) trong `bot_engine`. 

Kết quả kiểm thử cho thấy hệ thống hiện tại đang gặp **2 LỖI NGHIÊM TRỌNG (CRITICAL)** ảnh hưởng trực tiếp đến tính đúng đắn của việc quản trị rủi ro và tính ổn định của bot, cùng **3 LỖI MỨC ĐỘ CAO (HIGH)** và một số vấn đề về khớp nối chặt (tight coupling) / hardcoding.

### 📊 Bảng Tổng Hợp Mức Độ Nghiêm Trọng (Severity Matrix)

| STT | Phân Loại Lỗi | Vị Trí File | Mức Độ | Ảnh Hưởng |
| :--- | :--- | :--- | :--- | :--- |
| **1** | **Hardcoded Equity trong Strategy Context** | `helios_bridge.rs:311` | 🔴 **CRITICAL** | Tín hiệu phân bổ vốn & quản trị rủi ro của Strategy tính theo $10,000 cố định, hoàn toàn lệch khỏi vốn thực/paper của tài khoản. |
| **2** | **Deadlock cờ `is_submitting_order`** | `asset_actor.rs:435,500` | 🔴 **CRITICAL** | Khi xảy ra lỗi REST API lúc đặt lệnh, cờ không được reset về `false`, khiến `AssetActor` bị "tê liệt" vĩnh viễn không thể vào lệnh mới. |
| **3** | **Hardcoded Trade ID & Order IDs** | `helios_bridge.rs:282,289,290` | 🟠 **HIGH** | `trade_id = 1`, `sl_order_id = Some(1)`, `tp_order_id = Some(2)` bị cố định, gây rủi ro sai lệch khi quản lý nhiều vị thế/lệnh dời SL TP. |
| **4** | **Sai lệch `live_entry_bar` khi Reconnect** | `asset_actor.rs:270` | 🟠 **HIGH** | Vị thế khôi phục từ Binance WS lấy `entry_bar` là nến hiện tại tại thời điểm bot bật lại, làm sai lệch logic đếm tuổi nến (bar age). |
| **5** | **Mất Playbook Tag & SL/TP ban đầu** | `asset_actor.rs:276-284` | 🟠 **HIGH** | Vị thế khôi phục từ WebSocket bị gán `None` cho Tag, SL, TP ban đầu, làm chiến lược mất bối cảnh ban đầu. |
| **6** | **Hardcoded Min Notional & Commission** | `order_executor.rs:65`, `asset_actor.rs:81` | 🟡 **MEDIUM** | Bỏ qua `market_filter.min_notional` thực tế và cố định phí hoa hồng `0.0004`. |
| **7** | **Luồng WS Trade API bị bỏ trống** | `ws_manager.rs:273` | 🔵 **LOW** | Kết nối WS Trade API giữ lặp nhưng không được sử dụng để phát lệnh. |

---

## 🔍 2. Phân Tích Chi Tiết Các Lỗi & Bất Cập Kiến Trúc

### 🛑 Lỗi 1: Hardcoded Equity = 10,000.0 USDT trong Strategy Context (`helios_bridge.rs`)
* **Vị trí:** `src/optimizer/helios_bridge.rs`, dòng 311.
* **Chi tiết code:**
  ```rust
  let mut tick_context = TickContext {
      i: last_idx,
      equity: 10000.0, // ❌ HARDCODED FIX EQUITY!
      data_view: &data_view,
      indicator_manager: &indicator_manager,
      trades: &live_trades,
      actions: &mut actions_buffer,
      position: live_position_snapshot,
  };
  strategy.next(&mut tick_context);
  ```
* **Bản chất vấn đề:**
  - `AssetActor` khi gọi `HeliosBridge::evaluate_live_tick` (dòng 404 `asset_actor.rs`) **KHÔNG truyền thông tin vốn** (`equity` hoặc `available_balance`).
  - Trong `evaluate_live_tick`, `TickContext` bị gán cứng `equity: 10000.0`.
* **Hậu quả:**
  - Nếu tài khoản thật hoặc paper chỉ có **$100 USDT**, nhưng Strategy dựa trên `equity` ($10,000) để tính rủi ro (ví dụ 2% equity = $200), chiến lược sẽ phát ra tín hiệu mở vị thế $200 (tương đương 200% tài khoản thực), dẫn đến cháy tài khoản hoặc bị quá đòn bẩy.
  - Ngược lại, nếu tài khoản có **$100,000 USDT**, chiến lược vẫn tính toán rủi ro trên $10,000, làm giảm hiệu suất đầu tư xuống 10 lần.

---

### 🛑 Lỗi 2: Deadlock cờ `is_submitting_order` khi gặp lỗi REST API (`asset_actor.rs`)
* **Vị trí:** `src/controller/asset_actor.rs`, dòng 435 & 500.
* **Chi tiết code:**
  ```rust
  // Dòng 435: Kiểm tra cờ trước khi phát lệnh
  if position_amt > 0.0 || self.is_submitting_order {
      info!("ℹ️ [{}] Đang gửi lệnh, bỏ qua tín hiệu...", self.symbol);
      continue;
  }
  // Dòng 500: Set cờ = true trước khi spawn task bất đồng bộ
  self.is_submitting_order = true;
  tokio::spawn(async move {
      if let Err(e) = executor_clone.execute_action(...).await {
          error!("❌ Lỗi thực thi lệnh: {:?}", e);
      }
  });
  ```
* **Bản chất vấn đề:**
  - Cờ `self.is_submitting_order` được mở (`true`) trước khi bắn lệnh REST API.
  - Cờ này **chỉ được reset về `false`** khi `AssetActor` nhận được sự kiện `AccountEvent::PositionUpdate` hoặc `AccountEvent::OrderTradeUpdate` thông qua WebSocket (dòng 264 & 299).
  - Nếu lệnh gửi bị từ chối từ phía Binance (vd: lỗi mạng, lỗi Min Notional, lỗi API Key, Invalid Price), Binance **sẽ không gửi bất kỳ WebSocket event nào** về!
* **Hậu quả:**
  - `self.is_submitting_order` sẽ bị giữ ở giá trị `true` **MÃI MÃI**.
  - `AssetActor` rơi vào trạng thái Deadlock: tất cả các nến tiếp theo đều bị bỏ qua tín hiệu giao dịch vì tin rằng "đang có lệnh đang gửi".

---

### ⚠️ Lỗi 3: Hardcoded Trade ID (`1`) và Order IDs (`1`, `2`) (`helios_bridge.rs`)
* **Vị trí:** `src/optimizer/helios_bridge.rs`, dòng 282, 289, 290.
* **Chi tiết code:**
  ```rust
  let mut trade = ferro_trade::broker::trade::Trade::new(
      1, // ❌ HARDCODED trade_id = 1
      pos.position_amt,
      pos.entry_price,
      mapped_entry_bar.min(last_idx),
      None,
      pos.opening_playbook_tag.clone(),
  );
  trade.sl_price = pos.current_sl_price;
  trade.tp_price = pos.current_tp_price;
  trade.sl_order_id = if pos.current_sl_price.is_some() { Some(1) } else { None }; // ❌ HARDCODED order_id = 1
  trade.tp_order_id = if pos.current_tp_price.is_some() { Some(2) } else { None }; // ❌ HARDCODED order_id = 2
  ```
* **Bản chất vấn đề:**
  - Khi tiêm vị thế đang mở vào `TickContext` để Strategy đánh giá ở nến hiện tại, `trade_id`, `sl_order_id`, `tp_order_id` đều bị gán cứng.
  - Khi Strategy muốn phát tín hiệu `Action::UpdateTradeSlTp { trade_id, ... }`, việc gán cứng này khiến việc đối chiếu ID giữa lệnh thực tế và lệnh trong bộ nhớ của Strategy trở nên thiếu chính xác nếu mở rộng sang giao dịch nhiều vị thế (multi-position / grid) hoặc quản lý chi tiết từng order id.

---

### ⚠️ Lỗi 4: Sai lệch `live_entry_bar` khi Bot khởi động lại hoặc Reconnect (`asset_actor.rs`)
* **Vị trí:** `src/controller/asset_actor.rs`, dòng 270.
* **Chi tiết code:**
  ```rust
  AccountEvent::PositionUpdate { symbol, position_amt, entry_price, unrealized_profit } => {
      ...
      let current_bar = self.asset.get_candles_vec().len().saturating_sub(1);
      self.live_position = Some(LivePositionState {
          symbol: symbol.clone(),
          position_amt,
          entry_price,
          entry_timestamp: current_time,
          live_entry_bar: current_bar, // ❌ ĐÁNH DẤU ENTRY BAR LÀ NẾN HỆ THỐNG HIỆN TẠI
          ...
      });
  }
  ```
* **Bản chất vấn đề:**
  - Nếu vị thế đã mở trên Binance từ 5 tiếng trước (tương đương 300 nến 1m trước), nhưng bot bị restart hoặc ngắt kết nối WebSocket.
  - Khi nhận lại `PositionUpdate` từ Binance, `AssetActor` gán `live_entry_bar = current_bar` (nến hiện tại).
* **Hậu quả:**
  - Tuổi của vị thế (`pos_age = current_bar - entry_bar`) bị reset về `0`.
  - Các quy tắc chiến lược phụ thuộc vào thời gian nắm giữ vị thế (ví dụ: "Đóng vị thế sau 20 cây nến", "Nâng SL về Breakeven sau 10 nến") sẽ bị tính toán sai hoàn toàn.

---

### ⚠️ Lỗi 5: Mất Playbook Tag, SL Price và TP Price khi Reconnect (`asset_actor.rs`)
* **Vị trí:** `src/controller/asset_actor.rs`, dòng 276-284.
* **Chi tiết code:**
  ```rust
  self.live_position = Some(LivePositionState {
      symbol: symbol.clone(),
      position_amt,
      entry_price,
      entry_timestamp: current_time,
      live_entry_bar: current_bar,
      current_sl_price: None, // ❌ BỊ KHUYẾT SL
      current_tp_price: None, // ❌ BỊ KHUYẾT TP
      opening_playbook_tag: None, // ❌ BỊ KHUYẾT TAG
  });
  ```
* **Bản chất vấn đề:**
  - Sự kiện `PositionUpdate` của Binance WebSocket chỉ trả về `position_amt`, `entry_price`, `unrealized_profit`. Nó **không trả về** Playbook Tag hay mức SL/TP ban đầu mà bot đã đặt.
  - `AssetActor` khởi tạo `LivePositionState` mới với các giá trị `None`.
* **Hậu quả:**
  - Strategy khi đánh giá nến tiếp theo sẽ không biết vị thế này mở theo mô hình nào (Playbook Tag bị `None`), dẫn đến việc chọn sai quy tắc quản lý trailing stop.

---

### ⚠️ Lỗi 6: Hardcoded Min Notional & Các Tham Số Cấu Hình Execution
* **Vị trí:** `order_executor.rs:65`, `asset_actor.rs:81`, `order_executor.rs:128`.
* **Chi tiết code:**
  - `order_executor.rs:65`: `if notional < 5.0` (Gán cứng 5.0 USDT thay vì dùng `self.market_filter.min_notional`).
  - `asset_actor.rs:81`: `PaperTrader::new(..., 0.0004)` (Gán cứng phí hoa hồng 0.04%).
  - `order_executor.rs:128, 268`: Cố định số lần thử lại đặt SL (`3` lần) và thời gian chờ (`150ms`).

---

## 🛠️ 3. Đề Xuất Tái Cấu Trúc Chi Tiết (Refactoring & Clean Solutions)

### 💡 Giải pháp 1: Đồng bộ Vốn Động (Dynamic Equity Injection) vào Strategy Context

**1. Sửa hàm `HeliosBridge::evaluate_live_tick` nhận `current_equity: f64`:**
```rust
// In src/optimizer/helios_bridge.rs
pub fn evaluate_live_tick(
    symbol: &str,
    candles: &[Candle],
    slice_start: usize,
    live_position: Option<&crate::controller::asset_actor::LivePositionState>,
    strategy: &mut dyn Strategy,
    current_equity: f64, // 👈 THÊM THAM SỐ VỐN ĐỘNG
) -> Vec<Action> {
    ...
    let mut tick_context = TickContext {
        i: last_idx,
        equity: current_equity, // 👈 SỬ DỤNG VỐN THỰC THẾ
        data_view: &data_view,
        indicator_manager: &indicator_manager,
        trades: &live_trades,
        actions: &mut actions_buffer,
        position: live_position_snapshot,
    };
    strategy.next(&mut tick_context);
    ...
}
```

**2. Trong `AssetActor::on_new_candle`, tính toán Vốn thực tế theo ExecutionMode:**
```rust
// In src/controller/asset_actor.rs
let current_equity = match self.execution_mode {
    ExecutionMode::Paper => self.paper_trader.get_total_equity(current_close),
    ExecutionMode::Testnet | ExecutionMode::RealTrade => {
        let u_pnl = self.position_state.unrealized_profit;
        (self.account_state.wallet_balance + u_pnl).max(self.allocated_capital)
    }
    _ => self.allocated_capital,
};

let actions = HeliosBridge::evaluate_live_tick(
    &self.symbol, 
    rolling_buffer, 
    slice_start, 
    self.live_position.as_ref(), 
    strategy.as_mut(),
    current_equity // 👈 TRUYỀN VỐN CHÍNH XÁC
);
```

---

### 💡 Giải pháp 2: Xử lý an toàn cờ `is_submitting_order` chống Deadlock (Atomic RAII Guard)

Sử dụng cờ `Arc<AtomicBool>` và đảm bảo cờ luôn được reset về `false` khi Task kết thúc (dù thành công hay gặp lỗi):

```rust
// In src/controller/asset_actor.rs
if is_historical {
    info!("...");
} else if let Some(ref executor) = self.executor {
    let usable_capital = self.account_state.available_balance;
    let leverage = self.leverage;
    let symbol = self.symbol.clone();
    let action_clone = action.clone();
    let executor_clone = executor.clone();
    
    self.is_submitting_order = true;
    let account_tx_clone = self.account_tx.clone();

    tokio::spawn(async move {
        let res = executor_clone.execute_action(&symbol, action_clone, current_close, usable_capital, position_amt, leverage).await;
        if let Err(e) = res {
            error!("❌ [{}] Lỗi thực thi lệnh qua OrderExecutor: {:?}", symbol, e);
            // 🛑 BÁO PHÁT EVENT RESET CỜ KHI CÓ LỖI REST API
            let _ = account_tx_clone.send(AccountEvent::OrderTradeUpdate {
                symbol,
                side: "".into(),
                order_type: "".into(),
                order_status: "REJECTED".into(),
                avg_price: 0.0,
                orig_qty: 0.0,
                executed_qty: 0.0,
                client_order_id: "".into(),
                order_id: 0,
            });
        }
    });
}
```

---

### 💡 Giải pháp 3: Lưu Vết Trạng Thái Vị Thế vào Local DB (Position & State Persistence)

Để giải quyết triệt để **Lỗi 4** và **Lỗi 5** (mất `live_entry_bar`, `opening_playbook_tag`, `SL/TP` khi bot restart):

1. **Ghi nhận vào SQLite khi phát lệnh:** Khi `OrderExecutor` hoặc `PaperTrader` mở lệnh thành công, lưu bản ghi vào bảng `active_positions` trong SQLite bao gồm: `(symbol, entry_price, position_amt, entry_timestamp, entry_bar, playbook_tag, sl_price, tp_price)`.
2. **Khôi phục khi Reconnect:** Khi `AssetActor` nhận được `PositionUpdate` từ Binance WS mà chưa có `live_position` trong bộ nhớ, nó sẽ truy vấn SQLite để nạp lại đúng `entry_bar`, `playbook_tag`, `sl_price`, `tp_price` ban đầu thay vì gán `None` hay nến hiện tại.

---

### 💡 Giải pháp 4: Chuẩn hóa Min Notional & Cấu Hình Execution

1. **Sử dụng Min Notional động:**
```rust
// In src/execution/order_executor.rs
let min_notional = self.market_filter.min_notional.max(5.0);
if notional < min_notional {
    return Err(BotError::MarketFilterError(format!(
        "Giá trị lệnh ({:.2} USDT) nhỏ hơn Min Notional ({:.2} USDT)", 
        notional, min_notional
    )));
}
```

2. **Quản lý Trade ID sinh tự động:** Thay vì gán cứng `trade_id = 1`, khởi tạo `trade_id` bằng hash timestamp hoặc số tăng dần theo từng phiên giao dịch.

---

## 📋 4. Kế Hoạch Triển Khai (Action Plan)

1. **Bước 1 (Ưu tiên số 1):** Sửa lỗi Hardcoded Equity trong `helios_bridge.rs` và truyền vốn động từ `AssetActor`.
2. **Bước 2 (Ưu tiên số 1):** Sửa lỗi Deadlock cờ `is_submitting_order` khi lệnh REST API thất bại.
3. **Bước 3 (Ưu tiên số 2):** Cập nhật `OrderExecutor` sử dụng `market_filter.min_notional` thay cho giá trị gán cứng `5.0`.
4. **Bước 4 (Ưu tiên số 2):** Bổ sung lưu trữ và khôi phục trạng thái vị thế (`LivePositionState`) qua SQLite Store để bảo toàn Tag, SL/TP và `entry_bar` khi Reconnect.

---
*Báo cáo được khởi tạo tự động bởi Research Subagent - Hệ thống Trading Bot.*
