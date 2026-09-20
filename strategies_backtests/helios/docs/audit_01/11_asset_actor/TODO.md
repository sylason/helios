# DANH MỤC CÔNG VIỆC KHẮC PHỤC & NÂNG CẤP (TODO): MÔ-ĐUN `asset_actor.rs` VÀ ĐIỀU PHỐI HỆ THỐNG

> **Dự án:** Helios Core V3.3 - HFT/Intraday Quantitative Trading Engine  
> **Mô-đun kiểm toán:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)  
> **Tài liệu kiểm toán gốc:** [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/11_asset_actor/technical_audit.md)  
> **Ngày lập:** 19/09/2026  

---

## 📌 BẢNG TỔNG HỢP MỨC ĐỘ ƯU TIÊN

| Mức Độ | Số Lượng Task | Trạng Thái | Mô Tả Mức Độ Ảnh Hưởng |
| :---: | :---: | :---: | :--- |
| **P0** | 2 Tasks | 🔴 Cần xử lý ngay | Nguy cơ bế tắc giao dịch vĩnh viễn (Deadlock) hoặc cháy tài khoản trên Binance Live. |
| **P1** | 3 Tasks | 🟠 Rất quan trọng | Lệch tín hiệu MTF giữa Live và Backtest, méo mó chỉ báo nến 4h, bypass kiểm tra DNA. |
| **P2** | 2 Tasks | 🟡 Trung bình | Bẫy High Watermark trôi buffer, rớt nến do broadcast buffer lagged. |
| **P3** | 1 Task | 🔵 Cải tiến | Dọn dẹp dead code, tối ưu hóa cấp phát bộ nhớ Heap trong `evaluate_live_tick`. |

---

## 🚨 MỨC ĐỘ P0: KHẨN CẤP / CHÁY TÀI KHOẢN & BẾ TẮC HỆ THỐNG

### [P0-1] Khắc phục Lỗi Bế Tắc Cooldown Vĩnh Viễn khi Bộ Đệm Bão Hòa (VULN-AA-01)
- **File mục tiêu:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L528-L587)
- **Vấn đề:** Khi `Asset.ohlcv` đạt 43,200 nến, `current_bar_idx` bị ghim cố định ở 43,199. Điều kiện `current_bar_idx < locked_until_bar` vĩnh viễn đúng, bot bị liệt không bao giờ mở lệnh được nữa.
- **Giải pháp kỹ thuật:**
  1. Thêm trường `processed_candles_count: usize` vào struct `AssetActor` (khởi tạo bằng 0).
  2. Mỗi lần nhận nến mới trong `on_new_candle`, tăng `self.processed_candles_count += 1`.
  3. Sử dụng `self.processed_candles_count` làm `current_monotonic_bar` thay thế cho `current_bar_idx` khi tính `locked_until_bar` và trong hàm `should_veto`.
  4. (Tùy chọn nâng cao) Chuyển toàn bộ logic Cooldown sang so sánh bằng timestamp mili-giây: `locked_until_timestamp = candle.timestamp + effective_cooldown * 60_000`.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - Viết unit test giả lập buffer đầy 43,200 nến. Kích hoạt Cooldown 15 nến.
  - Cho thêm 16 nến mới vào hệ thống. Xác nhận `should_veto` chuyển từ `true` sang `false` thành công khi hết 15 nến.

---

### [P0-2] Đồng Bộ Trạng Thái Vị Thế Bị Đóng Từ Sàn Thật Vào Cooldown Guard (VULN-AA-02)
- **File mục tiêu:**
  - [`trading_bot/bot_engine/src/execution/live_engine.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs)
  - [`trading_bot/bot_engine/src/execution/engine.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/engine.rs)
  - [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L522-L570)
- **Vấn đề:** Khi lệnh Stop Loss khớp trên sàn Binance Live, WebSocket cập nhật `live_position = None` trước khi `on_candle_ctx` chạy, khiến `closed_intra_candle` nhận giá trị `false`. Toàn bộ cơ chế Cooldown chống Revenge Trade bị bypass 100% trên tiền thật.
- **Giải pháp kỹ thuật:**
  1. Trong trait `ExecutionEngine`, thêm phương thức:
     ```rust
     fn pop_recently_closed_trade(&mut self) -> Option<ClosedTradeSnapshot>;
     ```
  2. Trong `BinanceLiveExecutionEngine`, khi nhận sự kiện `PositionUpdate` từ vị thế mở sang 0.0 (hoặc `OrderTradeUpdate` FILLED của lệnh SL/TP):
     Lưu `ClosedTradeSnapshot { exit_price, pnl, side, exit_time }` vào một hàng đợi nhỏ.
  3. Trong `AssetActor::on_new_candle`:
     Kiểm tra `let external_closed = self.execution_engine.pop_recently_closed_trade();`.
     Nếu `closed_intra_candle || external_closed.is_some()`, kích hoạt tính toán `EngineCooldownGuard` và cập nhật chuỗi thua.
- **Tiêu chí nghiệm thu:**
  - Viết integration test mô phỏng WebSocket bắn `PositionUpdate(0.0)` với PnL âm.
  - Kiểm tra xem `cooldown_guard.consecutive_losses` có tăng lên 1 và `locked_until_bar` có được kích hoạt trên sàn Live hay không.

---

## 🟠 MỨC ĐỘ P1: NGHIÊM TRỌNG / LỆCH TÍN HIỆU MTF & NGUY CƠ CRASH

### [P1-1] Căn Chỉnh Resampling MTF Chuẩn UTC và Tăng Buffer Warmup Khung 4h (VULN-AA-03)
- **File mục tiêu:**
  - [`trading_bot/bot_engine/src/models/config.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/config.rs#L79)
  - [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L538)
  - [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L182-L190)
- **Vấn đề:** Buffer 15,000 nến chỉ tương đương 62 nến 4h, không đủ thời gian warmup cho các chỉ báo chu kỳ dài (EMA 100/200, KAMA) trên khung 4h. Việc cắt lát buffer tại phút lẻ làm nến 4h đầu tiên bị cắt cụt (truncated bar), gây lệch tín hiệu hoàn toàn so với Backtest.
- **Giải pháp kỹ thuật:**
  1. Tăng `EngineConfig::max_live_mem_candles` mặc định từ `15,000` lên `30,000` nến 1m.
  2. Khi tính toán `slice_start` trong `asset_actor.rs`, làm tròn lùi về mốc chia hết cho 240 phút (00:00, 04:00, 08:00, 12:00, 16:00, 20:00 UTC):
     ```rust
     let raw_slice_start = full_buffer.len().saturating_sub(max_mem);
     let slice_start = if raw_slice_start > 0 {
         let remainder_ms = full_buffer[raw_slice_start].timestamp % (4 * 3600 * 1000);
         let shift_bars = (remainder_ms / 60_000) as usize;
         raw_slice_start.saturating_sub(shift_bars)
     } else {
         0
     };
     ```
- **Tiêu chí nghiệm thu:**
  - Chạy `precompute_timeframes` trên slice và so sánh giá trị KAMA/EMA của nến 4h cuối cùng với Backtest chạy trên toàn bộ 43,200 nến. Sai số tương đối phải < $0.01\%$.

---

### [P1-2] Khắc Phục Lỗ Hổng Bypass Validation khi Nhận DNA Hot-Swap (VULN-AA-05)
- **File mục tiêu:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L286-L294, #L327-L335, #L611-L617, #L743-L749)
- **Vấn đề:** Các khối kích hoạt `pending_dna` và `dna_rx.changed()` gọi thẳng `strategy.update_from_dna` và `helios_config.update_from_dna` mà bỏ qua hàm kiểm duyệt an toàn `apply_dna`. Nếu DNA chứa `NaN` hoặc `Inf` từ Optimizer, bot sẽ bị panic runtime.
- **Giải pháp kỹ thuật:**
  Thay thế toàn bộ các lệnh cập nhật thủ công bằng hàm `self.apply_dna`:
  ```rust
  if self.apply_dna(&dna.dna, dna.fitness, "DNA Hot-Swap") {
      self.pending_dna = None;
  }
  ```
- **Tiêu chí nghiệm thu:**
  - Viết unit test tiêm DNA chứa giá trị `f64::NAN` và `f64::INFINITY` qua `dna_rx`.
  - Xác nhận Actor từ chối nạp, giữ nguyên cấu hình cũ an toàn và không bị crash.

---

### [P1-3] Truyền `last_closed_trade` Sang `TickContext` trong `HeliosBridge` (VULN-AA-04)
- **File mục tiêu:** [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L271)
- **Vấn đề:** `HeliosBridge::evaluate_live_tick` truyền `last_closed_trade: None`, làm Strategy bên trong bị mù thông tin về lệnh vừa đóng và kích hoạt sai cơ chế dời SL / Fallback PnL.
- **Giải pháp kỹ thuật:**
  Bổ sung tham số `last_closed_trade: Option<&ferro_trade::broker::trade::Trade>` vào hàm `evaluate_live_tick` và truyền chính xác trade vừa đóng sang `TickContext`.
- **Tiêu chí nghiệm thu:**
  - Xác nhận Strategy nhận diện đúng `tick.last_closed_trade`, tính toán PnL chính xác 1:1 theo dữ liệu thực tế.

---

## 🟡 MỨC ĐỘ P2: TRUNG BÌNH / PHÒNG NGỪA RÒ RỈ & NGHẼN MẠNG

### [P2-1] Sửa Lỗi High Watermark Buffer Bleed Khi Lệnh Ngâm Lâu (VULN-AA-06)
- **File mục tiêu:** [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L224-L233)
- **Vấn đề:** Khi vị thế ngâm lâu trôi khỏi biên trái của buffer, `mapped_entry_bar` fallback về 0 khiến chiến lược quét cả các nến trước khi vào lệnh để tính đỉnh High Watermark.
- **Giải pháp kỹ thuật:**
  Nếu `pos.entry_timestamp < candles[0].timestamp`:
  Không fallback về 0. Thay vào đó, gán `mapped_entry_bar = 0` và truyền đỉnh lợi nhuận lịch sử đã ghi nhận trước đó (`highest_watermark_price`) vào `Trade` snapshot để Strategy không quét lại nến quá khứ.
- **Tiêu chí nghiệm thu:**
  - Giả lập một lệnh Long mở tại giá 100, buffer bắt đầu từ giá 500 (nến quá khứ có đỉnh 1,000).
  - Xác nhận High Watermark không bị méo mó bởi đỉnh 1,000 của quá khứ trước lệnh.

---

### [P2-2] Chống Rớt Nến Broadcast WebSocket (`RecvError::Lagged`) (VULN-AA-07)
- **File mục tiêu:**
  - [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L308-L318, #L360-L362)
- **Vấn đề:** Khi `diagnostic_backtest` chạy mất nhiều giây, kênh broadcast `data_rx` bị đầy buffer và drop mất nến, gây thủng lỗ chuỗi thời gian trong `Asset.ohlcv`.
- **Giải pháp kỹ thuật:**
  1. Chuyển `HeliosBridge::diagnostic_backtest` sang chạy trong `tokio::task::spawn_blocking` trên luồng phụ để không chặn vòng lặp Actor.
  2. Khi bắt được lỗi `RecvError::Lagged(skipped)`:
     Chủ động gọi `self.candle_repo.get_candles_by_range` để bù đắp các nến bị khuyết (Gap Healing).
- **Tiêu chí nghiệm thu:**
  - Chạy tải nặng trong 30 giây, kiểm tra độ liên tục của timestamp trong `Asset.ohlcv`, không còn bị mất nến.

---

## 🔵 MỨC ĐỘ P3: TỐI ƯU HÓA HIỆU NĂNG & DỌN DẸP CODE

### [P3-1] Dọn Dẹp Mã Nguồn Rác & Giảm Cấp Phát Bộ Nhớ Heap
- **File mục tiêu:**
  - [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L444-L467)
  - [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L281-L316)
- **Giải pháp kỹ thuật:**
  1. Xóa bỏ hàm dead code `check_and_update_dna` không sử dụng.
  2. Tái sử dụng vùng đệm vector cột trong `candles_to_df` thay vì cấp phát mới 6 vector dài 15,000 phần tử ở mỗi cây nến.
- **Tiêu chí nghiệm thu:**
  - Mã nguồn sạch sẽ, không còn cảnh báo dead code khi biên dịch `cargo check`.
  - Giảm áp lực Garbage Collection / Heap Allocation trên luồng chính.
