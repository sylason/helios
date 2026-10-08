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

### [x] [P0-1] Khắc phục Lỗi Bế Tắc Cooldown Vĩnh Viễn khi Bộ Đệm Bão Hòa (VULN-AA-01) - **ĐÃ HOÀN THÀNH**
- **File mục tiêu:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
- **Vấn đề:** Khi `Asset.ohlcv` đạt 43,200 nến, `current_bar_idx` bị ghim cố định ở 43,199. Điều kiện `current_bar_idx < locked_until_bar` vĩnh viễn đúng, bot bị liệt không bao giờ mở lệnh được nữa.
- **Giải pháp kỹ thuật (Occam's Razor Gốc Rễ):**
  1. Thêm trường `pub processed_candles_count: usize` vào struct `AssetActor`. Khởi tạo bằng `0` và đồng bộ trong `sync_history`.
  2. Mỗi lần nhận nến mới trong `on_new_candle`, tăng `self.processed_candles_count = self.processed_candles_count.saturating_add(1)`.
  3. Sử dụng `self.processed_candles_count.saturating_sub(1)` làm `current_bar_idx` đơn điệu tăng dần.
  4. Sửa `pos_age = current_bar_idx.saturating_sub(pos.live_entry_bar)` đảm bảo tuổi vị thế không bị bóp nghẹt về 0 khi buffer đầy 43,200 nến.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] Đã viết unit test `test_monotonic_sequence_counter_cooldown_saturated_buffer`: kiểm chứng buffer 43,200 nến, phạt cooldown 15 bars, mở khóa chuẩn xác tại bar thứ 16.
  - [x] Đã viết unit test `test_pos_age_calculation_monotonic_counter_saturated_buffer`: kiểm chứng tuổi vị thế tính chuẩn 119 nến thay vì bị bóp về 0.
  - [x] Vượt qua 44/44 tests `bot_engine`, 105/105 tests `helios`, 50/50 tests `ferro_trade`.

---

### [P0-2] Đồng Bộ Trạng Thái Vị Thế Bị Đóng Từ Sàn Thật Vào Cooldown Guard (VULN-AA-02) - ✅ **ĐÃ HOÀN THÀNH**
- **File mục tiêu:**
  - [`trading_bot/bot_engine/src/models/account_state.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/account_state.rs)
  - [`trading_bot/bot_engine/src/execution/ws_manager.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/ws_manager.rs)
  - [`trading_bot/bot_engine/src/execution/live_engine.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/live_engine.rs)
  - [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)
- **Vấn đề:** Khi lệnh Stop Loss khớp trên sàn Binance Live, WebSocket cập nhật `live_position = None` trước khi `on_candle_ctx` chạy, khiến `closed_intra_candle` nhận giá trị `false`. Toàn bộ cơ chế Cooldown chống Revenge Trade bị bypass 100% trên tiền thật.
- **Giải pháp kỹ thuật (2-Piece Assembler Pattern):**
  1. Thêm `realized_pnl: f64` vào `AccountEvent::OrderTradeUpdate` và parse trường `rp` từ Binance WebSocket User Data Stream.
  2. Trong `BinanceLiveExecutionEngine`, triển khai cơ chế ghép 2 mảnh không phụ thuộc thứ tự mạng:
     - Mảnh A: `PositionUpdate` vị thế về 0 lưu snapshot `last_closed_pos_snapshot`.
     - Mảnh B: `OrderTradeUpdate` mang `realized_pnl` và giá khớp trung bình lưu `pending_closed_fill`.
     - Hàm `try_assemble_closed_trade(&mut self)` tự động ghép thành `PaperTradeRecord` hoàn chỉnh ngay khi đủ 2 mảnh.
  3. Override `ExecutionEngine::take_last_closed_trade(&mut self)` trong `BinanceLiveExecutionEngine` (drain semantics $O(1)$) để trả về bản ghi lệnh đóng.
  4. Trong `AssetActor::sync_and_handle_closed_positions`: lấy `engine_closed_trade`, cập nhật `cooldown_guard`, đồng thời lưu vào `self.last_closed_trade_for_bridge` để nạp sang `HeliosBridge`.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] Đã parse chuẩn xác `realized_pnl` từ Binance WS JSON `ORDER_TRADE_UPDATE`.
  - [x] Đã viết unit test `test_live_engine_assemble_closed_trade_order_after_position`: Mảnh vị thế đến trước, mảnh order đến sau -> Ghép thành công.
  - [x] Đã viết unit test `test_live_engine_assemble_closed_trade_order_before_position`: Mảnh order đến trước, mảnh vị thế đến sau -> Ghép thành công.
  - [x] Vượt qua 46/46 unit tests `bot_engine`, 4/4 `trailing_sl_test`, 109/109 tests `helios`, 50/50 tests `ferro_trade`.

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

---

## 🆕 PHÁT HIỆN MỚI TỪ LOG AUDIT (2026-09-22) — Nguồn: `WFSS_arbiter_weakpoint2.log`

> **Xác nhận bởi:** Đọc trực tiếp `asset_actor.rs:73-101, 655, 676, 729-736`

---

### [P0-3] 🔥 Same-Side Price Guard Deadlock Vĩnh Viễn (VULN-AA-09) — MỚI

- **File:** [`asset_actor.rs:73-85`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L73-L85)
- **Bản chất:** Khối 1 (`Same-Side Price Guard`) trong `should_veto` không có điều kiện `current_bar_idx < locked_until_bar`, chạy vĩnh viễn sau cooldown. Tạo vòng khóa chết: SL → Khối 1 chặn LONG → không có lệnh thắng → không reset `consecutive_losses` → Khối 1 tiếp tục chặn mãi mãi.
- **Bằng chứng:** VETO 2.64 ngày / 280 lần liên tiếp, bỏ lỡ ETH pump +7.5%.
- **Log sai:** Dòng 676 hardcode ký tự `<` → in `+55.28 < 4.45` (misleading). Thực tế Khối 2 không chạy, Khối 1 mới là thủ phạm.
- **Giải pháp:** Đưa Khối 1 vào TRONG `if current_bar_idx < locked_until_bar`. Dùng `enum CooldownVetoReason` để log đúng lý do.
- **Acceptance:** Sau hết cooldown (`Remaining: 0`), bot PHẢI mở được lệnh dù giá < `last_losing_price`.

---

### [P1-4] ATR Mismatch Engine vs Helios (VULN-AA-10) — MỚI

- **File:** [`asset_actor.rs:655`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L655)
- **Bản chất:** Engine dùng `rolling_atr(14 nến 1m) ≈ 2–5 USDT`. Helios dùng `sensors.atr_sl` (thường 14 nến 4h ≈ 50–100 USDT). Ngưỡng Early Release Engine thấp hơn ~10–20x so với ATR Helios. Gây bất nhất giữa 2 tầng và đây là lý do log hiện ra các số ATR nhỏ bất thường.
- **Giải pháp:** Truyền `current_atr` từ `HeliosBridge` (cùng nguồn `sensors.atr_sl`) xuống `EngineCooldownGuard`.

---

### [P1-5] `ClosePosition` Không Dùng Exponential Cooldown (VULN-AA-11) — MỚI

- **File:** [`asset_actor.rs:729-736`](file:///e:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L729-L736)
- **Bản chất:** Nhánh đóng lệnh qua `Action::ClosePosition` chỉ gán thẳng `locked_until_bar = current_bar_idx + base_bars` mà không gọi `calculate_exponential_cooldown`, không tăng `consecutive_losses`, không cập nhật `last_losing_price`. Lệch pha hoàn toàn với nhánh `intra-candle exit`.
- **Giải pháp:** Trích xuất hàm `trigger_engine_cooldown(pnl, side, exit_price)` và gọi thống nhất từ cả 2 nhánh.
