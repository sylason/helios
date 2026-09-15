# 📋 TODO: ĐẠI TU TOÀN DIỆN CHIẾN ĐỘI TREND PULLBACK (`trend_pb.rs`)

---

## 🎯 Mục Tiêu
- Đại tu khép kín `trend_pb.rs` theo Clean Architecture & Chuẩn hóa định lượng 100%.
- Tháo gỡ tử huyệt "Câm lặng 0 lệnh" trong `matrix.rs`.
- Loại bỏ toàn bộ các hằng số magic numbers `-0.1` và `40%-40%-20%`.
- Nâng DNA từ 121 lên 123 chiều.

---

## 🛠️ Danh Sách Nhiệm Vụ Baby Steps

- [x] **Bước 1: Nâng Cấp Cấu Hình DNA Dynamic 1% -> 99% (`config.rs`)**
  - [x] Bổ sung `max_pullback_dist_atr: f64` vào `TacticalPullbackParams`.
  - [x] Bổ sung `invalidation_buffer_atr: f64` vào `TacticalPullbackParams`.
  - [x] Đổi `rsi_buy_thresh` thành `rsi_max_cap: f64`.
  - [x] Nâng `DNA_WIDTH` của `HeliosConfigV3` lên 123 chiều.

- [x] **Bước 2: Tháo Gỡ Điểm Nghẽn Matrix (`matrix.rs`)**
  - [x] Bổ sung `slope_strength` của KAMA vào `trend_multiplier`.
  - [x] Đảm bảo ADX hạ nhiệt trong nhịp hồi không dập tắt $w_{\text{trend}}$.
  - [x] Giữ vững tính chuẩn hóa Simplex $\sum \equiv 1.00$.

- [x] **Bước 3: Đại Tu Toàn Diện Logic Tính Điểm Toán Học (`scanners/trend_pb.rs`)**
  - [x] Triển khai Trụ cột 1: Khoảng cách không gian KAMA $\delta_{\text{KAMA}}$ (Invalidation Floor & Distance Ceiling).
  - [x] Triển khai Trụ cột 2: Vùng hạ nhiệt Stoch & RSI Max Cap.
  - [x] Triển khai Trụ cột 3: Đạo hàm vận tốc giá CLV & Gradient dương.
  - [x] Xóa sạch toàn bộ hàm phi tuyến sigmoid `-0.1` và trọng số cứng.

- [x] **Bước 4: Đồng Bộ Tầng Điều Phối & Viết Bộ Unit Tests Mới (`core_v3/mod.rs`)**
  - [x] Cập nhật assertion `DNA_WIDTH == 123`.
  - [x] Viết 5 unit tests chuyên sâu kiểm chứng các edge cases.
  - [x] Chạy `cargo test --lib` trên `helios` (44/44 pass) và `bot_engine` (38/38 pass).

- [x] **Bước 5: Đồng Bộ Cổng Nạp DNA An Toàn & Database Trong `trading_bot`**
  - [x] Xóa file SQLite cũ không tương thích nếu cần (`walk_forward_smart_seed.db`).
  - [x] Kiểm chứng `cargo test --lib` trên toàn bộ workspace.
