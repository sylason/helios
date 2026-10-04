# KẾ HOẠCH HÀNH ĐỘNG CHO SIGNAL ARBITER & GATEKEEPER (ARBITER TODO)

> **Mục tiêu:** Khắc phục 6 kẽ hở logic, triệt tiêu mã chết, bổ sung bảo vệ chống bẻ lái liên nến và chuyển đổi cơ chế phòng ngự sang Dynamic Sizing.  
> **Tài liệu tham chiếu gốc:**
> - [01_arbiter_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/03_arbiter/01_arbiter_deep_dive.md)

---

## 🎯 GIAI ĐOẠN 1: TRIỆT TIÊU MÃ CHẾT & CHỐNG BẪY BẺ LÁI LIÊN NẾN

### 1. [ ] `TODO-ARB-01`: Xây Dựng Inter-Bar Flip-Flop Guard & Dọn Dẹp Nhánh Deadlock Nến Đơn
- **Tài liệu chi tiết:** [§2 (Kẽ hở 2 & 3) trong 01_arbiter_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/03_arbiter/01_arbiter_deep_dive.md#2-6-kẽ-hở-logic-cốt-tử-đã-phát-hiện)
- **Vị trí code:** [`core_v3/arbiter.rs:437-482`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L437) & [`core_v3/mod.rs:30-40`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L30)
- **Nội dung:**
  - [ ] **Bước 1.1:** Dọn dẹp hoặc cấu trúc lại nhánh Deadlock nến đơn (vốn không thể xảy ra do CLV).
  - [ ] **Bước 1.2:** Bổ sung cơ chế chống bẻ lái liên nến (Directional Flip-Flop Memory): Khi vừa bị dừng lỗ ở chiều Short/Long trong $K=2$ nến gần nhất, Arbiter cấm mở vị thế chiều ngược lại ngay lập tức trừ khi có nến xung lực bứt phá cực đại $> 2.0 \text{ ATR}$.
- **Tiêu chí nghiệm thu:**
  - [ ] Test `test_arbiter_blocks_whipsaw_flip_flop`: Lệnh Short vừa dính SL ở nến $i-1$, nến $i$ xuất hiện tín hiệu Long thông thường $\to$ Arbiter từ chối duyệt để tránh bẫy whipsaw.

---

## 🛡️ GIAI ĐOẠN 2: CHUYỂN ĐỔI PHÒNG NGỰ SANG DYNAMIC RISK SIZING

### 2. [ ] `TODO-ARB-02`: Thay Thế Binary Sniper Veto Bằng Dynamic Position Sizing (De-leverage)
- **Tài liệu chi tiết:** [§2 (Kẽ hở 4) trong 01_arbiter_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/03_arbiter/01_arbiter_deep_dive.md#2-6-kẽ-hở-logic-cốt-tử-đã-phát-hiện)
- **Vị trí code:** [`core_v3/arbiter.rs:218-234`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L218)
- **Nội dung:**
  - [ ] **Bước 2.1:** Khi SL cấu trúc của Scanner vượt trần ROE thông thường (nhưng $\le 25\%$ ROE), không VETO vứt bỏ tín hiệu.
  - [ ] **Bước 2.2:** Gán cờ yêu cầu giảm Position Size (De-leverage): Tự động giảm tỷ lệ phân bổ vốn sao cho tổng số tiền đô-la chịu rủi ro ($Risk USD) không đổi.
- **Tiêu chí nghiệm thu:**
  - [ ] Test `test_arbiter_approves_wide_sl_with_reduced_size`: Proposal có SL 18% ROE được phê duyệt kèm khuyến nghị giảm size thay vì bị `StructuralRiskVeto`.

---

## 📊 GIAI ĐOẠN 3: RÀO CẢN MA SÁT SÀN & HIỆU CHUẨN ĐIỂM TỰ TIN

### 3. [ ] `TODO-ARB-03`: Bổ Sung Frictional & EV Hurdle Gate
- **Tài liệu chi tiết:** [§2 (Kẽ hở 5) trong 01_arbiter_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/03_arbiter/01_arbiter_deep_dive.md#2-6-kẽ-hở-logic-cốt-tử-đã-phát-hiện)
- **Vị trí code:** [`core_v3/arbiter.rs:235`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L235)
- **Nội dung:**
  - [ ] Tính toán rào cản chi phí ma sát: $\text{Friction Cost} = 2 \times (\text{Taker Fee} + \text{Slippage}) \times \text{Leverage} \approx 2.6\% \text{ ROE}$.
  - [ ] Bắt buộc tiềm năng biến động tối thiểu của lệnh phải $\ge 2.5 \times \text{Friction Cost}$ ($\ge 6.5\%$ ROE), từ chối các cơ hội scalp quá mỏng.
- **Tiêu chí nghiệm thu:**
  - [ ] Test `test_arbiter_rejects_sub_friction_opportunity`: Cơ hội có kỳ vọng $< 5\%$ ROE bị từ chối do không đủ bù đắp phí sàn.

### 4. [ ] `TODO-ARB-04`: Hiệu Chuẩn Phân Phối Điểm Số (Score Quantile Normalization)
- **Tài liệu chi tiết:** [§2 (Kẽ hở 1) trong 01_arbiter_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/03_arbiter/01_arbiter_deep_dive.md#2-6-kẽ-hở-logic-cốt-tử-đã-phát-hiện)
- **Vị trí code:** [`core_v3/arbiter.rs:149-155`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L149)
- **Nội dung:**
  - [ ] Chuẩn hóa điểm số thô của các scanner về cùng một thang đo xác suất thực tế (ví dụ Percentile mapping qua Rolling Window) trước khi so sánh trong Two-Stage Auction, ngăn chặn việc Momentum luôn cướp quyền đấu thầu.
