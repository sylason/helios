# Helios Core V3: Tổng Quan Kiểm Toán Kiến Trúc (Strategic Audit)

Chào mừng đến với báo cáo kiểm toán chi tiết của kiến trúc `core_v3`. Sau khi chuyển dịch từ mô hình Regime-First (V2) sang mô hình Signal-First với Arbiter (V3), chúng ta đã mở ra một tiềm năng lớn cho tính linh hoạt và cạnh tranh tín hiệu. 

Tuy nhiên, trong quá trình chuyển đổi, một số nguyên tắc thiết kế quan trọng đã bị vi phạm, gây ra các "lỗ hổng" logic nghiêm trọng (Regression). Các lỗ hổng này làm tê liệt Thuật toán Di truyền (Optimizer) và gây sai lệch lớn trong kết quả thực thi lệnh.

Tài liệu kiểm toán này được chia thành các thư mục chủ đề sau:

## 1. Kiến trúc Hệ thống & Ma trận Lõi (Architecture & Matrix)
- [Mất Cân Bằng Trọng Số Matrix](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3/01_architecture_and_matrix/01_matrix_multipliers_imbalance.md)
- [Lỗ Hổng của Cấu Trúc Proposal & Arbiter](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3/01_architecture_and_matrix/02_arbiter_and_proposal_flaws.md)

## 2. Hệ Thống Quét Tín Hiệu & DNA (Scanners & DNA)
- [Sự Ngắt Kết Nối DNA và Magic Numbers](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3/02_scanners_and_dna/01_magic_numbers_and_dna_disconnect.md)

## 3. Thực Thi & Quản Trị Rủi Ro (Execution & Risk)
- [Chặn Nút Thắt Trong Thực Thi Lệnh (Size & TP)](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3/03_execution_and_risk/01_execution_bottlenecks.md)
