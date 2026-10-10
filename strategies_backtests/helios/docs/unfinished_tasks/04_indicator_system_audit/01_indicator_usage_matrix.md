# BẢNG MA TRẬN SỬ DỤNG CHỈ BÁO THỰC TẾ & KIỂM TOÁN TẦNG CẢM BIẾN HELIOS V3.3

> **Thư mục lưu trữ:** `strategies_backtests/helios/docs/unfinished_tasks/04_indicator_system_audit/`  
> **File:** `01_indicator_usage_matrix.md`  
> **Mục tiêu:** Hệ thống hóa toàn bộ các Indicator hiện có trong `HeliosSensors` và `MarketMatrixV3`, đối chiếu bên tiêu thụ thực tế (Consumers), phát hiện các chỉ báo ma (Ghost Sensors), phân định ranh giới Vi mô - Trung mô - Vĩ mô và xây dựng phương án tinh giản DNA.

---

## 🏛️ 1. TỔNG QUAN KIẾN TRÚC CẢM BIẾN

Hệ sinh thái Helios V3.3 chia tách dòng dữ liệu chỉ báo thành 2 tầng:
1. **Level 1 (Raw Sensors - `HeliosSensors`):** Con trỏ mảng dữ liệu thô nhận từ `ferro_trade::BulkIndicatorFactory` (tính toán song song hoặc đọc từ bộ nhớ đệm).
2. **Level 2 (Market Matrix - `MarketMatrixV3`):** Các chỉ số đã chuẩn hóa phi thứ nguyên ($[0.0, 1.0]$ hoặc $[-1.0, 1.0]$) phục vụ trực tiếp cho Arbiter và Risk Manager.

Hiện tại, Arbiter vận hành **4 Playbook chiến thuật chủ lực**:
* `TrendPB` (Trend Pullback): Săn sóng hồi thuận xu hướng KAMA.
* `MomentumIgnition` (Breakout): Bắt các cú bứt phá bùng nổ dải nén Bollinger Bands.
* `ClimaxReversal` (Mean Reversion): Bắt đảo chiều cực trị kiệt sức dòng tiền MFI và Z-Score.
* `AbsorptionScout` (VSA): Bắt dấu chân cá mập gom/xả theo Wyckoff (%B, VWMA, Squeeze).

*(Hai playbook `RangeOsc` và `Anomaly` đã được đưa vào chế độ ngủ đông trong phiên bản V3.3).*

---

## 📊 2. BẢNG MA TRẬN SỬ DỤNG CHỈ BÁO THỰC TẾ

| Nhóm chức năng | Tên Indicator | Timeframe DNA | Số Gen DNA | Bên Tiêu Thụ Thực Tế (Consumers) | Bản chất Vật lý / Nghiệp vụ | Tình trạng & Đánh giá |
| :--- | :--- | :---: | :---: | :--- | :--- | :--- |
| **Trục Xu Hướng Vĩ Mô** | `kama_mid`<br>`kama_slope_atr` | 15m, 1h, 4h | 2 gen | • `trend_pb`<br>• `momentum`<br>• `reversal`<br>• `matrix.rs` | Xác định la bàn xu hướng trung-dài hạn, kháng nhiễu cực tốt nhờ hệ số thích ứng ER. | 🟢 **RẤT TỐI ƯU.** Xương sống vĩ mô không thể thiếu. |
| **Độ Biến Động Vi Mô** | `atr_sl` | 1m, 3m, 5m | 2 gen | • Cả 4 Scanners<br>• `RiskManager`<br>• `Gatekeeper` | Đo dải biến động thực vi mô để tính Stop Loss động và định cỡ vị thế. | 🟢 **RẤT TỐI ƯU.** Bảo đảm tính linh hoạt theo biên độ từng nến. |
| **Sàn Biến Động Chết** | `natr` | 15m, 1h, 4h | 2 gen | • `Gatekeeper`<br>• `matrix.chaos_index` | Đo % biến động so với giá. Chốt chặn: $\text{NATR} \ge 2 \times \text{Fee} + \text{Slippage} + \text{Alpha}$. | 🟢 **RẤT TỐI ƯU.** Chặn đứng cào lệnh làm mồi cho phí sàn. |
| **Bão Hỗn Loạn Whipsaw** | `chaos_index` | Derived | 0 gen | • `Gatekeeper`<br>• `Arbiter` | Tích hợp từ `chaos_ratio` & `efficiency_ratio` để đo nhiễu giật râu 2 đầu. | 🟢 **TỐI ƯU.** Bảo vệ tài khoản khỏi bão thị trường. |
| **Độ Căng Biên Độ** | `price_z_score` | 15m, 1h, 4h | 1 gen | • `trend_pb`<br>• `reversal` | Đo khoảng cách giá lệch bao nhiêu độ lệch chuẩn ($\sigma$) so với đường trung bình. | 🟢 **RẤT TỐI ƯU.** Chuẩn hóa phi thứ nguyên, thay thế KAMA distance. |
| **Xung Lực Khối Lượng** | `vol_z_score` | 15m, 1h, 4h | 1 gen | • `momentum`<br>• `absorption` | Đo mức độ bùng nổ thể tích giao dịch so với mức bình quân lịch sử. | 🟢 **RẤT TỐI ƯU.** Độc lập, không bị pha tạp như khối lượng tuyệt đối. |
| **Biên Độ & Độ Nén** | `bbands` + `keltner` | 5m, 15m, 1h, 4h | 6 gen | • `momentum`<br>• `absorption`<br>• `trend_pb` (squeeze) | Đo biên độ dải giá và phát hiện trạng thái nén tích lũy TTM Squeeze. | 🟢 **TỐI ƯU.** Phát hiện thời điểm bùng nổ động lượng. |
| **Giá Trị Gia Quyền** | `vwma` | 5m, 15m, 1h, 4h | 2 gen | • `absorption` | Xác định trọng tâm dòng tiền nằm ở vùng giá cao hay thấp. | 🟢 **TỐI ƯU.** Phục vụ Wyckoff VSA rất chuẩn. |
| **Dòng Tiền Kiệt Sức** | `mfi` | 5m, 15m, 1h, 4h | 2 gen | • `reversal` | Đo áp lực dòng tiền nạp vào/rút ra để bắt điểm kiệt sức dòng tiền. | 🟢 **TỐI ƯU.** Kết hợp xuất sắc với Z-Score trong Reversal. |
| **Trần Động Lượng** | `rsi_trend`<br>`rsi_momentum` | 5m-4h<br>1m-15m | 4 gen | • `trend_pb` (rsi_t)<br>• `momentum` (rsi_m) | Đo tốc độ thay đổi giá, chặn mua đỉnh Pullback và xác nhận xung lực Breakout. | 🟡 **TƯƠNG ĐỐI TỐI ƯU.** Nhưng cần tinh chỉnh timeframe. |
| **Động Lượng Hồi Quy** | `stoch_trend` | 5m, 15m, 1h, 4h | **7 gen** | • `trend_pb` | Đo vị trí giá hạ nhiệt trong nhịp hồi. | 🔴 **CHƯA TỐI ƯU (BỊ LỖI).** Phình 7 gen, bẫy TF 4h, lỗi forward-fill. |
| **Sức Mạnh Xu Hướng** | `adx` | 5m, 15m, 1h, 4h | **2 gen** | • `matrix.trend_strength`<br>*(Chỉ `range_osc` dùng)* | Đo độ mạnh của xu hướng không kể hướng đi. | 🔴 **LÃNG PHÍ.** 4 Scanner chủ lực **KHÔNG AI DÙNG**. Tốn 2 gen DNA vô ích! |
| **Đảo Chiều RSI** | `rsi_reversal` | 5m, 15m, 1h, 4h | **2 gen** | • **0 Scanner** | Định nghĩa riêng cho Reversal nhưng Reversal dùng MFI + Z-Score! | 🔴 **CHỈ BÁO MA (GHOST).** Tốn 2 gen DNA, tốn RAM tính toán nhưng bỏ xó! |
| **Khối Lượng Trung Bình** | `vol_sma` | 1m, 5m, 15m, 1h | **2 gen** | • `reversal` *(chỉ làm fallback)* | Đường MA khối lượng thông thường. | 🟡 **DƯ THỪA.** Đã có `vol_z_score` ưu việt hơn nhiều. |
| **Vi Cấu Trúc Nến** | `clv` (Close Location Value) | **1m (Tick gốc)** | 0 gen *(tính trực tiếp)* | • **Cả 4 Scanners** | Đo tỷ lệ phe mua/bán chiếm ưu thế tại nến đóng: $\frac{2C - (H+L)}{H-L}$. | 💎 **XUẤT SẮC NHẤT.** Tức thì, không trễ, không lỗi MTF, độ tin cậy cao. |

---

## 🚨 3. BA VẤN ĐỀ GỐC RỄ ĐƯỢC PHÁT HIỆN

1. **Vấn nạn Chỉ báo Ma & Phình to Không gian DNA (Ghost Sensors & DNA Bloat):**
   * `rsi_reversal` (2 gen), `adx` (2 gen), `vol_ma` (2 gen) tiêu tốn **6 gen DNA** trong không gian tối ưu hóa GA nhưng hoàn toàn không tác động đến 4 scanner chủ lực đang hoạt động.
2. **Lệch pha Quy mô Thời gian (Semantic & Scale Mismatch):**
   * `stoch_trend` (chiếm 7 gen) cho phép cấu hình khung 1h, 4h cho nhịp Pullback vốn dĩ chỉ kéo dài 15-45 phút trên nến 1m.
   * Cơ chế forward-fill từ khung MTF xuống 1m khiến biểu thức $K_i < K_{i-1}$ bị tê liệt trong 93.3% thời gian.
3. **Sức mạnh chưa được khai phóng của Tầng Vi mô (CLV):**
   * CLV phản ánh trung thực phản ứng giá tức thì tại nến 1m, không bị trễ, không bị lỗi MTF, nhưng hiện tại mới chỉ được áp dụng như một bộ lọc nhị phân thô sơ thay vì đóng vai trò cò kích hoạt điểm uốn (Inflection Trigger).
