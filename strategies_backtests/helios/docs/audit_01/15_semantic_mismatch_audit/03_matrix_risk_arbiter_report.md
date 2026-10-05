# BÁO CÁO KIỂM TOÁN CHUYÊN SÂU: MA TRẬN PHÒNG NGỰ, TRỌNG TÀI & QUẢN TRỊ RỦI RO

> **Chuyên đề:** Semantic Mismatch & Cross-Scale Pollution Audit (`matrix.rs`, `arbiter.rs`, `risk.rs`, `config.rs`)  
> **Người thực hiện:** Chuyên gia Kiểm toán Ma Trận Phòng Ngự & Quản Trị Rủi Ro (Auditor Subagent)  
> **Hệ sinh thái:** Helios V3.3 Pure Rust Quant Architecture

---

## 1. Executive Summary (Tóm Tắt Điều Hành)

Sau khi rà soát độc lập và đối chiếu từng dòng mã nguồn giữa các tầng **Giác quan (`sensors.rs`)**, **Ma trận (`matrix.rs`)**, **Trọng tài (`arbiter.rs`)** và **Quản trị rủi ro (`risk.rs`)**, kiểm toán phát hiện **4 lỗ hổng nghiêm trọng (CRITICAL/HIGH)** xuất phát từ bẫy **Semantic Mismatch Trap** (không tương thích giữa thang đo biến động, ô nhiễm chu kỳ đa khung thời gian và xung đột thứ nguyên).

Các điểm nghẽn nguy hiểm nhất bao gồm:
1. **Bẫy Break-Even tự sát (Self-Destructive BE Trap):** Nhánh kích hoạt BE theo khoảng cách ATR vi mô (`peak_profit >= required_profit`) kích hoạt quá sớm khi lợi nhuận chưa đủ bù phí sàn 2 chiều, kích hoạt tiếp điều kiện `Emergency BE Rejection Close` ép bot bán tống bán tháo vị thế chịu lỗ phí sàn.
2. **Khuyết tật thang đo Trailing Stop ($\sqrt{T}$ Scale Instability):** Gen `atr.timeframe` được thả nổi (`1m, 3m, 5m`), khiến giá trị ATR chênh lệch tới $\sqrt{5} \approx 2.236$ lần. Một khoảng cách TSL 2.0 ATR trên nến 1m ($0.2\%$) bị quét sạch lập tức, trong khi trên nến 5m ($0.45\%$) lại hoạt động bình thường, gây vỡ không gian tối ưu hóa GA.
3. **Mù bách phân vị nén biến động (Blind Squeeze Flatline 0.50):** Khi Bollinger Bands chọn khung 4h (240 nến 1m/bar), việc dùng cửa sổ trượt `sqz_lb` từ 100 nến 1m khiến toàn bộ dữ liệu trong cửa sổ bị trùng lặp, ép `rolling_percentile` trả về hằng số chết `0.50`, làm tê liệt toàn bộ bộ lọc Chop Zone và Absorption.
4. **Lệch pha thời gian (Temporal Desynchronization) trong `chaos_index`:** Ghép nối cú sốc biến động 4h (`chaos_ratio`) với độ nhiễu 15m (`efficiency_ratio`) tạo ra tín hiệu bão ảo kéo dài hàng tiếng đồng hồ sau khi thị trường đã êm đềm.

---

## 2. 🔍 PHẦN 1: KIỂM TOÁN CHI TIẾT `matrix.rs` & `sensors.rs`

### 2.1. `kama_slope_atr`: Ô Nhiễm Chu Kỳ Vi Mô Vào Độ Dốc Vĩ Mô
* **Vị trí code:** 
  - `core_v3/mod.rs` (dòng 187-193):
    ```rust
    let (k_slope, k_slope_adj) = ctx.kama_slope(
        ind.kama.timeframe, "KAMA_Slope", 
        ind.kama.short_period, 2, ind.kama.long_period, 
        ind.kama.short_period, ind.atr.period
    );
    ```
  - `ferro_trade/src/indicators/calculate/bulk.rs` (dòng 1592-1607):
    ```rust
    let kama_series = kama(closes, er_period, fast_period, slow_period)?;
    let atr_series = atr(highs, lows, closes, atr_period)?;
    let slope = (current_kama - prev_kama) / current_atr;
    adj_slope[i] = slope * current_er;
    ```
  - `matrix.rs` (dòng 87-90):
    ```rust
    let scale_atr = if config.matrix.kama_slope_scale_atr <= 0.0 { 0.20 } else { config.matrix.kama_slope_scale_atr };
    let kama_slope_atr: Vec<f64> = sensors.kama_slope_adj.iter().map(|&s| {
        if s.is_finite() { (s / scale_atr).clamp(-1.0, 1.0) } else { 0.0 }
    }).collect();
    ```
* **Bản chất toán học & Dòng dữ liệu:**
  - ATR dùng để chia trong hàm `calculate::bulk::kama_slope` được tính trên chuỗi nến của `ind.kama.timeframe` (ví dụ 15m, 1h, 4h). Tuy nhiên, chu kỳ tính toán `atr_period` lại bị lấy từ `ind.atr.period` (thuộc struct `AtrParams` vốn dành cho `atr_sl` vi mô 1m/3m/5m).
  - Biến thiên tử số `current_kama - prev_kama` được đo qua bước nhảy `slope_period = ind.kama.short_period` nến MTF (ví dụ 10 nến 1h = 10 tiếng). Mẫu số lại là ATR của 1 nến MTF (1 tiếng).
* **Vấn đề Semantic Mismatch:**
  - Khi GA tinh chỉnh `ind.atr.period` nhằm thay đổi độ co giãn của Trailing Stop Loss vi mô, nó **vô tình làm thay đổi toàn bộ thước đo độ dốc KAMA vĩ mô** ở Market Matrix. Điều này vi phạm nguyên tắc bất khả xâm phạm: **Clean Data Lineage & No Cross-Pollution** (GEMINI.md mục 6.5).
  - Tử số là biến thiên qua $N$ nến MTF, mẫu số là ATR của 1 nến MTF, tạo ra giá trị $slope$ có độ lớn tự nhiên từ $1.0 \to 5.0$, sau đó nhân $ER$ rồi lại chia tiếp cho `scale_atr` (0.05 -> 0.50, tương đương nhân $2 \to 20$), dẫn đến giá trị bị đụng trần kẹp biên $[-1.0, 1.0]$ quá sớm, mất độ mịn của tín hiệu xu hướng.

### 2.2. `squeeze_intensity`: Hiện Tượng "Mù Bách Phân Vị" (Blind Percentile Flatline 0.50)
* **Vị trí code:**
  - `core_v3/mod.rs` (dòng 201-207): Keltner Channel đã được ép đồng bộ `ind.bbands.timeframe`.
  - `matrix.rs` (dòng 34-63, 81):
    ```rust
    let sqz_continuous = if kc_w > 1e-9 && bb_w.is_finite() {
        let ratio = (bb_w.max(0.0) / kc_w).max(0.0);
        let val = 1.0 / (1.0 + ratio * ratio);
        val.clamp(0.0, 1.0)
    } else { 0.0 };
    sqz_raw.push(sqz_continuous);
    let squeeze_intensity = normalization::rolling_percentile(&sqz_raw, sqz_lb);
    ```
* **Bản chất toán học & Dòng dữ liệu:**
  - Hàm `map_indicator_to_base_tf` rải đều các giá trị nến MTF ra nến cơ sở 1m dưới dạng bậc thang (Step Function).
  - Nếu `ind.bbands.timeframe` chọn khung **4h**, 1 nến 4h tương đương 240 nến 1m. Trong suốt 240 nến 1m đó, `bb_w` và `kc_w` giữ nguyên giá trị không đổi 100%.
  - Cấu hình `squeeze_lookback` có dải gen `[100..1000]` nến 1m. Nếu GA chọn `sqz_lb = 120` nến: Cửa sổ trượt 120 nến 1m chỉ chứa đúng một nửa cây nến 4h!
  - Trong hàm `normalization::rolling_percentile`:
    $$\text{result}[i] = \frac{\text{count\_smaller} + 0.5 \times \text{count\_equal}}{\text{valid\_window\_count}} = \frac{0 + 0.5 \times W}{W} = 0.50$$
* **Hậu quả thực tế:**
  - Cường độ nén `squeeze_intensity` bị kẹt cứng ở mức **0.50** (50%) liên tục trong nhiều tiếng đồng hồ.
  - Bộ lọc Chop Zone trong `TrendPB` (`squeeze > pb_cfg.max_squeeze_intensity`) và bộ kích hoạt `Absorption` (`squeeze >= sqz_thresh`) hoàn toàn bị mù, không nhận biết được thị trường đang nén chặt hay bung toạc.

### 2.3. `chaos_index` vs `efficiency_ratio`: Lệch Pha Động Lực Học Thời Gian (Temporal Aliasing)
* **Vị trí code:**
  - `core_v3/config.rs`: `ind.kama.timeframe` chọn trong `[15m, 1h, 4h]`, `ind.natr.timeframe` chọn độc lập trong `[15m, 1h, 4h]`.
  - `core_v3/mod.rs` (dòng 176-177):
    ```rust
    s.chaos_ratio = ctx.chaos_ratio(ind.natr.timeframe, "ChaosRatio", ind.natr.period);
    s.efficiency_ratio = ctx.efficiency_ratio(ind.kama.timeframe, "ER", ind.kama.short_period);
    ```
  - `matrix.rs` (dòng 92-107):
    ```rust
    let c_r = c_r_raw.clamp(0.0, 1.0);
    let noise = (1.0 - ent_raw.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    chaos_index = c_r * noise;
    ```
* **Bản chất toán học & Dòng dữ liệu:**
  - Nếu GA chọn `natr.timeframe = 4h` và `kama.timeframe = 15m`:
    + `chaos_ratio` đo độ biến động trên nến 4h so với SMA 50 nến 4h (chu kỳ 200 giờ). Giá trị này giữ nguyên suốt 240 nến 1m.
    + `efficiency_ratio` đo độ mượt mà của giá đóng cửa trên nến 15m trong chu kỳ ngắn (ví dụ 10 nến 15m = 2.5 giờ).
* **Hậu quả thực tế:**
  - Khi một cây nến 4h trước đó có biến động mạnh (ví dụ tin tức giật 1 cây rồi kết thúc), `chaos_ratio` vọt lên cao và giữ nguyên trong 4 giờ kế tiếp.
  - Trong 4 giờ đó, giá trên khung 15m đi vào trạng thái nén tích lũy rất êm đềm ($ER \to 0 \implies noise \to 1.0$).
  - Phép nhân $c\_r \times noise$ lập tức vọt lên $> 0.70$, kích hoạt cầu dao Gatekeeper (`chaos_veto_threshold`) và **khóa chặt toàn bộ hệ thống bot không cho giao dịch suốt 4 tiếng**, dù điều kiện thực tế trên khung tác chiến vi mô đang là cơ hội Pullback tuyệt hảo!
  - Ngược lại, nếu Flash Crash xảy ra trong 15 phút, nến 4h chưa đóng nên `chaos_ratio` 4h hoàn toàn bất động, bot lao vào bắt dao rơi vì Ma Trận báo "thị trường bình yên".

---

## 3. ⚖️ PHẦN 2: KIỂM TOÁN CHI TIẾT `arbiter.rs`

### 3.1. Thang Đo Điểm Tự Tin & Nguy Cơ "Playbook Cannibalization"
* **Vị trí code:** `arbiter.rs` (dòng 150-156, 208-209, 385-489) kết hợp với các scanner `scanners/*.rs`.
* **Phân tích thang đo:**
  - Trong mô hình Pure Confidence hiện tại, `compute_confidence(raw_score) = raw_score.clamp(0.0, 1.0)`.
  - Mọi Scanner đều sử dụng tổ hợp lồi `convex_combine` trên các hàm bão hòa `sat_*` từ `normalization.rs`, đảm bảo $raw\_score \in [0.0, 1.0]$.
* **Điểm lệch độ lớn thực tế (Scale Asymmetry):**
  - **`AbsorptionScout`:** Thành phần Volume sử dụng `sat_ratio_damped(raw_vol_z, norm_spread, vol_thresh)`. Khi nến nén thân hẹp ($norm\_spread = 0.10\text{ ATR}$) và Volume nổ nhẹ ($vol\_z = 2.5$), tỉ số $ratio = 25.0 \implies \sqrt{ratio} = 5.0$. Giá trị $\tanh(5.0 / 1.0) \approx 0.9999$. Điểm số của Absorption rất dễ đạt $0.85 \to 0.95$.
  - **`TrendPB`:** Để đạt điểm $0.85$, giá phải tiệm cận sát sạt đường KAMA ($delta\_kama \approx 0$), Stochastic chạm đáy ($stoch \approx 0$) và độ dốc KAMA phải gấp đôi ngưỡng kích hoạt. Điểm trung bình của TrendPB thường chỉ dao động trong khoảng $0.60 \to 0.78$.
  - **Hệ quả trong Đấu thầu Two-Stage Auction:**
    Khi cùng xuất hiện tín hiệu thuận chiều hoặc đối đầu, `Absorption` luôn sở hữu lợi thế điểm số áp đảo nhân tạo, đè bẹp và cướp quyền của `TrendPB` bất kể bối cảnh xu hướng vĩ mô đang thuộc về TrendPB.

### 3.2. Vấn Đề Ảo Ảnh Cấu Hình (Ghost / Orphan Parameters)
* **Vị trí code:** `core_v3/config.rs` dòng 346-374; `arbiter.rs` dòng 150-156, 196.
* **Chi tiết:**
  - Trong `ArbiterConfig`, các trường như `scanner_weight`, `max_chaos_penalty`, `min_confidence_trending`, `min_confidence_ranging` vẫn tồn tại và nhận giá trị mặc định.
  - Tuy nhiên, trong hàm `compute_confidence(raw_score, _chaos_index, _max_chaos_penalty)`, các tham số này đã bị gạch dưới `_` và loại bỏ hoàn toàn khỏi biểu thức toán học.
  - Sự tồn tại của các trường mồ côi này gây hiểu lầm nghiêm trọng trong tài liệu và cấu hình giao diện bot, tạo cảm giác hệ thống đang áp dụng hình phạt bão mềm trong khi thực tế đã chuyển sang Hard Gate ở Gatekeeper.

---

## 4. 🛡️ PHẦN 3: KIỂM TOÁN CHI TIẾT `risk.rs`

### 4.1. Cơ Chế Trailing Stop: Biến Động ATR Lệch $\sqrt{T}$
* **Vị trí code:** `risk.rs` dòng 267-270, 511-518; `core_v3/config.rs` dòng 189-195.
  ```rust
  let current_atr = sensors.atr_sl[i];
  let raw_sl = peak_val - (current_atr * current_multiplier);
  ```
* **Bản chất toán học:**
  - `sensors.atr_sl` được nạp từ `ind.atr.timeframe` với tập giá trị cho phép là `["1m", "3m", "5m"]`.
  - Theo nguyên lý bước ngẫu nhiên (Brownian Motion), biên độ biến động tỉ lệ thuận với căn bậc hai thời gian:
    $$\text{ATR}_{5\text{m}} \approx \sqrt{5} \times \text{ATR}_{1\text{m}} \approx 2.236 \times \text{ATR}_{1\text{m}}$$
  - Các tham số Trailing Stop (`tsl_atr_normal`, `tsl_atr_tight`, `tsl_atr_loose`) có cùng miền giá trị `[0.5 -> 4.5, Step 0.10]`.
* **Hậu quả chết người:**
  - Nếu GA chọn `timeframe = 1m`: Khoảng cách TSL $2.0 \times \text{ATR}_{1\text{m}} \approx 0.18\% \to 0.25\%$ giá. Đây là khoảng cách siêu hẹp, khiến lệnh bị quét sạch bởi nhiễu vi mô chỉ sau 1-3 nến.
  - Nếu GA chọn `timeframe = 5m`: Khoảng cách TSL $2.0 \times \text{ATR}_{5\text{m}} \approx 0.40\% \to 0.55\%$ giá, tạo đủ không gian cho giá thở.
  - **Kết luận:** Việc để ngỏ `atr.timeframe` biến `tsl_atr` thành một biến số bị "thả trôi đơn vị đo". Không gian tham số của Optimizer bị phân mảnh hoàn toàn vì gen 2.0 ATR ở 1m hoàn toàn khác bản chất với 2.0 ATR ở 5m!

### 4.2. Cơ Chế Break-Even: "Bẫy Tự Sát" Do Lệch Thang Đo Chi Phí (Self-Destructive BE Trap)
* **Vị trí code:** `risk.rs` (dòng 426-432, 521-535).
  ```rust
  let min_safe_be_roe = (fee_buffer_pct * leverage * 1.5).max(risk_cfg.base_be_roe_trigger.max(0.06));
  let required_profit = current_atr * be_trigger_mult * trade.size.abs();
  let is_be_triggered = peak_roe >= min_safe_be_roe || peak_profit >= required_profit;
  ```
  Và khi thực thi dời SL hòa vốn:
  ```rust
  if is_be_triggered && !is_mega_trend {
      if trade.is_long() {
          let min_be = trade.entry_price * (1.0 + fee_buffer_pct);
          if current_close <= min_be {
              tick.close(None); // 🛑 EMERGENCY BE REJECTION EXIT!
              continue;
          }
          ...
      }
  }
  ```
* **Bản chất toán học & Kịch bản sập bẫy:**
  1. Giả sử đòn bẩy $20\text{x}$, phí sàn 2 chiều + trượt giá $fee\_buffer\_pct = 0.10\%$. Ngưỡng an toàn bảo vệ phí sàn $min\_safe\_be\_roe = 6.0\%\text{ ROE}$ (tương đương giá phải tăng ít nhất $+0.30\%$).
  2. Tuy nhiên, nhánh điều kiện thứ hai: `peak_profit >= required_profit` $\iff \Delta P_{peak} \ge current\_atr \times be\_trigger\_mult$.
  3. Nếu `current_atr` là ATR 1m (ví dụ $0.02\%$ thị giá) và $be\_trigger\_mult = 1.0$:
     - Chỉ cần giá nhích nhẹ $+0.02\%$, điều kiện `peak_profit >= required_profit` **lập tức thỏa mãn**!
     - `is_be_triggered` chuyển thành `true`!
  4. Ngay tại nến đó hoặc nến kế tiếp:
     - Giá thị trường đang ở mức $+0.02\%$, trong khi $min\_be$ đòi hỏi giá phải ở mức $+0.10\%$ ($entry \times 1.0010$).
     - Điều kiện $current\_close \le min\_be$ hiển nhiên ĐÚNG!
     - **Hệ thống kích hoạt ngay lập tức `EMERGENCY BE REJECTION EXIT` và đóng lệnh khẩn cấp, chịu lỗ sòng phẳng toàn bộ chi phí sàn $0.08\%$!**
* **Mức độ nghiêm trọng: CRITICAL.** Lỗ hổng này biến cơ chế bảo vệ vốn thành cỗ máy tự động bào mòn tài khoản khi giao dịch ở khung nến vi mô.

### 4.3. Stagnant Exit & Underwater Tightening: Đếm Nến 1m Bóp Nghẹt Sóng MTF
* **Vị trí code:** `risk.rs` (dòng 336-361, 642-685).
* **Phân tích:**
  - Biến `bars_held = i.saturating_sub(entry_bar)` đếm số nến 1m thuần túy của Backtest Engine.
  - Cấu hình `max_stagnant_hold_bars` có trần tối đa là 720 nến (tương đương 12 giờ).
  - Một chiến lược theo xu hướng bậc thang vĩ mô (TrendPB trên KAMA 4h hoặc 1h) cần từ 24 đến 72 giờ để hoàn thành một chu kỳ sóng.
  - Sau 12 giờ (chỉ mới xuất hiện đúng 3 cây nến 4h), lệnh đã bị gán nhãn "ngâm vốn" và bị cưỡng bức đóng nếu chưa đạt `stagnant_min_profit_pct`.
  - Tương tự trong `underwater_cycles = bars_held / time_countdown_bars`: Nếu $time\_countdown\_bars = 15$ nến 1m, cứ sau 15 phút SL lại bị siết hẹp một lần, dẫn đến việc chạm giá hiện tại và force close (`should_force_close`) chỉ sau 1-2 tiếng ngâm lệnh âm nhẹ, tước đoạt hoàn toàn khả năng phục hồi của các vị thế Swing.

---

## 5. 📊 BẢNG TỔNG KẾT MA TRẬN KIỂM TOÁN (AUDIT MATRIX)

| Vị Trí (File & Dòng) | Phép Tính / Biến Số | Timeframe / Đơn Vị Vào | Ngưỡng / Thang Đo Đối Ứng | Vấn Đề Lệch Ngữ Nghĩa & Hậu Quả | Mức Độ | Đề Xuất Giải Pháp |
| :--- | :--- | :--- | :--- | :--- | :---: | :--- |
| **`risk.rs`**<br>`L426-432`<br>`L521-535` | `is_be_triggered = peak_roe >= min_safe_be_roe \|\| peak_profit >= required_profit` | `current_atr` (1m/3m/5m) $\times$ `trade.size` | `fee_buffer_pct` ($0.10\%$) vs `min_be` | **Self-Destructive BE Trap:** ATR 1m quá nhỏ kích hoạt BE khi lãi mới $+0.02\%$, không đủ bù $0.10\%$ phí sàn, kích hoạt Emergency Exit bán tháo chịu lỗ phí. | 🔴 **CRITICAL** | Loại bỏ hoàn toàn nhánh `peak_profit >= required_profit` hoặc ràng buộc bắt buộc: `peak_profit >= required_profit.max(entry * fee_buffer_pct * size * 1.5)`. |
| **`risk.rs`**<br>`L267-270`<br>`L511-518` | `raw_sl = peak_val - (current_atr * tsl_mult)` | `sensors.atr_sl`<br>(chọn 1m, 3m hoặc 5m) | `tsl_atr_normal`<br>($[0.5, 4.5]$ ATR) | **$\sqrt{T}$ Scale Instability:** ATR 5m gấp $2.24$ lần ATR 1m khiến cùng một khoảng cách 2.0 ATR bị lệch bản chất từ siêu hẹp ($0.2\%$) sang bình thường ($0.45\%$). | 🟠 **HIGH** | Khóa cứng `atr.timeframe = Timeframe::T1m` hoặc chuẩn hóa quy đổi mọi ATR về tương đương nến 1m: $\text{ATR}_{\text{norm}} = \text{ATR}_{T} / \sqrt{T}$. |
| **`matrix.rs`**<br>`L81`<br>`L55-63` | `squeeze_intensity = rolling_percentile(sqz_raw, sqz_lb)` | `sensors.bb_upper`, `kc_upper` (khung 4h rải ra 1m) | `sqz_lb` ($[100, 1000]$ nến 1m) | **Blind Squeeze Flatline:** Cửa sổ 100 nến 1m chỉ chứa nửa nến 4h, mọi giá trị bằng nhau khiến bách phân vị kẹt cứng ở hằng số chết $0.50$, làm mù bộ lọc Chop Zone. | 🟠 **HIGH** | Tính `squeeze_intensity` trên chuỗi nến nén gốc MTF trước khi rải mảng, hoặc buộc `sqz_lb \ge 2 \times \text{MTF\_bars}`. |
| **`matrix.rs`**<br>`L92-107`<br>`core_v3/mod.rs` | `chaos_index = c_r * (1.0 - entropy)` | `chaos_ratio` (4h) vs `efficiency_ratio` (15m) | `chaos_veto_threshold` ($0.50 \to 0.85$) | **Temporal Desynchronization:** Cú sốc nến 4h duy trì 4 tiếng kết hợp với độ nén 15m tạo bão giả, kích hoạt cầu dao VETO cấm giao dịch oan uổng. | 🟠 **HIGH** | Đồng bộ hóa `natr.timeframe` phải bằng `kama.timeframe` để đảm bảo tính cùng hệ quy chiếu thời gian. |
| **`core_v3/mod.rs`**<br>`L187-193`<br>`matrix.rs: L87` | `kama_slope(..., ind.atr.period)` | `ind.atr.period` (vốn của SL 1m) áp cho nến KAMA (1h/4h) | `scale_atr` ($0.05 \to 0.50$) | **Cross-Pollution:** Tinh chỉnh chu kỳ SL vi mô làm méo mó độ dốc KAMA vĩ mô; tử số $N$ nến chia mẫu số 1 nến làm giá trị chạm trần $[-1.0, 1.0]$ quá sớm. | 🟡 **MEDIUM** | Tách riêng chu kỳ ATR nội bộ của KAMA: dùng chính `kama.short_period` làm chu kỳ ATR của KAMA, không dùng chung `ind.atr.period`. |
| **`scanners/`**<br>`absorption`<br>`vs trend_pb` | `sat_ratio_damped` (Absorption) vs `sat_proximity` (TrendPB) | Dimensionless $[0.0, 1.0]$ | `final_confidence` đấu thầu Two-Stage Auction | **Playbook Cannibalization:** Absorption dễ đạt điểm $0.90+$ khi nến thân hẹp, trong khi TrendPB hiếm khi vượt $0.80$, dẫn đến Absorption cướp hết quyền vào lệnh. | 🟡 **MEDIUM** | Cân bằng lại hàm bão hòa của Absorption hoặc áp dụng chuẩn hóa phân vị (Rank/Percentile) cho từng Playbook trước khi đấu thầu. |
| **`risk.rs`**<br>`L336-361`<br>`L642-685` | `bars_held = i - entry_bar`<br>`underwater_cycles = bars / cd` | Nến 1m của Backtest Engine | `max_stagnant_hold_bars` ($60 \to 720$ nến 1m) | **Swing Premature Termination:** Đếm nến 1m để giải phóng vốn ngâm (tối đa 12h) bóp chết các vị thế Trend dài hạn trên khung 1h/4h (vốn cần 24-48h). | 🟡 **MEDIUM** | Quy đổi `bars_held` và `time_countdown_bars` theo số nến của Playbook tương ứng (nhân hệ số timeframe). |
| **`arbiter.rs`**<br>`L150-156` | `compute_confidence(..., _chaos, _max_penalty)` | `raw_score` | `_max_chaos_penalty` (No-Op) | **Ghost Configuration:** Cấu hình vẫn lưu các tham số hình phạt bão mềm cũ nhưng code bên trong đã bỏ qua hoàn toàn. | 🟢 **LOW** | Dọn sạch các trường mồ côi khỏi `ArbiterConfig` và signature của hàm để code phản ánh đúng thực tế. |

---

## 6. 🛠️ KIẾN NGHỊ HÀNH ĐỘNG CẤP BÁCH (ACTIONABLE ROADMAP)

1. **Khắc phục ngay lập tức Bẫy Break-Even tự sát trong `risk.rs` (Ưu tiên P0):**
   Sửa điều kiện kích hoạt Break-Even tại dòng 431:
   ```rust
   // ĐẢM BẢO LỢI NHUẬN TỐI THIỂU PHẢI VƯỢT TRẦN CHI PHÍ SÀN TRƯỚC KHI BẬT BE
   let min_profit_fee_floor = trade.entry_price * fee_buffer_pct * 1.5 * trade.size.abs();
   let is_be_triggered = peak_roe >= min_safe_be_roe || (peak_profit >= required_profit && peak_profit >= min_profit_fee_floor);
   ```
2. **Khóa cứng đơn vị đo cho Trailing Stop (Ưu tiên P0):**
   Trong `core_v3/config.rs`, khóa cứng `AtrParams.timeframe = Timeframe::T1m` (hoặc loại bỏ gen này khỏi DNA), đảm bảo mọi tham số `tsl_atr_*` luôn được tính toán trên nến 1m chuẩn xác và bất biến trên mọi asset.
3. **Đồng bộ hóa Timeframe giữa NATR và KAMA (Ưu tiên P1):**
   Ràng buộc `ind.natr.timeframe = ind.kama.timeframe` trong hàm `init()` và `update_from_dna()` để triệt tiêu hiện tượng lệch pha thời gian trong `chaos_index`.
4. **Cô lập chu kỳ ATR của KAMA Slope (Ưu tiên P1):**
   Trong lời gọi `ctx.kama_slope`, thay thế `ind.atr.period` bằng chính `ind.kama.short_period` để đảm bảo KAMA Slope là một thực thể khép kín, độc lập hoàn toàn với cấu hình SL vi mô của Risk Manager.
