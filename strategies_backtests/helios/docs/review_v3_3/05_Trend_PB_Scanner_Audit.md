# HELIOS V3.3 ARCHITECTURE REVIEW: PART 5 - TREND PULLBACK SCANNER AUDIT

Tài liệu này tổng hợp kết quả rà soát chi tiết **FILE #5: [`core_v3/scanners/trend_pb.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs)**.

---

## 🏛️ 1. MỤC ĐÍCH & TRIẾT LÝ GIAO DỊCH CỦA PLAYBOOK `TREND_PB`

### Triết Lý Giao Dịch:
- **Nguyên tắc**: Trong một xu hướng TĂNG (Bullish Trend), tuyệt đối **KHÔNG MUA Ở ĐỈNH (No Breakout Chasing)**. Kiên nhẫn chờ giá giật hồi (Pullback) về vùng giá thấp gần đường đệm `KAMA Mid` hoặc khi Stochastic nhún xuống vùng Oversold ($K < 30$), sau đó mới mở lệnh **MUA HỒI GIÁ (PULLBACK BUY)**.
- **Lợi ích**: Bắt được điểm xoay chiều ở mức giá thấp nhất của sóng hồi, tối ưu hóa điểm dừng lỗ (Stop Loss) cực hẹp, tạo nên tỷ lệ Risk:Reward (R:R) vượt trội.

---

## 🔬 2. KẾT QUẢ RÀ SOÁT VÀ CÁC THAY ĐỔI THIẾT KẾ CHO `TREND_PB`

### A. Sử Dụng Cảm Biến Độc Lập Per-Playbook:
- Thay vì dùng chung `sensors.stoch_k` và `sensors.rsi` với các Playbook khác, `TrendPBScanner` được trang bị riêng:
  - `stoch_trend_period` & `stoch_trend_tf`.
  - `rsi_trend_period` & `rsi_trend_tf`.

### B. Sử Dụng KAMA Slope Làm Thước Đo Độ Dốc Xu Hướng:
- Thay thế `LinReg Slope` bằng `KAMA Slope`.
- Khi thị trường sideway, `KAMA Slope` $\approx 0.00$ tự động vô hiệu hóa `TREND_PB`, tránh bẫy mở lệnh sóng hồi giả khi không có trend.

### C. Loải Bỏ Các Biến Dư Thừa Làm Phình Dải DNA (Redundant Parameter Pruning):
- **Loại bỏ `stoch_vs_kama_weight`**: Phép cộng trọng số gượng ép giữa Stochastic và khoảng cách KAMA làm kéo dài dải DNA và gây nhiễu cho GA Optimizer. 
- Khi dùng KAMA Slope, ta chỉ cần điều kiện tinh gọn: `stoch_k > stoch_d` ở vùng Oversold ($K < 30$) khi `KAMA Slope > +0.10`.

### D. Cơ Chế Kích Hoạt Lệnh Hồi Giá Tối Ưu (Phương Án A):
- **Điều kiện Mua (Long)**: `KAMA Slope > +0.10` AND `stoch_k > stoch_d` **KHI CẢ HAI ĐANG NẰM Ở VÙNG OVERSOLD ($K < 30$)**.
- **Điều kiện Bán (Short)**: `KAMA Slope < -0.10` AND `stoch_k < stoch_d` **KHI CẢ HAI ĐANG NẰM Ở VÙNG OVERBOUGHT ($K > 70$)**.
- **Thoát Lệnh (Exit Strategy)**: KHÔNG đặt Take Profit cố định (`suggested_tp = None`). Để vị thế thả rông cho Progressive Trailing SL tự động siết theo giá và chốt lời Siêu Trend!
