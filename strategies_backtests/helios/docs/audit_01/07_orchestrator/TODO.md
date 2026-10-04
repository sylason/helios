# DANH MỤC CÔNG VIỆC CẦN LÀM: BỘ ĐIỀU PHỐI CHIẾN LƯỢC TRUNG TÂM (TODO.md)

Tài liệu này quản lý các hạng mục công việc, kiểm tra logic và ca kiểm thử (test cases) phát sinh từ báo cáo kiểm toán chuyên sâu:
👉 [Báo cáo Kiểm toán Chuyên sâu Bộ Điều Phối Chiến Lược Trung Tâm (`01_strategy_mod_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/07_orchestrator/01_strategy_mod_deep_dive.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [ ] **`TODO-MOD-01`**: **Dọn Sạch Code Cooldown Thừa Trong `mod.rs` & Khẳng Định Quyền Sở Hữu Của Bot Engine**
  - *Mục tiêu:* Tách bạch 100% ranh giới trách nhiệm: Strategy chỉ thuần túy sinh tín hiệu Alpha. Cầu Dao Cooldown thuộc về quyền sở hữu độc quyền của Bot Engine (`AssetActor`) nơi quản lý tài khoản và PnL thực tế của sàn.
  - *Tham chiếu:* [Mục 2 - Vấn đề 1: Làm rõ kiến trúc Cooldown](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/07_orchestrator/01_strategy_mod_deep_dive.md#vấn-đề-1-làm-rõ-kiến-trúc-cooldown--dọn-sạch-code-thừa)
  - *Nhiệm vụ:*
    - Xóa bỏ khối `CooldownTracker` và logic theo dõi PnL thừa trong `HeliosStrategyV3`.
    - Bảo toàn và củng cố Cầu Dao Cooldown tại `trading_bot/bot_engine/src/controller/asset_actor.rs`.
  - *Test cases cần có:*
    - `test_strategy_runs_cleanly_without_local_cooldown`: Kiểm tra Strategy vận hành nhẹ nhàng, không lưu trạng thái thừa.

---

- [ ] **`TODO-MOD-02`**: **Thanh Lọc Toàn Diện Các Chỉ Báo Zombie Khỏi Hàm `init()`**
  - *Mục tiêu:* Loại bỏ các phép tính và cấp phát bộ nhớ không cần thiết trong hàm `init()` để giải phóng RAM và tăng tốc GA Optimizer.
  - *Tham chiếu:* [Mục 2 - Lỗ hổng 2: Lãng phí tính toán chỉ báo Zombie](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/07_orchestrator/01_strategy_mod_deep_dive.md#lỗ-hổng-2-sự-lãng-phí-tính-toán-của-chỉ-báo-zombie-trong-init)
  - *Nhiệm vụ:*
    - Xóa bỏ vòng lặp tính `bb_middle_slope_adj` và các chỉ báo ma không scanner nào sử dụng (`trend_strength`, `bb_width_norm`).
  - *Test cases cần có:*
    - `test_strategy_init_zero_unused_allocations`: Kiểm tra hàm `init()` hoàn tất thành công mà không cấp phát các mảng dữ liệu thừa.

---

- [ ] **`TODO-MOD-03`**: **Hợp Nhất Logic Kẹp Stop Loss & Triệt Tiêu Magic Number `0.15` (Đồng Bộ `risk.rs` & `config.rs`)**
  - *Mục tiêu:* Tránh việc gọi lặp lại 2 lần hàm `clamp_sl_price` và loại bỏ hoàn toàn con số gán cứng `0.15` trong `mod.rs`. Đưa toàn bộ định nghĩa trần ROE về `config.rs` làm Single Source of Truth (SSOT).
  - *Tham chiếu:* 
    - [Mục 2 - Lỗ hổng 3: Magic number 0.15 & gọi kép](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/07_orchestrator/01_strategy_mod_deep_dive.md#lỗ-hổng-3-magic-number-015--gọi-kép-kẹp-stop-loss-liên-kết-riskrs--configrs)
    - [Báo cáo Risk Manager (`05_risk/01_risk_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md)
    - [Báo cáo Config (`06_config/01_config_and_hardcode_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md)
  - *Nhiệm vụ:*
    - Cải tiến `RiskManager::clamp_sl_price` để trả về một struct `ClampedSlResult { sl_price: f64, is_hard_clamped: bool, is_playbook_clamped: bool }`.
    - Lấy trần ROE từ `config.risk_management.stop_management.default_sl_roe_cap`.
  - *Test cases cần có:*
    - `test_single_call_clamp_sl_returns_accurate_telemetry`: Kiểm tra một lần gọi duy nhất trả về chính xác cả mức SL và trạng thái kẹp.

---

- [ ] **`TODO-MOD-04`**: **Tường Minh Hóa Kiểu Dữ Liệu Khối Lượng Vị Thế (Type-Safe Position Sizing)**
  - *Mục tiêu:* Ngăn ngừa rủi ro nhầm lẫn giữa tỷ lệ vốn (% equity), số lượng coin và lot size khi mở rộng sang các sàn ngoài Binance Futures.
  - *Tham chiếu:* [Mục 2 - Lỗ hổng 4: Type-Safety cho Position Sizing](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/07_orchestrator/01_strategy_mod_deep_dive.md#lỗ-hổng-4-type-safety-cho-position-sizing-trong-ferro_trade--helios)
  - *Nhiệm vụ:*
    - Phối hợp bổ sung Type Wrapper bên phía `ferro_trade` (ví dụ `OrderSize::CapitalFraction(f64)`).
    - Cập nhật `HeliosStrategyV3` sử dụng kiểu dữ liệu Type-Safe này thay cho số `f64` trần trụi.
  - *Test cases cần có:*
    - `test_order_size_fraction_conversion_correctness`: Kiểm tra tính toán chuyển đổi từ % vốn sang số lượng hợp đồng chuẩn xác theo giá nến.
