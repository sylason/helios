# DANH MỤC CÔNG VIỆC CẦN LÀM: RISK MANAGER AUDIT (TODO.md)

Tài liệu này quản lý các hạng mục công việc, kiểm tra logic và ca kiểm thử (test cases) phát sinh từ báo cáo kiểm toán chuyên sâu:
👉 [Báo cáo Kiểm toán Chuyên sâu Risk Manager (`01_risk_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [ ] **`TODO-RSK-01`**: **Bổ Sung Chốt Chặn Khoảng Cách Thanh Lý Sàn (Liquidation & MMR Guard)**
  - *Mục tiêu:* Ngăn chặn tuyệt đối việc mở lệnh có Stop Loss nằm ngoài hoặc quá sát Giá Thanh lý ($P_{\text{liq}}$) khi giao dịch ở đòn bẩy cao ($\ge 50\text{x}$) hoặc trên Altcoins có tỷ lệ ký quỹ duy trì (MMR) lớn.
  - *Tham chiếu:* [Kẽ hở 1: MMR Blindness](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-1-rủi-ro-thanh-lý-ngầm-do-thiếu-nhận-thức-về-maintenance-margin-mmr-blindness)
  - *Nhiệm vụ:*
    - Nạp bảng thông số MMR từ sàn vào cấu hình rủi ro của cặp tiền.
    - Trong `clamp_sl_price` và `is_structural_sl_acceptable`, thêm điều kiện:
      $$\text{Dist}(\text{Entry}, \text{SL}) \le \text{Dist}(\text{Entry}, P_{\text{liq}}) - \Delta_{\text{MMR\_buffer}}$$
    - Nếu đòn bẩy làm khoảng cách thanh lý nhỏ hơn SL đề xuất $\to$ Tự động de-leverage hoặc từ chối vào lệnh an toàn.
  - *Test cases cần có:*
    - `test_liquidation_distance_rejection_at_100x`: Kiểm tra từ chối lệnh khi SL 15% ROE vượt qua giá thanh lý của Altcoin có MMR 1.5%.
    - `test_auto_de_leverage_to_fit_liquidation_safe_zone`: Kiểm tra tự động hạ đòn bẩy từ 50x xuống 30x để đảm bảo vùng đệm thanh lý $\ge 1.0\%$.

---

- [ ] **`TODO-RSK-02`**: **Tái Cấu Trúc Cơ Chế Thoát Lệnh Ngâm Vốn (Structural Invalidation Thay Cho Time Decay Mù)**
  - *Mục tiêu:* Loại bỏ hoàn toàn hiện tượng bot tự bóp nghẹt SL và cắt lỗ đúng ngay đáy tích lũy của nến 1m chỉ vì thời gian trôi qua.
  - *Tham chiếu:* [Kẽ hở 2: Underwater Tightening Trap](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/05_risk/01_risk_deep_dive.md#kẽ-hở-2-bẫy-underwater-tightening-bóp-chết-lệnh-ngay-đáy-premature-whipsaw-execution-trap)
  - *Nhiệm vụ:*
    - Thay thế công thức `underwater_mult = base_sl_atr * (0.8)^n` bằng cơ chế kiểm tra **Phá vỡ Cấu trúc Thị trường (Structural Invalidation)**:
      - Chỉ dời SL khi giá tạo Lower High / Lower Low rõ ràng hoặc đường trung bình KAMA đảo chiều ngược lại hướng lệnh.
      - Giữ nguyên mức SL cấu trúc ban đầu nếu giá vẫn dao động trong biên độ tích lũy an toàn.
  - *Test cases cần có:*
    - `test_underwater_position_holding_during_healthy_consolidation`: Kiểm tra lệnh không bị force-close khi ngâm 20 bars nhưng giá vẫn tôn trọng swing low.
    - `test_underwater_exit_on_kama_slope_inversion`: Kiểm tra lệnh chỉ bị thoát sớm khi KAMA dốc ngược chiều lệnh.

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
