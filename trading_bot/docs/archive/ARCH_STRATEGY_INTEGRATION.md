# 🧩 Architecture: Strategy Integration (Helios Bridge)

Tài liệu mô tả cách kết nối Rust Core với chiến lược `helios` và cơ chế cập nhật tham số tự động.

---

## 1. Cấu trúc hàm `run_optimize`
Hàm này là cầu nối chính giữa Bot và bộ máy tối ưu hóa:

```rust
// Logic giả định trong Rust Core
pub fn run_optimize(asset: String) {
    // 1. Lấy 30 ngày dữ liệu nến 1m từ DuckDB
    let candles = db.get_last_30_days_candles(&asset);
    
    // 2. Gọi Helios (Hardcoded) để tối ưu hóa
    // Hàm này sẽ tự ghi kết quả vào file/db khi xong
    helios::optimize_prospect(candles, config.opt_num_threads);
}
```

## 2. Cơ chế cập nhật Params (Hot-swap)
Để đảm bảo chiến lược luôn dùng bộ thông số mới nhất mà không cần khởi động lại:
1. **Trigger:** `ferro_trade` hoàn tất việc ghi `best_params.json`.
2. **Notification:** Gửi thông báo qua kênh `Internal Event Bus`.
3. **Reload:** `AssetActor` thực hiện nạp lại (Reload) cấu hình vào bộ nhớ đệm (Cache).
4. **Apply:** Mọi tính toán từ giây tiếp theo sẽ sử dụng cấu hình mới.

## 3. Quy trình Giao tiếp qua UI (Tauri Commands)
Các nút bấm trên UI sẽ gọi các hàm tương ứng trong Rust:

| Nút bấm | Tauri Command | Hành động Core |
| :--- | :--- | :--- |
| **Prepare Data** | `cmd_prepare_data` | Sync dữ liệu nến 1m vào DuckDB. |
| **Run Optimize** | `cmd_run_optimize` | Chạy GA trên 30 ngày dữ liệu, giới hạn CPU threads. |
| **Start Trade** | `cmd_start_trade` | Kết nối WebSocket, dùng Params mới nhất để trade. |
| **Terminate** | `cmd_emergency_stop` | Đóng toàn bộ lệnh sàn Binance và dừng Actor. |

---

## 4. Rủi ro & Xử lý (Safety First)
- **File Lỗi:** Nếu `best_params.json` bị lỗi định dạng khi đang đọc, Bot sẽ giữ nguyên bộ Params cũ và ghi Log cảnh báo.
- **CPU Overload:** Nếu người dùng nhấn Optimize khi đang Trade, Bot sẽ cảnh báo hoặc giới hạn luồng thấp hơn nữa để tránh trễ lệnh Trade.
