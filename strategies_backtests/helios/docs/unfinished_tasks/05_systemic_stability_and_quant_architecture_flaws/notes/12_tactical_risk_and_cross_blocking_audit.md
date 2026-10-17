# Ghi Chú Kiểm Toán Định Lượng: Rủi Ro Tầng Chiến Lược & Nghịch Lý Ngáng Chân Tự Triệt Tiêu
## HELIOS V3 TACTICAL RISK & CROSS-BLOCKING AUDIT

> **Dự án:** Helios V3 Strategy Core (`strategies_backtests/helios`)  
> **Thư mục:** [`05_systemic_stability_and_quant_architecture_flaws/notes/`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/unfinished_tasks/05_systemic_stability_and_quant_architecture_flaws/notes/)  
> **Mã tài liệu:** `NOTE-12-TACTICAL-RISK`  
> **Ngày lập:** 14/10/2026  
> **Trạng thái:** Báo cáo kiểm toán độc lập cấp cao (Line-by-line Verified)

---

## 🏛️ 1. TỔNG QUAN HIỆN TRẠNG KIẾN TRÚC RỦI RO CHIẾN LƯỢC

Hệ thống quản trị rủi ro chiến lược của Helios V3 được xây dựng trên nguyên tắc phân lớp phòng thủ:
- **Tầng 1 (Regime Gatekeeper - Cầu Dao Chế Độ):** Kiểm tra biến động sàn Flatline (`NATR < cost_floor`) và bão hỗn loạn Chaos (`chaos_index >= chaos_veto_threshold` kèm Hysteresis $K=5$).
- **Tầng 2 (Tactical Scanners - Bộ Lọc Nội Bộ):** Mỗi scanner lọc trạng thái (Squeeze, KAMA Slope, CLV, Z-Score).
- **Tầng 3 (Arbiter - Hội đồng Đấu Thầu & Thẩm Định):** Thẩm định Stop Loss cấu trúc (`validate_structural_sl`), kiểm tra khoảng cách thanh lý sàn (`safe_liq_dist`) và trần ROE $15.0\%$.
- **Tầng 4 (RiskManager & Trailing Stop):** Kẹp trần Stop Loss (`clamp_sl_price`), định cỡ vị thế (`calculate_size`), và dời Stop Loss thích ứng (`manage_trailing_stop`).

> ⚠️ **KẾT QUẢ KIỂM TOÁN TỔNG THỂ:**  
> Hệ thống phòng thủ đang mắc phải căn bệnh **"Lồng sắt quá nhiều lớp (Over-constraining)"** và **"Nghịch lý Catch-22 tự triệt tiêu cơ hội"**, khiến hệ thống bị tê liệt giao dịch ở các thời điểm thị trường có sóng biến động lớn nhất.

---

## 🚨 2. NGHỊCH LÝ CATCH-22 CỦA `validate_structural_sl` (TỬ HUYỆT CỐT TỬ)

* **Vị trí code:** [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs:219-280`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L219-L280) & [`risk.rs:84-138`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L84-L138).

### 2.1. Bản Chất Toán Học & Logic Code
Khi một Scanner phát hiện setup và tính toán mức Stop Loss cấu trúc theo hành động giá (Price Action) — ví dụ đáy nhịp hồi Pullback hoặc đáy cây nến nổ volume Absorption:
$$\text{suggested\_sl} = \text{low}_{\text{swing}} - \text{buffer\_atr}$$
Khoảng cách Stop Loss và tỷ lệ rủi ro trên vốn ký quỹ (ROE) tương ứng tại đòn bẩy $L = 20\text{x}$:
$$\text{dist\_pct} = \frac{|\text{Price} - \text{suggested\_sl}|}{\text{Price}}, \quad \text{suggested\_roe} = \text{dist\_pct} \times L$$

Tại `arbiter.rs:254-280`, Arbiter gọi `validate_structural_sl`:
- Nếu $\text{suggested\_roe} > \text{playbook\_max\_sl\_roe.min}(0.15)$ (với $0.15 = 15.0\%$ ROE toàn cục):
- Hàm trả về `SlValidationResult::StructuralRiskVeto`.
- Arbiter gán `passed = false` và **VỨT BỎ TOÀN BỘ PROPOSAL KHỎI SÀN ĐẤU THẦU**!

### 2.2. Vạch Trần Nghịch Lý Catch-22 Ngoài Đời Thực
1. **Trần $15.0\%$ ROE tại đòn bẩy $20\text{x}$ tương đương khoảng cách giá cực kỳ hẹp:**
   $$\text{Max Price Distance} = \frac{15.0\%}{20} = 0.75\%$$
   Trên ETH giá $\$2,500$, khoảng cách $0.75\%$ chỉ là **$\$18.75$**.
2. **Thực tế biên độ nến và đệm biến động ATR:**
   - Một cây nến 1m bùng nổ xung lực (`MomentumIgnition`) hoặc một nhịp rũ bỏ (`TrendPB`, `Reversal`) thường có thân nến + râu nến từ $\$10 \sim \$25$.
   - Cộng thêm đệm an toàn chống quét thanh khoản ($1.0 \sim 1.5 \times \text{ATR} \approx \$6 \sim \$12$), khoảng cách SL cấu trúc thực tế thường nằm trong khoảng **$0.85\% \sim 1.50\%$** ($17\% \sim 30\%$ ROE).
3. **Hậu Quả Triệt Tiêu Lợi Lộc (Perverse Incentive):**
   - **Tất cả các cơ hội bứt phá nến đẹp, xung lực rõ ràng đều bị VETO 100%!**
   - Chỉ các cây nến li ti còi cọc ($< 0.6\%$ biên độ) mới lọt qua được cửa Arbiter. Nhưng vì SL quá sát, vừa mở lệnh là bị râu nến nhiễu quét sạch SL ngay lập tức.
4. **Mâu Thuẫn Kiến Trúc Với `mod.rs` (Clamping Dead Code):**
   - Tại [`mod.rs:332-342`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L332-L342), hệ thống đã xây dựng hàm `RiskManager::clamp_sl_price` để kẹp mức SL quá xa về đúng trần an toàn $15\%$ ROE.
   - Nhưng vì Arbiter đã VETO và giết chết proposal từ trước, **khối lệnh `clamp_sl_price` KHÔNG BAO GIỜ ĐƯỢC CHẠM TỚI** đối với các proposal có SL cấu trúc!
   - **Nghịch lý kép:** Scanner nào lười biếng **không tính SL** (`suggested_sl: None`) thì Arbiter cho qua (`Acceptable`) và được kẹp SL ở `mod.rs`. Còn Scanner nào **tính toán SL cẩn thận** mà hơi rộng một chút thì bị Arbiter giết chết!

### 2.3. Giải Pháp Tinh Giản (Dynamic Scaling Thay Vì Binary Veto)
Trong `arbiter.rs`: **Tuyệt đối không VETO khi SL cấu trúc $> 15\%$ ROE**. Thay vào đó:
1. Tự động kẹp mức SL về đúng trần an toàn: $\text{final\_sl} = \text{clamp\_sl\_price}(\dots)$.
2. Co giãn kích thước vị thế (Dynamic Position Sizing) theo tỷ lệ để cố định rủi ro danh nghĩa (\$ rủi ro không đổi):
   $$\text{Scale Factor} = \frac{\text{Max Allowed ROE (15\%)}}{\text{Suggested ROE}}$$
   $$\text{Final Order Size} = \text{Base Size} \times \text{Scale Factor}$$
*Ưu điểm:* Vừa bảo toàn $100\%$ an toàn vốn (không bao giờ mất quá $15\%$ ROE), vừa không bỏ lỡ bất kỳ cơ hội bùng nổ nến đẹp nào của thị trường.

---

## 🧱 3. HIỆN TƯỢNG "LỒNG SẮT 5 LỚP" CHỒNG CHÉO TÊ LIỆT `Trend_PB`

* **Vị trí code:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs:64, 98, 104, 109, 114`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/trend_pb.rs#L64)

### 3.1. Sự Chồng Chéo Ngáng Chân Giữa Các Tầng
`Trend_PB` hiện tại phải vượt qua 5 lớp khóa cứng đồng thời:
1. **Lớp 1 (Gatekeeper Flatline):** `NATR < Required_Min_NATR` đã chặn sạch các vùng thị trường đóng băng.
2. **Lớp 2 (Chop Zone Squeeze Guard - `trend_pb.rs:64`):**
   ```rust
   if squeeze > pb_cfg.max_squeeze_intensity { return None; }
   ```
   *Mâu thuẫn:* Code ghi chú *"Chop zone là đất diễn của range_osc"*. Nhưng trong V3.3, **`RangeOscScanner` ĐÃ BỊ TẮT KHỎI HỆ THỐNG** (`mod.rs:48-53`). `Trend_PB` tự trói chân để nhường sân cho một Scanner "không hề tồn tại", tạo ra một **Vùng Chân Không Tê Liệt (Dead Zone)** không ai giao dịch.
3. **Lớp 3 (Boundary Guard Z-Score Lệch Pha Đa Khung - `trend_pb.rs:98`):**
   ```rust
   if z_score > pb_cfg.max_boundary_z { return None; } // Mặc định 0.50
   ```
   *Lệch pha:* `z_score` này được tính trên khung **5m lookback 50 bars** (~4 tiếng đồng hồ). Khi thị trường có một con sóng Uptrend 4h mạnh mẽ, giá liên tục bám biên trên nên `z_score` 5m hầu như luôn $> +0.50\sigma$. Bộ lọc này cấm mở Long trong suốt toàn bộ con sóng tăng mạnh nhất, chỉ cho phép Long khi sóng tăng đã gãy hoặc suy yếu!
4. **Lớp 4 & 5 (Stochastic Oversold + CLV + Soft Hook):** Ép $\text{Stoch}_K \le 30$ và $CLV \ge 0.20$.

### 3.2. Giải Pháp Tháo Gỡ
1. **Gỡ bỏ hoặc nâng trần Squeeze trong `trend_pb.rs`:** Vì Gatekeeper đã đảm nhiệm việc lọc Flatline, `Trend_PB` chỉ cần lọc khi thị trường bung nén cực đại hoặc nâng trần `max_squeeze_intensity` lên $0.90$.
2. **Thay thế Boundary Guard 5m bằng Biên Bollinger Vi mô 1m:** Thay vì dùng Z-Score 5m lag pha 4 tiếng, chỉ cần kiểm tra nến 1m vi mô: `current_close < bb_upper_1m` để chống mua ngay đỉnh râu nến phân phối ngắn hạn.

---

## 🔄 4. NGHỊCH LÝ TRIỆT TIÊU KHỐI LƯỢNG BÁN THÁO TRONG `Reversal`

* **Vị trí code:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs:85, 105-107`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs#L85)

### 4.1. Lỗi Toán Học Trong Chấm Điểm Volume
Trong `reversal.rs:105-107`:
$$\text{vol\_ratio} = \frac{\text{Volume}[i]}{\text{Vol\_SMA}[i]}$$
$$f_{\text{vol}} = \text{sat\_proximity}(\text{vol\_ratio},\ \text{exhaust\_drop})$$
Hàm `sat_proximity(x, target)` được định nghĩa để thưởng điểm cao nhất khi $x \approx \text{target}$ (mặc định $\text{exhaust\_drop} = 0.35 \sim 0.50$), và **GIẢM VỀ 0 KHI $x$ TĂNG CAO**!

### 4.2. Hậu Quả Thực Chiến
- Trong các đợt sập gãy thanh lý kinh hoàng (Capitulation Dump) — thời điểm vàng để bắt đáy đảo chiều theo Wyckoff — khối lượng giao dịch bùng nổ gấp $200\% \sim 400\%$ trung bình ($\text{vol\_ratio} = 2.0 \sim 4.0$).
- Vì công thức dùng `sat_proximity`, điểm khối lượng $f_{\text{vol}}$ bị **ĐÈ XUỐNG BẰNG ĐÚNG 0.0**!
- Cú bão volume bán tháo đẹp nhất bị hệ thống chấm 0 điểm và Arbiter vứt bỏ!

### 4.3. Xung Đột CLV Rút Chân vs. MFI Quá Bán
- Điều kiện Reversal yêu cầu nến rút chân mạnh: $CLV \ge 0.65$ (giá đóng cửa sát đỉnh cao nhất của nến).
- Đồng thời lại ép chỉ số dòng tiền $MFI \le 30$ (vùng quá bán).
- Nhưng bản thân một cây nến biên độ lớn rút chân mạnh từ đáy lên đỉnh với khối lượng khủng sẽ ngay lập tức đẩy giá trị Typical Price và dòng tiền dương lên cao, kéo $MFI$ vọt lên $> 35$! Hai điều kiện này tự đá văng nhau ở các cây nến rút chân hoàn hảo.

---

## 🔒 5. LỖ HỔNG AN TOÀN & KIỂM TOÁN TRAILING STOP LOSS

### 5.1. Lỗ Hổng Thiếu Chiều Giao Dịch `is_long` trong `validate_structural_sl`
* **Vị trí:** `risk.rs:84-138`
* **Bản chất:** Hàm chỉ tính $\text{dist\_pct} = \frac{|\text{Price} - \text{SL}|}{\text{Price}}$ bằng trị tuyệt đối.
* **Nguy cơ:** Nếu một Scanner bị lỗi sinh ra lệnh Long nhưng $SL > Price$, hàm vẫn thẩm định `Acceptable`! Lệnh này gửi sang sàn sẽ bị kích hoạt Stop Market cắn lỗ ngay lập tức tại mili-giây đầu tiên.
* **Khắc phục:** Bổ sung tham số `is_long: bool`. Long bắt buộc $SL < Price \times (1 - \text{min\_dist})$, Short bắt buộc $SL > Price \times (1 + \text{min\_dist})$.

### 5.2. Vùng Chân Không Khóa Lãi (+6% Đến +20% ROE) Trong Trailing Stop
* **Vị trí:** `risk.rs:431-445`, `config.rs:1187`
* **Hiện trạng:**
  - Break-Even kích hoạt tại $+6.0\%$ ROE, dời SL lên Entry $+ 2.6\%$ ROE.
  - Trần khóa lãi `min_lock_roe` lại nằm tít ở **$+20.0\%$ ROE**.
  - Toàn bộ khoảng lợi nhuận từ $+6\%$ đến $+20\%$ ROE (ví dụ lệnh đã lãi $+16.5\%$ ROE) **hoàn toàn không có nấc bảo vệ trung gian**. Khi thị trường đảo chiều, lệnh rơi thẳng về mức hòa vốn $+2.6\%$ ROE, làm bốc hơi toàn bộ lợi nhuận.
* **Khắc phục:** Triển khai **Tiered Profit Lock** chuẩn mực:
  - $\text{ROE} \ge +6.0\% \implies \text{Khóa Hòa Vốn} +2.5\%$ ROE.
  - $\text{ROE} \ge +10.0\% \implies \text{Khóa Bảo Toàn Lãi} +5.0\%$ ROE.
  - $\text{ROE} \ge +15.0\% \implies \text{Khóa Siết Lãi} +9.0\%$ ROE.

---

## 📋 6. KẾ HOẠCH HÀNH ĐỘNG KHẮC PHỤC (ACTION ITEMS)

| Ưu Tiên | Nhiệm Vụ | File Mục Tiêu | Giải Pháp Kỹ Thuật |
| :---: | :--- | :--- | :--- |
| **P0** | **Phá Bẫy Catch-22 Veto SL** | `arbiter.rs:254-280` | Bỏ Binary Veto khi SL $> 15\%$ ROE; tự động kẹp SL về trần và co giãn khối lượng (Dynamic Sizing). |
| **P0** | **Vá Lỗ Hổng Inverted SL** | `risk.rs:84-138` | Bổ sung `is_long: bool`, kiểm tra chiều logic giá $SL$ trước khi duyệt. |
| **P1** | **Khai Thông Trend_PB** | `trend_pb.rs:64, 98` | Gỡ bỏ `squeeze_intensity` ngáng chân; thay Z-Score 5m lag pha bằng dải vi mô 1m. |
| **P1** | **Sửa Hàm Volume Reversal** | `reversal.rs:105-107` | Đổi `sat_proximity` sang hàm hỗ trợ cả 2 thái cực: Cực cạn kiệt HOẶC Bùng nổ bán tháo. |
| **P1** | **Bổ Sung Tiered Profit Lock** | `risk.rs:431-450` | Thiết lập 3 nấc khóa lãi bậc thang ($+6\% \to +2.5\%$, $+10\% \to +5\%$, $+15\% \to +9\%$). |
