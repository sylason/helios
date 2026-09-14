# 📋 TODO: CỤM 1 (FLAW-REG.1 & FLAW-TSL.9) - 4-STATE SIMPLEX VECTOR & FAIR AUCTION

> **Thư mục quản lý:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_reg1_whipsaw_and_absorption_churn/`  
> **Nguyên tắc:** Baby Steps, 100% Dynamic & Universal, Không Hardcode, Viết test đi kèm, Báo cáo để User tự commit Git.

---

## 📌 TIẾN ĐỘ CÔNG VIỆC

- [x] **Bước 1: Thiết kế & Khởi tạo Tài liệu Kỹ thuật**
  - [x] Tạo thư mục `flaw_reg1_whipsaw_and_absorption_churn/`.
  - [x] Viết `TECHNICAL_SPEC.md` mô tả toán học 4-State Simplex Vector, bóc tách điểm mù Whipsaw và giải mã hiện tượng Absorption độc tôn.
  - [x] Tạo `TODO.md` theo dõi tiến độ chi tiết.

- [x] **Bước 2: Nâng cấp Ma trận Vĩ mô 4 Trạng thái (`matrix.rs`)**
  - [x] Bổ sung các trường `w_comp_norm` và `w_chaos_norm` vào struct `MarketMatrixV3`.
  - [x] Triển khai công thức $w_{\text{trend\_raw}}$, $w_{\text{clean\_range\_raw}}$, $w_{\text{comp\_raw}}$, $w_{\text{chaos\_raw}}$.
  - [x] Triển khai thuật toán Continuous Simplex Normalization đảm bảo $\sum \equiv 1.00$ với Fallback an toàn $0.25$ khi nến đầu bị `NaN`.
  - [x] Viết unit test `test_4state_simplex_normalization_sum_is_one`.
  - [x] Viết unit test `test_whipsaw_discrimination_prevents_fake_range`.

- [x] **Bước 3: Nâng cấp Signal Arbiter Đấu thầu Công bằng (`arbiter.rs`)**
  - [x] Cập nhật phân bổ `regime_weight` 4 trạng thái cho 6 Tactical Scanners.
  - [x] Tính toán ngưỡng phê duyệt động `required_confidence` từ 4 trọng số bối cảnh.
  - [x] Viết unit test `test_fair_auction_balance_scanners`.
  - [x] Viết unit test `test_whipsaw_suppression_blocks_bad_entries`.

- [x] **Bước 4: Cập nhật Cấu hình DNA & Kiểm soát Số chiều (`config.rs`)**
  - [x] Cập nhật `ArbiterConfig`: Bổ sung `min_confidence_compression` và chuẩn hóa `min_confidence_chaos`.
  - [x] Đảm bảo toàn bộ gene tuân thủ Universal `[min = 0.01, max = 0.99, step = 0.03]`.
  - [x] Kiểm tra số chiều DNA, đảm bảo tính tương thích và bảo vệ an toàn bộ nhớ (DNA 118 chiều).

- [x] **Bước 5: Xác minh Toàn diện Hệ thống**
  - [x] Chạy `cargo test --lib` trên `helios` (Mục tiêu: 100% tests pass -> 34/34 passed).
  - [x] Chạy `cargo test --lib` trên `bot_engine` (Mục tiêu: 100% tests pass -> 38/38 passed).
  - [x] Cập nhật `TODO_CRITICAL.md` tích `[x]` cho FLAW-REG.1 và FLAW-TSL.9.

- [ ] **Bước 6: Hướng dẫn Người dùng Kiểm thử Thực nghiệm Walk-Forward**
  - [ ] Cung cấp tài liệu hướng dẫn tham số cần thay đổi để chạy Walk-Forward.
  - [ ] Chạy kiểm thử đối chiếu trên **ETHUSDT** (Thanh khoản cao).
  - [ ] Chạy kiểm thử đối chiếu trên **LSKUSDT** (Altcoin thanh khoản mỏng, độ ồn cao).
  - [ ] Cung cấp câu lệnh Git commit để người dùng tự lưu trữ thủ công.
