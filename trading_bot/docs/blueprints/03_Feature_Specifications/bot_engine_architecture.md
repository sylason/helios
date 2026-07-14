# Kiến trúc Engine Cốt lõi (Bot Engine Architecture)

Tài liệu này đặc tả cấu trúc mã nguồn bên trong `trading_bot/bot_engine` và cách nó kết nối với thư viện nghiên cứu.

## 1. Tích hợp Helios Crate (Cargo Path Dependency)
Để chia tách môi trường Research và Production nhưng vẫn không vi phạm DRY (Don't Repeat Yourself), thuật toán Helios sẽ KHÔNG được copy vào source code của Bot.
Thay vào đó, `bot_engine` xem `helios` như một thư viện nội bộ.

Trong `bot_engine/Cargo.toml`:
```toml
[dependencies]
helios = { path = "../../strategies_backtests/helios" }
```
**Quy trình:**
- Mọi nghiên cứu, test, viết hàm sinh tín hiệu sẽ làm ở `strategies_backtests/helios`.
- `bot_engine` chỉ việc khởi tạo struct của `helios`, nạp chuỗi JSON vào, và gọi hàm `on_data()` để lấy tín hiệu.

## 2. Cấu trúc Thư mục Đề xuất (Folder Skeleton)
Cấu trúc `bot_engine/src` được tối ưu hóa theo các trọng tâm của dự án:

```text
bot_engine/src/
 ├── main.rs
 ├── controller/       
 │   └── mod.rs        # Quản lý vòng đời các Actor (Mỗi Asset là một Actor).
 ├── data_engine/      
 │   ├── ws_client.rs  # Lắng nghe K-line 1m và MarkPrice.
 │   ├── gap_filler.rs # Logic gọi REST API bù khoảng trống nến.
 │   └── duckdb_repo.rs# Các hàm ghi nến, xóa nến cũ (> 1 tháng 3 ngày).
 ├── optimizer/        
 │   └── ga_runner.rs  # Vòng lặp chạy Genetic Algorithm (ferro_trade) ngầm. Ghi kết quả vào DB.
 ├── execution/        
 │   ├── order_mgr.rs  # Bắn lệnh Market/Limit (Local/Testnet/Mainnet).
 │   └── trailing_sl.rs# Observer theo dõi MarkPrice để gỡ vị thế.
 ├── strategy_runner/  
 │   └── engine.rs     # Tương tác với crate helios. Nạp tham số JSON từ DB. Nạp lại (Hot-reload) nếu có cấu hình mới.
 ├── interface/        
 │   └── tauri_ipc.rs  # Các trait/channel giao tiếp với React Frontend.
 └── models/           
     └── shared.rs     # Định nghĩa Enums, K-Line struct dùng chung.
```

## 3. Hoạt động của Asset Actor (Mỗi đồng coin 1 luồng)
Mỗi cặp giao dịch (VD: ETHUSDT) sẽ là một luồng (Task) hoạt động độc lập (Actor). 
Bên trong Actor đó chứa:
- 1 Data Engine.
- 1 Strategy Runner.
- 1 Execution Manager.
- 1 Optimizer (Chạy độc lập trên Thread pool khác để không chặn luồng nhận nến).

Nếu ETHUSDT bị lỗi và sập, thuật toán Auto-Panic sẽ chỉ tắt ETHUSDT. Các cặp khác như BTCUSDT vẫn tiếp tục trade bình thường.
