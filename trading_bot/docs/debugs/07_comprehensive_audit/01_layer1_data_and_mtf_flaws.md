# 🚨 Báo Cáo Kiểm Toán Tầng 1: Dữ Liệu & Khung Thời Gian MTF (Data Pipeline & MTF)

**Dự án:** `ferro_trade` & `trading_bot`  
**Ngày kiểm toán:** 2026-09-04  
**Trạng thái:** ⚠️ **PHÁT HIỆN 6 LỖ HỔNG LOGIC (2 CRITICAL, 3 HIGH/MAJOR, 1 MEDIUM)**

---

## 📌 Bảng Tổng Hợp Lỗ Hổng Phát Hiện Tại Tầng 1

| STT | Mức Độ | Tên Lỗ Hổng | File & Vị Trí Code | Hậu Quả Thực Tế |
| :---: | :---: | :--- | :--- | :--- |
| **1** | 🔴 **CRITICAL** | **Rò rỉ Nến Chưa Đóng (Unclosed Candle) từ Binance REST** | [`binance_rest.rs:65-71`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/binance_rest.rs#L65-L71)<br>[`models.rs:94-98`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/models.rs#L94-L98) | `fill_gap` không có `endTime` nên nạp nến 1m dở dang vào DB & RAM. Khi WebSocket gửi nến đóng thật, `push_candle` thấy trùng timestamp nên **vứt bỏ vĩnh viễn nến đóng thật**! |
| **2** | 🔴 **CRITICAL** | **Signal Replay Bắn Lệnh Live Trên Nến Quá Khứ Khi Reconnect** | [`asset_actor.rs:326-367`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L326-L367) | Khi mất mạng có lại, `fill_gap` bơm 10-15 nến quá khứ vào. Code chạy chiến lược **không kiểm tra `!is_historical`**, dẫn đến bắn lệnh MUA/BÁN thật lên sàn theo giá quá khứ. |
| **3** | 🟠 **MAJOR** | **Lỗi Cột Rào (Off-by-One) Làm Khuyết Nến Khi Sync Lịch Sử** | [`binance_rest.rs:184-196`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/data_engine/binance_rest.rs#L184-L196) | Công thức lùi 1,000 phút tạo ra đoạn $[t_{start}, t_{end}]$ chứa 1,001 nến. Binance giới hạn `.limit(1000)` nên **cắt bỏ nến tại biên `current_end_time`**. Sync 33 ngày làm mất ~48 nến 1m trong DB! |
| **4** | 🟡 **HIGH** | **Vỡ Ranh Giới (Boundary Fracture) Cây Nến 4h Đầu Tiên** | [`asset_actor.rs:339-340`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L339-L340) | Cắt lát `rolling_buffer` không căn chỉnh theo mốc 4h (`timestamp % 14_400_000 == 0`). Cây nến 4h đầu tiên bị cắt cụt ở giữa, mất đỉnh/đáy và volume thật, gây méo mó chỉ báo. |
| **5** | 🟡 **HIGH** | **`max_live_mem_candles` = 15,000 Làm Liệt Chiến Lược** | [`config.rs:79`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/models/config.rs#L79)<br>[`helios_bridge.rs:208`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L208) | 15,000 nến 1m chỉ tạo được 62 nến 4h. Nếu KAMA cần 80 nến 4h (19,200 nến 1m) để Warmup thì `full_len <= warmup`, bot âm thầm trả về rỗng, **không bao giờ ra lệnh giao dịch**! |
| **6** | 🔵 **MEDIUM** | **`DataView::close_at` Không Kiểm Tra `current_len`** | [`data.rs:181-205`](file:///E:/Projects/Automations/trading_backtest/ferro_trade/src/data.rs#L181-L205) | `open_at`, `close_at` kiểm tra theo tổng chiều dài mảng gốc thay vì `current_len`. Nếu chiến lược gọi `close_at(i + 10)`, hệ thống sẽ trả về giá tương lai thay vì chặn lại. |

---

## 🛡️ Điểm Sáng Đã Kiểm Chứng (An Toàn Tuyệt Đối)

1. **Lookahead Bias MTF trong `ferro_trade`:** Đạt chuẩn toán học $100\%$. Nhờ Polars `Label::Right` kết hợp bảng Two-Pointer Lookup Table $O(N)$, nến 1m không bao giờ nhìn thấy trước dữ liệu của nến 5m/15m/1h/4h chưa đóng.
2. **Khóa Chính SQLite:** Bảng `candles_1m` sử dụng `PRIMARY KEY (symbol, open_time)` và `INSERT OR REPLACE`. Đảm bảo $100\%$ không bao giờ có bản ghi trùng lặp trên đĩa.
3. **Thứ Tự Sắp Xếp & Đơn Vị:** Timestamp đồng nhất là milliseconds (`i64`), câu truy vấn luôn chỉ định tường minh `ORDER BY open_time ASC`.

---

## 🛠️ Kế Hoạch Khắc Phục Khuyến Nghị (Action Plan)

1. **Khắc phục Lỗ hổng 1 & 3 (`binance_rest.rs` + `models.rs`):**
   - Giới hạn `end_time` tối đa trong `fill_gap` là nến 1m đã đóng gần nhất.
   - Sửa công thức lùi trong `ensure_history`: Đổi `1000 * 60 * 1000` thành `999 * 60 * 1000` (đúng 1,000 nến).
   - Trong `Asset::push_candle`, nếu `candle.timestamp == last.timestamp`, cập nhật lại nến hoàn chỉnh thay vì drop.
2. **Khắc phục Lỗ hổng 2 (`asset_actor.rs`):**
   - Bổ sung kiểm tra `!is_historical` trước khi gọi `evaluate_live_tick` và gửi lệnh thực thi.
3. **Khắc phục Lỗ hổng 4 & 5 (`asset_actor.rs` + `config.rs`):**
   - Căn chỉnh `slice_start` về mốc bắt đầu của cây 4h (`full_buffer[idx].timestamp % 14_400_000 == 0`).
   - Nâng `max_live_mem_candles` mặc định từ `15,000` lên `43,200` nến (30 ngày).
4. **Khắc phục Lỗ hổng 6 (`ferro_trade/src/data.rs`):**
   - Sửa các hàm `close_at`, `open_at` kiểm tra theo `self.current_len.load(Ordering::Relaxed)`.
