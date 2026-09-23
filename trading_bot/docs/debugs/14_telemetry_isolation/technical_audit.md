# 🏛️ KIẾN TRÚC KỸ THUẬT: HẠ TẦNG LOGGING & CÔ LẬP HAI LUỒNG (DUAL-STREAM TELEMETRY)

---

## 1. Bối cảnh & Vấn đề Cốt Lõi

Trước khi nâng cấp, hệ thống ghi nhận thông tin (Logging & Telemetry) tồn tại các hạn chế nghiêm trọng ảnh hưởng đến khả năng kiểm toán định lượng và hiệu năng vận hành:
1. **Console Spam từ GA Optimizer (`ferro_trade/prospector.rs`)**:
   Hơn 20 câu lệnh `println!` trực tiếp từ các luồng Scout, Worker, Prune, Promotion và Restart làm nghẽn terminal I/O, che lấp toàn bộ các thông điệp quan trọng của quá trình Out-of-Sample.
2. **Thiếu Telemetry Đấu Thầu tại Arbiter (`helios/arbiter.rs`)**:
   Tham số `_debug_mode` bị bỏ xó (`unused`). Khi xảy ra xung đột hoặc khi một Scanner bị từ chối, người kiểm toán không có bất kỳ thông tin nào về điểm raw score, mức phạt Soft Chaos Penalty hay lý do bị loại.
3. **Quadratic Log Spam trong `PaperTrader` (`bot_engine/paper_trader.rs`)**:
   Hàm `log_all_trades` được gọi định kỳ mỗi 4 giờ in lặp lại toàn bộ danh sách lệnh từ #1 đến #N. Với 100 giao dịch, số dòng log tăng theo cấp số nhân ($O(K \times N)$). Đồng thời khi chưa có giao dịch nào, hệ thống vẫn in 7 dòng bảng rỗng gây nhiễu log.
4. **Thiếu Dữ Liệu Khám Nghiệm Lệnh (MFE / MAE / Post-Trade Autopsy)**:
   Log đóng lệnh chỉ có giá Entry, Exit và PnL. Không biết được trong suốt thời gian giữ lệnh, giá đã từng đi có lợi tối đa bao nhiêu phần trăm (MFE) và sụt giảm tối đa bao nhiêu phần trăm (MAE) để đánh giá độ chuẩn xác của điểm vào lệnh và Stop Loss.
5. **Pha Tạp Hai Luồng Log (Mixed Logging)**:
   Toàn bộ thông tin từ WebSocket, SQLite, Rest API, GA Optimizer và Paper Trade bị nhồi chung vào một file `{strategy_id}.log` duy nhất, gây khó khăn cho việc phân tích và debug.

---

## 2. Giải Pháp Triển Khai Chi Tiết

### 2.1. Silent Mode cho GA Optimizer (`ferro_trade`)
- Thêm trường `pub verbose: bool` (default: `false`) vào `ProspectorConfig`.
- Toàn bộ các khối log terminal trong `prospector.rs` được bọc điều kiện `if self.config.verbose` / `if config_task.verbose`.
- Trong `ga_worker.rs`, chuyển `println!` sang `eprintln!` khi xảy ra lỗi nghiêm trọng.

### 2.2. Arbiter Auction Telemetry (`helios`)
- Kích hoạt tham số `debug_mode: bool` trong `HeliosArbiter::process_tick`.
- Khi `debug_mode == true` và có ít nhất 1 proposal được sinh ra từ các Scanner, hệ thống xuất biên bản đấu thầu:
  + Tên Scanner, Loại tín hiệu (Long/Short), Điểm hoàn hảo mô hình (`raw_score`), Điểm tự tin sau phạt Chaos (`confidence`), Mức SL và TP đề xuất.
  + Kết quả phân xử: Thắng cuộc (`APPROVED`), Bị từ chối do không đạt ngưỡng (`REJECTED`), hoặc Triệt tiêu hòa hoãn giữa Long và Short (`FLAT`).

### 2.3. MFE, MAE & Post-Trade Autopsy trong `PaperTrader`
- **Theo dõi giá cực hạn**:
  + Khi mở lệnh: `highest_price_seen = current_price; lowest_price_seen = current_price;`
  + Khi nhận nến mới: Cập nhật `highest_price_seen` theo `candle.high` và `lowest_price_seen` theo `candle.low`.
- **Công thức MFE & MAE**:
  $$\text{MFE}_{\text{Long}} = \frac{\max(\text{highest}, P_{\text{exit}}) - P_{\text{entry}}}{P_{\text{entry}}} \times 100 \times \text{Leverage}$$
  $$\text{MAE}_{\text{Long}} = \frac{\min(\text{lowest}, P_{\text{exit}}) - P_{\text{entry}}}{P_{\text{entry}}} \times 100 \times \text{Leverage}$$
  $$\text{MFE}_{\text{Short}} = \frac{P_{\text{entry}} - \min(\text{lowest}, P_{\text{exit}})}{P_{\text{entry}}} \times 100 \times \text{Leverage}$$
  $$\text{MAE}_{\text{Short}} = \frac{P_{\text{entry}} - \max(\text{highest}, P_{\text{exit}})}{P_{\text{entry}}} \times 100 \times \text{Leverage}$$
- **Post-Trade Autopsy Event**: In ngay khi lệnh đóng kèm đầy đủ các chỉ số:
  `🔴 [CLOSE LONG] #001 [LONG] [TREND_PB] Entry: 2000.00 -> Exit: 2200.00 [TP] | 💰 PnL: $+200.00 (+20.0% ROE) | MFE: +21.0% | MAE: -1.0% | Held: 12 bars | Fee: $1.60 | Balance: $1198.40`
- **Delta Logging**: Sử dụng `AtomicUsize` lưu `last_logged_trade_idx` để chỉ in các lệnh mới hoàn tất trong kỳ.
- **Final Summary**: Hàm `log_final_summary` xuất 1 lần duy nhất toàn bộ lịch sử khi kết thúc bài test.

### 2.4. Dual-Appender Logging (`common/mod.rs`)
Triển khai hệ thống 3 tầng xuất log:
1. `{strategy_id}_system.log`: Ghi nhận sự kiện hệ thống (WebSocket, DB, Re-Opt cycles).
2. `{strategy_id}_trades.log`: Ghi nhận OOS Telemetry (Auction, Veto, Orders, Autopsy, Milestones).
3. `Console (Stdout)`: Giao diện dòng lệnh tinh gọn, chỉ xuất hiện các mốc quan trọng, không làm tràn màn hình.

---

## 3. Kết Quả Kiểm Chứng

| Kiểm thử | Lệnh thực thi | Kết quả |
| :--- | :--- | :--- |
| `ferro_trade` compile check | `cargo check --package ferro_trade` | ✅ Thành công (Exit 0) |
| `helios` Arbiter unit tests | `cargo test --lib strategy::core_v3::arbiter` | ✅ 8/8 tests passed (100%) |
| `bot_engine` PaperTrader tests | `cargo test --lib execution::paper_trader` | ✅ 9/9 tests passed (100%) |
| `bot_engine` Examples check | `cargo check --examples` | ✅ Thành công (Exit 0) |
