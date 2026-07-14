# 💾 Architecture: Database & Data Flow (MVP Demo)

Thiết kế hệ thống lưu trữ và luồng dữ liệu tối giản cho bản Demo ETHUSDT, tập trung vào tính chính xác 1:1.

---

## 🏗️ 1. Công nghệ & Mô hình (Storage Stack)
- **Engine:** **DuckDB** (In-process, duy nhất 1 file `trading_bot.db`).
- **Pattern:** **Repository Pattern** kết hợp với **DTO (Data Transfer Object)**.
- **DTOs:**
    - `CandleDTO`: `{symbol, timestamp, o, h, l, c, v}`.
    - `MarketTickDTO`: `{price, volume, timestamp}` (Dùng trong RAM).

## 📊 2. Cấu trúc Bảng (Schema)
Chỉ duy nhất một bảng cần thiết cho vận hành:
- **`candles_1m`:**
    - `symbol` (TEXT), `open_time` (BIGINT - PK), `o`, `h`, `l`, `c`, `v` (DOUBLE).
    - Lưu trữ window 30-45 ngày dữ liệu "sạch" từ sàn.

## 🔄 3. Luồng dữ liệu (Data Pipeline)
Hệ thống phân tách rõ ràng thành các module độc lập:
1. **MarketDataManager (Aggregator):** 
    - Thu thập `aggTrade` từ WebSocket.
    - Tổng hợp thành `CandleDTO` sau mỗi phút.
    - Phát tán nến mới qua **Tokio Broadcast Channel**.
2. **DatabaseWriter:** 
    - Lắng nghe Channel và thực hiện `INSERT` nến vào DuckDB thông qua Repository.
3. **AssetActor (Trader):** 
    - Lắng nghe Channel để kích hoạt chu kỳ Phân tích (Helios) và Optimize (Ferro-trade).

## 🚀 4. Quy trình Khởi động (Clean-Slate Sync)
Để đảm bảo dữ liệu không bao giờ bị lỗi hoặc thiếu:
1. **Reset:** Xóa sạch dữ liệu trong bảng `candles_1m`.
2. **History Fetch:** Gọi REST API (Binance Klines) để tải toàn bộ 30 ngày dữ liệu nến 1m (chia làm nhiều đợt).
3. **Hot-Start:** Nạp dữ liệu vào DB và RAM, sau đó mới kích hoạt WebSocket để tiếp nhận dữ liệu thời gian thực.

## 🛠️ 5. Tính Linh hoạt (Future-Proof)
- Toàn bộ logic nghiệp vụ chỉ làm việc với `CandleRepository` (Trait).
- Trong tương lai, việc thay đổi từ DuckDB sang TimescaleDB chỉ yêu cầu viết một bản Implementation mới cho Trait này mà không ảnh hưởng đến Logic Trading.
