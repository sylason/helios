# 🔬 Báo Cáo Kiểm Toán TẦNG 8: Hạ Tầng Mạng, Rate Limit, Quy Chuẩn Sàn & Desktop UI

> **Dự án kiểm toán:** `trading_bot` (`bot_engine` & `bot_desktop_client`)  
> **Trọng tâm:** WebSocket Streaming, Phòng Ngự API 2400 Weight, Làm Tròn Số Thập Phân (Market Rules), Tích Hợp Tauri v2 IPC, và Mã Nguồn Treo.  
> **Ngày thực hiện:** 2026-09-04  

---

## Executive Summary

Sau khi hoàn thành 7 Tầng cốt lõi của thuật toán và khớp lệnh, việc mở rộng rà soát toàn bộ các module ngoại vi, hạ tầng và giao diện của `trading_bot` đã phát hiện **10 lỗ hổng nghiêm trọng**, giải thích vì sao bot có thể bị tê liệt ngoài đời thực:

| Mã Lỗi | Module | Mức Độ | Tóm Tắt Bản Chất Lỗi |
| :--- | :--- | :---: | :--- |
| **Flaw 8.1** | `ws_manager.rs:139` | **CRITICAL** | WebSocket rơi vào vòng lặp `sleep(60)` vĩnh viễn khi đứt mạng, không bao giờ reconnect |
| **Flaw 8.2** | `market_info.rs:45` | **CRITICAL** | `round_qty` dùng `.round()` thay vì `.floor()`, gây lỗi sàn `-2019 Margin Insufficient` hoặc bán khống vượt vị thế |
| **Flaw 8.3** | `order_executor.rs:77,84` | **CRITICAL** | `.unwrap()` trên `Decimal::from_f64(price/qty)` làm panic sập toàn bộ Actor nếu dính `NaN`/`Inf` |
| **Flaw 8.4** | `bot_desktop_client` | **CRITICAL** | Hố ngăn cách 100% giữa Tauri v2 IPC và Engine: Backend Tauri chỉ là template `greet`, 0 lệnh bot được nối |
| **Flaw 8.5** | `ws_manager.rs:74` | **MAJOR** | Vi phạm thiết kế Combined Streams: Mỗi symbol mở 1 kết nối WS riêng và nhồi chung vào 1 broadcast bus |
| **Flaw 8.6** | `ws_manager.rs:248` | **MAJOR** | Trade API WebSocket chỉ là vòng lặp `sleep(60)` rỗng (Dummy Stub), lệnh thật vẫn bắn qua REST trễ |
| **Flaw 8.7** | `weight_monitor.rs` | **MAJOR** | Xung đột & trùng lặp giữa 2 bộ `WeightMonitor` độc lập với ngưỡng và logic reset không đồng nhất |
| **Flaw 8.8** | `asset_actor.rs:211` | **HIGH** | Kênh Broadcast bị `Lagged` làm rớt nến vĩnh viễn, không có cơ chế bù nến khiến chuỗi nến bị khuyết |
| **Flaw 8.9** | `main.rs:219` | **HIGH** | Thoát bàn phím (Ctrl+C) tự động bán tháo Market (`PanicCloseAll`), bỏ qua Bảng Thoát Tổng Hợp |
| **Flaw 8.10**| `duckdb/interface` | **MEDIUM** | Mã nguồn ma: `duckdb_impl.rs` (308 dòng), `binance_stream.rs`, `interface/mod.rs` (0B) bị bỏ hoang |

---

## 🔍 Chi Tiết 10 Lỗ Hổng Kỹ Thuật

### 🚨 Flaw 8.1 (CRITICAL): Bẫy Ngủ Vô Tận Khiến WebSocket Không Bao Giờ Tự Reconnect Khi Rớt Mạng
- **File:** [`trading_bot/bot_engine/src/execution/ws_manager.rs:139-151`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs#L139-L151)
- **Bằng chứng mã nguồn:**
  ```rust
  stream.on_message(move |message| { ... });

  loop {
      sleep(Duration::from_secs(60)).await;
  }
  // Khối kết nối lại dưới đây không bao giờ chạm tới được!
  warn!("♻️ Đang thử kết nối lại Market Data sau 5 giây...");
  sleep(Duration::from_secs(5)).await;
  ```
- **Phân tích:** Callback `on_message` được SDK đăng ký ngầm. Sau đó luồng rơi vào `loop { sleep(60).await; }`. Khi Binance ngắt kết nối WebSocket (theo quy định sàn tự ngắt mỗi 24h hoặc khi rớt mạng), `stream` bị đóng nhưng vòng lặp `sleep` vẫn chạy mãi mãi. Khối lệnh kết nối lại ở dòng 149 **không bao giờ được thực thi**. Bot rơi vào trạng thái "chết lâm sàng" (zombie): không có nến mới nhưng tiến trình không hề báo lỗi.

---

### 🚨 Flaw 8.2 (CRITICAL): Làm Tròn Khối Lượng Dùng `.round()` Gây Sập Lệnh Sàn Do Thiếu Ký Quỹ
- **File:** [`trading_bot/bot_engine/src/models/market_info.rs:41-50`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/market_info.rs#L41-L50)
- **Bằng chứng mã nguồn:**
  ```rust
  pub fn round_qty(&self, qty: f64) -> f64 {
      if self.step_size <= 0.0 { return qty; }
      let rounded = (qty / self.step_size).round() * self.step_size;
      ...
  }
  ```
- **Phân tích:** Trong giao dịch phái sinh Futures, việc làm tròn khối lượng (`step_size`) **bắt buộc phải làm tròn xuống (`floor`)**, tuyệt đối không được dùng `.round()`:
  1. Khi mở lệnh: Nếu số dư khả dụng tối đa cho phép mua 0.1235 BTC, `.round()` với `step_size = 0.001` sẽ làm tròn thành **0.124 BTC**. Khi gửi lên Binance, sàn lập tức từ chối lệnh với mã lỗi `-2019: Margin is insufficient`.
  2. Khi đóng lệnh: Nếu vị thế đang nắm giữ là 0.1234 BTC, `.round()` làm tròn thành 0.124 BTC, bot gửi lệnh đóng vượt quá số lượng đang có, dẫn đến lỗi sàn hoặc vô tình mở vị thế ngược chiều.

---

### 🚨 Flaw 8.3 (CRITICAL): Nguy Cơ Panic Sập Toàn Bộ Bot Do `.unwrap()` Trực Tiếp Trên Decimal
- **File:** [`trading_bot/bot_engine/src/execution/order_executor.rs:74-85`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs#L74-L85)
- **Bằng chứng mã nguồn:**
  ```rust
  fn format_price_decimal(&self, price: f64) -> rust_decimal::Decimal {
      let rounded = self.market_filter.round_price(price);
      let decimals = MarketFilter::count_decimals(self.market_filter.tick_size);
      rust_decimal::Decimal::from_f64(rounded).unwrap().round_dp(decimals as u32)
  }
  ```
- **Phân tích:** `rust_decimal::Decimal::from_f64` trả về `None` nếu số thực đầu vào là `NaN` hoặc `+Inf / -Inf`. Khi một chỉ báo lỗi hoặc giá Stop Loss tính toán ra `NaN`, gọi `.unwrap()` sẽ gây **Panic trực tiếp làm sập luồng của Actor**, kéo theo ngắt toàn bộ tiến trình bot đang chạy tiền thật!

---

### 🚨 Flaw 8.4 (CRITICAL): Hố Ngăn Cách Hoàn Toàn Giữa Desktop UI (Tauri v2) Và Bot Engine
- **File:** [`trading_bot/bot_desktop_client/src-tauri/src/lib.rs:1-15`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_desktop_client/src-tauri/src/lib.rs#L1-L15)
- **Bằng chứng mã nguồn:**
  ```rust
  #[tauri::command]
  fn greet(name: &str) -> String { format!("Hello, {}!", name) }

  pub fn run() {
      tauri::Builder::default()
          .invoke_handler(tauri::generate_handler![greet])
          ...
  }
  ```
- **Phân tích:**
  - `bot_desktop_client/src-tauri/Cargo.toml` **không hề khai báo dependency** tới `bot_engine`.
  - Frontend React (`botService.ts`) gọi các hàm IPC: `cmd_activate_asset`, `cmd_terminate_graceful`, `cmd_terminate_immediate`, `cmd_emergency_stop_all`.
  - Backend Tauri v2 chỉ có đúng hàm mẫu `greet`. Không có bất kỳ lệnh nào được cài đặt. UI hiện tại chỉ là "vỏ rỗng" chạy ở chế độ Mock và hoàn toàn không thể điều khiển được Bot Engine.

---

### ⚠️ Flaw 8.5 (MAJOR): Vi Phạm Kiến Trúc "Single Shared WS Combined Streams"
- **File:** [`trading_bot/bot_engine/src/execution/ws_manager.rs:74-93`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs#L74-L93)
- **Phân tích:** Kiến trúc chuẩn quy định gom tất cả các stream nến vào 1 kết nối WebSocket duy nhất (`combined streams`) và định tuyến qua bảng băm $O(1)$. Tuy nhiên trong code:
  - Mỗi khi gọi `start_market_data_stream(symbol)`, một kết nối vật lý mới được mở. Nếu chạy 10 cặp tiền, bot mở 10 kết nối WebSocket riêng biệt.
  - Toàn bộ nến từ mọi cặp tiền đều đẩy chung vào một kênh `data_tx: broadcast::Sender<(Candle, bool)>`, ép tất cả `AssetActor` phải đọc và tự kiểm tra `if candle.symbol == self.symbol`, gây lãng phí chu kỳ CPU nghiêm trọng.

---

### ⚠️ Flaw 8.6 (MAJOR): Trade API WebSocket Bị Bỏ Hoang (Dummy Stub Loop)
- **File:** [`trading_bot/bot_engine/src/execution/ws_manager.rs:248-284`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs#L248-L284)
- **Bằng chứng mã nguồn:**
  ```rust
  pub async fn start_trade_api_stream(&self) {
      ...
      match client.connect().await {
          Ok(_connection) => {
              // Ở phiên bản hiện tại, chỉ loop để giữ kết nối không bị ngắt.
              loop { sleep(Duration::from_secs(60)).await; }
          }
      }
  }
  ```
- **Phân tích:** Luồng WebSocket API siêu tốc phục vụ đặt lệnh chỉ được kết nối rồi `sleep`, hoàn toàn không có kênh chuyển phát lệnh. Trong thực tế, `OrderExecutor` vẫn sử dụng 100% REST API, gây hao tổn giới hạn Request Weight của Binance và có độ trễ cao hơn WS API từ 50-200ms.

---

### ⚠️ Flaw 8.7 (MAJOR): Trùng Lặp & Mâu Thuẫn Quản Trị API Weight Giữa 2 Monitor
- **Files:** [`models/weight_monitor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/weight_monitor.rs) & [`execution/rate_limit_monitor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/rate_limit_monitor.rs)
- **Phân tích:** Hệ thống có tới 2 struct `WeightMonitor` nằm ở 2 file khác nhau với logic xung đột:
  - `models/weight_monitor.rs`: Ngưỡng ngắt 90% (2160), tự reset về 0 bằng phép chia `now / 60000 > last_update / 60000`.
  - `execution/rate_limit_monitor.rs`: Ngưỡng cảnh báo 1920 (80%), khẩn cấp 2200 (92%), khi chạm ngưỡng thì ngủ 30s rồi gán về 0.
  - `rate_limit_monitor.rs` là mã nguồn mồ côi không hề được nối vào `OrderExecutor`, dẫn đến việc kiểm soát weight bị phân mảnh.

---

### ⚠️ Flaw 8.8 (HIGH): Bỏ Rơi Nến Vĩnh Viễn Khi Kênh Broadcast Bị Lag
- **File:** [`trading_bot/bot_engine/src/controller/asset_actor.rs:211-213`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L211-L213)
- **Bằng chứng mã nguồn:**
  ```rust
  Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
      debug!("⚠️ [{}] AssetActor data_rx LAGGED! Dropped {} candles!", self.symbol, skipped);
  }
  ```
- **Phân tích:** Khi một Actor bận tính toán nặng (ví dụ JIT GA hoặc Diagnostic Backtest), bộ đệm kênh `broadcast(4096)` bị đầy và Tokio tự động hủy bỏ các nến cũ. Khi nhận `RecvError::Lagged`, Actor chỉ ghi log mức `debug` rồi... tiếp tục chạy bình thường! Không có cơ chế kích hoạt `fill_gap` để bù nến thiếu. Hậu quả là `rolling_buffer` bị thủng lỗ (mất nến), làm sai lệch toàn bộ mảng tính toán kỹ thuật (EMA, ATR) từ đó về sau.

---

### ⚠️ Flaw 8.9 (HIGH): Dừng Bot Bằng Bàn Phím (Ctrl+C) Tự Động Bán Tháo Market Không Hỏi Ý Kiến
- **File:** [`trading_bot/bot_engine/src/main.rs:219-224`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/main.rs#L219-L224)
- **Bằng chứng mã nguồn:**
  ```rust
  _ = tokio::signal::ctrl_c() => {
      warn!("🚨 Nhận lệnh dừng từ bàn phím. Đang dọn dẹp...");
      let _ = emergency_tx.send(crate::models::models::EmergencyCmd::PanicCloseAll).await;
      tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
  }
  ```
- **Phân tích:** Khi người dùng nhấn Ctrl+C, bot tự động kích hoạt `PanicCloseAll` để bán tháo tất cả vị thế ở giá Market. Điều này vi phạm quy chuẩn thiết kế tại `system_tray_and_exit_protocol.md`: bot phải cho phép người dùng lựa chọn giữ nguyên vị thế và để Trailing Stop Loss trên đám mây của Binance tiếp tục bảo vệ (`Leave Cloud SL & Exit`) thay vì chịu trượt giá bán tháo.

---

### ⚠️ Flaw 8.10 (MEDIUM): Rác Mã Nguồn & Module Bỏ Hoang (Ghost Modules)
- **File:**
  - `trading_bot/bot_engine/src/data_engine/duckdb_impl.rs`: 308 dòng mã nguồn DuckDB không ai gọi.
  - `trading_bot/bot_engine/src/data_engine/binance_stream.rs`: Luồng WebSocket 1m riêng lẻ bị bỏ hoang.
  - `trading_bot/bot_engine/src/interface/mod.rs`: File rỗng (0 bytes).
  - `trading_bot/bot_engine/src/strategy_runner/mod.rs`: File rỗng (0 bytes).
- **Hệ quả:** Làm phân tán sự chú ý khi bảo trì, gây hiểu nhầm về kiến trúc lưu trữ và giao tiếp của bot.
