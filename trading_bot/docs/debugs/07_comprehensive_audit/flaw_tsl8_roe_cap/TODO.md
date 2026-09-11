# 📋 MASTER TODO: QUẢN TRỊ RỦI RO BẰNG PER-PLAYBOOK ROE CAP & FALLBACK HIERARCHY (FLAW-TSL.8)

> **Mục tiêu:** Quản trị các đầu việc theo phương pháp Baby Steps cho việc loại bỏ magic number `0.80 / lev` và chuẩn hóa Stop Loss theo ROE Cap.  
> **Thư mục:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_tsl8_roe_cap/TODO.md`  
> **Trạng thái:** `Hoàn thành (Passed 100% Tests - 59/59)`

---

## 📌 DANH SÁCH ĐẦU VIỆC CHI TIẾT (BABY STEPS)

### Giai Đoạn 1: Thiết Kế & Tài Liệu Hóa (Documentation)
- [x] **Task 1.1:** Khởi tạo thư mục tính năng `trading_bot/docs/debugs/07_comprehensive_audit/flaw_tsl8_roe_cap/`.
- [x] **Task 1.2:** Viết tài liệu đặc tả kỹ thuật `TECHNICAL_SPEC.md` (Toán học, Phân cấp Thẩm quyền, Min Distance Floor Guard, Edge Cases).
- [x] **Task 1.3:** Viết tài liệu theo dõi tiến độ `TODO.md`.

### Giai Đoạn 2: Chuẩn Hóa Cấu Hình DNA (`config.rs`)
- [x] **Task 2.1:** Cập nhật dải DNA của `min_initial_sl_pct` trong `RiskStopParams`:
  - `#[dna(min = 0.0015, max = 0.040, step = 0.0005)]` (Sàn an toàn chống cắn spread và phí sàn).
- [x] **Task 2.2:** Bổ sung trường `default_sl_roe_cap` vào `RiskStopParams`:
  - `#[dna(min = 0.04, max = 0.40, step = 0.015)] pub default_sl_roe_cap: f64` (Trần rủi ro cho lệnh Untagged / Adopted, default `0.20`).
- [x] **Task 2.3:** Chuẩn hóa dải DNA của 6 trường Scanner trong `TacticalSniperConfig` (`pullback`, `momentum`, `range`, `reversal`, `anomaly`, `absorption`):
  - Đổi dải DNA từ `% giá` sang `[min = 0.04, max = 0.40, step = 0.015]` (Trần rủi ro ROE Cap của từng Playbook).
- [x] **Task 2.4:** Cập nhật test DNA 112 chiều `test_dna_width_and_parameter_defs_integrity` trong `strategies_backtests/helios/src/strategy/core_v3/mod.rs` (Đã pass 100%).

### Giai Đoạn 3: Triển Khai Clamp SL Khởi Tạo & Min Distance Floor Guard (`risk.rs` & `mod.rs`)
- [x] **Task 3.1:** Sửa hàm `RiskManager::clamp_sl_price` trong `risk.rs`:
  - Nhận `max_sl_roe_cap: f64` thay vì `max_initial_sl_pct`.
  - Tính khoảng cách giá: `let cap_price_pct = max_sl_roe_cap / lev;`.
  - Triển khai **Min Distance Floor Guard**: `let effective_cap = cap_price_pct.max(min_pct);`.
  - Xóa bỏ vĩnh viễn dòng hardcode `let max_sl_by_lev = 0.80 / lev;`.
- [x] **Task 3.2:** Sửa hàm `RiskManager::get_initial_sl` trong `risk.rs` đồng bộ với signature mới.
- [x] **Task 3.3:** Trong `mod.rs` (hàm `next`):
  - Triển khai Fallback Hierarchy: Đọc `max_sl_roe_cap` từ `winning_scanner`, nếu `None` thì fallback về `config.risk_management.stop_management.default_sl_roe_cap` (xóa bỏ hardcode `0.05`).

### Giai Đoạn 4: Triển Khai Fallback Hierarchy Cho Trailing Stop (`risk.rs`)
- [x] **Task 4.1:** Trong `RiskManager::manage_trailing_stop`:
  - Đọc ROE Cap theo Playbook tag (`"TREND_PB"`, `"RANGE_OSC"`, ...), nếu tag `None` hoặc không khớp thì fallback về `risk_cfg.default_sl_roe_cap` (xóa bỏ hardcode `0.05`).
- [x] **Task 4.2:** Xóa bỏ vĩnh viễn dòng 267 `let lev_cap_pct = (playbook_max_sl_pct).min(0.80 / leverage);`.
  - Thay bằng:
    ```rust
    let cap_price_pct = playbook_max_sl_roe / leverage;
    let effective_cap = cap_price_pct.max(risk_cfg.min_initial_sl_pct);
    ```
- [x] **Task 4.3:** Bảo toàn tính đơn điệu Monotonic Ratchet cho Trailing SL.

### Giai Đoạn 5: Kiểm Thử Độc Lập & Xác Minh (Verification)
- [x] **Task 5.1:** Viết 5 unit tests độc lập trong `risk.rs`:
  - Test 1: `test_clamp_sl_price_spot_1x` (Đòn bẩy 1x).
  - Test 2: `test_clamp_sl_price_standard_20x` (Đòn bẩy 20x).
  - Test 3: `test_clamp_sl_price_nuke_50x_floor_guard` (Đòn bẩy 50x xác minh Floor Guard chống kẹp ngược).
  - Test 4: `test_clamp_sl_price_extreme_100x` (Đòn bẩy 100x).
  - Test 5: `test_fallback_hierarchy_untagged_trade_roe_cap` (Lệnh không có tag tự động áp dụng default ROE Cap).
- [x] **Task 5.2:** Chạy kiểm thử toàn diện: Helios (25/25 passed) và Bot Engine (34/34 passed) -> 59/59 tests pass 100%.
- [x] **Task 5.3:** Cập nhật `TODO.md` và `TODO_CRITICAL.md`. Cung cấp câu lệnh Git commit thủ công cho người dùng.
