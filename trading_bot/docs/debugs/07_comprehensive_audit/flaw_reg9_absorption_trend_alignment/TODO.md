# 📋 TODO: FLAW-REG.9 - ABSORPTION MACRO TREND ALIGNMENT

> **Thư mục quản lý:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_reg9_absorption_trend_alignment/`  
> **Nguyên tắc:** Baby Steps, 100% Dynamic & Universal, Không Hardcode, Viết test đi kèm, Báo cáo để User tự commit Git.

---

## 📌 TIẾN ĐỘ CÔNG VIỆC

- [x] **Bước 1: Thiết kế & Khởi tạo Tài liệu Kỹ thuật**
  - [x] Tạo thư mục `flaw_reg9_absorption_trend_alignment/`.
  - [x] Viết `TECHNICAL_SPEC.md` phân tích nguyên nhân VWMA lag, chứng minh toán học và thiết kế Dual-Layer Confirmation 100% Dynamic.
  - [x] Tạo `TODO.md` theo dõi tiến độ chi tiết.
  - [x] Cập nhật `TODO_CRITICAL.md` bổ sung `FLAW-TSL.10` và `FLAW-REG.9`.

- [x] **Bước 2: Cập nhật Cấu hình DNA & Số Chiều (`config.rs` & `mod.rs`)**
  - [x] Thêm gene `pub max_adverse_slope: f64` vào `TacticalAbsorptionParams` trong `config.rs`.
  - [x] Khai báo hàm default `default_absorption_max_adverse_slope() -> f64 { 0.15 }`.
  - [x] Nâng `DNA_WIDTH` từ 118 lên **119 chiều**.
  - [x] Cập nhật unit test `test_dna_width_and_parameter_defs_integrity` trong `core_v3/mod.rs` với 119 chiều.

- [x] **Bước 3: Triển khai Dual-Layer Confirmation trong `absorption.rs`**
  - [x] Tầng 1 (Price Action): Kiểm tra `close >= bb_middle` cho Long và `close <= bb_middle` cho Short.
  - [x] Tầng 2 (Macro Trend Veto): Kiểm tra `trend_dir < -max_adverse_slope` cấm Long và `trend_dir > max_adverse_slope` cấm Short.
  - [x] Đảm bảo 100% Dynamic, đọc trực tiếp từ `abs_cfg.max_adverse_slope`.

- [x] **Bước 4: Viết Bộ Unit Tests Chuyên Sâu Kiểm Chứng**
  - [x] Viết unit test `test_absorption_falling_knife_blocked_by_macro_trend`: Xác minh khi thị trường sập dốc mạnh, `scan()` trả về `None` dù VWMA > BB_Middle.
  - [x] Viết unit test `test_absorption_valid_accumulation_in_neutral_range`: Xác minh khi thị trường phẳng và giá giữ trên BB_Middle, `scan()` phát sinh proposal Long bình thường.

- [x] **Bước 5: Xác minh Toàn diện Hệ thống**
  - [x] Chạy `cargo test --lib` trên `helios` (36/36 tests pass 100%).
  - [x] Chạy `cargo test --lib` trên `bot_engine` (38/38 tests pass 100%).
  - [x] Cập nhật `TODO_CRITICAL.md` tích `[x]` cho FLAW-REG.9.

- [ ] **Bước 6: Báo Cáo Hoàn Thành & Lệnh Git Commit**
  - [ ] Báo cáo kết quả chi tiết cho người dùng.
  - [ ] Cung cấp câu lệnh Git commit thủ công.
