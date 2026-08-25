# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU `ASSET_ACTOR` (DEEP AUDIT REPORT)
**Dự án:** `trading_bot/bot_engine`  
**Tệp kiểm toán:** [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)  
**Ngày thực hiện:** 24/08/2026  
**Mục đích:** Ghi nhận toàn bộ các mâu thuẫn logic, hằng số hardcoded, vi phạm Clean Code và thắt chặt kiến trúc chưa decoupled trong `AssetActor`.

---

## I. MÂU THUẪN LOGIC VÀ SAI LỆCH TRẠNG THÁI (LOGIC CONTRADICTIONS & STATE BUGS)

### 1. Bất đồng bộ Vị thế kép (Dual Position State Desync)
- **Hiện trạng:** `AssetActor` quản lý 2 struct vị thế độc lập:
  - `self.position_state` (cập nhật qua `AccountEvent::PositionUpdate` từ WebSocket Binance: chỉ chứa `position_amt`, `entry_price`, `unrealized_profit`).
  - `self.live_position` (chứa `live_entry_bar`, `current_sl_price`, `current_tp_price`, `opening_playbook_tag`).
- **Mâu thuẫn:** Khi bot bị đứt mạng WS kết nối lại hoặc khởi động lại, sự kiện `PositionUpdate` của Binance gửi về chỉ cập nhật `position_amt` và `entry_price`. Lúc này `live_entry_bar` bị gán sai về cây nến HIỆN TẠI (`current_bar`), đồng thời `opening_playbook_tag`, `current_sl_price`, `current_tp_price` bị biến thành `None`! Điều này làm hỏng toàn bộ logic Trailing SL của Helios Strategy khi reconnection xảy ra.

### 2. Phương thức Chết không được sử dụng (Dead Method)
- **Hiện trạng:** Hàm `check_and_update_dna(&mut self)` (Dòng 325) tồn tại nhưng **không bao giờ được gọi** trong vòng lặp `run()`.
- **Mâu thuẫn:** Vòng lặp `run()` lại dùng nhánh `Ok(_) = self.dna_rx.changed() => { ... }` ở Dòng 160 để cập nhật DNA. Tồn tại 2 cơ chế cập nhật DNA làm nhập nhằng luồng xử lý.

### 3. Trùng lặp cập nhật `LivePositionState` trong Paper Mode
- **Hiện trạng:** Trong chế độ `Paper`, `AssetActor` vừa gọi `self.paper_trader.process_action(...)`, vừa phải viết lại hơn 30 dòng code thủ công (Dòng 464 - 495) để gán lại từng trường sang `self.live_position`.
- **Mâu thuẫn:** Dễ dẫn đến lệch giá SL/TP giữa `PaperTrader` và `LivePositionState` nếu quên gán 1 trường.

---

## II. HẰNG SỐ CỨNG VÀ CON SỐ MA THUẬT (HARDCODED CONSTANTS & MAGIC NUMBERS)

1. **Dòng 14:** `pub const MAX_LIVE_MEM_CANDLES: usize = 15000;` bị hardcode cứng ở mức module thay vì đọc từ file cấu hình `EngineConfig`. Trong khi đó `prospector_runner.rs` và `helios_bridge.rs` lại dùng mốc `43200`.
2. **Dòng 81:** `PaperTrader::new(symbol.clone(), allocated_capital, 0.0004)` -> Tỷ lệ phí giao dịch `0.0004` (0.04% Taker fee) bị hardcode cứng trong constructor của `AssetActor`.
3. **Dòng 340:** `until_ts - (33 * 24 * 60 * 60 * 1000)` -> Con số `33` ngày tải nến lịch sử bị hardcode trực tiếp trong logic sync.
4. **Dòng 385:** `self.max_live_mem_candles.max(pos_age + 200)` -> Con số `200` nến đệm bị hardcode.

---

## III. VI PHẠM CLEAN CODE (CLEAN CODE FLAWS)

1. **Trường dư thừa gây Cảnh báo Biên dịch (Unused Fields):**
   - `monitor: Arc<WeightMonitor>` (Dòng 36) và `market_filter` (Dòng 46) được lưu trong struct nhưng **không bao giờ được đọc hay sử dụng** trong bất kỳ hàm nào của `AssetActor`.
   - `_trade_tx` (Dòng 43) bị bỏ xám `_`, không được dùng để phát `TradeSignal` lên EventBus.
2. **Hàm "Phồng Đại" (God Method Smell):**
   - Hàm `on_new_candle` (Dòng 363 đến 552) dài tới **190 dòng**, ôm xô quá nhiều nhiệm vụ (Đẩy nến, lưu DB, cắt bộ nhớ, tính vốn thực tế, gọi Bridge, lọc Exclusive Order, phân nhánh 5 chế độ `ExecutionMode`, drain event channel).

---

## IV. THẮT CHẶT KIẾN TRÚC, CHƯA DECOUPLED (COUPLING FLAWS)

1. **Phụ thuộc trực tiếp vào `HeliosBridge` cụ thể:**
   - `AssetActor` trực tiếp gọi `HeliosBridge::evaluate_live_tick(...)` (Dòng 414) và `HeliosBridge::diagnostic_backtest(...)` (Dòng 171) thay vì thông qua một Trait trừu tượng (`dyn StrategyBridge`). Điều này làm `bot_engine` bị trói chặt vào dự án `helios`, không thể dùng lại bot engine cho chiến thuật khác (ví dụ `artemis`, `zeus`...).
2. **Tự khởi tạo `PaperTrader` bên trong Actor:**
   - `AssetActor` tự gọi `PaperTrader::new(...)` thay vì nhận một đối tượng `PaperTrader` hoặc `ExecutionEngine` được tiêm từ ngoài vào qua Constructor.

---
*Báo cáo kiểm toán chuyên sâu tệp `asset_actor.rs` đã được lưu lưu trữ để làm cơ sở tái cấu trúc cho Bước 2 và Bước 3.*
