# Helios V3: 5 Trụ Cột Trạng Thái Thị Trường & Dòng Dữ Liệu Tiêu Thụ (Market Pillars & Data Lineage)

---

## 🏛️ 1. Bản Chất Nghiệp Vụ & Nền Tảng Định Lượng

Trong giao dịch thuật toán hiện đại, thị trường tài chính không phải là một chuỗi ngẫu nhiên đơn thuần, mà là một hệ động lực phức tạp liên tục dịch chuyển qua nhiều trạng thái (Regimes). Để máy tính có thể phân loại và phản ứng khách quan mà không phụ thuộc vào cảm tính con người, **Helios V3 chia tách thị trường thành 5 Không Gian Đo Lường Vật Lý Độc Lập**:

1. **Thanh Khoản & Khối Lượng (Liquidity & Volume):** Thước đo *Nỗ Lực (Effort)* của dòng tiền lớn (Smart Money).
2. **Xu Hướng (Trend):** Thước đo *Chiều vector dịch chuyển* bền vững của giá.
3. **Biến Động (Volatility):** Thước đo *Năng lượng tiềm năng và độ dãn biên độ*.
4. **Động Lượng (Momentum & Price Location):** Thước đo *Vận tốc và vị trí nến tức thời*.
5. **Hỗn Loạn & Nhiễu (Chaos & Noise):** Thước đo *Mức độ bất định và rủi ro giật quét (Whipsaw)*.

> [!IMPORTANT]
> **LÀM RÕ VỀ THANH KHOẢN TRONG HỆ THỐNG:**
> Hệ thống **hoàn toàn KHÔNG loại bỏ thanh khoản**. Trái lại, thanh khoản là trụ cột cốt tử để phát hiện dòng tiền tổ chức.
> - **Cắt bỏ `matrix.liquidity`:** Đây là một chỉ báo phái sinh lỗi thời, áp dụng sai lầm hàm `rolling_percentile` lên chỉ số `MFI`, gây ra ảo giác thanh khoản giả mạo và không được bất kỳ module nào tiêu thụ.
> - **Giữ vững 4 thước đo thanh khoản thực sự:** `sensors.vol_z_score`, `sensors.vol_sma`, `sensors.vwma`, và `sensors.mfi` trực tiếp.

---

## 📊 2. Bảng Thống Kê Toàn Diện 5 Trụ Cột & Ánh Xạ Tiêu Thụ (Playbooks & Modules)

| Trụ Cột | Chỉ Số Cụ Thể (Sensors / Matrix) | Nơi Tiêu Thụ (Module & File) | Bản Chất Toán Học & Logic Nghiệp Vụ Ra Quyết Định |
| :--- | :--- | :--- | :--- |
| **1. THANH KHOẢN & KHỐI LƯỢNG (Liquidity & Volume)** | **`sensors.vol_z_score`** *(Z-Score khối lượng so với SMA 20)* | - **Absorption** ([`absorption.rs:47`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L47))<br>- **Momentum Ign** ([`momentum_ignition.rs:62`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L62))<br>- **Range Osc** ([`range_osc.rs:55`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L55)) | - **Absorption:** Phát hiện cá mập gom/xả hàng đột biến ($Z_{\text{vol}} > \text{vol\_z\_threshold}$).<br>- **Momentum:** Xác nhận bứt phá thực sự có dòng tiền lớn hậu thuẫn ($Z_{\text{vol}} > 1.5\sigma$).<br>- **Range:** Bộ lọc an toàn cấm vào lệnh khi khối lượng bùng nổ ($Z_{\text{vol}} > 2.0\sigma$ sẽ xé toạc dải Range). |
| | **`sensors.vol_sma`** & **`tick.volumes`** | - **Absorption** ([`absorption.rs:69`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L69))<br>- **Reversal** ([`reversal.rs:104`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L104)) | - **Absorption:** Mô hình Wyckoff VSA (*Effort vs Result*): Khối lượng bùng nổ nhưng biên độ nến hẹp $\implies$ Sự hấp thụ thanh khoản đỉnh cao.<br>- **Reversal:** Đo kiệt sức khối lượng ($\text{Vol} / \text{SMA} < 0.5$) để xác nhận phe đẩy giá đã cạn kiệt năng lượng. |
| | **`sensors.vwma`** *(Đường trung bình gia quyền khối lượng)* | - **Absorption** ([`absorption.rs:95-97`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L95-L97)) | - Định vị trọng tâm dòng tiền thực tế: Nếu Long Accumulation thì bắt buộc $\text{VWMA} < \text{BB Middle}$ (dòng tiền lớn đang gom ở vùng giá chiết khấu thấp). |
| | **`sensors.mfi`** *(Money Flow Index $[0, 100]$)* | - **Reversal** ([`reversal.rs:56-96`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L56-L96)) | - Bắt đáy/đỉnh kiệt sức dòng tiền: Long khi $MFI \le 30$ (kiệt cung bán tháo); Short khi $MFI \ge 70$ (quá tải lực mua). |
| **2. XU HƯỚNG (Trend)** | **`matrix.kama_slope_atr`** *(Độ dốc KAMA chuẩn hóa theo ATR)* | - **Trend PB** ([`trend_pb.rs:47`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L47))<br>- **Absorption** ([`absorption.rs:107`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L107))<br>- **Momentum Ign** ([`momentum_ignition.rs:86`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L86))<br>- **Range Osc** ([`range_osc.rs:122`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L122)) | - **Trend PB:** Điều kiện tiên quyết để kích hoạt pullback (độ dốc phải đủ dốc vượt ngưỡng `slope_thresh`).<br>- **Absorption & Momentum:** Cầu dao Macro Veto chống bắt dao rơi: Nếu KAMA dốc ngược chiều quá mạnh $\implies$ Cấm mở lệnh cản tàu.<br>- **Range:** Cấm mở lệnh đánh Range khi KAMA đang nghiêng dốc. |
| | **`matrix.trend_strength`** *(ADX tuyến tính $[0.0, 1.0]$)* | - **Range Osc** ([`range_osc.rs:69`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L69)) | - Bộ lọc Quant chống bẫy xu hướng: Chỉ cho phép dao động Range khi $ADX / 100 < \text{max\_trend}$ (bảo đảm thị trường không có xu hướng). |
| | **`matrix.bb_middle_slope_adj`** *(Độ dốc trục giữa SMA 20)* | - **Range Osc** ([`range_osc.rs:74`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L74)) | - Đảm bảo dải đi ngang phẳng lì, không bị uốn lượn theo sóng xu hướng. |
| | **`sensors.kama_mid`** | - **Trend PB** ([`trend_pb.rs:67`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L67))<br>- **Reversal** ([`reversal.rs:65`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L65)) | - Làm mốc quy chiếu hồi quy (Displacement / Pullback distance). |
| **3. BIẾN ĐỘNG (Volatility)** | **`matrix.squeeze_intensity`** *(Tỷ lệ nén Bollinger Bands nằm gọn trong Keltner)* | - **Absorption** ([`absorption.rs:48`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/absorption.rs#L48))<br>- **Risk Manager** ([`risk.rs:323`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L323)) | - **Absorption:** Gom hàng trong vùng nén chặt ($Squeeze > \text{threshold}$).<br>- **Risk Manager:** Khi thị trường nén mạnh, nới rộng ngưỡng Breakeven để tránh bị quét râu nến trước khi bung nổ. |
| | **`matrix.bb_width_norm`** *(Độ rộng dải Bollinger Bands kẹp trần $[0, 1]$)* | - **Range Osc** ([`range_osc.rs:53`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs#L53)) | - Đảm bảo biên độ dao động trong hộp không quá hẹp (đủ R:R tối thiểu để bù đắp phí sàn). |
| | **`sensors.atr_sl`** *(Average True Range)* | - **Toàn bộ 6 Scanners**<br>- **Risk Manager** ([`risk.rs:97`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L97)) | - Chuẩn hóa mọi khoảng cách giá theo biến động thực tế.<br>- Tính khoảng cách Stop Loss ban đầu và Trailing Stop Loss đám mây động. |
| **4. ĐỘNG LƯỢNG (Momentum & Price Location)** | **`sensors.z_score`** *(Độ lệch chuẩn giá so với trung bình)* | - **Reversal** ([`reversal.rs:48`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L48))<br>- **Anomaly** | - Đo độ dãn giá cực hạn để kích hoạt đánh đảo chiều hồi quy (Mean Reversion) khi giá lệch $> 2.0\sigma$. |
| | **`CLV (Close Location Value)`** *($[-1.0, 1.0]$)* | - **Toàn bộ Scanners** (`absorption`, `momentum`, `reversal`, `trend_pb`) | - Đọc vị lực mua/bán nến ngay tại thời điểm đóng: $CLV > 0.5$ (rút chân gom/mua mạnh), $CLV < -0.5$ (đè đầu xả/bán mạnh). |
| | **`sensors.rsi_momentum`** & **`rsi_trend`** | - **Momentum Ign** ([`momentum_ignition.rs:60`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/momentum_ignition.rs#L60))<br>- **Trend PB** ([`trend_pb.rs:50`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L50)) | - Xác nhận xung lực bứt phá ($RSI > 60$) hoặc trạng thái điều chỉnh lành mạnh. |
| | **`sensors.stoch_trend_k/d`** | - **Trend PB** & **Range Osc** | - Đo điểm rơi nhịp hồi (Pullback timing) và điểm chạm biên dao động Range. |
| **5. HỖN LOẠN & NHIỄU (Chaos & Noise)** | **`matrix.chaos_index`** *($\text{chaos\_ratio} \times (1 - \text{entropy})$)* | - **Signal Arbiter** ([`arbiter.rs:136`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L136))<br>- **Risk Manager** ([`risk.rs:103`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L103)) | - **Arbiter:** Tính hệ số phạt điểm tự tin phi tuyến (*Confidence Penalty*): Khi bão Whipsaw bùng nổ, điểm tự tin của tất cả Scanner bị trừ nặng $\implies$ VETO không cho mở lệnh mới.<br>- **Risk:** Phát hiện bất thường để siết chặt Trailing SL bảo vệ vốn. |

---

## 🔬 3. Cơ Chế Phối Hợp Đa Trạng Thái Giữa Các Playbooks

Không có bất kỳ Playbook nào hoạt động đơn độc trên một chiều dữ liệu:
- **Absorption:** Kết hợp **Thanh khoản (`vol_z_score`, `vwma`)** + **Biến động (`squeeze`)** + **Động lượng nến (`CLV`, `%B`)** + **Bộ lọc Xu hướng (`kama_slope_atr`)**.
- **Momentum Ignition:** Kết hợp **Động lượng bứt phá (`close > bb_u`, `rsi_mom`)** + **Thanh khoản (`vol_z_score`)** + **Bộ lọc Xu hướng (`kama_slope_atr`)**.
- **Trend Pullback:** Kết hợp **Xu hướng chính (`kama_slope_atr`)** + **Hồi quy không gian (`delta_kama`)** + **Động lượng điểm vào (`stoch_trend`, `CLV`)**.
- **Reversal:** Kết hợp **Động lượng kéo dãn (`z_score`, `displacement`)** + **Kiệt sức thanh khoản & dòng tiền (`mfi`, `vol_ratio`)** + **Động lượng nến (`CLV`)**.
- **Range Osc:** Kết hợp **Thấp biến động (`bb_width_norm`)** + **Triệt tiêu Xu hướng (`trend_strength < max`, `bb_middle_slope_adj ~ 0`)** + **Giới hạn Thanh khoản (`vol_force < max`)**.
- **Signal Arbiter & Risk Manager:** Dùng **Hỗn loạn (`chaos_index`)** làm chốt chặn bảo vệ toàn cục, hạ đòn bẩy hoặc VETO mọi tín hiệu khi thị trường rơi vào trạng thái bão Whipsaw.
