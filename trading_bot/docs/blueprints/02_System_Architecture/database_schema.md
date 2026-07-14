# Thiết Kế Cơ Sở Dữ Liệu (Database Schema)

Dự án sử dụng **DuckDB** lưu trữ cục bộ.
> **File:** `~/.gemini/antigravity/trading_bot_data/market_data.duckdb`

## 1. Bảng `kline_1m` (Source of Truth)
Đây là bảng lưu dữ liệu nến 1 phút được lấy trực tiếp từ WebSocket hoặc REST API của Binance. Dữ liệu này dùng để chạy Strategy và Optimizer.

| Column | Type | Chú thích |
| :--- | :--- | :--- |
| `timestamp` | BIGINT (PK) | Unix millis (Thời điểm MỞ nến) |
| `symbol` | VARCHAR (PK) | |
| `open` | DOUBLE | |
| `high` | DOUBLE | |
| `low` | DOUBLE | |
| `close` | DOUBLE | |
| `volume` | DOUBLE | |

> **Chính sách Lưu trữ (Retention Policy):** Dữ liệu trong bảng này chỉ được giữ tối đa **1 tháng 3 ngày**. Mỗi khi khởi động Bot, một Query DELETE sẽ được gọi để dọn dẹp các nến cũ hơn mốc này. Optimizer sẽ chỉ sử dụng dữ liệu 1 tháng.

## 2. Bảng `strategy_configs` (Lưu trữ Tham số Tối ưu)
Bảng này lưu lại kết quả từ Optimizer. Đây là giải pháp **JSON Dynamic Parameters** siêu linh hoạt, cho phép lưu trữ bất kỳ loại tham số nào của bất kỳ chiến lược nào mà không cần tạo bảng mới.

| Column | Type | Chú thích |
| :--- | :--- | :--- |
| `id` | UUID (PK) | |
| `symbol` | VARCHAR | Tên tài sản (VD: "BTCUSDT") |
| `strategy_name` | VARCHAR | Tên chiến lược (VD: "helios") |
| `params_json` | JSON | Chuỗi JSON chứa params (VD: `{"rsi_len": 14, "sl": 0.05}`) |
| `fitness_score` | DOUBLE | Điểm tối ưu/Lợi nhuận kỳ vọng |
| `created_at` | BIGINT | Unix millis thời điểm Optimizer hoàn thành |
| `is_active` | BOOLEAN | Bộ tham số đang được bot dùng |

*Cách hoạt động:* Khi Optimizer ra kết quả mới, nó INSERT vào bảng này. Bot giao dịch sẽ định kỳ (hoặc qua Event) SELECT dòng có `created_at` mới nhất và nạp JSON ngược vào Struct của Rust (Hot-reload).

## 3. Bảng `positions` (Vị thế đang mở)
Lưu lại trạng thái vị thế hiện tại để bot phục hồi nếu bị crash.

| Column | Type | Chú thích |
| :--- | :--- | :--- |
| `symbol` | VARCHAR (PK) | |
| `direction` | VARCHAR | "LONG" / "SHORT" |
| `avg_entry_price`| DOUBLE | |
| `size` | DOUBLE | Khối lượng |
| `trailing_sl` | DOUBLE | Giá kích hoạt Trailing Stoploss hiện tại |
