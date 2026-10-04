# MỔ XẺ CHUYÊN SÂU SIGNAL ARBITER & REGIME GATEKEEPER (ARBITER V3.3)

> **Ngày thực hiện:** 04/10/2026  
> **Phạm vi kiểm toán:** `core_v3/arbiter.rs`, `core_v3/gatekeeper.rs`, `core_v3/mod.rs:395-500`.  
> **Mục tiêu:** Kiểm toán toàn diện logic nghiệp vụ, tính đúng đắn toán học, các điểm mù và kẽ hở kiến trúc trong tầng phân xử tín hiệu tối cao của Helios V3.

---

## 1. HIỆN TRẠNG KIẾN TRÚC & QUY TRÌNH RA QUYẾT ĐỊNH

Signal Arbiter là trọng tài tối cao giải quyết xung đột và điều phối lệnh từ 4 Tactical Scanners, vận hành qua chu trình 3 bước:
1. **Tầng 1 (Regime Gatekeeper):** Thẩm định điều kiện vi mô qua `RegimeGatekeeper::evaluate`. Nếu thị trường rơi vào bão (`chaos_index >= threshold` hoặc dải trễ Hysteresis) hoặc biến động chết (`NATR < min_natr`), Arbiter thoát sớm $O(1)$ (`is_approved: false`).
2. **Tầng 2 (Sniper Entry Guard & Proposal Collection):** Quét qua các Scanner, tính điểm tự tin `compute_confidence(raw_score)`. Kiểm tra khoảng cách SL đề xuất qua `RiskManager::is_structural_sl_acceptable`. Nếu SL vượt trần ROE của playbook, loại bỏ với trạng thái `StructuralRiskVeto`.
3. **Tầng 3 (Two-Stage Auction):**
   - *Intra-Side Auction:* Chọn ra Long có điểm tự tin cao nhất và Short có điểm tự tin cao nhất.
   - *Inter-Side Face-Off:* So sánh chênh lệch giữa Long và Short qua `conflict_confidence_margin`. Nếu $|L - S| \le margin \implies$ Triệt tiêu cả hai (Flat).

---

## 2. 6 KẼ HỞ LOGIC CỐT TỬ ĐÃ PHÁT HIỆN

| # | Kẽ hở logic | Dòng code | Bản chất & Hậu quả vi cấu trúc |
|:-:|:---|:---|:---|
| **1** | **Điểm Tự Tin Không Cùng Thước Đo (Uncalibrated Scores)** | [`arbiter.rs:149-155`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L149) | `raw_score` được gán trực tiếp làm `confidence`. Các scanner dùng các hàm phi tuyến khác nhau: Momentum dùng `sat_unbounded` trên Volume nên điểm thường đạt 0.95–0.99; Absorption/Reversal tính điểm thận trọng chỉ đạt 0.30–0.60. Arbiter so sánh 2 đại lượng không cùng phân phối xác suất, khiến Momentum luôn áp đảo phi lý. |
| **2** | **Cơ Chế Phân Xử Xung Đột Nến Đơn Là "Mã Chết" (Deadlock Illusion)** | [`arbiter.rs:468-478`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L468) | Cả 4 scanner đều bắt buộc dấu CLV khớp với chiều lệnh (Long cần $CLV > 0$, Short cần $CLV < 0$). Trên 1 nến $i$, CLV chỉ có 1 giá trị duy nhất $\implies$ **Không bao giờ có Long và Short cùng xuất hiện trong 1 nến**. Toàn bộ nhánh hóa giải xung đột giằng co (`|L - S| <= margin -> Flat`) là mã chết 100%. |
| **3** | **Bất Lực Trước Xung Đột Đảo Chiều Liên Nến (Inter-Bar Flip-Flop Whipsaw)** | [`arbiter.rs:157-192`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L157) | Xung đột thực sự không nằm trên 1 nến tĩnh mà diễn ra giữa nến $i-1$ và nến $i$. Arbiter là hàm phi trạng thái (Stateless): Vừa dính SL lệnh Short ở nến $i-1$, nến $i$ xanh giật ngược lại duyệt ngay Long $\implies$ Bot bị lật mặt liên tục, cào nát vốn trong vùng Whipsaw khi Gatekeeper chưa kịp nhảy vào bão. |
| **4** | **Phủ Quyết Nhị Phân Cực Đoan (All-or-Nothing Sniper Veto)** | [`arbiter.rs:225-234`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L225) | Khi SL đề xuất vượt trần ROE (ví dụ 11.5% > 11.0%), Sniper Guard vứt bỏ 100% proposal. Trong khi đó, `RiskManager` lại có sẵn hàm `clamp_sl_price` và cơ chế giảm position size. Hàng chục cơ hội gom hàng của Absorption và Pullback (Raw Score 0.96) bị giết chết vô lý chỉ vì râu nến dài hơn 0.5% ROE. |
| **5** | **Tù Mù Trước Chi Phí Sàn & Trượt Giá (Frictional Blindness)** | [`arbiter.rs:235`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L235) | Arbiter duyệt lệnh chỉ dựa vào `confidence >= min_threshold`. Với đòn bẩy 20x, phí sàn 2 chiều + trượt giá ngốn $\approx \mathbf{2.6\% \text{ ROE}}$. Arbiter không có cơ chế kiểm tra biên độ kỳ vọng so với chi phí ma sát, dẫn đến duyệt các lệnh scalp mỏng khiến tài khoản bị bào mòn vì phí. |
| **6** | **Điểm Mù Dư Chấn Sau Bão (Post-Storm Residual Shock)** | [`gatekeeper.rs:143-168`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs#L143) | Hysteresis Schmitt Trigger chỉ quét ngược $K=5$ bars. Ngay khi nến thứ 6 hạ nhiệt, Gatekeeper mở khóa cửa ngay lập tức khi sổ lệnh (Orderbook) còn rất mỏng, thanh khoản rỗng. Arbiter duyệt lệnh ngay nến này khiến vị thế bị trượt giá (Slippage) cực nặng. |

---

## 3. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành phần Arbiter | Ưu điểm | Khuyết điểm | Edge Cases nguy hiểm |
| :--- | :--- | :--- | :--- |
| **Cầu Dao Gatekeeper O(1)** | Tiết kiệm CPU, bảo vệ hệ thống tuyệt đối khi thị trường rơi vào bão Whipsaw hoặc mất thanh khoản. | Hysteresis 5 bars phản ứng trễ với các cú sập nến đơn (Single-bar Flash Crash). | Flash Crash 1 nến: Nến sập 3% trong 30 giây, chaos chưa kịp tăng $\to$ Gatekeeper cho PASS, Momentum Short đuổi đáy ngay trước cú giật chữ V. |
| **Two-Stage Auction** | Độc lập, giao hoán (Commutative), đảm bảo thứ tự thêm scanner không làm đổi kết quả. | Giả định sai về sự tồn tại của xung đột Long vs Short nến đơn. Không giải quyết được xung đột liên nến. | Hai scanner cùng chiều (ví dụ Trend_PB và Momentum) cùng báo Long: Scanner có công thức điểm lỏng lẻo hơn sẽ luôn thắng dù chất lượng setup kém hơn. |
| **Sniper Entry Guard** | Ngăn chặn các vị thế có SL cấu trúc quá xa gây cháy tài khoản. | Thiết kế nhị phân (Binary All-or-Nothing): Từ chối hoàn toàn thay vì điều chỉnh quy mô vị thế. | Thị trường biến động giãn nở (Volatility Expansion): SL cấu trúc tự nhiên rộng ra $\to$ 100% lệnh bị VETO, bot đứng ngoài nhìn sóng lớn chạy mất. |
