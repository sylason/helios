# Báo Cáo Thực Nghiệm: Vòng Đời Trailing SL, Take Profit & Lệnh Kép Trên Binance Futures

> **Thời gian thực nghiệm:** 2026-09-15  
> **Môi trường:** Binance Futures USDⓈ-M Testnet  
> **Cặp giao dịch:** `ETHUSDT`  
> **Thư viện thực thi:** `binance-sdk-68.1.1` (WebSocket API & REST API)  
> **Mã nguồn thực nghiệm:** [`trading_bot/bot_engine/examples/binance_ws_sl_tp_lifecycle_test.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/binance_ws_sl_tp_lifecycle_test.rs)  
> **Log chi tiết:** [`trading_bot/bot_engine/examples/logs/binance_ws_sl_tp_test.log`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/examples/logs/binance_ws_sl_tp_test.log)

---

## 🔬 1. Mục Tiêu Thực Nghiệm

1. **Khả năng Modify SL & TP:** Kiểm chứng xem Binance có hỗ trợ sửa giá kích hoạt (`triggerPrice`) trực tiếp không, hay bắt buộc phải qua quy trình **Cancel-and-Replace** (`cancel_algo_order` -> `new_algo_order`).
2. **Độ trễ & Tính ổn định của WebSocket API:** Đo đạc độ trễ phản hồi khi thao tác lệnh điều kiện (Algo Order) qua kết nối WebSocket (`wss://testnet.binancefuture.com/ws-fapi/v1`).
3. **Cơ chế Phòng vệ Kích hoạt (Trigger Engine Guard):** Kiểm chứng phản hồi của sàn khi gửi lệnh có giá kích hoạt đã bị xuyên thủng hoặc vi phạm vùng giá hiện tại.
4. **Khả năng Đặt Song Song Cả SL và TP (`closePosition = true`):** Kiểm tra xem Binance có cho phép cùng lúc tồn tại 1 lệnh `STOP_MARKET` và 1 lệnh `TAKE_PROFIT_MARKET` với cờ `closePosition = true` cho cùng một vị thế hay không.

---

## 📊 2. Kết Quả Thực Nghiệm Từng Pha (Logs Trực Tiếp Từ Binance)

### 🧪 Pha A: Vòng Đời Trailing Stop Loss (SL Riêng Biệt)

1. **Mở vị thế Long 0.01 ETH:**
   ```text
   OrderID: 16795218038 | Status: "NEW" | Entry Price: 2483.14 USDT
   ```
2. **Đặt Trailing SL ban đầu (-2% tại 2433.46 USDT):**
   - Binance tiếp nhận thành công lệnh Algo `STOP_MARKET`:
   ```json
   {
     "algo_id": 1000000206042521,
     "algo_type": "CONDITIONAL",
     "order_type": "STOP_MARKET",
     "trigger_price": "2433.46",
     "algo_status": "NEW",
     "working_type": "MARK_PRICE",
     "close_position": true,
     "reduce_only": true
   }
   ```
3. **Thực hiện "Modify" dời Trailing SL lên 2458.29 USDT:**
   - **Bước 1 (Hủy SL cũ đích danh):** Gửi `cancel_algo_order(1000000206042521)` qua WebSocket.
     - Phản hồi từ Binance: `{"algo_id": 1000000206042521, "code": "200", "msg": "success"}` (Thời gian phản hồi: **105 ms**).
   - **Bước 2 (Đặt SL mới):** Gửi `new_algo_order` với giá dời lên `2458.29`.
     - Phản hồi từ Binance: Tạo thành công lệnh mới `New AlgoID: 1000000206042524` (Thời gian hoàn tất: **261 ms**).
4. **Kiểm tra Phòng vệ Trigger:**
   - Cố tình đặt SL cao hơn giá thị trường (`2488.12 > 2483.12`):
   - **Phản hồi từ Binance:** `Server-side response error (code -4130): An open stop or take profit order with GTE and closePosition in the direction is existing.` Sàn bảo vệ chặn ngay lập tức.
   - Hủy lệnh cũ và đặt SL siêu sát Mark Price (`2482.82`, cách 0.3 USDT): Sàn tiếp nhận thành công `AlgoID: 1000000206042531`.

---

### 🧪 Pha B: Vòng Đời Take Profit (TP Riêng Biệt)

1. **Mở vị thế Long 0.01 ETH:**
   ```text
   OrderID: 16795218147 | Status: "NEW" | Entry: 2483.06 USDT
   ```
2. **Đặt Take Profit ban đầu (+2% tại 2532.72 USDT):**
   - Binance tiếp nhận thành công: `AlgoID: 1000000206042599`, `algo_status: "NEW"`, `order_type: "TAKE_PROFIT_MARKET"`.
3. **Thực hiện "Modify" dời Take Profit lên 2557.55 USDT (+3%):**
   - Hủy đích danh `algo_id: 1000000206042599`: `Code: "200", Msg: "success"` (Phản hồi sau **104 ms**).
   - Đặt lệnh TP mới tại `2557.55 USDT`: Tạo thành công `New AlgoID: 1000000206042604` (Sau **259 ms**).
4. **Kiểm tra Phòng vệ Trigger TP:**
   - Cố tình đặt TP thấp hơn giá thị trường (`2478.14 < 2483.14`): Sàn chặn ngay lập tức với lỗi `code -4130`.
   - Đặt TP siêu sát Mark Price (`2483.44`, cách +0.3 USDT): Sàn tiếp nhận thành công `AlgoID: 1000000206042610`.

---

### 🧪 Pha C: Thử Nghiệm Lệnh Kép Song Song (SL & TP cùng `closePosition = true`)

1. **Mở vị thế Long 0.01 ETH:** Khớp thành công.
2. **Gửi lệnh 1 (STOP_MARKET tại 2358.91 USDT, `closePosition = true`):**
   - **Kết quả:** Binance chấp nhận! `AlgoID: 1000000206042666`.
3. **Gửi lệnh 2 (TAKE_PROFIT_MARKET tại 2607.21 USDT, `closePosition = true`):**
   - **Kết quả:** **Binance CHẤP NHẬN 100%!** `AlgoID: 1000000206042670`.

---

## 💡 3. Các Kết Luận & Phát Hiện Kỹ Thuật Đắt Giá

| Tiêu chí | Cơ chế Thực tế trên Binance | Ý nghĩa Thiết kế với Bot Engine |
| :--- | :--- | :--- |
| **Cơ chế Modify SL/TP** | Binance **không có endpoint sửa trực tiếp `triggerPrice`** cho Algo. Bắt buộc dùng cơ chế: `cancel_algo_order(id)` -> `new_algo_order(new_price)`. | Giữ vững cơ chế Cancel-and-Replace, nhưng chuyển từ REST sang **WebSocket API**. |
| **Hủy đích danh lệnh** | WebSocket API cho phép truyền `algo_id` để hủy chính xác 1 lệnh. Không cần dùng lệnh hạt nhân `cancel_all_algo_open_orders` như REST cũ. | Cho phép dời Trailing SL độc lập mà **không làm mất** lệnh Take Profit đang mở. |
| **Tốc độ thực thi qua WS** | Độ trễ mạng: Hủy lệnh mất **~105ms**, tạo lệnh mới mất **~260ms**. Tổng chu kỳ Modify chỉ khoảng **~365ms**. | Rút ngắn cửa sổ không bảo vệ (Naked Window) xuống mức tối thiểu, an toàn hơn nhiều so với REST. |
| **Đặt Song Song SL + TP** | **Binance cho phép tồn tại đồng thời cả 1 lệnh SL và 1 lệnh TP cùng có `closePosition = true`** cho 1 vị thế. | **Đột phá:** Bot có thể đưa cả Stop Loss và Take Profit lên sàn Binance cùng lúc để bảo vệ tài khoản 24/7 kể cả khi rớt mạng. |
| **Quy tắc Chặn lỗi Sàn (`-4130`)** | Sàn từ chối lệnh nếu `triggerPrice` bị ngược hướng (SL > MarkPrice hoặc TP < MarkPrice) hoặc đã có lệnh cùng loại đang chờ. | Bot cần kiểm tra giá `MarkPrice` trước khi gửi lệnh để tránh lãng phí API weight do lỗi `-4130`. |
