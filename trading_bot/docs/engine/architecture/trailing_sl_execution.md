# 🛡️ Trailing Stop Loss (Trailing SL) Execution Architecture

Tài liệu thiết kế chi tiết về cơ chế thực thi, độ chính xác giá (Mark Price) và thuật toán chống xung đột (Failsafe Retry Loop) cho lệnh Trailing SL trên Binance Futures.

---

## 🎯 1. Tại Sao Phải Dùng `Mark Price` Cho Trailing SL?

Trên thị trường Binance USDⓈ-M Futures, sàn hỗ trợ 2 loại giá kích hoạt (`workingType`):
1. `CONTRACT_PRICE` (Last Price - Giá khớp lệnh gần nhất trên sổ lệnh).
2. `MARK_PRICE` (Giá Đánh Dấu - Được tính toán từ Chỉ số giá trung bình các sàn lớn + Funding Rate).

### ⚠️ Rủi Ro Của `Last Price`:
Khi thị trường có biến động mạnh hoặc "râu nến quét thanh khoản" (wick spikes) ảo trong $0.1$ giây, giá `Last Price` có thể giật sập rồi bật lên ngay lập tức $\rightarrow$ Khiến bot bị cắn Stop Loss oan uổng ở đáy.

### 🛡️ Giải Pháp Chuẩn Mực Với `Mark Price`:
`Mark Price` loại bỏ hoàn toàn các râu nến ảo và hiện tượng thao túng giá ngắn hạn. Trong [`order_executor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/order_executor.rs), mọi lệnh Algo `STOP_MARKET` đều bắt buộc cấu hình:
```rust
working_type: WorkingTypeEnum::MarkPrice
```

---

## 🔄 2. Luồng Cập Nhật Trailing SL An Toàn (Anti-Race Condition Flow)

Khi `AssetActor` phát tín hiệu dịch chuyển Trailing SL (`Action::UpdateTradeSlTp`):

```mermaid
sequenceDiagram
    participant Actor as 🧠 AssetActor
    participant Exec as 🛡️ OrderExecutor
    participant Binance as 🌐 Binance Futures REST API

    Actor->>Exec: execute_action(UpdateTradeSlTp, new_sl_price)
    
    rect rgb(30, 40, 60)
    Note over Exec,Binance: BƯỚC 1: HỦY LỆNH ALGO SL CŨ
    Exec->>Binance: cancel_all_algo_open_orders(symbol)
    Binance-->>Exec: Ok (Đã nhận yêu cầu hủy)
    end

    rect rgb(60, 50, 20)
    Note over Exec: BƯỚC 2: CHỜ AN TOÀN 150MS<br>(Đảm bảo sổ lệnh Binance đã xóa sạch lệnh cũ)
    Exec->>Exec: tokio::time::sleep(150ms)
    end

    rect rgb(20, 60, 30)
    Note over Exec,Binance: BƯỚC 3: VÒNG LẶP RETRY 3 LẦN ĐẶT LỆNH SL MỚI
    loop Thử tối đa 3 lần (mỗi lần cách 150ms)
        Exec->>Binance: new_algo_order(STOP_MARKET, MarkPrice, closePosition=true)
        alt Thành công (200 OK)
            Binance-->>Exec: Ok (Lệnh SL mới đã active)
        else Thất bại do độ trễ mạng
            Binance-->>Exec: Err (Race condition)
            Exec->>Exec: Chờ 150ms rồi thử lại
        end
    end
    end
```

---

## 📐 3. Chuẩn Hóa Số Thập Phân (Price & Quantity Precision)

Binance Futures sẽ lập tức từ chối lệnh với mã lỗi `Precision is over the maximum defined` nếu giá hoặc khối lượng bị dư số thập phân do sai số `f64`.

Hệ thống xử lý thông qua 2 hàm cốt lõi trong `order_executor.rs`:
- `format_price_decimal(price)`: Làm tròn chính xác theo số chữ số thập phân của `market_filter.tick_size`.
- `format_qty_decimal(qty)`: Làm tròn chính xác theo số chữ số thập phân của `market_filter.step_size`.
