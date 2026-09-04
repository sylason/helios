# BÁO CÁO KIỂM TOÁN TẦNG 7: KỊCH BẢN WALK-FORWARD, VÒNG ĐỜI ACTOR & QUẢN LÝ TRẠNG THÁI (SYSTEM HARNESS)

> **Mục tiêu:** Rà soát toàn diện phương pháp luận Walk-Forward Out-Of-Sample (`walk_forward_smart_seed.rs`, `walk_forward_evaluator.rs`), cơ chế DNA Hot-Swapping qua `watch::channel`, tính toàn vẹn của Máy trạng thái Actor, khả năng phục hồi lỗi từ SQLite và cơ chế Adopt & Protect trên sàn Binance.

---

## 📌 TỔNG HỢP DANH SÁCH LỖ HỔNG PHÁT HIỆN TẠI TẦNG 7

| Mã Lỗi | Thành phần | Vị trí (File & Dòng) | Mức độ | Bản chất Lỗ hổng |
| :--- | :--- | :--- | :---: | :--- |
| **FLAW-7.1** | `bot_engine` | `account_manager.rs:174-178` | 🚨 **CRITICAL** | **"Clean Slate Market Dump" Thay Vì "Adopt & Protect":** Khi khởi động bot, nếu phát hiện vị thế có sẵn trên Binance, bot tự động hủy toàn bộ SL và gửi lệnh Market ép đóng vị thế ngay lập tức! |
| **FLAW-7.2** | `bot_engine` | `live_engine.rs:69`, `order_executor.rs:148` | 🚨 **CRITICAL** | **Order Submission Deadlock:** Khi dời Trailing SL, bot bật cờ `is_submitting_order = true` nhưng không bao giờ reset về `false` (vì Algo SL chưa khớp nên sàn không gửi trade event), khóa chết toàn bộ lệnh Buy/Sell tiếp theo! |
| **FLAW-7.3** | `bot_engine` | `walk_forward_evaluator.rs:158-170` | 🚨 **CRITICAL** | **Rò Rỉ Dữ Liệu Tương Lai (Lookahead Leakage):** Quên truyền `.with_sync_until(train_end_ts)` khiến Actor nạp trước toàn bộ nến Out-Of-Sample (ngày 30–32) vào buffer trước khi OOS bắt đầu chạy! |
| **FLAW-7.4** | `bot_engine` | `live_engine.rs:45-89, 126-136` | 🚨 **CRITICAL** | **Phá Vỡ Tính Đơn Điệu (Monotonic Ratchet) Trên Live:** `live_engine.rs` không lưu `current_sl_price` trong RAM; khi DNA mới có SL rộng hơn, bot hủy SL chặt trên Binance và nới lỏng SL lùi xuống! |
| **FLAW-7.5** | `bot_engine` | `prospector_runner.rs:270`, `asset_actor.rs:154` | 🚨 **CRITICAL** | **Thiếu Validate DNA (`len`, `NaN`, `Inf`):** Lệch độ dài gene gây panic `index out of bounds` làm sập Actor; `NaN` trong DNA làm tê liệt hoàn toàn logic dời Trailing SL. |
| **FLAW-7.6** | `bot_engine` | `sqlite_impl.rs:30-72`, `repository.rs` | 🚨 **CRITICAL** | **Zero SQLite Position Storage:** SQLite không có bảng vị thế; crash/restart là mất sạch trạng thái Paper Trading và metadata Live (100% RAM-only). |
| **FLAW-7.7** | `helios` | `risk.rs:242`, `live_engine.rs:132` | 🚨 **CRITICAL** | **Vị Thế Tiếp Quản Bị Thả Nổi Không Có SL (Naked Position):** Nếu vị thế tiếp quản đang âm (`current_pnl <= 0.0`), `risk.rs` bỏ qua logic đặt SL vì chỉ kích hoạt khi `current_pnl > 0.0`! |
| **FLAW-7.8** | `bot_engine` | `walk_forward_smart_seed.rs:215` | 🚨 **MAJOR** | **Trùng Lặp Nến Ranh Giới (Boundary Overlap):** Do SQLite dùng `>= train_end_ts`, nến ranh giới bị nạp 2 lần (lần 1 trong buffer lịch sử, lần 2 trong stream OOS), kích hoạt tick kép. |
| **FLAW-7.9** | `bot_engine` | `account_manager.rs:85-90` | 🚨 **MAJOR** | **Cross Margin API Failure:** Binance báo lỗi `-4067 / -4068` khi đổi margin type lúc có vị thế; bot bỏ qua lỗi và chạy ngầm với rủi ro Cross Margin toàn bộ ví Futures. |
| **FLAW-7.10** | `helios` | `risk.rs:133-146` | 🟠 **HIGH** | **Bẫy Đóng Lệnh Cưỡng Bức Do `decay_cycles` Jump:** Hot-swap DNA có `time_countdown_bars` nhỏ hơn làm `decay_cycles` nhảy vọt, đóng ngay lập tức các vị thế đang âm nhẹ do phí. |
| **FLAW-7.11** | `bot_engine` | `paper_trader.rs:60-80` | 🟠 **HIGH** | **Thiếu Đo Đạc Max Drawdown Trong PaperTrader:** Không theo dõi `peak_equity` và `max_drawdown_pct`, khiến người dùng mù về rủi ro sụt giảm vốn thực tế trong Forward Test. |
| **FLAW-7.12** | `bot_engine` | `walk_forward_smart_seed.rs:241` | 🟠 **HIGH** | **Ngộ Nhận "Diagnostic Backtest" Là Out-Of-Sample:** Chạy backtest 30 ngày độc lập với $60 thực chất là đo lại In-Sample, không đại diện cho hiệu suất Out-Of-Sample thực tế. |
| **FLAW-7.13** | `bot_engine` | `live_engine.rs:130`, `helios_bridge.rs:220` | 🟠 **HIGH** | **High-Watermark & Age Reset:** `entry_timestamp` bị gán `now()`, biến tuổi nến về 0, làm mất toàn bộ đỉnh lợi nhuận lịch sử của vị thế trước khi bot restart. |
| **FLAW-7.14** | `bot_engine` | `asset_actor.rs:173-181` | 🟡 **MEDIUM** | **Nghẽn Async Event Loop Do `diagnostic_backtest`:** Chạy backtest 43k nến đồng bộ trong `tokio::select!` làm rơi rụng các gói tin nến WebSocket (`data_rx LAGGED`). |
| **FLAW-7.15** | `bot_engine` | `resilience_recovery_test.rs:54-76` | 🟡 **MEDIUM** | **Bài Test Phục Hồi Giả Tạo (Mock Assertion):** Testcase chỉ tự gán biến struct trong bộ nhớ rồi assert chính biến đó, không hề kiểm tra SQLite hay phục hồi thực tế. |
| **FLAW-7.16** | `bot_engine` | `models/models.rs:37-43` | 🟡 **MEDIUM** | **Lệch Chuẩn FSM Giữa Thiết Kế Và Code:** Code thiếu các trạng thái `INACTIVE`, `PAUSED`, `GRACEFUL_STOPPING`, khiến bot không thể tạm dừng hay dừng an toàn. |

---

## 🔍 CHI TIẾT CÁC LỖ HỔNG ĐẶC BIỆT NGUY HIỂM

### 1. FLAW-7.1 [CRITICAL]: "Clean Slate Market Dump" Thay Vì "Adopt & Protect"
- **Hiện trạng:**
  Tại `bot_engine/src/execution/account_manager.rs` (dòng 174–178):
  ```rust
  if let Some(positions) = acc_info.positions {
      for pos in positions {
          if pos.symbol.as_deref() == Some(symbol) {
              // ⚠️ CLEAN SLATE: Tự động hủy lệnh và bán tháo vị thế!
              let _ = self.cancel_all_orders(symbol).await;
              let _ = self.force_close_all_positions(symbol, position_amt).await;
          }
      }
  }
  ```
- **Hậu quả:**
  Thay vì tiếp quản vị thế có sẵn và đặt Trailing SL bảo vệ như thiết kế, code thực hiện **tiêu diệt vị thế (Clean Slate)**: Hủy sạch SL và gửi lệnh MARKET đóng vị thế ngay lập tức! Người dùng vừa bật bot thì vị thế thủ công đẹp bị bán tháo chịu lỗ và phí Taker!

---

### 2. FLAW-7.2 [CRITICAL]: Deadlock Đóng Băng Giao Dịch Vì Lệnh Trailing SL
- **Hiện trạng:**
  Tại `bot_engine/src/execution/live_engine.rs` (dòng 69) và `order_executor.rs` (dòng 148):
  1. Khi nhận hành động `Action::UpdateTradeSlTp`, hàm `execute_action` bật:
     `self.is_submitting_order = true;`
  2. Lệnh Algo `STOP_MARKET` được gửi lên sàn thành công và trả về `Ok(())`.
  3. Nhưng vì là lệnh điều kiện chưa khớp, sàn Binance **không hề gửi sự kiện khớp lệnh (`OrderTradeUpdate`)** về WebSocket!
  4. Cờ `is_submitting_order` **MÃI MÃI LÀ `true`**!
  5. Ở các cây nến tiếp theo, mọi lệnh `Action::Buy` và `Action::Sell` đều bị chặn đứng:
     ```rust
     if position_amt > 0.0 || self.is_submitting_order {
         return Ok(()); // ⚠️ Từ chối 100% lệnh giao dịch mới!
     }
     ```
- **Hậu quả:** Ngay sau lần dời Trailing SL đầu tiên, bot Live rơi vào trạng thái **Deadlock hoàn toàn**, không thể mở bất kỳ vị thế nào nữa!

---

### 3. FLAW-7.3 [CRITICAL]: Rò Rỉ Dữ Liệu Tương Lai Trong `walk_forward_evaluator.rs`
- **Hiện trạng:**
  Tại dòng 158–170 của `walk_forward_evaluator.rs`:
  ```rust
  let actor = AssetActor::new(...).with_strategy(strategy);
  // ⚠️ THIẾU HOÀN TOÀN: .with_sync_until(train_end_ts)
  ```
- **Hậu quả:**
  Trong `asset_actor.rs:293-299`, khi `initial_sync_until` là `None`, nó mặc định lấy thời gian hiện tại (`now`). Toàn bộ 32 ngày nến (bao gồm cả giai đoạn Out-Of-Sample ngày 30–32) **đã được nạp sẵn vào Rolling Buffer của Actor trước khi OOS bắt đầu chạy**! Sau đó vòng lặp OOS lại tiếp tục bơm các nến này lần thứ 2, làm sai lệch hoàn toàn tính khoa học của bài kiểm thử.

---

### 4. FLAW-7.4 [CRITICAL]: Phá Vỡ Tính Đơn Điệu (Monotonic Ratchet) Trên Sàn Binance Live
- **Hiện trạng:**
  - Trong `PaperExecutionEngine`, `pos.current_sl_price` được lưu trữ liên tục trong RAM.
  - Nhưng trong `live_engine.rs`, `execute_action` gửi lệnh lên sàn nhưng **không hề cập nhật `current_sl_price`**! Khi nhận event từ sàn, `current_sl_price` luôn bị gán là `None`.
  - Nến tiếp theo gọi `HeliosBridge::evaluate_live_tick`, `trade.sl_price` nhận `None` $\rightarrow$ trở thành `-INFINITY`.
- **Hậu quả:**
  Điều kiện `calculated_sl > -INFINITY` **luôn luôn đúng trên mọi cây nến**! Khi nhận DNA mới có biên độ SL rộng hơn, bot sẽ hủy SL chặt hiện tại trên Binance và đặt lại mức SL nới lỏng xuống dưới, vi phạm trực tiếp nguyên tắc bảo vệ vốn Monotonic Ratchet!

---

### 5. FLAW-7.7 [CRITICAL]: Vị Thế Tiếp Quản Bị Bỏ Mặc Trần Trụi Không Có SL Khi Đang Âm
- **Hiện trạng:**
  Trong `strategies_backtests/helios/src/strategy/core_v3/risk.rs` (dòng 242):
  ```rust
  // Chỉ kích hoạt khi vị thế có lãi danh nghĩa (current_pnl > 0.0)
  if current_pnl > 0.0 {
      let should_update = ...;
      if should_update { tick.update_trade_sl_tp(...); }
  }
  ```
- **Hậu quả:**
  Khi tiếp quản vị thế có sẵn mà vị thế đó đang âm (`current_pnl <= 0.0`) và chưa có SL (`trade.sl_price = None`): Khối code trên bị bỏ qua hoàn toàn! Bot **không bao giờ đặt Stop Loss lên sàn Binance**, bỏ mặc vị thế trần trụi (Naked Position) cho đến khi tài khoản bị thanh lý cháy sạch!

---

## 🛡️ ĐIỂM SÁNG XÁC NHẬN TRONG TẦNG 7

- **Bảo toàn Tính Liên Tục Vị Thế (State Continuity):** Trong `paper_forward_test.rs`, khi nhận DNA mới, vị thế mở không bị đóng ép mà được chuyển tiếp trơn tru cho bộ tham số mới quản lý.
- **Tính Liên Tục Của Vốn Kép (Compounding):** Trong `PaperTrader`, vốn `equity` và khối lượng lệnh được tích lũy liên tục 100% qua tất cả các chu kỳ Out-Of-Sample, phản ánh chuẩn xác cơ chế lãi kép thực tế.
- **Phòng Ngừa Double Entry:** 2 chốt chặn độc lập tại Strategy (`is_pos_open`) và ExecutionEngine (`position_amt > 0.0`) ngăn chặn tuyệt đối việc mở lệnh kép cùng chiều.
