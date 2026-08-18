# Kế Hoạch & Danh Mục Kiểm Thử Chi Tiết (Testing Matrix Roadmap)

Tài liệu này hệ thống hóa toàn bộ các kịch bản kiểm thử (Testcases Matrix) cho hệ thống **Trading Bot (`bot_engine`)**, được phân chia theo từng giai đoạn ưu tiên nhằm đảm bảo tính đúng đắn 100% của logic trước khi tối ưu hiệu năng.

---

## 🏛️ Lộ Trình Triển Khai (Execution Roadmap)

- 🎯 **Giai đoạn 1: Paper Forward Test Trọn Vẹn Hệ Thống (Full Engine Paper Forward Test)**
  - Tập trung kiểm thử 4 mô-đun lõi: `PaperTrader`, `Full Event-Driven Flow`, `LivePositionState Sync`, `Event-Driven DNA Swap`.
- 🛡️ **Giai đoạn 2: Kiểm Thử Thực Thi Lệnh Binance API & Phục Hồi Trạng Thái**
  - Tập trung kiểm thử tương tác với Binance API (Trailing SL, Precision, Retry Loop, Panic Close) và phục hồi trạng thái hệ thống sau khi sập (Crash Recovery, Gap Filling, Weight Defense).

---

## 📊 Bảng Danh Mục Kịch Bản Kiểm Thử (Testing Matrix)

| STT | Mô-đun / Chức năng | Kịch bản Kiểm thử (Testcase Scenario) | Kết quả Mong đợi (Expected Behavior) | Mức độ Ưu tiên |
| :---: | :--- | :--- | :--- | :---: |
| **1** | **PaperTrader (Sàn giả lập)** | Giả lập khớp lệnh Mua/Bán (Long/Short), tính toán Realized PnL, Wallet Balance và nến quét giá dính Trailing SL/TP. | Khớp lệnh chính xác 1:1 theo giá nến, cập nhật số dư khả dụng và PnL không bị dính số âm hay nhảy rác. | 🟢 **Giai đoạn 1** |
| **2** | **Full Event-Driven Flow** | Bơm luồng nến lịch sử nối tiếp qua `EventBus` $\rightarrow$ `AssetActor` $\rightarrow$ `PaperTrader` $\rightarrow$ `StatusUpdate`. | Toàn bộ hệ thống chạy mượt mà, không kẹt channel (deadlock), ghi log trạng thái UI chuẩn xác. | 🟢 **Giai đoạn 1** |
| **3** | **LivePositionState Sync** | Giả lập mở vị thế Long/Short và kiểm tra việc đồng bộ thanh bar mở lệnh (`entry_bar`), mức Trailing SL hiện tại và Playbook Tag. | Trạng thái vị thế trong memory luôn khớp 100% với vị thế đang có trên sàn/paper trader. | 🟢 **Giai đoạn 1** |
| **4** | **Event-Driven DNA Swap** | Gửi bộ DNA mới qua kênh `watch::channel` từ Optimizer sang `AssetActor` khi đang chạy dở. | `AssetActor` lập tức nạp DNA mới cho chiến lược `HeliosStrategyV3` ở nến kế tiếp mà không làm sập bot. | 🟢 **Giai đoạn 1** |
| **5** | **Trailing SL Long/Short (Binance API)** | Gửi lệnh Algo Order `STOP_MARKET` cho vị thế Long (`SELL`) và Short (`BUY`) kèm cờ `closePosition = true`. | Binance nhận lệnh thành công, giá `triggerPrice` được làm tròn chuẩn 2 chữ số thập phân (`format_price_decimal`). | 🛡️ **Giai đoạn 2** |
| **6** | **Latency & 3x Retry Loop** | Giả lập sàn Binance phản hồi chậm hoặc báo lỗi HTTP 400 `An open stop order existing`. | Bot tự động nghỉ `150ms`, chạy vòng lặp Retry 3 lần và đặt thành công Trailing SL mới. | 🛡️ **Giai đoạn 2** |
| **7** | **Emergency Panic Close** | Phát tín hiệu `PanicCloseAll` (Ctrl+C hoặc nút bấm khẩn cấp từ UI). | Bot lập tức gửi lệnh Hủy toàn bộ Algo Orders cũ và Market Close tất cả vị thế đang mở trong < 1 giây. | 🛡️ **Giai đoạn 2** |
| **8** | **MarketFilter Precision** | Truyền các giá trị khối lượng/giá lẻ số thập phân bất kỳ (VD: 0.12345678 BTC/ETH/DOGE). | `round_qty` và `round_price` làm tròn chính xác theo `step_size`, `tick_size` và `min_qty` của sàn. | 🛡️ **Giai đoạn 2** |
| **9** | **Crash & State Recovery** | Giả lập Bot bị sập đột ngột khi Binance đang có vị thế mở, sau đó khởi động lại bot. | `AccountManager` query REST API, khôi phục lại vị thế mở và gắn lại Trailing SL bảo vệ lập tức. | 🧱 **Giai đoạn 2** |
| **10** | **WebSocket Reconnect & Gap Fill** | Giả lập đứt mạng WebSocket 5 phút, sau đó có mạng trở lại. | `WsManager` tự kết nối lại, `InitialSync::fill_gap` tự bù 5 nến thiếu vào SQLite mà không trùng lặp timestamp. | 🧱 **Giai đoạn 2** |
| **11** | **WeightMonitor Defence** | Giả lập gọi API dồn dập vượt mốc 2400 weight/phút. | `WeightMonitor` tự động ép các luồng gọi REST API tạm dừng (sleep) chờ hết phút để tránh dính IP Ban. | 🧱 **Giai đoạn 2** |
| **12** | **Multi-Asset Scalability** | Khởi chạy 2+ đồng coin song song (VD: ETHUSDT và BTCUSDT) trên cùng bộ khung `bot_engine`. | Các `AssetActor` xử lý độc lập trên EventBus, không bị nhiễu tín hiệu hay ghi đè lệnh của nhau. | 🧱 **Giai đoạn 2** |
