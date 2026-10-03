# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU: TREND PULLBACK SCANNER (TREND_PB)
> **Thư mục:** `strategies_backtests/helios/docs/audit_01/08_trend_pb/`  
> **Ngày hoàn thành:** 03/10/2026  
> **Tác giả:** Antigravity Quant Team & 3 Sub-Agents Chuyên Trách  
> **Phương pháp:** Bóc tách 100% từng dòng mã nguồn, First-Principles Microstructure & Mathematical Proof.

---

## 📑 HỆ THỐNG TÀI LIỆU KIỂM TOÁN
1. 🔬 [`01_mathematical_logic.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/08_trend_pb/01_mathematical_logic.md): Kiểm toán bản chất toán học, vi cấu trúc nến, bẫy Pinbar rút chân, KAMA suction trap và mất tính định hướng trong chấm điểm khoảng cách.
2. 📊 [`02_dna_config_space.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/08_trend_pb/02_dna_config_space.md): Kiểm toán 10 tham số của `TacticalPullbackParams`, phân tích 2 bẫy GA Overfitting (VETO 100% và Cào lệnh sideway) và đề xuất dải DNA chuẩn Quant.
3. 🛡️ [`03_systemic_risk_interaction.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/08_trend_pb/03_systemic_risk_interaction.md): Kiểm toán dòng dữ liệu đầu vào (Z-score MTF trap), độ lệch phổ điểm Arbiter (bị Momentum đè bẹp) và Hybrid TTP bóp nghẹt siêu sóng.
4. 📋 [`TODO.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/08_trend_pb/TODO.md): Sổ tay 10 đầu việc cải tổ được phân loại theo mức độ ưu tiên từ Critical đến Medium kèm test cases.

---

## 🎯 TỔNG HỢP CÁC PHÁT HIỆN CỐT TỬ CẦN ĐẠI PHẪU

| Hạng mục | Dị tật phát hiện | Hệ lụy thực tế | Giải pháp khắc phục |
| :--- | :--- | :--- | :--- |
| **Stop Loss Logic** | `low.min(kama_mid)` kéo SL lùi sâu về KAMA MTF cách 3.0 - 4.5 ATR | SL vượt trần 15% ROE $\implies$ **Arbiter VETO 100% lệnh** | Bỏ `.min(kama_mid)`, neo SL trực tiếp dưới đáy nến nhịp hồi `low - buffer` |
| **Candle Confirmation** | Cấm nến $Close < Open$ | **Bóp chết nến Pinbar/Hammer rút chân kinh điển** ($CLV \ge 0.80$) chỉ vì thân đỏ 0.1 pip | Cho phép nến đỏ nếu $CLV \ge 0.50$ (có lực hấp thụ rút chân mạnh) |
| **Distance Scoring** | Dùng trị tuyệt đối `delta_kama.abs()` | Thủng KAMA -0.4 ATR (sắp gãy trend) vẫn nhận điểm cao ngang giữ vững hỗ trợ | Áp dụng hệ số phạt khi $\text{delta\_kama} < 0$ |
| **Oscillator Cooldown** | Khuyến khích Stoch về 0 | Bắt dao rơi giữa thác đổ thanh lý (Liquidation cascade) | Yêu cầu Stochastic có tín hiệu ngóc đầu đảo chiều $\text{stoch\_k} \ge \text{stoch\_k}[i-1]$ |
| **MTF Z-Score Trap** | Z-score đọc từ khung lớn 1h (thường $> 1.0\sigma$) trong khi `max_boundary_z` bị kẹp $\le 0.80$ | Khóa chết 90%+ tín hiệu Trend_PB khi thị trường vào siêu sóng | Nới rộng dải DNA của `max_boundary_z` lên $[0.50, 2.00]$ |
| **Arbiter Competition** | `raw_score` bị kẹp trong dải 0.55 - 0.70 | **Luôn bị Momentum (0.85 - 0.98) đè bẹp** tại Arbiter | Đưa lực nến $CLV$ vào tổ hợp lồi, nâng phổ điểm lên $0.75 - 0.88$ |
| **Hybrid TTP** | `ttp_callback_atr` mặc định 0.80 ATR quá hẹp | Lệnh bị đá văng ngay nhịp thở đầu tiên, không nuôi được Mega Trend | Bổ sung `ttp_callback_atr_trend` riêng cho Trend ($1.50 \sim 3.00$ ATR) |
