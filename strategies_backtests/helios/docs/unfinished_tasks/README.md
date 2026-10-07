# KHO LƯU TRỮ CÁC TÍNH NĂNG & Ý TƯỞNG BẢO LƯU TƯƠNG LAI (UNFINISHED TASKS & FUTURE ENHANCEMENTS)

> **Mục đích:** Nơi quy hoạch, lưu trữ tập trung các bản thiết kế kiến trúc, ý tưởng nâng cấp dài hạn và tính năng chưa hoàn thiện được tạm gác lại.  
> **Nguyên tắc quản trị:** Không làm loãng phạm vi (Scope Creep) của giai đoạn hiện tại (ưu tiên Forward Test & Paper Trading sinh lời ổn định). Khi hệ thống đạt đến giai đoạn trưởng thành tương ứng, các task trong thư mục này sẽ được kích hoạt trở lại và đưa vào chu kỳ phát triển.

---

## 🗺️ MỤC LỤC CÁC CHỦ ĐỀ BẢO LƯU

| STT | Chủ Đề & Thư Mục | Tài Liệu Thiết Kế (Blueprint) | Danh Mục Công Việc (TODO) | Thời Điểm Kích Hoạt (Trigger Condition) |
| :---: | :--- | :--- | :--- | :--- |
| **01** | **Dynamic Fee Context (Tiêm Phí Sàn Tự Động)** | [📄 Architecture Blueprint](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/01_dynamic_fee_context/01_dynamic_exchange_fee_context_blueprint.md) | [📋 TODO.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/01_dynamic_fee_context/TODO.md) | Khi mở rộng `trading_bot` sang nền tảng Multi-Exchange (nhiều sàn) hoặc Multi-Asset VIP Tier. |
| **02** | **Liquidation Simulation & Dynamic Auto-Deleverage** | [📄 Architecture Blueprint](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/02_liquidation_and_auto_deleverage/01_liquidation_simulation_and_auto_deleverage_blueprint.md) | [📋 TODO.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/02_liquidation_and_auto_deleverage/TODO.md) | Khi triển khai các chiến lược siêu đòn bẩy ($\ge 50\text{x}$) hoặc nâng cấp engine `ferro_trade` thành Crypto Futures Simulator chuyên nghiệp. |
| **03** | **Multi-Regime Harmonization & Adaptive Gatekeeper** | [📄 Architecture Blueprint](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/01_gatekeeper_and_multi_regime_harmonization_blueprint.md) | [📋 TODO.md](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/03_gatekeeper_and_multi_regime_harmonization/TODO.md) | Khi hoàn tất Forward Test và tiến hành tối ưu hóa chuyên sâu sự tương tác giữa 6 Scanners, Per-Playbook Adaptive Gating. |

---

## 🧭 NGUYÊN TẮC QUẢN TRỊ BẢO LƯU (DEFERRED WORK PRINCIPLES)
1. **Lý do bảo lưu minh bạch:** Mỗi chủ đề phải nêu rõ tại sao chưa làm ngay ở hiện tại (tránh Over-engineering, tránh phân tán nguồn lực).
2. **Giải pháp hiện tại đầy đủ:** Phải có giải pháp tạm thời (workaround/sweet spot) an toàn để hệ thống hiện tại chạy trơn tru mà không có lỗ hổng rủi ro.
3. **Kế hoạch hành động cụ thể:** File `TODO.md` trong từng thư mục phải có sẵn checklist và test cases rõ ràng, để khi quay lại làm không phải nghiên cứu lại từ đầu.
