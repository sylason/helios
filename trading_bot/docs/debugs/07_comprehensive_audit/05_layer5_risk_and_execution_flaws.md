# BÁO CÁO KIỂM TOÁN TẦNG 5: QUẢN TRỊ RỦI RO & KHỚP LỆNH NẾN (RISK & EXECUTION MATCHING)

> **Mục tiêu:** Rà soát toàn diện động cơ khớp lệnh mô phỏng (`ferro_trade::Broker`), thuật toán quản trị rủi ro / Trailing SL (`helios::RiskManager`), và sự đồng nhất khớp lệnh giữa Backtest, Paper Trader và Live Binance.

---

## 📌 TỔNG HỢP DANH SÁCH LỖ HỔNG PHÁT HIỆN TẠI TẦNG 5

| Mã Lỗi | Thành phần | Vị trí (File & Dòng) | Mức độ | Bản chất Lỗ hổng |
| :--- | :--- | :--- | :---: | :--- |
| **FLAW-5.1** | `ferro_trade` | `broker.rs:332-345` & `engine.rs:331-338` | 🚨 **CRITICAL** | **Bug Cháy Tài Khoản (`equity <= 0`):** Lệnh bị hoãn không đóng ngay, trượt sang nến cuối cùng (`final_bar_index`) làm sai lệch toàn bộ PnL và thời gian thoát lệnh. |
| **FLAW-5.2** | `ferro_trade` | `broker.rs:19, 326-332` | 🚨 **CRITICAL** | **Bỏ qua râu nến khi thanh lý trong nến:** `equity` chỉ tính tại giá `Close`. Nến quét râu làm âm tài khoản trong phiên nhưng rút chân cuối nến vẫn sống sót; `margin_rate` là dead field. |
| **FLAW-5.3** | `trading_bot` | `order_executor.rs:240-270` | 🚨 **CRITICAL** | **Naked Position Vulnerability (Live Binance):** Hủy SL cũ -> chờ 150ms -> đặt SL mới. Nếu đặt SL mới lỗi sau 3 lần retry, hàm nuốt lỗi trả về `Ok(())`, bỏ mặc vị thế không có SL bảo vệ. |
| **FLAW-5.4** | `trading_bot` | `live_engine.rs` & `order_executor.rs` | 🚨 **CRITICAL** | **Thiếu Take Profit trên Binance Live:** Bot Live chỉ xử lý lệnh SL, hoàn toàn không gửi lệnh TP lên Binance Futures, làm lỡ nhịp chốt lời khi giá chạm TP rồi giật ngược. |
| **FLAW-5.5** | `bot_engine` | `paper_trader.rs` vs `broker.rs:137` | 🚨 **CRITICAL** | **Lệch pha Khớp Nến 2 Đầu giữa Backtest & Paper:** Backtest luôn bi quan 100% SL thắng; Paper Trader lại đo khoảng cách tới `Open`, ưu tiên cắn TP trước nếu TP gần `Open` hơn $\rightarrow$ Thổi phồng PnL ảo ở Paper. |
| **FLAW-5.6** | `helios` | `risk.rs:13-24` | 🟠 **HIGH** | **Nguy cơ `NaN` trong `calculate_size` khi Vốn $\le 0$:** Phép chia `0.0 / 0.0` sinh ra `NaN`. `NaN.min(0.20)` trả về `0.20`, tiếp tục cấp margin 20% cho tài khoản đã cháy. |
| **FLAW-5.7** | `helios` | `risk.rs:211` | 🟡 **MEDIUM** | **Break-even Buffer (0.08%) không bù được Trượt Giá:** Chỉ đủ bù phí danh nghĩa 0.04% x 2. Khi gặp trượt giá sàn lúc khớp lệnh Market, lệnh dời hòa vốn vẫn bị lỗ ròng (Net Negative). |
| **FLAW-5.8** | `ferro_trade` | `broker.rs:137-142` | 🟡 **MEDIUM** | **Pessimistic Bias Tuyệt Đối ở Nến Quét 2 Đầu:** Bất kể khoảng cách từ Open, lệnh SL luôn chèn `push_front` và ăn 100%, dìm Sharpe/Winrate của backtest xuống dưới mức thực tế. |

---

## 🔍 CHI TIẾT CÁC LỖ HỔNG NGHIÊM TRỌNG

### 1. FLAW-5.1 [CRITICAL]: Bug Cháy Tài Khoản (`equity <= 0.0`) Bị Trượt Sang Nến Cuối Cùng
- **Hiện trạng:**
  Trong `ferro_trade/src/broker/broker.rs` (dòng 332–345):
  ```rust
  if equity <= 0.0 {
      let last_price = self.data.close_at(self.i);
      let trades_to_close: Vec<usize> = self.trades.iter().map(|t| t.id).collect();
      for id in trades_to_close {
          self.close_trade(id, 1.0); // ⚠️ Chỉ tạo Order đẩy vào queue self.orders!
      }
      self.cash = 0.0;
      return Err("Out of money");
  }
  ```
  Và tại `ferro_trade/src/backtest/engine.rs` (dòng 249, 331–338):
  ```rust
  if broker.next(i).is_err() { break; } // Vòng lặp ngắt ngay lập tức!
  // Sau đó chạy khối finalize_trades:
  if self.finalize_trades {
      let final_bar_index = full_len.saturating_sub(1);
      // Gọi broker.next(final_bar_index) -> Trade bị ép đóng tại nến CUỐI CÙNG!
  }
  ```
- **Hậu quả:**
  Hàm `self.close_trade(id, 1.0)` không hề đóng lệnh ngay mà chỉ đẩy một lệnh đóng vào hàng đợi `orders`. Nhưng dòng tiếp theo lại gọi `return Err("Out of money")`, khiến chu trình khớp lệnh bị hủy bỏ!
  Sau khi ngắt vòng lặp, khối `finalize_trades` đóng vị thế này ở nến **`final_bar_index`** (ví dụ nến 43,100 thay vì nến 500 lúc tài khoản bị cháy). Kết quả: Thời gian giữ lệnh (Duration), giá đóng lệnh (Exit Price) và PnL bị bóp méo hoàn toàn so với thực tế!

---

### 2. FLAW-5.2 [CRITICAL]: Bỏ Qua Râu Nến Khi Thanh Lý Trong Nến & Dead Field `margin_rate`
- **Hiện trạng:**
  - Struct `Broker` khai báo `pub margin_rate: f64` (`broker.rs:19`), nhưng trường này **hoàn toàn không được sử dụng ở bất kỳ dòng code nào** trong cả file.
  - Điều kiện kiểm tra cháy tài khoản chỉ thực hiện ở cuối nến thông qua `self.equity()` (`broker.rs:326-332`), lấy giá đóng cửa `Close`:
    ```rust
    let last_price = self.data.get_current_close().unwrap_or(0.0);
    ```
- **Hậu quả:**
  Nếu trong phiên nến, giá sụt giảm cực mạnh tạo râu nến sâu (`Low` đối với Long hoặc `High` đối với Short) vượt xa mức thanh lý của sàn Binance, nhưng đến cuối nến giá giật ngược rút chân đóng nến ở mức an toàn $\rightarrow$ **Backtest vẫn cho vị thế sống sót bình thường!** Trong khi trên thực tế sàn Binance Futures đã kích hoạt Liquidation Call và cháy sạch tài khoản từ râu nến.

---

### 3. FLAW-5.3 [CRITICAL]: Naked Position Vulnerability Trên Binance Live Execution
- **Hiện trạng:**
  Tại `trading_bot/bot_engine/src/execution/order_executor.rs` (dòng 240–270):
  ```rust
  // Hủy Stop Loss cũ
  if let Some(old_id) = current_sl_order_id {
      let _ = self.client.cancel_order(symbol, old_id).await;
      tokio::time::sleep(Duration::from_millis(150)).await; // Chờ hủy hoàn tất
  }
  // Thử đặt Stop Loss mới (retry tối đa 3 lần)
  for attempt in 0..3 {
      match self.place_algo_order(...).await {
          Ok(res) => return Ok(res.order_id),
          Err(e) => warn!("Retry đặt SL thất bại lần {attempt}: {e}"),
      }
  }
  // ⚠️ Sau 3 lần thất bại: Nuốt lỗi, trả về Ok(0) hoặc log warning rồi tiếp tục!
  ```
- **Hậu quả:**
  Hệ thống áp dụng cơ chế "Hủy trước -> Đặt sau" để tránh vi phạm quy tắc số lượng lệnh Algo của sàn. Tuy nhiên, nếu sau khi hủy SL cũ, đường truyền mạng bị ngắt hoặc sàn trả về lỗi `LOT_SIZE / MIN_NOTIONAL / PRICE_FILTER` trong 3 lần retry, vị thế trên Binance Futures sẽ **HOÀN TOÀN TRẦN TRỤI (0% Stop Loss)** mà bot không hề dừng lại hay cảnh báo khẩn cấp. Nếu giá đảo chiều mạnh sau đó, tài khoản sẽ bị cháy rụi mà không có cơ chế tự vệ!

---

### 4. FLAW-5.4 [CRITICAL]: Hoàn Toàn Thiếu Lệnh Take Profit Trên Binance Live
- **Hiện trạng:**
  Khi kiểm tra toàn bộ luồng khớp lệnh của `trading_bot/bot_engine/src/execution/live_engine.rs` và `order_executor.rs`:
  - Bot chỉ khởi tạo và dời lệnh `STOP_MARKET` cho Trailing Stop Loss.
  - Hoàn toàn **không có bất kỳ dòng code nào** gửi lệnh Take Profit (`tp_price`) lên sàn Binance!
- **Hậu quả:**
  Trên Backtest và Paper Trading, chiến lược có cơ chế chốt lời theo TP đề xuất từ Playbook Scanners (`tp_price`). Tuy nhiên trên sàn Live thực tế, lệnh TP không bao giờ được đặt. Khi giá chạm đỉnh mục tiêu rồi quay đầu sụt giảm, bot bỏ lỡ 100% lợi nhuận và phải chờ Trailing SL kéo theo sau hoặc bị quét hòa vốn.

---

### 5. FLAW-5.5 [CRITICAL]: Lệch Pha Khớp Nến Quét 2 Đầu Giữa Backtest & Paper Trader
- **Hiện trạng:**
  - Trong `ferro_trade::Broker` (Backtest): Lệnh SL được ưu tiên xếp vào đầu hàng đợi (`push_front`), lệnh TP xếp cuối hàng đợi (`push_back`). Khi nến quét cả SL và TP, **SL luôn thắng 100% (Giả định bi quan nhất)**.
  - Trong `PaperTrader` (`paper_trader.rs`):
    ```rust
    let sl_dist = (open - sl).abs();
    let tp_dist = (open - tp).abs();
    if tp_dist < sl_dist {
        // Khớp TP trước!
    } else {
        // Khớp SL trước!
    }
    ```
- **Hậu quả:**
  Paper Trader và Backtest có cơ chế mô phỏng xung đột trực diện. Khi TP gần giá mở cửa hơn SL, Paper Trader tuyên bố chốt lời thành công, trong khi Backtest ghi nhận thua lỗ do dính SL. Điều này tạo ra sự **thổi phồng hiệu suất (Optimistic PnL Bias)** trong quá trình Paper Forward Test, khiến chiến lược nhìn có vẻ thắng lớn trên Paper nhưng Backtest lại cho kết quả xấu.

---

### 6. FLAW-5.6 [HIGH]: Nguy Cơ `NaN` Trong `calculate_size` Khi Vốn $\le 0$
- **Hiện trạng:**
  Tại `strategies_backtests/helios/src/strategy/core_v3/risk.rs` (dòng 13–24):
  ```rust
  pub fn calculate_size(equity: f64, risk_pct: f64, min_order_usd: f64, leverage: f64) -> f64 {
      let mut target_margin = equity * risk_pct;
      ...
      let size_fraction = target_margin / equity;
      size_fraction.min(0.20)
  }
  ```
- **Hậu quả:**
  Nếu `equity <= 0.0` (tài khoản cạn vốn hoặc âm do trượt giá/phí), phép chia `0.0 / 0.0` cho kết quả `NaN`. Theo đặc tả IEEE-754 của Rust, `NaN.min(0.20)` trả về `0.20`. Hệ thống vẫn tiếp tục duyệt mở vị thế với tỷ lệ ký quỹ 20% trên một tài khoản đã phá sản!

---

## 🛡️ ĐIỂM SÁNG XÁC NHẬN TRONG TẦNG 5

- **Enhancement 3: Monotonic Ratchet Guard (`risk.rs:242-252`)**:
  Hoạt động hoàn hảo về mặt toán học. Với Long, điều kiện `calculated_sl > trade.sl_price` đảm bảo dãy SL đơn điệu tăng ($SL_{t+1} \ge SL_t$). Khi thị trường bùng nổ biến động làm ATR phình to, công thức dời SL tính ra mức thấp hơn sẽ bị chặn đứng ngay lập tức, ngăn chặn việc nới lỏng SL làm tăng rủi ro.
- **Tính toán Phí Sàn (Commission) trong `Broker`**:
  Trừ trực tiếp vào `cash` cả 2 chiều vào/ra lệnh, bảo toàn toán học chính xác 100% kể cả khi đóng từng phần (`reduce_trade`).
