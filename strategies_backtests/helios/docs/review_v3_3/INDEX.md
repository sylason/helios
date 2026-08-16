# HELIOS V3.3 ARCHITECTURE REVIEW: INDEX & SITEMAP

Tài liệu này tổng hợp toàn bộ kết quả rà soát, thảo luận và định hướng phát triển kiến trúc **Helios Core V3.3** trong thư mục `docs/review_v3_3`.

---

## 📂 SITEMAP THƯ MỤC `docs/review_v3_3`

1. **[`01_V3_Risk_Management_Architecture.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/01_V3_Risk_Management_Architecture.md)**
   - Hệ thống hóa 3 Vòng bảo vệ Risk Management.
   - Sơ đồ Mermaid luồng thực thi lệnh.
   - Cơ chế vị thế Exclusive 20% max equity cap.
   - Thực nghiệm v3_19 với Profit Factor = 15.36.

2. **[`02_Modules_Audit_File1_Mod_File2_Sensors_File3_Matrix.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/02_Modules_Audit_File1_Mod_File2_Sensors_File3_Matrix.md)**
   - Rà soát File #1 (`mod.rs`): Phát hiện nguyên nhân trade gaps và loại bỏ 100% `cooldown_duration_bars`.
   - Rà soát File #2 (`sensors.rs`): Phân tích đa ngành (Quant, Rust Dev, Trader, Math).
   - Rà soát File #3 (`matrix.rs`): Nâng cấp ADX Tanh Scale & Chuẩn hóa Volume MFI + Vol Z-Score (loại bỏ `vol_ratio` rác).
   - Danh sách biến dư thừa được tinh giảm cho File #1, #2, #3.

3. **[`03_Per_Playbook_Dedicated_Sensors_And_Regime_Router.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/03_Per_Playbook_Dedicated_Sensors_And_Regime_Router.md)**
   - Phát kiến Cảm biến Độc lập Per-Playbook (mỗi Playbook sở hữu bộ chỉ báo riêng).
   - Công tắc chuyển mạch bối cảnh Arbiter (Mutually Exclusive Regime Router) triệt tiêu 100% Playbook Bias.

4. **[`04_Arbiter_Auction_Audit_And_Kama_Slope_Upgrade.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/04_Arbiter_Auction_Audit_And_Kama_Slope_Upgrade.md)**
   - Rà soát File #4 (`arbiter.rs`): Quyết định nâng cấp KAMA Slope thay thế LinReg Slope.
   - Cơ chế Vùng Chuyển Mạch Trung Tính (Neutral Transition Zone).
   - Thiết kế 3 dải ngưỡng duyệt động `min_confidence` theo bối cảnh cho Arbiter Config.
   - Danh sách tinh giảm các biến dư thừa trong File #4.

5. **[`05_Trend_PB_Scanner_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/05_Trend_PB_Scanner_Audit.md)**
   - Rà soát File #5 (`scanners/trend_pb.rs`): Triết lý mua giá thấp trong UpTrend và bán giá cao trong DownTrend.
   - Tích hợp KAMA Slope và Cảm biến độc lập `stoch_trend`/`rsi_trend`.
   - Loại bỏ biến `stoch_vs_kama_weight` dư thừa.

6. **[`06_Momentum_Ignition_Scanner_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/06_Momentum_Ignition_Scanner_Audit.md)**
   - Rà soát File #6 (`scanners/momentum_ignition.rs`): Săn sóng bùng nổ động lượng nổ dải BB.
   - Loại bỏ `rsi_climax` gây Overfit và `vol_max_multiplier` rác.
   - Chuyển sang mốc Breakout Bollinger Band Upper (`close > bb_upper`) và Volume Z-Score.
   - Bảng tổng hợp các biến dư thừa trong File #6.

7. **[`07_Range_Oscillator_Scanner_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/07_Range_Oscillator_Scanner_Audit.md)**
   - Rà soát File #7 (`scanners/range_osc.rs`): Đánh đảo chiều hai biên Sideway.
   - Trang bị bộ cảm biến Stochastic độc lập `stoch_range`.
   - Loại bỏ biến `bb_vs_stoch_weight` và `range_bb_tolerance_pct` dư thừa.

8. **[`08_Reversal_Scanner_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/08_Reversal_Scanner_Audit.md)**
   - Rà soát File #8 (`scanners/reversal.rs`): Săn sóng kiệt sức Mean Reversion.
   - Mốc TP chốt lời cố định tại `kama_mid` nến mở lệnh ($P_0$).
   - Cơ chế thoát lệnh linh hoạt qua Progressive Trailing SL Decay khi giá không chạm TP.
   - Loại bỏ biến `z_vs_rsi_weight` dư thừa.

9. **[`09_Anomaly_Scanner_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/09_Anomaly_Scanner_Audit.md)**
   - Rà soát File #9 (`scanners/anomaly.rs`): Săn sóng bão tố quét thanh khoản Pinbar.
   - Đo râu nến theo bội số ATR: `upper_wick / current_atr >= wick_atr_threshold`.
   - Khắc phục bẫy soi nến $i$ đơn lẻ bằng chỉ số biến động `chaos_ratio` (NATR).
   - Trang bị bộ cảm biến RSI độc lập `rsi_anomaly`.
   - Loại bỏ các biến trọng số `pattern_vs_wick_weight` và `z_vs_rsi_weight` dư thừa.

10. **[`10_Master_Flaws_And_Solutions_Summary_Files_1_to_9.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/10_Master_Flaws_And_Solutions_Summary_Files_1_to_9.md)**
    - Bảng Tổng hợp Toàn diện Khuyết điểm, Bẫy Logic & Giải pháp Tinh giảm Biến dư thừa từ File #1 đến File #9.

11. **[`11_Absorption_Scanner_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/11_Absorption_Scanner_Audit.md)**
    - Rà soát File #10 (`scanners/absorption.rs`): Săn sóng gom hàng trước breakout.
    - Loại bỏ bẫy MACD trễ, thay bằng **`VWMA vs SMA`**.
    - Loại bỏ `vol_ratio` rác, thay bằng **Volume Z-Score**.
    - Khóa lãnh thổ trong Volatile Regime.

12. **[`12_Risk_Management_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/12_Risk_Management_Audit.md)**
    - Rà soát File #11 (`risk.rs`): Trọng tài Quản lý Rủi ro và Progressive Trailing SL.
    - Tự động siết trần SL ban đầu theo đòn bẩy ($\min(\text{playbook\_max\_sl\_pct}, 0.80 / \text{leverage})$).
    - Cơ chế Stagnant Capital Release thoát bẫy False Breakout khi ngâm nến $PNL \le 0$.
    - Bảo tồn 100% dải tham số động `time_countdown_bars` và `tighten_step_multiplier`.

13. **[`13_Config_And_DNA_Space_Audit.md`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/review_v3_3/13_Config_And_DNA_Space_Audit.md)**
    - Rà soát File #12 (`config.rs`): Cấu trúc dải tham số DNA V3.3 và Đánh giá Clean Code (Phương án B).
    - Bảng tổng hợp tất cả các biến đã tinh giảm trên toàn hệ thống.

---

## 🎯 TỔNG KẾT
Toàn bộ 12 Modules của Helios Core V3 đã được rà soát, phản biện và hệ thống hóa $100\%$ vào thư mục `docs/review_v3_3`!
