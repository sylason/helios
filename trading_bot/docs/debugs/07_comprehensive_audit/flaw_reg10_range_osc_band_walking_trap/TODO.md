# 📋 TODO: FLAW-REG.10 - ĐẠI TU TOÀN DIỆN RANGE_OSC & TRIỆT TIÊU BẪY BAND-WALKING TRAP

> **Thư mục quản lý:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_reg10_range_osc_band_walking_trap/`  
> **Nguyên tắc:** Baby Steps, 100% Dynamic qua DNA, Additive-Only, Non-Regression, Viết test đi kèm, Báo cáo để User tự commit Git.

---

## 📌 TIẾN ĐỘ 9 BƯỚC BABY STEPS

- [x] **Bước 1: Thiết Kế Kỹ Thuật & Quản Trị Ranh Giới (Design Docs)**
  - [x] Soạn thảo `PLAYBOOK_BOUNDARY_MATRIX.md` phân định ranh giới tác chiến 6 Playbooks.
  - [x] Soạn thảo `TECHNICAL_SPEC.md` chi tiết 7 thành phần khép kín.
  - [x] Tạo `TODO.md` theo dõi tiến độ chi tiết.

- [x] **Bước 2: Nâng Cấp Tầng Cảm Biến (`sensors.rs`)**
  - [x] Bổ sung trường `bb_middle_slope_adj: Arc<Vec<f64>>` vào `HeliosSensors`.
  - [x] Triển khai hàm tính độ dốc chuẩn hóa của SMA(20) qua ATR ($K=5$ bars).
  - [x] Chạy `cargo test --lib` trên `helios`: Đảm bảo 36/36 tests cũ tiếp tục PASS 100%.

- [x] **Bước 3: Nâng Cấp Tầng Ma Trận Vĩ Mô (`matrix.rs`)**
  - [x] Bổ sung trường `bb_middle_slope_adj: Arc<Vec<f64>>` vào `MarketMatrixV3`.
  - [x] Tích hợp hệ số phẳng $\text{flatness\_factor}$ vào $w_{\text{range\_raw}}$ và `range_multiplier`.
  - [x] Bảo tồn bất biến Simplex: $\sum (w_t + w_r + w_c + w_{ch}) \equiv 1.0000$.
  - [x] Chạy `cargo test --lib` trên `helios`: Đảm bảo 36/36 tests cũ tiếp tục PASS 100%.

- [x] **Bước 4: Cấu Hình DNA Dynamic 1% -> 99% (`config.rs`)**
  - [x] Thêm gene `pub max_bb_slope: f64` (dải `0.02 -> 0.50`, step `0.02`) vào `TacticalRangeParams`.
  - [x] Thêm gene `pub tp_target_ratio: f64` (dải `0.10 -> 0.90`, step `0.05`) vào `TacticalRangeParams`.
  - [x] Khai báo các hàm default tương ứng.
  - [x] Nâng `DNA_WIDTH` của `HeliosConfigV3` từ 119 lên **121 chiều**.

- [x] **Bước 5: Đại Tu 4 Khối Của Chiến Đội Range (`range_osc.rs`)**
  - [x] Khối 1: Xóa bỏ bẫy `.abs()`, kiểm tra giá nằm trong dải BB (Strict Half-Range Boundary: Long chỉ nửa dưới, Short chỉ nửa trên).
  - [x] Khối 2: Kiểm tra nến xác nhận phản ứng (Rejection Candle).
  - [x] Khối 3: Tính toán Take Profit linh hoạt hướng về Trục Giữa (`bb_middle`).
  - [x] Khối 4: Xóa sạch 7 hằng số hardcode, thay thế bằng bội số ATR và gene DNA.

- [x] **Bước 6: Nâng Cấp Tầng Trọng Tài Chống Cướp Thầu (`arbiter.rs`)**
  - [x] Bổ sung cơ chế Preemption Guard: Phủ quyết tín hiệu Range khi thị trường đang có xu hướng định hướng mạnh ($w_{\text{trend\_norm}} > 0.40$).
  - [x] Chạy `cargo test --lib` trên `helios`: Kiểm tra tính toàn vẹn của Arbiter.

- [x] **Bước 7: Đồng Bộ Tầng Quản Trị Rủi Ro & Thoát Lệnh (`risk.rs`)**
  - [x] Tiếp nhận `suggested_tp` động theo Trục Giữa cho Tag `"RANGE_OSC"`.
  - [x] Đồng bộ công thức Dynamic TP theo dải Bollinger và gene `tp_target_ratio`.
  - [x] Đảm bảo cơ chế Trailing Stop và Stagnant Decay không làm nghẽn vị thế Range.

- [x] **Bước 8: Đồng Bộ Tầng Điều Phối & Cooldown (`core_v3/mod.rs`)**
  - [x] Cập nhật assertion `DNA_WIDTH == 121` trong test `test_dna_width_and_parameter_defs_integrity`.
  - [x] Kích hoạt Directional Cooldown cho Range sau khi cắt lỗ.
  - [x] Viết bộ Unit Tests chuyên sâu cho Range mới:
    - [x] `test_range_osc_band_walking_drift_blocked`: Chặn đứng bẫy trôi dốc.
    - [x] `test_range_osc_rejection_candle_confirmed`: Phê duyệt khi có nến phản ứng tại dải.
    - [x] `test_range_osc_breakdown_penetration_blocked`: Chặn đứng khi nến đâm thủng đáy.

- [x] **Bước 9: Xác Minh Toàn Hệ Thống & Lệnh Git Commit**
  - [x] Chạy `cargo test --lib` trên `helios` (39/39 tests PASS 100%).
  - [x] Chạy `cargo test --lib` trên `bot_engine` (38/38 tests PASS 100%).
  - [x] Cập nhật `TODO_CRITICAL.md` tích `[x]` cho FLAW-REG.10.
  - [x] Báo cáo kết quả và cung cấp câu lệnh Git commit thủ công cho người dùng.
