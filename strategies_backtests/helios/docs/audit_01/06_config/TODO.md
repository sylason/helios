# DANH MỤC CÔNG VIỆC CẦN LÀM: CẤU HÌNH DNA & MAGIC NUMBERS AUDIT (TODO.md)

Tài liệu này quản lý các hạng mục công việc, kiểm tra logic và ca kiểm thử (test cases) phát sinh từ báo cáo kiểm toán chuyên sâu:
👉 [Báo cáo Kiểm toán Chuyên sâu Cấu hình DNA & Magic Numbers (`01_config_and_hardcode_deep_dive.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [ ] **`TODO-CFG-01`**: **Triệt Tiêu Magic Numbers & Tham Số Hóa Toàn Diện Vào `config.rs`**
  - *Mục tiêu:* Xóa bỏ hoàn toàn các hằng số bị gán cứng trong các Scanners, Gatekeeper, Matrix và Risk Manager; chuyển chúng thành các trường cấu hình có giá trị mặc định rõ ràng.
  - *Tham chiếu:* [Mục 2: Bảng Tổng Kết Magic Numbers](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md#2-bảng-tổng-kết-magic-numbers-các-giá-trị-hardcode-trong-toàn-bộ-codebase)
  - *Nhiệm vụ:*
    - Trong `trend_pb.rs`: Bổ sung `min_clv` vào `TacticalPullbackParams` (thay thế hardcode `0.15`); thay thế trọng số `[1.0, 1.0, 1.0]` bằng `w_slope`, `w_dist`, `w_stoch`.
    - Trong `momentum_ignition.rs`: Đưa `target_breakout_atr` (thay thế hardcode `0.25`) và `rsi_excess_scale` (thay thế hardcode `15.0`) vào config.
    - Trong `gatekeeper.rs`: Đưa `hysteresis_lookback_bars` (thay thế hardcode `5`) vào `ArbiterConfig`.
    - Trong `matrix.rs`: Xóa bỏ độn sàn `0.35` của Chaos Index và thay thế `rolling_percentile(data, 20)` cứng bằng tham số lookback động.
  - *Test cases cần có:*
    - `test_trend_pb_respects_configured_min_clv`: Kiểm tra Trend_PB đọc đúng `min_clv` từ config thay vì 0.15.
    - `test_gatekeeper_hysteresis_lookback_configurable`: Kiểm tra Gatekeeper sử dụng đúng số bar lookback từ cấu hình.

---

- [ ] **`TODO-CFG-02`**: **Mở Rộng Độ Bao Quát Dải DNA Cho Mọi Asset (Forex Đến Memecoin)**
  - *Mục tiêu:* Đảm bảo các dải tìm kiếm $[Min, Max]$ của DNA bao quát được cả tài sản biến động siêu thấp (Forex $0.02\%$ NATR) lẫn tài sản biến động cực mạnh (Memecoins $1.0\%$ NATR).
  - *Tham chiếu:* [Mục 3.1: Nghịch Lý Sàn Biến Động `min_natr_threshold`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md#31-nghịch-lý-sàn-biến-động-min_natr_threshold-forex-vs-memecoins)
  - *Nhiệm vụ:*
    - Mở rộng dải DNA của `min_natr_threshold`: từ `[0.0008, 0.0030]` sang `[0.0001, 0.0100]` với `step = 0.0002` (50 nấc tìm kiếm).
    - Mở rộng trần `min_confidence_threshold` của Arbiter: từ `[0.05, 0.50]` sang `[0.05, 0.80]` với `step = 0.02` (38 nấc) để cho phép GA chọn chiến thuật siêu chắt lọc lệnh.
  - *Test cases cần có:*
    - `test_gatekeeper_permits_forex_low_volatility`: Kiểm tra với `min_natr_threshold = 0.0002`, thị trường EUR/USD ($NATR = 0.03\%$) vượt qua Gatekeeper thành công.
    - `test_gatekeeper_filters_memecoin_high_noise`: Kiểm tra với `min_natr_threshold = 0.0050`, các vùng nhiễu $0.4\%$ của Memecoin bị chặn lại.

---

- [ ] **`TODO-CFG-03`**: **Tái Cấu Trúc Thứ Bậc Dải Phân Tầng Trailing Stop (`tsl_atr_tight / normal / loose`)**
  - *Mục tiêu:* Xóa bỏ nghịch lý toán học khi GA chọn giá trị của phân tầng Tight lớn hơn Normal hoặc Loose.
  - *Tham chiếu:* [Mục 3.3: Nghịch Lý Dải DNA Của Bộ 3 Phân Tầng TSL](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md#33-nghịch-lý-dải-dna-của-bộ-3-phân-tầng-trailing-stop-tsl_atr_tight--normal--loose)
  - *Nhiệm vụ:*
    - Thiết lập 3 dải độc lập, tăng dần có trật tự:
      - `tsl_atr_tight`: `min = 0.40, max = 1.80, step = 0.05` (29 nấc).
      - `tsl_atr_normal`: `min = 1.20, max = 3.20, step = 0.10` (21 nấc).
      - `tsl_atr_loose`: `min = 2.50, max = 6.00, step = 0.15` (24 nấc).
    - Thêm ràng buộc thẩm định tự động (Validation Guard): Nếu `tight >= normal` hoặc `normal >= loose` $\to$ Tự động clamp có trật tự.
  - *Test cases cần có:*
    - `test_tsl_tiers_strictly_ordered`: Kiểm tra 3 dải TSL luôn đảm bảo $TSL_{\text{tight}} < TSL_{\text{normal}} < TSL_{\text{loose}}$.

---

- [ ] **`TODO-CFG-04`**: **Tổng Quát Hóa Kích Thước Lệnh Danh Nghĩa (`min_order_usd` Độc Lập Sàn)**
  - *Mục tiêu:* Loại bỏ việc trói cứng hệ thống vào con số \$21.0 của Binance Futures, hỗ trợ các sàn giao dịch và thị trường khác.
  - *Tham chiếu:* [Mục 3.4: Ràng Buộc Sàn Binance `min_order_usd: 21.0`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/06_config/01_config_and_hardcode_deep_dive.md#34-ràng-buộc-sàn-binance-min_order_usd-210)
  - *Nhiệm vụ:*
    - Cho phép nạp `min_notional` trực tiếp từ thông tin sàn (`MarketFilter` / `ExchangeInfo`) thay vì dựa vào hằng số cố định trong struct config.
  - *Test cases cần có:*
    - `test_calculate_size_adapts_to_external_min_notional`: Kiểm tra hàm `calculate_size` nhận đúng `min_notional = 5.0` (Bybit) hoặc `1.0` (OKX).
