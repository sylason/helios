# KẾ HOẠCH HÀNH ĐỘNG CHO MARKET MATRIX (MATRIX TODO)

> **Mục tiêu:** Khắc phục bẫy suy biến Percentile, khử bão hòa độ dốc KAMA và dọn dẹp các chỉ báo zombie trong Level 2 Market Matrix.  
> **Tài liệu tham chiếu gốc:**
> - [01_matrix_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_matrix/01_matrix_deep_dive.md)

---

## 🎯 GIAI ĐOẠN 1: KHỬ BẪY SUY BIẾN PERCENTILE CHO SQUEEZE & CHAOS

### 1. [ ] `TODO-MAT-01`: Cải Tổ Cơ Chế Chuẩn Hóa Squeeze Intensity (Khử Bẫy Tụt Về 0.50)
- **Tài liệu chi tiết:** [§2 (Lỗ hổng 1) trong 01_matrix_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_matrix/01_matrix_deep_dive.md#lỗ-hổng-1-bẫy-tê-liệt-của-cửa-sổ-trượt-percentile-the-rolling-percentile-memory-trap)
- **Vị trí code:** [`core_v3/matrix.rs:52-63, 81`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L52)
- **Nội dung:**
  - [ ] **Bước 1.1:** Đánh giá việc sử dụng trực tiếp giá trị thực phi thứ nguyên $sqz\_continuous = \frac{1}{1 + (BB_w / KC_w)^2}$ thay vì đẩy qua `rolling_percentile(20)`. Khi $BB_w < KC_w$, giá trị luôn $\ge 0.50$ phản ánh đúng thực trạng nén bất kể nén bao lâu.
  - [ ] **Bước 1.2:** Nếu vẫn dùng percentile, mở rộng cửa sổ lookback từ 20 nến lên tối thiểu 100 nến hoặc sử dụng Cumulative Distribution Function (CDF).
- **Tiêu chí nghiệm thu:**
  - [ ] Test `test_squeeze_persists_during_long_consolidation`: Thị trường nén chặt 40 nến liên tiếp $\to$ `squeeze_intensity` tại nến 35 vẫn phải duy trì mức cao ($> 0.70$), không được tụt về trung vị 0.50.

---

### 2. [ ] `TODO-MAT-02`: Khắc Phục Điểm Mù Cơn Bão Kéo Dài Trong `chaos_index`
- **Tài liệu chi tiết:** [§2 (Lỗ hổng 1) trong 01_matrix_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_matrix/01_matrix_deep_dive.md#lỗ-hổng-1-bẫy-tê-liệt-của-cửa-sổ-trượt-percentile-the-rolling-percentile-memory-trap)
- **Vị trí code:** [`core_v3/matrix.rs:83-85, 92-107`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L83)
- **Nội dung:**
  - [ ] Neo `chaos_ratio` theo ngưỡng biên độ biến động tuyệt đối chuẩn hóa (ví dụ: $NATR / SMA(NATR, 100)$ vượt ngưỡng 2.0x) thay vì chỉ dùng rolling percentile ngắn hạn 20 nến.
  - [ ] Đảm bảo khi một cơn bão kéo dài 40 nến, chỉ số `chaos_index` vẫn giữ vững trên ngưỡng 0.70 để Gatekeeper duy trì đóng cửa an toàn.
- **Tiêu chí nghiệm thu:**
  - [ ] Test `test_chaos_index_maintains_high_during_prolonged_storm`: Cơn bão kéo dài 50 nến $\to$ nến thứ 35 vẫn có `chaos_index >= 0.70`.

---

## 📈 GIAI ĐOẠN 2: LÀM MỊN ĐỘ DỐC KAMA & TẬN DỤNG CHỈ BÁO ZOMBIE

### 3. [ ] `TODO-MAT-03`: Làm Mịn Phi Tuyến Độ Dốc KAMA (Khử Bão Hòa ±1.00)
- **Tài liệu chi tiết:** [§2 (Lỗ hổng 3) trong 01_matrix_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_matrix/01_matrix_deep_dive.md#lỗ-hổng-3-bão-hòa-độ-dốc-kama-triệt-tiêu-độ-nhạy-saturation-trap)
- **Vị trí code:** [`core_v3/matrix.rs:87-90`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L87)
- **Nội dung:**
  - [ ] Thay thế phép kẹp thô `clamp(-1.0, 1.0)` bằng hàm làm mịn hyperbol phi tuyến:
    $$\text{kama\_slope\_atr} = \tanh\left(\frac{\text{kama\_slope\_adj}}{\text{scale\_atr}}\right)$$
  - [ ] Giữ được độ nhạy gradient khi sóng mở rộng mạnh, không bị kẹt cứng ở hằng số $\pm 1.00$.
- **Tiêu chí nghiệm thu:**
  - [ ] Test `test_kama_slope_atr_smooth_gradient`: Độ dốc $0.40$ và $0.80$ phải cho ra 2 giá trị khác nhau rõ rệt thay vì cùng bị kẹp ở 1.00.

### 4. [ ] `TODO-MAT-04`: Cắm `trend_strength` (ADX) Vào `Trend_PB` Hoặc Dọn Dẹp Trục Đo Thừa
- **Tài liệu chi tiết:** [§2 (Lỗ hổng 2) trong 01_matrix_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_matrix/01_matrix_deep_dive.md#lỗ-hổng-2-chỉ-báo-zombie-lãng-phí-bộ-nhớ-zombie-indicators)
- **Vị trí code:** [`core_v3/matrix.rs:69-79`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/matrix.rs#L69) & [`scanners/trend_pb.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs)
- **Nội dung:**
  - [ ] Tích hợp `matrix.trend_strength` vào `trend_pb.rs` để nhận diện nhịp hồi nông khi trend cực mạnh ($ADX \ge 40$).
  - [ ] Loại bỏ hoặc lazy-calculate các biến không dùng (`bb_width_norm`, `bb_middle_slope_adj`) để tối ưu hóa bộ nhớ RAM và tốc độ backtest GA.
