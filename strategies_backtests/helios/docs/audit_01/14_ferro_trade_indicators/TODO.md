# 📋 DANH MỤC CÔNG VIỆC: KHẮC PHỤC LỖI ĐỘNG CƠ CHỈ BÁO FERRO_TRADE
> **Phân hệ:** `ferro_trade/src/indicators/`  
> **Thời điểm khởi tạo:** 2026-09-28  
> **Tài liệu kiểm toán kỹ thuật:** [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/14_ferro_trade_indicators/technical_audit.md)  
> **Trạng thái:** 🟡 Sẵn sàng thực thi (Pending Execution)

---

## 🎯 MỤC TIÊU CỐT LÕI
Loại bỏ 100% các lỗ hổng toán học, bẫy nuốt chửng warmup, ngộ độc trạng thái NaN và sai số dấu phẩy động trong động cơ tính toán chỉ báo `ferro_trade`, đảm bảo tính đúng đắn toán học tuyệt đối và bảo toàn dữ liệu cho GA Optimizer (`Prospector`) và Strategy (`Helios`).

---

## 📌 BẢNG THEO DÕI TIẾN ĐỘ THỰC THI

- [x] **TASK-FT-01: Khắc phục bẫy nuốt chửng nến Warmup khi lỗi (`DEFECT-FT-01`)**
  - **Mức độ:** `CRITICAL`
  - **Vị trí tác động:** [`ferro_trade/src/backtest/manager.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/manager.rs), [`ferro_trade/src/backtest/context.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/context.rs), [`ferro_trade/src/backtest/engine.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/backtest/engine.rs)
  - **Nghiệp vụ:** Khi indicator gặp lỗi tham số và trả về toàn bộ mảng `NaN`, helper `register_indicator` chặn đứng việc tăng `warmup_period`. Kích hoạt cơ chế **Early Infeasible Abort** tại bar 0, trả về điểm phạt di truyền trong im lặng (Zero-Log Spam). Áp dụng trần Warmup 40% và sàn 50 nến giao dịch.
  - **Kiểm thử:** Đã viết test suite [`ferro_trade/tests/infeasible_warmup_guard_test.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/tests/infeasible_warmup_guard_test.rs) bao phủ 5/5 kịch bản (Pass 100%).

- [ ] **TASK-FT-02: Vá lỗi Z-Score tự sát khi thị trường đi ngang (`DEFECT-FT-02`)**
  - **Mức độ:** `HIGH`
  - **Vị trí tác động:** [`ferro_trade/src/indicators/calculate/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L125-L130)
  - **Nghiệp vụ:** Khi độ lệch chuẩn $\sigma \approx 0.0$ và khoảng cách tới trung bình $|X - \mu| \approx 0.0$, trả về $Z = 0.0$ (tâm phân phối) thay vì trả về `f64::NAN`.
  - **Kiểm thử:** Unit test với mảng giá trị không đổi (vd: `[100.0, 100.0, 100.0, 100.0]`), kiểm tra Z-score sau warmup trả về đúng `0.0`.

- [ ] **TASK-FT-03: Triệt tiêu ngộ độc trạng thái nội bộ Yata SMA (`DEFECT-FT-03`)**
  - **Mức độ:** `HIGH`
  - **Vị trí tác động:** [`ferro_trade/src/indicators/calculate/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L201-L207)
  - **Nghiệp vụ:** Quét tìm phần tử hợp lệ đầu tiên (`!val.is_nan()`) để khởi tạo `SMA::new`, không truyền `&source[0]` mù quáng khi nến đầu là `NaN` của chỉ báo trước.
  - **Kiểm thử:** Unit test tính SMA trên một mảng dữ liệu có chứa các phần tử đầu là `NaN`, xác minh các phần tử sau warmup cho kết quả hợp lệ.

- [ ] **TASK-FT-04: Khắc phục sai số trôi nổi tích lũy trong Kaufman ER (`DEFECT-FT-04`)**
  - **Mức độ:** `MEDIUM`
  - **Vị trí tác động:** [`ferro_trade/src/indicators/calculate/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L162)
  - **Nghiệp vụ:** Thêm kẹp sàn bảo vệ `.max(0.0)` cho `volatility_sum` sau mỗi bước cập nhật trượt để chống Catastrophic Cancellation.
  - **Kiểm thử:** Unit test Kaufman ER trên chuỗi dài 10,000 nến, kiểm tra không xuất hiện giá trị âm giả mạo.

- [ ] **TASK-FT-05: Khắc phục bẫy kẹp ép kiểu `u8::MAX` trong `to_yata_ma` (`DEFECT-FT-05`)**
  - **Mức độ:** `MEDIUM`
  - **Vị trí tác động:** [`ferro_trade/src/indicators/calculate/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L28-L30)
  - **Nghiệp vụ:** Kẹp `period.clamp(1, 255)` an toàn và cảnh báo nếu `period > 255`.
  - **Kiểm thử:** Unit test kiểm tra trường hợp `period = 0` và `period = 300`.

- [ ] **TASK-FT-06: Bổ sung guard input trong `keltner_channel` (`DEFECT-FT-06`)**
  - **Mức độ:** `LOW`
  - **Vị trí tác động:** [`ferro_trade/src/indicators/calculate/bulk.rs`](file:///e:/Projects/Automations/trading_backtest/ferro_trade/src/indicators/calculate/bulk.rs#L1390-L1405)
  - **Nghiệp vụ:** Kiểm tra `period == 0` hoặc `multiplier <= 0.0` ngay đầu hàm, trả về lỗi tường minh thay vì để Yata panic.
  - **Kiểm thử:** Unit test gọi Keltner Channel với `period = 0`.
