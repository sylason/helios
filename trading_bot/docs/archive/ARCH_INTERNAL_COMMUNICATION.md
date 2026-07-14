# 🧠 Architecture: Internal Communication (Headless Core)

Tài liệu mô tả cách các thành phần trong Bot giao tiếp với nhau để đảm bảo độ trễ thấp và ưu tiên lệnh giao dịch trên CPU 8 luồng.

---

## 1. Hệ thống Kênh truyền tin (Rust Channels)
Bot sử dụng thư viện `tokio::sync` để điều phối thông tin:

| Loại Kênh | Tên | Nhiệm vụ | Độ ưu tiên |
| :--- | :--- | :--- | :--- |
| **MPSC (Unbounded)** | **Emergency Bus** | Lệnh PANIC, CLOSE_ALL, STOP hệ thống. | **1 (Cao nhất)** |
| **MPSC (Bounded)** | **Trade Bus** | Gửi tín hiệu vào/đóng lệnh từ Strategy sang Execution. | **2 (Trung bình)** |
| **Broadcast** | **Data Bus** | Lan tỏa nến 1m cho Actor, DB và Logger. | **3 (Thấp)** |

## 2. Quy trình Xử lý Sự kiện (Event Loop)
Mỗi `AssetActor` chạy một vòng lặp `tokio::select!` để lắng nghe đa kênh:
```rust
loop {
    tokio::select! {
        // 1. Kiểm tra lệnh khẩn cấp trước
        Some(cmd) = emergency_rx.recv() => {
            handle_emergency(cmd);
        }
        // 2. Nhận nến mới để tính toán
        Ok(candle) = data_rx.recv() => {
            let signal = strategy.analyze(candle);
            if signal.is_some() {
                trade_tx.send(signal).await;
            }
        }
    }
}
```

## 3. Quản lý tài nguyên trên i7-3770 (8 Cores)
- **Core Affinity (Dự kiến):**
    - Cores 0-3: Dành cho Main Loop, Market Data và Execution (Yêu cầu độ trễ thấp).
    - Cores 4-7: Dành cho Optimizer (`ferro_trade`).
- **Backpressure:** Nếu `Trade Bus` bị đầy (do Binance API phản hồi chậm), Strategy sẽ tạm dừng gửi tín hiệu mới để tránh tràn bộ nhớ.

## 4. Giao diện điều khiển (CLI/Logs)
Vì tạm hoãn UI, việc giám sát sẽ thông qua:
- **Live Logs:** Ghi chi tiết từng bước nhảy của giá và quyết định của Strategy vào `logs/trade_exec.log`.
- **State File:** Một file JSON (`state.json`) cập nhật liên tục PnL, trạng thái ví và các vị thế đang mở để người dùng kiểm tra nhanh.
