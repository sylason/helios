# Thư Mục Khảo Sát & Nghiên Cứu Lỗi Hệ Thống: 05_systemic_stability_and_quant_architecture_flaws

> [!NOTE]
> **TRẠNG THÁI THƯ MỤC: KHẢO SÁT & ĐỐI CHẤT HỌC THUẬT (UNVERIFIED HYPOTHESES & LAB NOTES)**  
> Thư mục này phục vụ việc tập hợp mọi phát hiện, giả thuyết và phản biện kiến trúc trước khi bắt tay vào triển khai thực tế. Mọi tài liệu phân tích chi tiết được phân tách vật lý vào thư mục con `notes/`.

---

## 🗂️ Cấu Trúc Thư Mục

- [`ROADMAP.md`](./ROADMAP.md): Kế hoạch tổng thể và lộ trình dứt điểm các lỗi kiến trúc.
- [`TODO.md`](./TODO.md): Danh sách các đầu việc chi tiết và trạng thái kiểm toán.
- **Thư mục con `notes/` (Tài liệu nghiên cứu & phân tích chuyên sâu):**
  - [`01_quant_architecture_flaws_and_stability_blueprint.md`](./notes/01_quant_architecture_flaws_and_stability_blueprint.md): 4 tử huyệt định lượng ban đầu (Overfitting, Split-brain, Monotonicity, Cooldown).
  - [`02_helios_and_trading_bot_architectural_flaws.md`](./notes/02_helios_and_trading_bot_architectural_flaws.md): 5 điểm yếu giao thoa giữa chiến lược Helios và nền tảng TradingBot.
  - [`03_full_system_codebase_audit_flaws.md`](./notes/03_full_system_codebase_audit_flaws.md): Tổng hợp 42 phát hiện kiểm toán từ 3 sub-agents chuyên sâu.
  - [`04_subagents_audit_detailed_tables.md`](./notes/04_subagents_audit_detailed_tables.md): Bảng biểu chi tiết nguyên văn từ 3 sub-agents kiểm toán.
  - [`05_scanner_independence_and_regime_auction_architecture.md`](./notes/05_scanner_independence_and_regime_auction_architecture.md): Phân tích sự sụp đổ của triết lý cấm trùng indicator & thiết kế sàn đấu thầu 3 tầng.
  - [`06_sprint_plan_and_3tier_architecture_notes.md`](./notes/06_sprint_plan_and_3tier_architecture_notes.md): Kế hoạch tác chiến 7 ngày dứt điểm đưa bot vào demo & bản thiết kế 3 tầng.
  - [`07_micro_vs_macro_timeframe_quant_standards.md`](./notes/07_micro_vs_macro_timeframe_quant_standards.md): Tiêu chuẩn định lượng và nguyên lý vật lý phân chia khung thời gian Vi mô vs Vĩ mô.
  - [`08_forward_fill_matrix_flaws_and_architectural_sanity_check.md`](./notes/08_forward_fill_matrix_flaws_and_architectural_sanity_check.md): Mổ xẻ dị tật forward-fill trong matrix, nguyên nhân thất bại của Arbiter và giải pháp triệt tiêu xung đột đa khung thời gian.
  - [`09_dynamic_timeframe_agnostic_architecture_and_deep_sanity_audit.md`](./notes/09_dynamic_timeframe_agnostic_architecture_and_deep_sanity_audit.md): Thiết kế độc lập khung thời gian (Timeframe-Agnostic) và thẩm định toàn diện tính tối ưu, nguy cơ Over/Under-engineering và xung đột logic.
  - [`10_ga_fitness_expectancy_alignment.md`](./notes/10_ga_fitness_expectancy_alignment.md): Tái cấu trúc hàm mục tiêu Fitness GA, đồng trục kỳ vọng toán học (EV Alignment) và triệt tiêu 4 tử huyệt lệch tâm.
  - [`11_binance_fee_friction_and_fitness_refinement.md`](./notes/11_binance_fee_friction_and_fitness_refinement.md): Chi phí ma sát Binance Futures thực tế & Tái cấu trúc hàm Fitness GA tinh gọn theo nguyên lý Occam's Razor.

