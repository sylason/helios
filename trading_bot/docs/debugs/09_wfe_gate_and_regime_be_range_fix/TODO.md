# 📋 DANH MỤC CÔNG VIỆC: WFE GATE, DYNAMIC BREAK-EVEN THEO REGIME & SỬA LỖI RANGE_OSC

> **Thư mục quản lý:** `trading_bot/docs/debugs/09_wfe_gate_and_regime_be_range_fix/`  
> **Các tài liệu kỹ thuật liên quan:**  
> - `01_wfe_validation_gate_spec.md`  
> - `02_regime_dynamic_break_even_spec.md`  
> - `03_range_osc_trend_filter_and_tp_fix.md`  

---

## 🎯 DANH SÁCH NHIỆM VỤ THEO BABY STEPS

### 🌿 BƯỚC 1: CẤU HÌNH & GEN DNA TRONG `HELIOS` (`config.rs`)
- [x] **Task 1.1:** Thêm 6 gen DNA phân tầng Dynamic Break-Even vào `StopManagementConfig` (`config.rs`):
  - `be_atr_trigger_trend`, `be_buffer_atr_trend`
  - `be_atr_trigger_range`, `be_buffer_atr_range`
  - `be_atr_trigger_volatile`, `be_buffer_atr_volatile`
- [x] **Task 1.2:** Thêm 2 gen DNA lọc xu hướng vào `TacticalRangeParams` (`config.rs`):
  - `max_trend_strength`
  - `max_adverse_slope`
- [x] **Task 1.3:** Kiểm tra biên dịch `cargo check --package helios` đảm bảo macro derive DNA hợp lệ.

---

### 🌿 BƯỚC 2: TÁI THIẾT KẾ BREAK-EVEN TRONG `risk.rs`
- [x] **Task 2.1:** Cập nhật hàm `manage_trailing_stop` trong `risk.rs` tra cứu bộ `(be_trigger_mult, be_buffer_mult)` theo Tag của Playbook.
- [x] **Task 2.2:** Xóa bỏ hoàn toàn con số hardcode $0.0008$, thay bằng công thức tính SL hòa vốn neo theo `High Watermark - ATR * be_buffer_mult`.
- [x] **Task 2.3:** Giữ nguyên comment và các log telemetry quan trọng.

---

### 🌿 BƯỚC 3: SỬA LỖI `RANGE_OSC` (FLAW-4.2 & FLAW-4.3)
- [x] **Task 3.1:** Thêm bộ lọc `matrix.trend_strength` và `matrix.trend_dir` trong `scanners/range_osc.rs`.
- [x] **Task 3.2:** Sửa FLAW-4.2: Chuẩn hóa `suggested_tp` cho Long $\ge \text{Close} \times 1.002$ và Short $\le \text{Close} \times 0.998$.
- [x] **Task 3.3:** Sửa FLAW-4.3 trong `core_v3/mod.rs`: Bỏ `.abs()`, chỉ mở khóa Cooldown Long khi giá hồi phục tăng ngược lên.

---

### 🌿 BƯỚC 4: TRIỂN KHAI WFE VALIDATION GATE TRONG `prospector_runner.rs` (FLAW-6.4)
- [x] **Task 4.1:** Cắt cửa sổ huấn luyện thành In-Sample và Validation OOS (48 giờ) trong `run_optimization_cycle_range_smart_seed`.
- [x] **Task 4.2:** Gọi `diagnostic_backtest` trên tập Validation nến.
- [x] **Task 4.3:** Thêm logic phân nhánh PASS (lưu DB & broadcast) vs REJECT (từ chối nạp, giữ DNA cũ) kèm log chi tiết cho việc debug.

---

### 🌿 BƯỚC 5: KIỂM THỬ XÁC MINH & MASTER TODO
- [x] **Task 5.1:** Chạy `cargo test --package helios --lib` (6/6 passed).
- [x] **Task 5.2:** Chạy `cargo test --package bot_engine --lib` (31/31 passed, 1 ignored).
- [x] **Task 5.3:** Chạy `cargo test --package bot_engine --test trailing_sl_test` (4/4 passed).
- [x] **Task 5.4:** Cập nhật trạng thái `[x]` cho FLAW-6.4, FLAW-4.2, FLAW-4.3 trong `TODO_CRITICAL.md`.
- [x] **Task 5.5:** Thông báo và cung cấp lệnh commit git cho người dùng.
