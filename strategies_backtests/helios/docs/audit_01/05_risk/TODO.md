# DANH MỤC CÔNG VIỆC CẦN LÀM: RISK MANAGER AUDIT (TODO.md)

Tài liệu này quản lý các hạng mục công việc, kiểm tra logic và ca kiểm thử (test cases) phát sinh từ báo cáo kiểm toán chuyên sâu:
👉 [Báo cáo Kiểm toán Chuyên sâu Risk Manager (`01_risk_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [x] **`TODO-RSK-01`**: **Bổ Sung Chốt Chặn Khoảng Cách Thanh Lý Sàn (Liquidation & MMR Guard)** - *(ĐÃ HOÀN THÀNH)*
  - *Mục tiêu:* Ngăn chặn tuyệt đối việc mở lệnh có Stop Loss nằm ngoài hoặc quá sát Giá Thanh lý ($P_{\text{liq}}$) khi giao dịch ở đòn bẩy cao ($\ge 50\text{x}$) hoặc trên Altcoins có tỷ lệ ký quỹ duy trì (MMR) lớn.
  - *Tham chiếu:* [Kẽ hở 1: MMR Blindness](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-1-rủi-ro-thanh-lý-ngầm-do-thiếu-nhận-thức-về-maintenance-margin-mmr-blindness) | [📄 Kế hoạch Cải tiến Nâng cao Tương lai (Simulated Liq trong Engine & Dynamic Auto-Deleverage)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/02_liquidation_and_auto_deleverage/01_liquidation_simulation_and_auto_deleverage_blueprint.md)
  - *Kết quả Triển khai:*
    - ✅ Bổ sung `maintenance_margin_rate` ($1.0\%$) & `liquidation_buffer_ratio` ($15\%$) vào `RiskBaseParams`.
    - ✅ Triển khai chốt chặn toán học an toàn tại `calculate_safe_sl_distance`, `validate_structural_sl` và `clamp_sl_price` trong `risk.rs`.
    - ✅ Bổ sung trạng thái `AuctionStatus::LiquidationRiskVeto` vào Signal Arbiter và hiển thị Thẻ Đỏ Telemetry `💀 [LIQ-VETO]`.
    - ✅ Vượt qua 100% 26/26 unit tests trong `risk.rs`: `test_clamp_sl_price_with_liquidation_guard`, `test_liquidation_guard_kamikaze_rejection_at_100x`, `test_liquidation_guard_safe_execution_at_20x`, `test_liquidation_guard_veto_high_mmr_altcoin`.
  - *Ghi chú Mở rộng Tương lai:* Đã lưu trữ tài liệu thiết kế và danh mục công việc cho các tính năng nâng cao (Simulated Liquidation trong Broker `ferro_trade` và Dynamic Auto-Deleverage khi vào lệnh) tại [`docs/unfinished_tasks/02_liquidation_and_auto_deleverage/`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/02_liquidation_and_auto_deleverage/TODO.md), sẽ quay lại triển khai khi tối ưu hóa sâu nền tảng.

---

- [x] **`TODO-RSK-02`**: **Tái Cấu Trúc Cơ Chế Thoát Lệnh Ngâm Vốn (Structural Invalidation Thay Cho Time Decay Mù)**
  - *Mục tiêu:* Loại bỏ hoàn toàn hiện tượng bot tự bóp nghẹt SL và cắt lỗ đúng ngay đáy tích lũy của nến 1m chỉ vì thời gian trôi qua.
  - *Tham chiếu:* [Kẽ hở 2: Underwater Tightening Trap](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-2-bẫy-underwater-tightening-bóp-chết-lệnh-ngay-đáy-premature-whipsaw-execution-trap)
  - *Thực hiện & Kết quả:*
    - ✅ Xóa bỏ triệt để khối siết SL lũy thừa $(0.8)^n$ khi vị thế đang âm ($current\_pnl \le 0.0$); bảo toàn 100% Initial Structural SL do Scanner thiết lập (đã có trần 15% ROE và MMR Liquidation Guard bảo vệ tuyệt đối).
    - ✅ Phân định ranh giới kiến trúc sạch (Clean Separation of Concerns): Nhiệm vụ giải phóng vốn ngâm lâu được độc quyền chuyển giao cho khối `Stagnant Capital Release` (`max_stagnant_hold_bars` $\ge 60$) và `Zombie Trade Purge` (48h), triệt tiêu hoàn toàn xung đột logic nội tại.
    - ✅ Đánh dấu `#[dna(skip)]` cho `base_sl_atr` trong `config.rs`, giảm không gian tìm kiếm DNA từ 103 về 102 chiều sạch sẽ.
    - ✅ Bổ sung 2 unit tests kiểm chứng: `test_underwater_position_preserves_initial_structural_sl` và `test_stagnant_capital_release_logic`. Toàn bộ 27/27 tests `risk.rs` và 104/104 tests crate `helios` đều PASS 100%.

---

- [ ] **`TODO-RSK-03`**: **Tối Ưu Hóa Ngưỡng Kích Hoạt Dynamic Break-Even Thích Ứng Biến Động**
  - *Mục tiêu:* Chống bẫy dời Break-Even quá sớm khi giá chỉ mới nhích $0.3\%$ (6% ROE tại 20x), ngăn chặn việc bị râu nến ngẫu nhiên quét hòa vốn làm mất sóng lớn.
  - *Tham chiếu:* [Kẽ hở 3: Premature BE Trap](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-3-bẫy-kích-hoạt-break-even-quá-sớm-the-03-price-jitter-be-trap)
  - *Nhiệm vụ:*
    - Neo ngưỡng kích hoạt BE vào độ biến động ATR thực tế của tài sản:
      $$\text{Trigger}_{\text{BE}} = \max(1.5 \times \text{ATR},\ 10.0\%\ \text{ROE})$$
    - Đảm bảo khoảng cách từ Entry đến điểm kích hoạt BE vượt ra ngoài vùng nhiễu tự nhiên của nến 1m.
  - *Test cases cần có:*
    - `test_break_even_not_triggered_by_single_wick_noise`: Kiểm tra râu nến 0.3% không làm kích hoạt dời SL lên BE.
    - `test_break_even_triggered_cleanly_on_real_momentum`: Kiểm tra khi giá bứt phá $> 1.5$ ATR thì BE được kích hoạt và khóa lãi an toàn.

---

- [ ] **`TODO-RSK-04`**: **Hoàn Thiện Cơ Chế Sizing Thích Ứng Vốn Nhỏ & Bảo Vệ Capital Fraction**
  - *Mục tiêu:* Đảm bảo tính toán Margin và kích thước vị thế không làm phình to tỷ lệ cược của tài khoản khi số dư nhỏ ($\le \$30$).
  - *Tham chiếu:* [Kẽ hở 4: Min Notional Bump](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-4-bẫy-min-notional-bump-làm-phình-to-rủi-ro-vốn-nhỏ-capital-bleed-trap)
  - *Nhiệm vụ:*
    - Trong `calculate_size`, nếu việc nâng lên `min_notional` (\$21) đòi hỏi margin vượt quá `max_capital_cap` (25% vốn) $\to$ Trả về `0.0` ngay lập tức, ghi log cảnh báo vốn không đủ an toàn để mở vị thế.
    - Tuyệt đối không cho phép lệnh mở với tỷ lệ margin $> 25\%$ vốn để tránh rủi ro sụt giảm dây chuyền.
  - *Test cases cần có:*
    - `test_calculate_size_rejects_overleveraged_small_equity`: Kiểm tra tài khoản \$15 tại đòn bẩy 5x (đòi hỏi \$4.2 margin = 28%) bị từ chối mở lệnh.
    - `test_calculate_size_permits_safe_small_equity_at_20x`: Kiểm tra tài khoản \$60 tại 20x mở lệnh chuẩn xác.

---

- [ ] **`TODO-RSK-05`**: **Xây Dựng Cơ Chế Xác Nhận Kép Đồng Bộ SL Giữa RAM và Cloud (Closed-Loop Sync)**
  - *Mục tiêu:* Xóa bỏ hoàn toàn rủi ro "vị thế mồ côi không có SL trên sàn" do cơ chế fire-and-forget và độ trễ 150ms khi hủy/đặt lại lệnh Algo.
  - *Tham chiếu:* [Kẽ hở 5: The Orphaned SL Gap](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-5-mất-đồng-bộ-phản-hồi-giữa-sl-trong-ram-và-cloud-binance-the-orphaned-sl-gap)
  - *Nhiệm vụ:*
    - Bổ sung kênh phản hồi (Ack/Nack channel) từ `ExecutionEngine` về `RiskManager`.
    - Nếu lệnh đặt SL mới trên Binance bị sàn Reject $\to$ Lập tức kích hoạt cơ chế khẩn cấp: Thử lại 3 lần với giá trigger an toàn hơn, hoặc nếu bất khả kháng thì đóng lệnh thị trường ngay lập tức để bảo vệ tài khoản.
  - *Test cases cần có:*
    - `test_cloud_sl_rejection_triggers_emergency_market_close`: Mô phỏng sàn Binance từ chối đặt SL mới $\to$ Bot tự động gửi lệnh đóng vị thế ngay lập tức.
