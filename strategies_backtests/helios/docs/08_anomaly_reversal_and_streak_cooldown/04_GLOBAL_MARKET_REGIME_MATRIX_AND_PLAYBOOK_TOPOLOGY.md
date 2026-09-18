# 🗺️ KIẾN TRÚC TOÀN CẢNH: MA TRẬN TRẠNG THÁI THỊ TRƯỜNG & ĐỊNH TUYẾN PLAYBOOKS (HELIOS V3.3)

---

## 🏛️ 1. Bản Đồ Không Gian Trạng Thái Thị Trường (Global Regime Topology)

Không gian thị trường được đo lường bởi 4 trục trực giao độc lập trong `MarketMatrixV3`:
1. **Trục Xu Hướng:** $\delta_{\text{KAMA}} \in [-1.0, 1.0]$ và $\text{trend\_axis} \in [0.0, 1.0]$ (ADX).
2. **Trục Nén Biến Động:** $\text{vol\_axis} \in [0.0, 1.0]$ (Squeeze Percentile).
3. **Trục Dòng Tiền:** $\text{flow\_axis} \in [0.0, 1.0]$ (Volume Force: Z-Score Vol + MFI).
4. **Trục Bão Hỗn Loạn:** $\text{chaos\_index} \in [0.0, 1.0]$ (Whipsaw Chaos Ratio + Step Z-Score).

---

## 📊 2. Ma Trận Toàn Cảnh: Trạng Thái Trade Được vs Trạng Thái Tránh/Cấm Trade

| STT | Trạng Thái Thị Trường | Đặc Trưng Định Lượng (Matrix Coordinates) | Hành Vi Hệ Thống (Trade / Cấm Trade) | Playbook Chủ Lực Ứng Phó | Cơ Chế Bảo Vệ & Quản Trị Rủi Ro |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | **Bão Nhiễu Whipsaw (Choppy Chaos)** | $\text{chaos\_index} \ge 0.60$<br>Nhiễu loạn cao, không xu hướng, giá giật 2 đầu. | 🚫 **CẤM TRADE TOÀN CỤC (Flat)** | **Không có (Đứng ngoài bảo toàn vốn)** | $\text{Confidence} = \text{Score} \times (1.0 - \text{Chaos})$. Tự động triệt tiêu điểm tự tin xuống dưới ngưỡng tối thiểu. |
| **2** | **Thác Đổ Xu Xuống (Cascading Downtrend)** | $\delta_{\text{KAMA}} < -0.10$<br>$\text{trend\_axis} \ge 0.40$<br>$\text{vol\_axis}$ thấp (BB bung toạc). | 🚫 **CẤM LONG (Directional Veto)**<br>✅ **MỞ RỘNG CHO SHORT** | • **`TREND_PB` (Short)**<br>• **`MOMENTUM` (Breakdown)** | Cấm tuyệt đối bắt dao rơi Long (Reversal/Anomaly). Short gồng bằng Trailing Stop (TTP). |
| **3** | **Bơm Dựng Cột (Parabolic Uptrend)** | $\delta_{\text{KAMA}} > +0.10$<br>$\text{trend\_axis} \ge 0.40$<br>$\text{vol\_axis}$ thấp (BB bung toạc). | 🚫 **CẤM SHORT (Directional Veto)**<br>✅ **MỞ RỘNG CHO LONG** | • **`TREND_PB` (Long)**<br>• **`MOMENTUM` (Breakout)** | Cấm tuyệt đối cản tàu Short. Long đón đầu sóng và gồng bằng Trailing Stop (TTP). |
| **4** | **Nén Tích Lũy Gom Hàng (Compression / Squeeze)** | $\text{vol\_axis} \ge 0.50$ (Squeeze cao)<br>$\|\delta_{\text{KAMA}}\| \le 0.10$ (Giá nén phẳng)<br>$\text{flow\_axis} \ge 0.40$ (Có dòng tiền gom). | ✅ **TRADE ĐƯỢC (Chờ Bứt Phá)** | • **`ABSORPTION` (Độc Quyền Sân Chơi)** | Vào lệnh đón đầu pha gom hàng. SL chặt dưới đáy vùng nén $\implies$ R:R cực cao. |
| **5** | **Cạn Kiệt Đỉnh/Đáy (Exhaustion Climax)** | $\|Z\text{-score}\| \ge 2.0$ (Lệch cực đại)<br>Lệch xa KAMA $\ge 1.5\text{ ATR}$<br>$\text{flow\_axis} \le 0.20$ (**Volume cạn kiệt**). | ✅ **TRADE ĐƯỢC (Bắt Hồi Quy KAMA)** | • **`REVERSAL` (Chỉ vào khi Volume đã tắt hẳn)** | Take Profit neo tại `kama_mid`. Yêu cầu Volume phải kiệt quệ để tránh bẫy bắt dao rơi khi xả lũ. |
| **6** | **Sideway Đi Ngang Êm Ả (Clean Neutral)** | $\|\delta_{\text{KAMA}}\| \le 0.05$<br>$\text{trend\_axis} < 0.30$<br>$\text{chaos\_index} < 0.30$. | ✅ **ĐẤU THẦU TỰ DO THEO R:R** | Đấu thầu tự do giữa các proposal hợp lệ. | Ưu tiên lệnh có Stop Loss gần nhất. Nếu Long và Short ngang cơ $(\|\Delta\| \le 0.15) \implies$ Flat. |

---

## ⚓ 3. Cơ Chế Neo Khung Vĩ Mô (Macro Anchor): Khóa Cận Dưới Timeframe KAMA $\ge 15\text{m}$

### ⚠️ Lỗ hổng cũ:
Trước đây, Optimizer tự do chọn timeframe từ `1m` đến `4h` cho mọi chỉ báo. Nếu GA chọn `ind.kama.timeframe = 1m`, KAMA chỉ đo độ dốc của vài phút vi mô, bị bão nhiễu nến 1m đánh lừa và mất hoàn toàn tư cách "La bàn Vĩ mô".

### 🛡️ Giải pháp chuẩn Quant:
Trong `HeliosConfigV3` và bộ gen DNA:
* Khóa dải chọn timeframe của **KAMA Xu Hướng Vĩ Mô** chỉ được phép nằm trong:
  $$\text{KAMA Timeframe} \in \{15\text{m},\ 1\text{h},\ 4\text{h}\}$$
  (Tuyệt đối cấm lùi về $1\text{m}$ và $5\text{m}$).
* **Lợi ích:** KAMA luôn đại diện cho cấu trúc xu hướng khung lớn của thị trường (H1/H4), lọc sạch 95% bão nhiễu vi mô của từng cây nến 1 phút.

---

## ⚔️ 4. Cơ Chế Giải Quyết Xung Đột Giữa Các Playbooks (Arbiter Conflict Resolution)

### 1. Xung Đột CÙNG CHIỀU (Co-directional Conflict):
* Khi 2 Playbooks cùng phát tín hiệu cùng chiều (VD: `TrendPB` và `Momentum` cùng đòi LONG):
  * Arbiter tính khoảng cách Stop Loss tuyệt đối: $\text{dist} = |\text{Close} - \text{SL}|$.
  * **Lệnh có khoảng cách SL ngắn hơn sẽ chiến thắng** (R:R tốt hơn, rủi ro tài khoản thấp hơn).
  * Nếu khoảng cách SL bằng nhau, lệnh có điểm tự tin ($\text{Confidence}$) cao hơn sẽ thắng.

### 2. Xung Đột NGƯỢC CHIỀU (Opposite Conflict):
* Khi 1 Playbook đòi Long và 1 Playbook đòi Short:
  * Arbiter tính độ chênh lệch tự tin: $\Delta_{\text{conf}} = \text{Conf}_{\text{Long}} - \text{Conf}_{\text{Short}}$.
  * Nếu một bên vượt trội rõ rệt ($|\Delta_{\text{conf}}| > 0.15$): Chọn bên mạnh hơn.
  * Nếu hai bên ngang ngửa ($|\Delta_{\text{conf}}| \le 0.15$): **HỦY CẢ HAI LỆNH, FLAT ĐỨNG NGOÀI! (Cash is King)**.
