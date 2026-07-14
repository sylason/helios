# Luồng Logic Thực thi Lệnh (Execution Logic Flow)

Tài liệu này mô tả **luồng xử lý tuyến tính** từ khi có nến mới cho đến khi lệnh được gửi lên sàn hoặc bị hủy an toàn.

---

## Sơ đồ Tổng quan (High-Level Flow)

```
[Nến K-Line 1m hoàn thành] (WebSocket)
         │
         ▼
[1. Thêm nến vào Rolling Buffer]
         │
         ▼
[2. Gọi Helios Strategy.next()]
         │
         ├─── Action::Hold ───────────────────────────────► [Kết thúc, chờ nến tiếp theo]
         │
         └─── Action::Buy / Action::Sell
                   │
                   ▼
         [3. Gatekeeper: Bộ lọc Quy chuẩn (Khối 1)]
                   │
                   ├─ Size không hợp lệ ──────────────────► [Ghi Log, skip lệnh]
                   │
                   └─ Size hợp lệ
                             │
                             ▼
                   [4. Kiểm tra Margin (Khối 2)]
                             │
                             ├─ Không đủ balance ─────────► [Ghi Log cảnh báo, skip lệnh]
                             │
                             └─ Đủ balance
                                       │
                                       ▼
                             [5. Rate Limit Check (Khối 4)]
                                       │
                                       ├─ Nguy hiểm (>92%) ─► [Sleep 30s, retry sau]
                                       │
                                       └─ An toàn
                                                 │
                                                 ▼
                                       [6. Order Executor (Khối 3)]
                                       connection.new_order(ws_api)
                                                 │
                                                 ├─ Thành công ─► [Binance đẩy sự kiện ORDER_TRADE_UPDATE]
                                                 │                 [Cập nhật State từ sự kiện]
                                                 │
                                                 └─ Lỗi API ───► [Error Handler (Khối 6)]
                                                                         │
                                                                         ├─ Lỗi tạm thời → Retry
                                                                         └─ Lỗi vĩnh viễn → Skip + Log
```

---

## Luồng Chi tiết Từng Bước

### Bước 0: Khởi động Bot (Startup)
```
Đọc file .env (API Keys)
  → Gọi GET /fapi/v1/exchangeInfo → Lưu SymbolFilter vào RAM
  → Gọi POST /fapi/v1/marginType (ISOLATED) → Ghi Log
  → Gọi POST /fapi/v1/leverage   (Leverage X) → Ghi Log
  → Tải 30 ngày nến từ DuckDB    → Nạp vào Rolling Buffer
  → Khởi động WebSocket K-Line Stream
  → Khởi động Optimizer (background thread)
Bot sẵn sàng nhận nến.
```

### Bước 1: Nhận Nến Mới (WebSocket Event)
- Khi nến 1m đóng (`x: true`), trích xuất OHLCV.
- Thêm vào cuối Rolling Buffer, xóa nến cũ nhất nếu vượt quá 30 ngày.
- **Phân nhánh:** Nếu nến mở (đang hình thành), bỏ qua.

### Bước 2: Chạy Chiến lược (Strategy Execution)
- Gọi `HeliosStrategy::next(rolling_buffer)`.
- Helios tự tính MTF (Multi-Timeframe), Indicator, và cho ra:
  - `Action::Buy`, `Action::Sell`, hoặc `Action::Hold`.
  - Kèm theo `size` (kích thước vị thế thô, tính từ % vốn).

### Bước 3: Gatekeeper — Bộ lọc Quy chuẩn Sàn
```rust
let rounded_qty = floor(raw_qty / step_size) * step_size;
let rounded_price = floor(current_price / tick_size) * tick_size;

if rounded_qty < min_qty {
    log::warn!("Size {} quá nhỏ, bỏ qua lệnh.", rounded_qty);
    return; // STOP
}
```

### Bước 4: Kiểm tra Số dư & Margin
```rust
let available = get_account_balance().await?; // GET /fapi/v2/account
let required_margin = (rounded_qty * current_price) / leverage as f64;
let buffer = required_margin * 1.05; // +5% dự phòng

if available < buffer {
    log::warn!("Không đủ margin. Cần {}, có {}. Bỏ qua lệnh.", buffer, available);
    return; // STOP
}
```

### Bước 5: Kiểm tra Rate Limit
```rust
if weight_monitor.used_weight() >= EMERGENCY_THRESHOLD {
    log::warn!("Rate limit nguy hiểm ({}). Hoãn lệnh.", weight);
    tokio::time::sleep(Duration::from_secs(30)).await;
    // Tùy chiến lược: thử lại hoặc bỏ qua
}
```

### Bước 6: Gửi Lệnh lên Binance
- Xác định hướng lệnh (mở Long/Short hay đóng vị thế hiện tại).
- Gọi `POST /fapi/v1/order`.
- Parse response, ghi Log `orderId`, cập nhật state máy trạng thái.

### Bước 7: Xử lý Lỗi API (Error Handler)
```
Nhận được lỗi API:
  → Parse mã lỗi Binance
  → Tra bảng Error Handling (xem TECHNICAL_DESIGN.md Khối 6)
  → Thực thi chiến lược: Retry / Sleep / Skip / Log
  → Không bao giờ Panic
```

---

## Luồng Dự phòng: Mất Kết nối WebSocket

```
WebSocket gặp lỗi / đứt kết nối
  → Ghi Log: "WS ngắt, reconnect sau 5s"
  → Sleep 5 giây
  → Gọi REST: Lấy nến bị thiếu trong lúc offline (Gap Filler)
  → Kết nối lại WebSocket
  → Tiếp tục Bước 1 như bình thường
```

---

## Quản lý Trạng thái Vị thế (Position State)

Bot phải biết mình đang ở trạng thái nào để ra lệnh đúng:

```
NO_POSITION  ──► BUY signal  ──► LONG_OPEN
NO_POSITION  ──► SELL signal ──► SHORT_OPEN
1. Khởi tạo 3 kết nối WebSocket (Market Data, User Data, Trade API).
2. Gọi API REST duy nhất 1 lần để cấu hình tài khoản (Margin = ISOLATED, Leverage = 20).
3. Nạp 30 ngày nến từ SQLite vào DataView.
4. Tải bộ DNA khởi điểm.

### Bước 1: Nhận Nến & Lấy DNA
- Nến 1m đóng, đẩy vào Rolling Buffer.
- Lấy **Bộ DNA Mới Nhất** từ RAM (hoặc SQLite) mà Optimizer vừa tạo ra.
- DNA này quyết định thông số Indicator (VD: EMA 20, 50).

### Bước 2: Chuẩn bị Ngữ Cảnh (Context)
- Tạo `InitContext` với DNA mới, tính toán lại Indicator cho cây nến vừa đóng.
- Tạo một giỏ `actions_buffer` rỗng.
- Ghép các dữ liệu trên thành một `TickContext` trỏ vào cây nến cuối cùng (hiện tại).

### Bước 3: Ra Quyết Định (Strategy Execution)
- Gọi `HeliosStrategy::next(&tick_context)`.
- Chiến lược đọc Indicator, so sánh điều kiện, nếu thỏa mãn sẽ ném `Action::Buy` hoặc `Action::Sell` vào giỏ `actions_buffer`.

### Bước 4: Gatekeeper — Bộ lọc Quy chuẩn Sàn
- `BinanceExecutor` mở giỏ ra, thấy có lệnh Buy/Sell.
- Đọc `exchangeInfo` trong RAM để làm tròn số lượng (qty) theo `stepSize` và giá (price) theo `tickSize`.
- Chặn lệnh nếu `qty < minQty`.

### Bước 5: Thực Thi Lệnh (WebSocket API)
- Dùng `DerivativesTradingUsdsFuturesWsApi`.
- Gọi hàm `connection.new_order(side=BUY, positionSide=LONG)`.
- Gửi đi ngay lập tức với độ trễ 10-30ms. KHÔNG kiểm tra REST `GET /account` vì tốn thời gian.

### Bước 6: Lắng Nghe Kết Quả (User Data Stream)
- Lệnh khớp, sàn Binance đẩy sự kiện `ORDER_TRADE_UPDATE` qua User Data Stream.
- Bot nhận sự kiện, lập tức cập nhật trạng thái vị thế (Position State) trong RAM (Đang giữ Long/Short).
- Ghi log lịch sử giao dịch vào SQLite để sau này hiển thị lên biểu đồ.

---

## Xử lý Ngoại lệ (Error & Resilience)

1. **Mất mạng WS Market Data:** Tạm dừng, gọi REST lấy nến bù (Gap Filler), kết nối lại, tiếp tục.
2. **Rate Limit Cảnh Báo:** Nhánh 2 (gửi lệnh WS) KHÔNG BỊ ẢNH HƯỞNG. Chỉ hoãn các lệnh gọi REST (như lấy exchangeInfo mới) trong 30 giây.
3. **Mất kết nối WS User Data:** Gọi lại `POST /listenKey` để lấy key mới và mở lại Stream.
