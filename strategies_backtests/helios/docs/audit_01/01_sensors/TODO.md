# DANH MỤC CÔNG VIỆC CẦN LÀM: CẢM BIẾN & DÒNG DỮ LIỆU MTF (TODO.md)

Tài liệu này quản lý các hạng mục công việc, kiểm tra logic và ca kiểm thử (test cases) phát sinh từ báo cáo kiểm toán chuyên sâu:
👉 [Báo cáo Kiểm toán Dòng Dữ liệu Cảm biến (`01_data_lineage_verification.md`)](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/01_data_lineage_verification.md)

---

## 📋 DANH SÁCH HẠNG MỤC CÔNG VIỆC (ACTION ITEMS)

- [x] **`TODO-SEN-01`**: **Tách Bạch Khung Thời Gian Volume Z-Score Khỏi KAMA (Khử Ô Nhiễm MTF 15 Bars)**
  - *Mục tiêu:* Ngăn chặn hiện tượng một cây nến 15m bùng nổ volume làm lây nhiễm giá trị Z-score cao chót vót giả tạo sang suốt 15 cây nến 1m tiếp theo, gây bẫy kích hoạt sai cho Momentum và Reversal.
  - *Tham chiếu:* [Mục 2.1: Cảm biến Volume Z-Score bị neo cứng theo KAMA](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/01_data_lineage_verification.md#21-cảm-biến-volume-z-score-bị-neo-cứng-theo-kama)
  - *Nhiệm vụ:*
    - Trong `config.rs`, bổ sung `VolZScoreParams` với `lookback: usize` [20, 600, 20] và `timeframe: Timeframe` khóa trong vi mô `#[dna(subset = "1m, 3m, 5m")]`.
    - Trong `mod.rs:250`, gọi `ctx.z_score_volume(ind.vol_z_score.timeframe, "VolZScore", ind.vol_z_score.lookback)` thay vì dùng ké `ind.kama.timeframe`.
  - *Test cases cần có:*
    - `test_volume_zscore_responds_immediately_to_single_1m_candle`: Kiểm tra volume nến 1m hạ nhiệt thì Vol Z-score phản ứng ngay lập tức thay vì giữ nguyên mức cao suốt 15 bars.

---

- [x] **`TODO-SEN-02`**: **Tách Bạch Timeframe Bollinger Bands Đo SL Khỏi Khung Vĩ Mô MTF Trong Absorption**
  - *Mục tiêu:* Xóa bỏ bẫy BBands MTF Suction Trap trong Absorption Scanner, chấm dứt việc lấy biên dải 1h/4h kéo Stop Loss xa tới 34% ROE khiến 100% lệnh bị Sniper Guard VETO.
  - *Tham chiếu:* [Mục 2.2: Dải Bollinger Bands MTF làm méo mó hình học Stop Loss](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/01_data_lineage_verification.md#22-dải-bollinger-bands-mtf-làm-méo-mó-hình-học-stop-loss)
  - *Nhiệm vụ:*
    - Phân tách rõ ràng: BBands MTF (15m, 1h, 4h) chỉ dùng đo độ nén vĩ mô (`squeeze_intensity`) và vị trí %B trong vùng nén.
    - Trong `scanners/absorption.rs:125-142`, xóa bỏ hoàn toàn `low.min(bb_lower)` và `high.max(bb_upper)`. Điểm neo SL chuẩn xác theo Wyckoff Spring/Upthrust là `low - buffer` (Long) và `high + buffer` (Short), với `buffer = current_atr * struct_sl_buffer_atr`.
  - *Test cases cần có:*
    - `test_absorption_sl_anchors_to_local_1m_structure`: Kiểm tra Stop Loss của Absorption không bị kéo giãn theo dải BB 4h, giữ khoảng cách trong dải an toàn $\le 10\%$ ROE.

---

- [x] **`TODO-SEN-03`**: **Chuẩn Hóa Đơn Vị & Tính Nhất Quán Của Cặp Chỉ Báo Biến Động ATR vs NATR**
  - *Mục tiêu:* Khắc phục độ lệch 2.2 lần giữa `NATR` và `ATR_SL` do GA ghép 2 khung thời gian khác nhau, đảm bảo các phép chuẩn hóa phi thứ nguyên và bộ lọc Gatekeeper có sự nhất quán về quy mô giá trị. Chấm dứt hiện tượng Cầu Dao Flatline Guard bị tê liệt khi so sánh ngưỡng vi mô ($0.12\%$) với biến động vĩ mô ($1\text{h}/4\text{h}$ luôn $> 1.0\%$).
  - *Tham chiếu:* [Mục 1: Giải mã độ lệch NATR 0.89% vs ATR/Price 0.41%](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/01_data_lineage_verification.md#1-giải-mã-độ-lệch-natr-089-vs-atrprice-041-lệch-22-lần) & [Blueprint Phương Án A Phí Sàn Tương Lai](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/03_dynamic_exchange_fee_context_blueprint.md)
  - *Nhiệm vụ Đã Hoàn Thành:*
    - Khóa `NAtrParams.timeframe` trong miền vĩ mô `#[dna(subset = "15m, 1h, 4h")]` (loại bỏ $5\text{m}$) để `s.natr` phản ánh trung thực bão biến động vĩ mô phục vụ tính toán `chaos_index`.
    - Tách dứt điểm logic Flatline Guard trong `gatekeeper.rs`: Tính toán biến động vi mô thực tế `micro_natr_ratio = (current_atr_sl / current_price).max(0.0)`.
    - Thiết lập **Sàn Chi Phí Sinh Tồn (Physical Cost Floor)** $= 2 \times \text{taker\_fee\_rate} + \text{slippage\_allowance\_pct}$ (tận dụng 2 trường Non-DNA có sẵn trong `RiskBaseParams`) kết hợp với **DNA Net Alpha** (`min_natr_threshold`).
    - Cập nhật Gatekeeper VETO: Nếu `micro_natr_ratio < (roundtrip_cost + min_natr_threshold)` $\to$ Kích hoạt `GateVetoReason::VolatilityFlatline`.
    - Cập nhật Telemetry Cards: Phân tách rõ `NATR_Micro (Flatline)` và `NATR_Macro (Chaos)` để tránh gây hiểu nhầm trong log.
    - Lập tài liệu kiến trúc bảo lưu Phương án A (Engine-injected fee qua `InitContext`/`TickContext` + WebSocket sàn) vào `03_dynamic_exchange_fee_context_blueprint.md` để triển khai sau khi hoàn tất Paper Trade.
  - *Test cases đã có & PASS:*
    - `test_gatekeeper_volatility_flatline_veto`: Kiểm tra Flatline Guard kích hoạt VETO khi `micro_natr < roundtrip_cost + min_natr_threshold`.
    - `test_gatekeeper_immune_to_macro_natr_pollution`: Kiểm tra Gatekeeper Flatline Guard miễn nhiễm hoàn toàn với `s.natr` vĩ mô.
    - `test_micro_sensors_timeframe_subset`: Kiểm tra `NAtrParams` chỉ nằm trong tập `["15m", "1h", "4h"]`.

---

- [x] **`TODO-SEN-04`**: **Tách Bạch Timeframe Price Z-Score Khỏi KAMA Vĩ Mô (Khử Đóng Băng Độ Lệch Chuẩn 15-60 Bars)**
  - *Mục tiêu:* Giải phóng `s.z_score` (Price Z-Score) khỏi việc bị neo cứng vào `ind.kama.timeframe` (15m, 1h, 4h). Ngăn chặn tình trạng độ lệch chuẩn giá bị forward-fill giữ nguyên giá trị suốt 15-60 bars, làm mù Boundary Guard của `Trend_PB` và bộ lọc bắt đỉnh đáy của `Reversal`.
  - *Tham chiếu:* [Mục 1.2: Price Z-Score bị trói cứng theo KAMA](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/02_micro_vs_macro_separation.md#🔴-12-sz_score-price-z-score---độ-lệch-chuẩn-giá)
  - *Nhiệm vụ:*
    - Trong `config.rs`, bổ sung `PriceZScoreParams` với `lookback: usize` [20, 600, 20] và `timeframe: Timeframe` khóa trong vi mô `#[dna(subset = "1m, 3m, 5m")]`.
    - Trong `mod.rs:249`, gọi `ctx.z_score(ind.price_z_score.timeframe, "ZScore", ind.price_z_score.lookback)` thay vì dùng ké `ind.kama.timeframe`.
  - *Test cases cần có:*
    - `test_price_zscore_updates_every_1m_bar`: Kiểm tra giá nến 1m biến động thì Price Z-score cập nhật động ngay lập tức tại mỗi nến 1m, không bị giữ nguyên giá trị qua nhiều nến liên tiếp.

---

- [x] **`TODO-SEN-05`**: **Khóa Dải Timeframe ATR_SL Trong Miền Vi Mô [1m, 3m, 5m] (Chống Bẫy Kéo Giãn SL 25-35% ROE)**
  - *Mục tiêu:* Chặn đứng việc GA Optimizer chọn khung thời gian 1h hoặc 4h cho `s.atr_sl` trong `config.rs`, khiến khoảng cách Stop Loss bị phình to theo ATR nến 1h ($35 trên ETH) thay vì nến vào lệnh 1m ($3), dẫn đến 100% lệnh bị Sniper Guard VETO vì vượt trần ROE.
  - *Tham chiếu:* [Mục 1.3: ATR dùng làm đệm Stop Loss](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/02_micro_vs_macro_separation.md#🔴-13-satr_sl-atr-dùng-làm-đệm-stop-loss)
  - *Nhiệm vụ:*
    - Trong `config.rs:193`, sửa macro `AtrParams.timeframe` từ `subset = "5m, 15m, 1h, 4h"` thành miền vi mô `subset = "1m, 3m, 5m"`.
    - Đảm bảo ranh giới: `ATR_SL` chỉ phục vụ đo biên độ nến vi mô cho Stop Loss / Trailing Stop, không dùng cho phân tích xu hướng vĩ mô.
  - *Test cases cần có:*
    - `test_atr_sl_constrained_to_micro_timeframe`: Kiểm tra `s.atr_sl` luôn nằm trong phạm vi biến động nến 1m-5m, khoảng cách SL ban đầu của mọi lệnh luôn $\le 10\%$ ROE ở đòn bẩy 20x.

---

- [ ] **`TODO-SEN-06`**: **Quy Hoạch Ranh Giới 3 Cặp Phạm Trù Vi Mô vs Vĩ Mô Giữa Các Cảm Biến Cốt Lõi**
  - *Mục tiêu:* Thiết lập chuẩn mực ranh giới kiến trúc bất biến giữa 3 cặp phạm trù: (1) Biến động vĩ mô (NATR 1h/4h) vs Biến động vi mô (ATR 1m/5m), (2) Xu hướng vĩ mô (KAMA 1h/4h) vs Bóp cò vi mô (CLV 1m), (3) Độ nén vĩ mô (Macro Squeeze 15m/1h) vs Điểm nổ vi mô (Vol Z-score 1m).
  - *Tham chiếu:* [Mục 2: Ba Cặp Phạm Trù Vi Mô vs Vĩ Mô Cốt Tử Cần Tách Bạch](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/01_sensors/02_micro_vs_macro_separation.md#🌐-2-ba-cặp-phạm-trù-vi-mô-vs-vĩ-mô-cốt-tử-cần-tách-bạch)
  - *Nhiệm vụ:*
    - Xây dựng tài liệu chuẩn mực và rà soát toàn bộ các cảm biến trong `sensors.rs` và `matrix.rs` để đảm bảo tuân thủ nguyên tắc "Đúng người - Đúng việc - Đúng miền không gian".
    - Bổ sung assert hoặc comment cảnh báo kiến trúc trong code để ngăn chặn việc tái ô nhiễm trong tương lai.
  - *Test cases cần có:*
    - `test_architectural_domain_separation`: Kiểm thử tích hợp xác nhận không có scanner nào sử dụng chỉ báo vĩ mô làm đệm SL hoặc chỉ báo vi mô làm bộ lọc xu hướng dài hạn.
