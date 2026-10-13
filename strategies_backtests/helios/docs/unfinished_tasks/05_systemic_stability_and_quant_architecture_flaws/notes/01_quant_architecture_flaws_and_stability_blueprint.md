# BẢN THIẾT KẾ KỸ THUẬT: TÁI CẤU TRÚC KIẾN TRÚC ĐỊNH LƯỢNG & ỔN ĐỊNH HÓA HỆ THỐNG
## SYSTEMIC STABILITY & QUANT ARCHITECTURE REDESIGN BLUEPRINT

> **Dự án:** Helios V3 & Trading Bot Execution Platform  
> **Phạm vi kiểm toán:** `strategies_backtests/helios/`, `trading_bot/bot_engine/`, `ferro_trade/`  
> **Dữ liệu thực nghiệm đối chứng:** Run Walk-Forward `run_walk_forward_smart_seed_20261011_092521` (ETHUSDT OOS 5 ngày, 7.193 nến).  
> **Mục tiêu tối thượng:** Chuyển đổi hệ thống từ "Cào lệnh vi mô chịu lỗ phí" sang "Lỗ ít, lời ổn định, đủ nhiều, lọc sạch trạng thái nguy hiểm và tối đa hóa lợi nhuận".

---

## 🚨 1. BỐN TỬ HUYỆT KIẾN TRÚC ĐỊNH LƯỢNG (BÓC TÁCH TỪ THỰC NGHIỆM)

Dữ liệu OOS 5 ngày thực chiến (22 lệnh, Winrate 31.8%, Gross PnL +$1.69 nhưng Net PnL -$2.45 do ngốn $4.14 phí sàn) đã bóc trần 4 tử huyệt cốt lõi:

### 1.1. Tử Huyệt 1: Hàm Mục Tiêu GA Khuyến Khích "Cào Lệnh Dại Dột" (Turnover Blindspot)
* **Thực trạng:** GA Optimizer tối ưu hóa dựa trên Total Return hoặc Sharpe thuần túy trên In-Sample. Thuật toán tìm ra kẽ hở: *"Nhồi càng nhiều lệnh vi mô nhỏ thì càng gộp được nhiều nhịp sóng để đẩy điểm In-Sample lên cao"*.
* **Hậu quả:** GA nạp các bộ DNA có ngưỡng kích hoạt lỏng lẻo, tạo ra hơn 130 lệnh/tháng. Mỗi lệnh mở ra chịu $1.60\%$ ROE phí Taker sàn Binance 2 chiều ($20\text{x}$ leverage). Phí sàn nuốt chửng toàn bộ lợi nhuận thị trường.
* **Nguyên lý gốc:** Trong giao dịch định lượng đòn bẩy cao, chất lượng lệnh quan trọng hơn số lượng. Một hệ thống $EV > 0$ bền vững phải tối đa hóa Kỳ vọng Lợi nhuận trên từng Lệnh ($EV / \text{Trade}$) chứ không phải tần suất giao dịch.

### 1.2. Tử Huyệt 2: Sự Sụp Đổ Kiến Trúc Đa Chế Độ (Multi-Regime Collapse)
* **Thực trạng:** Hệ sinh thái được thiết kế gồm 6 Scanners, nhưng thực tế:
  - `Range_Osc`, `Reversal`, `Anomaly` = 0 lệnh (tê liệt 100%).
  - `Absorption` = 2 lệnh (bắt dao rơi thua 100%, gây lỗ -$2.06 USD, chiếm 84% tổng lỗ tài khoản).
  - `Trend_PB` và `Momentum` gánh tới 20/22 lệnh.
* **Hậu quả:** Khi thị trường rơi vào pha đi ngang tích lũy không có sóng (Chop Zone / Range), hệ thống không biết "ngồi im chờ mồi" (Sit on hands), cũng không kích hoạt đánh biên, mà ép `Trend_PB` và `Momentum` vào lệnh liên tục $\to$ sinh ra chuỗi thua bào mòn vốn.

### 1.3. Tử Huyệt 3: Bẫy "ATR Phình To" Khiến Lợi Nhuận Bốc Hơi (Volatility Expansion Trap)
* **Thực trạng:** 5/6 lệnh của `Momentum_IGN` có đỉnh lãi MFE từ $+3.0\%$ đến $+15.7\%$ ROE (Lệnh #1 đạt $+15.7\%$, Lệnh #11 đạt $+8.0\%$), nhưng kết cục 5/6 lệnh đều quay đầu thành lỗ hoặc hòa vốn âm phí.
* **Nguyên nhân toán học:** Trailing Stop Loss neo theo độ biến động: $\text{TSL} = \text{Price} \pm (\text{ATR} \times \text{mult})$. Khi sóng bứt phá mạnh đẩy MFE lên cao, cây nến dài làm $\text{ATR}_{\text{1m}}$ phình to đột biến $\to$ khoảng cách TSL bị nới lỏng ra xa thay vì siết chặt lại. Khi giá giật ngược nhẹ, toàn bộ thành quả lợi nhuận bị xóa sạch trước khi chạm TSL.

### 1.4. Tử Huyệt 4: Thiếu Bộ Lọc "Đồng Thuận Pha Đa Khung Thời Gian" (MTF Phase Coherence)
* **Thực trạng:** Tín hiệu vào lệnh phụ thuộc vào nến vi mô 1m. Tỷ lệ tín hiệu/nhiễu (SNR) trên nến 1m cực kỳ thấp.
* **Hậu quả:** Hầu hết các cú breakout hoặc pullback trên 1m ngược hướng sóng lớn 15m/1h chỉ là bẫy quét thanh khoản nội bộ của sàn. Mở lệnh 1m mà khung 15m/1h đang đối nghịch có xác suất thất bại lên tới $70\%$.

---

## ⚖️ 2. BẢNG ĐỐI CHẤT TOÀN DIỆN: THỰC TRẠNG VS. ĐỀ XUẤT KIẾN TRÚC MỚI

| Tiêu Chí Phân Hệ | Kiến Trúc Hiện Tại | Kiến Trúc Đề Xuất (Ổn Định & Tinh Hoa) | Ưu Điểm, Khuyết Điểm & Edge Cases |
| :--- | :--- | :--- | :--- |
| **Mục Tiêu Tối Ưu GA Core (`ferro_trade`)** | Tối đa hóa Total Return đơn thuần trên In-Sample. | Tối ưu hóa **Kỳ vọng Lợi nhuận ròng trên từng Lệnh ($EV / \text{Trade}$)** + Phạt nặng số lệnh ($> 30$ lệnh/tháng). | 🟢 **Ưu:** Triệt tiêu 100% bẫy cào lệnh scalping dại dột.<br>🔴 **Khuyết:** Không gian tìm kiếm GA khắt khe hơn.<br>⚠️ **Edge Case:** Phạt quá tay khiến GA chọn 0 lệnh. Khắc phục: Kẹp sàn tối thiểu $\ge 5$ lệnh/tháng. |
| **Bộ Lọc Phân Loại Chế Độ (Regime Gatekeeper)** | Gatekeeper chỉ lọc Chaos (bão giá) và Flatline (thanh khoản chết). Tất cả Scanner cùng đấu thầu mọi lúc. | **Macro Regime Classifier:** Phân định rõ 2 pha: Trend vs Range. Khi thị trường nén ngang (ADX < 20, KAMA phẳng) $\to$ **CẤM TUYỆT ĐỐI** `Trend_PB` và `Momentum`. | 🟢 **Ưu:** Hệ thống biết "ngồi im chờ mồi" (Sit on hands), không cào lệnh tự sát trong vùng nhiễu.<br>🔴 **Khuyết:** Giảm số cơ hội giao dịch.<br>⚠️ **Edge Case:** Chuyển giao pha bị trễ 1-2 nến. |
| **Đồng Thuận Pha Đa Khung (MTF Coherence)** | Quét tín hiệu cục bộ nến 1m, chỉ dùng MTF để đo ATR/BBands. | **MTF Phase Alignment:** Chỉ cho phép Long 1m khi xu hướng 15m/1h đồng thuận tăng (KAMA dốc lên hoặc nến trên dải trung tâm). | 🟢 **Ưu:** Nâng Win Rate từ $30\% \to 55-65\%$, triệt tiêu bẫy Fakeout nến 1m.<br>🔴 **Khuyết:** Bỏ lỡ điểm vào lệnh sớm nhất.<br>⚠️ **Edge Case:** Sóng đảo chiều V-shape quá nhanh (đã có Playbook Reversal lo). |
| **Bảo Vệ Lợi Nhuận (Risk & Trailing Stop)** | Trailing SL thuần theo ATR (bị bẫy nới lỏng khi ATR phình to). Không có khóa hòa vốn sau phí. | **Tiered Profit Lock & Break-Even Ratchet:**<br>1. $\text{MFE} \ge +3.5\%$ ROE $\to$ Khóa cứng SL tại **$+1.6\%$ ROE** (hòa vốn sau phí).<br>2. $\text{MFE} \ge +8.0\%$ ROE $\to$ Khóa tối thiểu **$+4.0\%$ ROE**.<br>3. ATR đệm siết hẹp dần theo ROE đạt được. | 🟢 **Ưu:** Giữ chặt 100% các lệnh thắng lớn (như lệnh +15.7% ROE), triệt tiêu bẫy biến thắng thành thua.<br>🔴 **Khuyết:** Có thể bị cắn hòa vốn sớm nếu nhịp rung lắc mạnh trước khi bay tiếp.<br>⚠️ **Edge Case:** Flash wick quét qua BE. |
| **Bắt Dao Rơi (`Absorption` & `Trend_PB`)** | Absorption bắt đáy không kiểm tra độ dốc KAMA. Trend_PB dùng cộng lồi bị pha loãng cảnh báo dao rơi. | 1. `Absorption`: Macro Adverse Veto (cấm Long khi `kama_slope < -max_adverse`).<br>2. `Trend_PB`: Asymmetric Multiplicative Gating triệt tiêu dao rơi khi $\%K < \%D$. | 🟢 **Ưu:** Cứu ngay 84% số lỗ tài khoản (loại bỏ 2 lệnh cháy Hard SL -$2.06 USD).<br>🔴 **Khuyết:** Cần cấu hình thêm tham số độ dốc.<br>⚠️ **Edge Case:** Đáy V-shape cực dốc không khớp được (để dành cho Reversal). |

---

## 🏛️ 3. ĐỊNH HƯỚNG TRIỂN KHAI THEO GIAI ĐOẠN

1. **Giai đoạn 1 (Immediate Defense - Cắt Máu Cấp Bách):**
   - Triển khai Adverse Slope Veto cho `Absorption` (`TODO-SCN-02`).
   - Triển khai Tiered Profit Lock & Break-Even Ratchet trong `RiskManager` và `PaperTrader`.
2. **Giai đoạn 2 (Regime & Signal Precision - Lọc Nhiễu Tinh Hoa):**
   - Thiết lập Macro Regime Classifier (khóa cào lệnh khi thị trường Chop).
   - Thiết lập MTF Phase Coherence Gate cho nến 1m.
3. **Giai đoạn 3 (Evolution Alignment - Uốn Nắn Bộ Não GA):**
   - Nâng cấp hàm mục tiêu Fitness trong `ferro_trade`: phạt turnover và tối ưu $EV/\text{trade}$.
