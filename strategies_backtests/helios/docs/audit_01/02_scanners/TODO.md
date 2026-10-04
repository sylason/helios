# KẾ HOẠCH HÀNH ĐỘNG 4 CHIẾN ĐỘI SCANNER (SCANNERS TODO)

> **Mục tiêu:** Khắc phục toàn bộ các lỗ hổng vi cấu trúc, hình học Stop Loss và bẫy kiệt sức đã được phát hiện trong đợt kiểm toán chuyên sâu.  
> **Nguyên tắc:** Clean code, không lặp lại nội dung đã có trong docs, mỗi task chia nhỏ (Baby Steps) kèm tiêu chí kiểm thử (Acceptance Tests) độc lập.  
> **Tài liệu tham chiếu gốc:**
> - [01_momentum_ignition_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/01_momentum_ignition_deep_dive.md)
> - [02_reversal_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/02_reversal_deep_dive.md)
> - [03_trend_pb_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/03_trend_pb_deep_dive.md)
> - [04_absorption_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/04_absorption_deep_dive.md)

---

## 🎯 GIAI ĐOẠN 1: KHỬ BẪY HÌNH HỌC STOP LOSS & MỞ KHÓA TÍN HIỆU BỊ VETO

### 1. [ ] `TODO-SCN-01`: Khử KAMA Suction Trap & Nới Lỏng Nến Rút Chân (`Trend_PB`)
- **Tài liệu chi tiết:** [§4 trong 03_trend_pb_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/03_trend_pb_deep_dive.md#4-đề-xuất-cải-tiến-cốt-lõi-cho-trend_pb)
- **Vị trí code:** [`scanners/trend_pb.rs:112, 122-126, 159, 170-174`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L122)
- **Các bước triển khai:**
  - [ ] **Bước 1.1:** Sửa hình học SL Long: Bỏ `.min(kama_mid)`, chuyển thành `let sl = low - buffer;`.
  - [ ] **Bước 1.2:** Sửa hình học SL Short: Bỏ `.max(kama_mid)`, chuyển thành `let sl = high + buffer;`.
  - [ ] **Bước 1.3:** Nâng cấp bộ lọc nến xác nhận: Cho phép nến thân đỏ nếu lực rút chân mạnh (`if clv < 0.15 || (close < open && clv < 0.40) { return None; }`).
  - [ ] **Bước 1.4:** Áp dụng tương tự cho chiều Short (`if clv > -0.15 || (close > open && clv > -0.40) { return None; }`).
- **Tiêu chí nghiệm thu (Unit Tests):**
  - [ ] Test `test_trend_pb_sl_geometry_no_kama_suction`: Khi giá cao hơn KAMA 2 ATR, SL phải neo theo `low - buffer`, không bị kéo xuống KAMA.
  - [ ] Test `test_trend_pb_allows_red_hammer`: Nến thân đỏ nhẹ nhưng $CLV = 0.50$ vẫn được chấp thuận mở Long.

---

### 2. [ ] `TODO-SCN-02`: Khử BBands MTF Suction Trap & Giải Cứu Tần Suất (`Absorption`)
- **Tài liệu chi tiết:** [§4 trong 04_absorption_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/04_absorption_deep_dive.md#4-đề-xuất-cải-tiến-cốt-lõi-cho-absorption)
- **Vị trí code:** [`scanners/absorption.rs:95-98, 126-141`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L126)
- **Các bước triển khai:**
  - [ ] **Bước 2.1:** Sửa hình học SL: Bỏ `.min(bb_lower)` và `.max(bb_upper)`. Neo trực tiếp theo đáy/đỉnh nến hấp thụ: `low - buffer` (Long) và `high + buffer` (Short).
  - [ ] **Bước 2.2:** Nới lỏng điều kiện %B: Cho phép nến hấp thụ đóng ở nửa dưới dải nén (`pct_b <= max_bound.max(0.50)` cho Long và `pct_b >= (1.0 - max_bound).min(0.50)` cho Short).
  - [ ] **Bước 2.3:** Bỏ điều kiện cứng phụ thuộc `vwma < bb_middle` (giảm over-constrain).
- **Tiêu chí nghiệm thu (Unit Tests):**
  - [ ] Test `test_absorption_sl_tight_within_roe_cap`: Mức SL đề xuất luôn nằm trong phạm vi $\le 10\%$ ROE (đòn bẩy 20x).
  - [ ] Test `test_absorption_trigger_without_vwma_lock`: Nến nỗ lực gom volume lớn tại biên dưới kích hoạt thành công proposal.

---

## 🛡️ GIAI ĐOẠN 2: CHỐNG BẪY KIỆT SỨC CLIMAX & BẢO VỆ TỶ LỆ R:R

### 3. [ ] `TODO-SCN-03`: Bổ Sung Chốt Chặn Nén Trước Đó & Trần Climax (`Momentum_IGN`)
- **Tài liệu chi tiết:** [§3 trong 01_momentum_ignition_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/01_momentum_ignition_deep_dive.md#3-các-phương-án-đề-xuất-cải-tiến)
- **Vị trí code:** [`scanners/momentum_ignition.rs:51-57, 138-157`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L51) & [`core_v3/config.rs:480-522`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L480)
- **Các bước triển khai:**
  - [ ] **Bước 3.1:** Thêm cấu hình DNA `max_climax_z` (trần Z-Score) và `min_prior_squeeze` vào `TacticalMomentumParams`.
  - [ ] **Bước 3.2:** Bắt buộc kiểm tra độ nén ở nến trước đó: `matrix.squeeze_intensity[i.saturating_sub(1)] >= min_prior_squeeze` (hoặc dải BB hẹp).
  - [ ] **Bước 3.3:** Chốt chặn trần kiệt sức: VETO khi $|Price\_Z| > max\_climax\_z$ (chặn mua đỉnh nến thanh lý dây chuyền).
  - [ ] **Bước 3.4:** Kẹp trần khoảng cách SL: `sl = close - min(close - low + buffer, max_sl_dist)`.
- **Tiêu chí nghiệm thu (Unit Tests):**
  - [ ] Test `test_momentum_blocks_climax_high_z`: Khi $Z = +4.0$, scanner dứt khoát trả về `None`.
  - [ ] Test `test_momentum_requires_prior_squeeze`: Breakout trên thị trường không nén bị từ chối.

---

### 4. [ ] `TODO-SCN-04`: Thêm Cổng Kiểm Tra R:R & Macro Trend Alignment (`Reversal`)
- **Tài liệu chi tiết:** [§4 trong 02_reversal_deep_dive.md](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/02_scanners/02_reversal_deep_dive.md#4-đề-xuất-cải-tiến-cốt-lõi-cho-reversal)
- **Vị trí code:** [`scanners/reversal.rs:44, 80-96, 120-139`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L44) & [`core_v3/config.rs:582-630`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L582)
- **Các bước triển khai:**
  - [ ] **Bước 4.1:** Kích hoạt tham số `matrix`: Đọc `matrix.kama_slope_atr`.
  - [ ] **Bước 4.2:** Bổ sung Macro Adverse Trend Veto: Cấm Long khi `kama_slope_atr < -max_adverse_slope`, cấm Short khi `kama_slope_atr > max_adverse_slope`.
  - [ ] **Bước 4.3:** Thêm cấu hình DNA `min_expected_rr` (mặc định 1.20, dải 1.0 - 2.5).
  - [ ] **Bước 4.4:** Bổ sung Hard Gate R:R: Tính $\text{Expected\_RR} = |\text{close} - \text{kama\_mid}| / |\text{close} - \text{suggested\_sl}|$. Nếu $< min\_expected\_rr \implies$ `return None`.
- **Tiêu chí nghiệm thu (Unit Tests):**
  - [ ] Test `test_reversal_veto_adverse_trend`: Dốc KAMA âm mạnh $\to$ Cấm bắt dao rơi Long.
  - [ ] Test `test_reversal_rejects_poor_rr`: Khoảng cách tới KAMA quá hẹp so với đệm SL $\to$ Loại bỏ vì R:R $< 1.2$.
