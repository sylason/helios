# Quản Lý Công Việc & Kiểm Thử Đặc Tả Phần Mềm Helios V3 (TODO.md)

---

## 📌 Trạng Thái Đặc Tả Kỹ Thuật (Specification Status)

- [x] **Tài liệu hóa 5 Trụ Cột Trạng Thái Thị Trường:** Đã hoàn thành bảng ánh xạ chi tiết tại [01_market_state_pillars_and_data_lineage.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/09_software_specification/01_market_state_pillars_and_data_lineage.md).
- [x] **Phân định rõ ràng về Thanh khoản:** Đã làm rõ sự khác biệt giữa `sensors.mfi` (chỉ số dòng tiền thật, đang dùng) và `matrix.liquidity` (chỉ báo ma bị bọc rolling percentile méo mó, cần loại bỏ).
- [x] **Hoàn thành Phần 2 của TASK-MAT-07:**
  - [x] Rút gọn `MarketMatrixV3` trong `matrix.rs` từ 13 trường về đúng 6 trường vật lý cốt lõi.
  - [x] Đồng bộ hóa lưới biên nến `self.matrix.chaos_index.len() <= i` tại `mod.rs:269`.
  - [x] Dọn dẹp các assert cũ trong unit tests của `matrix.rs` và `mod.rs`.

---

## 🧪 Danh Sách Test Cases Xác Minh 5 Trụ Cột

| Mã Test | Tên Bài Test | Mục Đích Kiểm Thử | Trạng Thái |
| :--- | :--- | :--- | :--- |
| **TEST-PIL-01** | `test_orthogonal_matrix_nan_resilience` | Kiểm tra khả năng kháng NaN của 6 trường ma trận cốt lõi khi dữ liệu rác. | [x] PASS |
| **TEST-PIL-02** | `test_matrix_independence_no_zero_sum` | Xác minh tính độc lập giữa Trục Xu Hướng (`trend_strength`) và Trục Thanh Khoản (`vol_z_score`). | [x] PASS |
| **TEST-PIL-03** | `test_whipsaw_discrimination_chaos_index` | Kiểm tra `chaos_index` kích hoạt phạt tự tin khi bão Whipsaw bùng nổ. | [x] PASS |
| **TEST-PIL-04** | `test_squeeze_continuous_no_half_stuck` | Kiểm tra cường độ nén `squeeze_intensity` hoạt động mượt mà, không bị kẹt ở 0.50. | [x] PASS |
| **TEST-PIL-05** | `test_kama_slope_atr_scale_asset_agnostic` | Kiểm tra tính phi thứ nguyên của `kama_slope_atr` trên các tài sản giá khác nhau (ETH vs DOGE). | [x] PASS |
| **TEST-PIL-06** | `test_volume_zscore_differentiation_from_price` | Kiểm tra tính độc lập tuyệt đối giữa Z-Score giá (`sensors.z_score`) và Z-Score khối lượng (`sensors.vol_z_score`). | [x] PASS |
