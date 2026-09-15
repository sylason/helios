# 📘 THIẾT KẾ KỸ THUẬT: FLAW-REG.10 - ĐẠI TU TOÀN DIỆN RANGE_OSC & TRIỆT TIÊU BẪY BAND-WALKING TRAP

> **Mã định danh:** `FLAW-REG.10`  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/07_comprehensive_audit/flaw_reg10_range_osc_band_walking_trap/TECHNICAL_SPEC.md`  
> **Phạm vi tác động:** `sensors.rs`, `matrix.rs`, `range_osc.rs`, `config.rs`, `arbiter.rs`, `risk.rs`, `core_v3/mod.rs`.  
> **Mục tiêu cốt lõi:** Đại tu toàn diện cơ chế nhận diện và giao dịch trong vùng đi ngang (Clean Range), triệt tiêu bẫy bắt dao rơi bám dải dưới khi thị trường trôi dốc (Band-Walking Trap), xóa sạch 7 hằng số hardcode, mở khóa 100% Dynamic DNA bao phủ dải Universal 1% -> 99%.

---

## 1. 🔬 PHÂN TÍCH HIỆN TRẠNG & BẰNG CHỨNG ĐỊNH LƯỢNG (ROOT CAUSE)

### 1.1. Bằng chứng thực nghiệm từ Log SOL (`WFSS_SOL_FLAW-REG.1_FLAW-TSL.9.log`)
Trong chu kỳ giá SOL trôi dốc từ $102.30 xuống $98.50:
* `RANGE_OSC` thực hiện 6 giao dịch: **1 Thắng (+0.32$), 5 Thua (-5.36$) $\implies$ Net PnL = -$5.04 (Win Rate = 16.7%)**.
* Trong đó, **4 lệnh LONG liên tiếp (#004, #005, #006, #008) thua 100% (-$4.40)** do liên tục bắt đáy khi giá bám sát dải dưới `bb_lower`.

### 1.2. Chuỗi sai lệch toán học liên hoàn (Chain of Failure)
1. **Tầng Vĩ mô (`matrix.rs`):** Thị trường trôi dốc chậm với biên độ nhỏ khiến ADX $< 20$. Ma trận tính $w_{\text{range\_raw}} = (1 - t_{\text{mul}}) \times (1 - c_r) \times (1 - sqz)$ vọt lên $> 65\%$ và `range_multiplier = 0.85`, ngộ nhận là Clean Range phẳng!
2. **Tầng Trinh sát (`range_osc.rs`):**
   * Bẫy trị tuyệt đối đối xứng: `dist_l = (close - bb_l).abs()`. Khi nến đỏ đâm thủng hoặc bám sát dải dưới, `dist_l` tiệm cận 0 khiến điểm giá $f_{\text{price\_long}} = e^0 = 1.0$ (tối đa 100%).
   * Chỉ báo Stochastic bị bẹp dí ở đáy: $stoch_k < 15$ cho điểm quá bán $f_{\text{stoch\_long}} \approx 0.90$. Chỉ cần 1 nhịp nảy tick giá li ti là kích hoạt tín hiệu Hook ảo.
   * Chứa 7 hằng số hardcode: trọng số 50-30-20, hệ số Sigmoid -0.1, Hook -0.5, đệm cố định 0.2%.
3. **Tầng Trọng tài (`arbiter.rs`):** Nhận điểm ảo $0.85$ từ `RangeOsc`, Arbiter phê duyệt thầu và cướp quyền đi lệnh của `TrendPullback`, ép bot mở lệnh Long cản tàu.
4. **Tầng Quản trị Rủi ro (`risk.rs`):** Đặt Take Profit cố định tại dải trên `bb_upper` lúc vào lệnh. Khi thị trường trôi dốc, dải trên trôi xuống theo khiến TP ban đầu nằm tít trên cao, lệnh không bao giờ chạm TP và bị ngâm 200-300 nến rồi chết vì Stagnant Decay.

---

## 2. 🏛️ THIẾT KẾ ĐẠI TU TOÀN DIỆN (7 THÀNH PHẦN KHÉP KÍN)

### 2.1. Cảm Biến Mới Trong `sensors.rs` (Nguyên tắc Additive 100%)
Bổ sung trường đo độ dốc chuẩn hóa của trục giữa Bollinger Bands (SMA 20) qua ATR để độc lập hoàn toàn với thị giá coin:
$$\text{bb\_middle\_slope\_adj}[i] = \frac{\text{bb\_middle}[i] - \text{bb\_middle}[i - K]}{K \cdot \text{ATR}[i]}$$
*(Với $K = 5$ nến để lọc nhiễu micro-noise 1m. Hoàn toàn không chỉnh sửa bất kỳ sensor cũ nào).*

### 2.2. Chuẩn Hóa Simplex Trong `matrix.rs`
Gắn điều kiện độ phẳng thực sự vào $w_{\text{range\_raw}}$:
* Định nghĩa hệ số phẳng: $\text{flatness\_factor} = 1.0 - \tanh\left(\frac{|\text{bb\_middle\_slope\_adj}[i]|}{0.10}\right)$.
* $w_{\text{range\_raw}} = ((1.0 - t_{\text{mul}}) \times (1.0 - c_r) \times (1.0 - sqz) \times \text{flatness\_factor})$.
* Nếu trục giữa BB dốc cắm đầu hoặc dựng ngược $\implies \text{flatness\_factor} \to 0 \implies w_{\text{range}} \to 0$, nhường Simplex cho $w_{\text{trend}}$.

### 2.3. Đại Tu 4 Khối Của `range_osc.rs`
1. **Khối 1 (Vị thế trong dải - Xóa bỏ `.abs()`):**
   * Long: Bắt buộc $\text{Close} \ge \text{bb\_lower}$ (giá nằm bên trong hoặc ngay trên mép dải, không bị breakdown).
   * Short: Bắt buộc $\text{Close} \le \text{bb\_upper}$ (giá nằm bên trong hoặc ngay dưới mép dải, không bị breakout).
2. **Khối 2 (Nến xác nhận phản ứng - Rejection Candle):**
   * Long: Nến phải đóng xanh (`Close > Open`) HOẶC có râu dưới dài ($\text{Lower Wick} \ge \text{Upper Wick}$ và $\text{Close} > \text{Mid(Candle)}$).
   * Short: Nến phải đóng đỏ (`Close < Open`) HOẶC có râu trên dài ($\text{Upper Wick} \ge \text{Lower Wick}$ và $\text{Close} < \text{Mid(Candle)}$).
3. **Khối 3 (Take Profit thực tế hướng về Trục Giữa):**
   * Thay vì tham lam đặt $\text{TP} = \text{bb\_upper}$, mục tiêu TP thực tế được định vị linh hoạt quanh trục giữa `bb_middle` dựa trên gene DNA `tp_target_ratio`.
4. **Khối 4 (Xóa bỏ 7 hardcodes & Chuẩn hóa DNA):**
   * Thay toàn bộ các hằng số $0.2\%$ cố định bằng bội số ATR động.

### 2.4. Cấu Hình DNA Dynamic 1% -> 99% Trong `config.rs`
Cập nhật `TacticalRangeParams` bao phủ toàn bộ không gian tham số:
```rust
pub struct TacticalRangeParams {
    /// Độ rộng tối thiểu dải Range (0.2% -> 6.0% giá, Step 0.2%)
    #[dna(min = 0.002, max = 0.060, step = 0.002)]
    pub min_width_pct: f64,

    /// Trần Volume né tin tức (Universal 1% -> 99%, Step 3%)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub volume_max_threshold: f64,

    /// Ngưỡng kích hoạt mềm Range Oscillator (Universal 1% -> 99%, Step 3%)
    #[dna(min = 0.01, max = 0.99, step = 0.03)]
    pub min_activation_score: f64,

    /// Ngưỡng sức mạnh Trend tối đa mà Range được phép hoạt động (0.10 -> 0.80, Step 0.05)
    #[dna(min = 0.10, max = 0.80, step = 0.05)]
    pub max_trend_strength: f64,

    /// Ngưỡng dốc xu hướng KAMA tối đa cấm bắt dao rơi (0.05 -> 0.50, Step 0.05)
    #[dna(min = 0.05, max = 0.50, step = 0.05)]
    pub max_adverse_slope: f64,

    /// Đệm ATR ngoài biên dải Bollinger Bands (0.10 -> 3.00 ATR, Step 0.15)
    #[dna(min = 0.10, max = 3.00, step = 0.15)]
    #[serde(default = "default_range_sl_buffer_atr")]
    pub struct_sl_buffer_atr: f64,

    /// 🛡️ FLAW-REG.10: Ngưỡng dốc tối đa của trục giữa BB Middle (0.02 -> 0.50 ATR, Step 0.02)
    #[dna(min = 0.02, max = 0.50, step = 0.02)]
    #[serde(default = "default_range_max_bb_slope")]
    pub max_bb_slope: f64,

    /// 🛡️ FLAW-REG.10: Tỷ lệ mục tiêu TP từ Entry đến dải đối diện (0.10 -> 0.90, Step 0.05)
    /// 0.50 = Đúng trục giữa BB Middle; < 0.50 = Chốt non an toàn; > 0.50 = Hướng về dải đối diện
    #[dna(min = 0.10, max = 0.90, step = 0.05)]
    #[serde(default = "default_range_tp_target_ratio")]
    pub tp_target_ratio: f64,
}
```

### 2.5. Chống Cướp Thầu Trong `arbiter.rs`
Nếu $w_{\text{trend\_norm}} > 0.40$ mà `RangeOsc` phát tín hiệu ngược chiều xu hướng chính $\implies$ Arbiter áp dụng Veto phủ quyết, bảo vệ quyền tác chiến của `TrendPullback`.

### 2.6. Đồng Bộ Thoát Lệnh Trong `risk.rs`
Tiếp nhận `suggested_tp` động hướng về trục giữa `bb_middle` cho các lệnh mang Tag `"RANGE_OSC"`, giảm thiểu thời gian ngâm lệnh và triệt tiêu bẫy Stagnant Decay.

### 2.7. Điều Phối Cooldown & DNA Width Trong `core_v3/mod.rs`
* Nâng `DNA_WIDTH` từ 119 lên **121 chiều** (bổ sung `max_bb_slope` và `tp_target_ratio`).
* Kích hoạt Directional Cooldown sau khi lệnh Range bị cắt lỗ, chặn đứng việc mở lại lệnh cùng chiều ngay nến kế tiếp.

---

## 3. ⚖️ ĐỐI CHẤT CÁC EDGE CASES

1. **Edge Case 1: Giá đi ngang tạo đáy sau một cú dump mạnh:**
   * *Hiện tượng:* Giá đã dừng rơi 5 nến nhưng SMA(20) vẫn còn dốc xuống do độ trễ quán tính.
   * *Xử lý:* Hệ thống sẽ kiên nhẫn chờ thêm 3-5 nến để SMA(20) bẻ ngang $\implies$ Chậm vài nến nhưng an toàn tuyệt đối, loại bỏ 100% rủi ro bắt dao rơi giữa chừng.
2. **Edge Case 2: Bão Whipsaw quét râu 2 đầu nhưng trục giữa vẫn phẳng:**
   * *Hiện tượng:* Trục giữa phẳng nhưng nến giật 2 đầu làm loạn nhịp.
   * *Xử lý:* $w_{\text{chaos\_norm}}$ của FLAW-REG.1 sẽ vọt lên dìm $w_{\text{range\_norm}}$ xuống $< 10\%$, ngăn chặn `RangeOsc` nhảy vào bão.
3. **Edge Case 3: Nến rút chân giả tạo (Fake Rejection Wick):**
   * *Hiện tượng:* Nến rút chân nhỏ nhưng cây sau lại sập tiếp.
   * *Xử lý:* Structural SL neo dưới dải BB cộng đệm ATR bảo vệ vốn, kết hợp Cooldown Tracker cách ly ngay nếu dính SL.
