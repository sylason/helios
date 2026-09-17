# 📋 TODO: DE-OVERENGINEER `matrix.rs` & TINH GIẢN ARBITER

---

## 🎯 Mục Tiêu Cốt Lõi
- Xóa bỏ triệt để 15 hằng số hardcoded và bẫy liên kết phi tuyến 3 tầng trong `matrix.rs` và `arbiter.rs`.
- Chuẩn hóa các trục đo lường vật lý thành đại lượng phi thứ nguyên (KAMA Slope chia theo ATR), thích ứng 100% cho mọi đồng coin bất kể thị giá.
- Xóa bỏ trò chơi Zero-Sum Simplex để giải phóng hoàn toàn `REVERSAL`, `ABSORPTION`, `TREND_PB` và `MOMENTUM_IGNITION`.
- Dọn dẹp gen ma (Phantom Genes) trong `config.rs`.
- Bảo toàn mã nguồn của `range_osc.rs` và `anomaly.rs`.

---

## 🛠️ Danh Sách Nhiệm Vụ Baby Steps

- [x] **Bước 1: Hoàn Thiện Hồ Sơ Thiết Kế & Tài Liệu Kỹ Thuật**
  - [x] Tạo folder `docs/07_matrix_deoverengineer/`.
  - [x] Viết `01_PROBLEM_AUDIT_AND_ARCHITECTURAL_DEBT.md` (Món nợ kiến trúc & 15 hằng số hardcode).
  - [x] Viết `02_TECHNICAL_SPEC_ORTHOGONAL_MATRIX.md` (Đặc tả 4 trục trực giao & Arbiter tuyến tính).
  - [x] Viết `TODO.md` (Kế hoạch hành động).

- [x] **Bước 2: Cập Nhật & Tinh Giản Không Gian DNA (`config.rs`)**
  - [x] Bổ sung `adx_trend_threshold`, `kama_slope_scale_atr`, `vol_z_sensitivity` vào `MatrixConfig`.
  - [x] Comment out 5 gen ma trong `ArbiterConfig` (`min_confidence_trending`, `ranging`, `compression`, `chaos`, `min_multiplier_veto`).
  - [x] Thay thế bằng `min_confidence_threshold: f64` duy nhất.
  - [x] Cập nhật số chiều `DNA_WIDTH` của `HeliosConfigV3` (từ 123 về 121 chiều).

- [x] **Bước 3: Tái Cấu Trúc Toàn Diện `matrix.rs` (De-overengineer Toàn Diện)**
  - [x] Triển khai công thức KAMA Slope chia cho ATR $\delta_{\text{KAMA}} / \text{ATR}$ phi thứ nguyên.
  - [x] Triển khai ADX Tanh Scale theo gen DNA `adx_trend_threshold`.
  - [x] Triển khai Volume Force theo gen DNA `vol_z_sensitivity`.
  - [x] Bổ sung 5 trường dữ liệu trực giao độc lập (`kama_slope_atr`, `trend_axis`, `vol_axis`, `flow_axis`, `chaos_index`).
  - [x] Xóa bỏ triệt để 7 Multipliers và 5 trường Simplex Vector trong `MarketMatrixV3`.
  - [x] Cập nhật `risk.rs` kết nối trực tiếp với `chaos_index` và `vol_axis`.
  - [x] Pass các unit tests kiểm chứng tính phi thứ nguyên, tính trực giao và khả năng chịu lỗi NaN.

- [x] **Bước 4: Tinh Giản Bộ Điều Phối Arbiter (`arbiter.rs`)**
  - [x] Loại bỏ công thức lũy thừa và căn bậc hai phức tạp ở `compute_confidence`.
  - [x] Triển khai chấm điểm tuyến tính $O(1)$: $\text{Confidence} = \text{Raw Score} \times (1.0 - \text{Chaos Penalty})$.
  - [x] Cơ chế giải quyết xung đột cùng chiều ưu tiên lệnh có Stop Loss gần hơn (R:R tốt hơn).
  - [x] Cơ chế giải quyết xung đột ngược chiều chênh lệch nhỏ ($\le 0.15$) thì Flat đứng ngoài bảo toàn vốn.

- [x] **Bước 5: Đồng Bộ Hóa Scanners (`trend_pb.rs`) & Bộ Unit Tests Mới**
  - [x] Đồng bộ `scanners/trend_pb.rs`: Đọc trực tiếp `kama_slope_atr` và `trend_axis` từ `MarketMatrixV3`.
  - [x] Chặn dứt điểm bẫy bắt dao rơi: Yêu cầu $\text{kama\_slope} > \text{slope\_thresh}$ cho Long và $< -\text{slope\_thresh}$ cho Short.
  - [x] Nhân hệ số định lượng xu hướng `trend_axis` vào điểm số để tự động phòng thủ khi thị trường không có sóng.
  - [x] Pass toàn bộ 5 unit tests của `trend_pb` và 4 unit tests của `arbiter`.

- [x] **Bước 6: Xác Minh Toàn Workspace & Chuẩn Bị Nghiệm Thu Walk-Forward**
  - [x] Chạy `cargo test --lib` trên `strategies_backtests/helios` (46/46 passed).
  - [x] Chạy `cargo test --lib` trên `trading_bot/bot_engine` (38/38 passed).
  - [x] Thông báo câu lệnh commit thủ công cho người dùng.
