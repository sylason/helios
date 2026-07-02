# 05. Sniper Thực thi (Tactical Execution)

Tài liệu phản ánh file `src/strategy/logic/tactical.rs`.

Đây là nơi Đội SWAT đưa ra quyết định cuối cùng, kết hợp bối cảnh vĩ mô (Playbook) và tín hiệu Vi mô (Ngòi nổ).

## 1. Các Ngòi Nổ (Triggers)
Code hiện tại có 7 Đội SWAT tương ứng với 7 logic bóp cò khác nhau. Mọi đội đều dùng chung một bộ quản lý Rủi ro (Stoploss / Take Profit).
- **Squeeze:** Đòi hỏi Dải BB phải đang nở rộng (hôm nay rộng hơn hôm qua) + Volume xác nhận + MACD thuận chiều.
- **TrendPullback:** Đòi hỏi giá hồi về đường KAMA + Stochastic chạm vùng Quá bán/Quá mua + MACD có dấu hiệu bẻ góc (Pivot).
- **RangeOscillator:** Đánh Ping-pong chạm cản trên/dưới của BB + Stochastic kiệt sức + Volume yếu.
- **ClimaxReversal / AnomalyFade:** Bắt đỉnh bắt đáy khi RSI cực đoan + Z-Score bất thường + MACD sụp đổ.

### Tính năng Emergency Override (Lật Vị Thế)
Riêng 2 đội `Climax` và `Anomaly` có quyền lực Tối cao: Nếu Điểm số của họ `> 0.98`, họ được quyền Đóng ép (Force Close) lệnh hiện tại dù nó đang lãi hay lỗ, để đảo chiều ngay lập tức vì phát hiện cú sập hầm (Flash crash).

## 2. Quản lý Vốn & Rủi ro (SL/TP)
**Tính năng SL Co giãn (Elastic Trailing Stop):** Code trong `risk.rs` quy định:
- Khi đánh Squeeze/Anomaly: Stoploss nới rộng (`sl_factor` lớn).
- Khi đánh Trend: Stoploss bình thường.
- Dời SL (Trailing): Chỉ dời theo chiều có lợi. Khi lãi đạt R:R nhất định, tự dời SL về mức Hòa vốn (Breakeven).

> [!CAUTION] VẤN ĐỀ NGUY HIỂM VỀ QUẢN LÝ VỐN VÀ CODE SMELL
> 1. **Size Cố định (Fixed Fraction):** Lệnh hiện đang fix đánh 20% vốn (với tài khoản < 100$). Ở đòn bẩy 100x, lệnh này mang danh nghĩa x20 lần tài khoản, chỉ chịu nổi cú giật 5%.
> 2. **Code lặp (Violates DRY):** Khối lượng code tính toán SL = ATR * Factor, TP = SL * RR được copy-paste y hệt nhau 14 lần ở 7 Playbook. Cần Refactor thành một hàm `calculate_risk_params()`.
