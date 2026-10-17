# KIỂM TOÁN KIẾN TRÚC ĐỘNG CƠ ĐẤU THẦU TÍN HIỆU & PHÂN XỬ XUNG ĐỘT (ARBITER AUCTION & CONFLICT RESOLUTION AUDIT)

> **Mã tài liệu:** `NOTE-21-ARBITER-AUCTION-AUDIT`  
> **Dự án:** Helios V3 Quantitative Strategy Core  
> **Phạm vi kiểm toán:** `arbiter.rs`, `config.rs` (`ArbiterConfig`), `dna_mapping.rs`, các Tactical Scanners (`trend_pb.rs`, `momentum_ignition.rs`, `absorption.rs`, `reversal.rs`, `range_osc.rs`, `anomaly.rs`)  
> **Đối chiếu chéo:** `docs/audit_01/00_arbiter_rr_fallacy.md`, `03_arbiter/`, `04_arbiter/`, `audit_01/ROADMAP.md`, `notes/05_scanner_independence_and_regime_auction_architecture.md`  
> **Mục tiêu:** Kiểm toán toàn diện tính đúng đắn toán học của sàn đấu thầu tín hiệu Two-Stage Auction, bẫy độc tài Playbook (Mono-playbook Monopoly), chứng minh toán học về mã chết của ảo tưởng xung đột nến đơn, điểm mù bẻ lái liên nến và dọn dẹp các trường ma trong cấu hình.

---

## 🧭 1. BẢNG ĐỐI CHIẾU TRẠNG THÁI TỔNG THỂ (STATUS MAPPING)

| Mã Lỗi / Task | Vị Trí Code (File & Dòng) | Hiện Trạng Thực Tế Trong Codebase | Phân Loại & Đánh Giá |
| :--- | :--- | :--- | :---: |
| **`TODO-ARB-01`**<br>(Deadlock Nến Đơn) | `core_v3/arbiter.rs`<br>`L521-538` | Do $CLV$ chỉ có 1 dấu trên mỗi nến, các scanner không bao giờ cùng phát Long và Short. Nhánh `\|diff\| <= margin -> Flat` là **MÃ CHẾT 100%**. | 🔴 **MÃ CHẾT 100%**<br>*(Ảo tưởng đối kháng nến đơn)* |
| **`TODO-ARB-02`**<br>(Inter-bar Flip-Flop) | `core_v3/arbiter.rs`<br>`L111-158` | Arbiter hoàn toàn phi trạng thái (Stateless), không nhớ nến $i-1$. Vừa dính SL Short ở nến trước, nến sau giật rút chân duyệt Long ngay $\to$ Bị cào nát vốn trong Chop. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Bẫy vả 2 đầu liên nến)* |
| **`TODO-ARB-04`**<br>(Lệch Pha Điểm Số) | `core_v3/arbiter.rs`<br>`L150-156` | `compute_confidence` chỉ làm `raw_score.clamp(0.0, 1.0)`. Điểm thô của Absorption đạt 0.85–0.96 trong khi Reversal chỉ 0.50–0.65 $\to$ Absorption độc quyền thắng thầu. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Độc tài Playbook)* |
| **`TODO-H-04`**<br>(Score Normalization) | `core_v3/arbiter.rs`<br>`L150-156` | Chưa có chuẩn hóa phân vị xác suất nội bộ (Quantile / Z-Score Calibration) cho từng Scanner trước khi đưa vào đấu thầu. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(So sánh cam với táo)* |
| **Tight SL Fallacy**<br>(`00_arbiter_rr_fallacy`) | `core_v3/arbiter.rs`<br>`L509-517` | Đã xóa sạch $100\%$ logic `dist_new < dist_curr`. Tuy nhiên, phát sinh biến thể mới ở tầng Sniper Veto (`arbiter.rs:269-279`). | 🟢 **ĐÃ FIX TRONG AUCTION**<br>*(Chuyển sang Confidence-First)* |
| **`ZOMBIE-ARBITER-CFG`**<br>(8 Biến Ma ArbiterConfig) | `core_v3/config.rs`<br>`L450-478` | 6 dead fields cũ (`min_multiplier_veto`, `scanner_weight`, 4 `min_confidence_*`) + 2 trường bù nhìn (`max_chaos_penalty`, `conflict_confidence_margin`). | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Rác cấu hình)* |

---

## 🔬 2. MỔ XẺ CHI TIẾT CÁC LỖ HỔNG TOÁN HỌC & KIẾN TRÚC

### 2.1. Chứng Minh Toán Học: Nhánh Xung Đột Nến Đơn Là Mã Chết 100% (`TODO-ARB-01`)
- **Cơ chế hiện tại trong `arbiter.rs:521-538`:**
  ```rust
  // TẦNG 2: PHÂN XỬ LIÊN CHIỀU (INTER-SIDE FACE-OFF: LONG VS SHORT)
  match (best_long, best_short) {
      (Some(long_candidate), None) => Some(long_candidate),
      (None, Some(short_candidate)) => Some(short_candidate),
      (Some((l_prop, l_conf)), Some((s_prop, s_conf))) => {
          let diff = l_conf - s_conf;
          if diff > conflict_margin { Some((l_prop, l_conf)) }
          else if diff < -conflict_margin { Some((s_prop, s_conf)) }
          else { None /* Triệt tiêu (Flat) */ }
      }
      (None, None) => None,
  }
  ```
- **Chứng minh toán học về tính đơn dấu của CLV (Close Location Value):**
  $$CLV_i = \frac{2 \cdot \text{Close}_i - (\text{High}_i + \text{Low}_i)}{\text{High}_i - \text{Low}_i} \in [-1.0, +1.0]$$
  Trên bất kỳ cây nến đơn $i$ nào, $CLV_i$ là một đại lượng vô hướng đơn nhất.
- **Rà soát điều kiện kích hoạt của cả 4 Tactical Scanners:**
  1. `TrendPullback` (`trend_pb.rs:114, 159`):
     - Long: Bắt buộc $CLV_i \ge \text{min\_clv} > 0$ (mặc định $\ge +0.05 \sim +0.20$).
     - Short: Bắt buộc $CLV_i \le -\text{min\_clv} < 0$.
  2. `MomentumIgnition` (`momentum_ignition.rs:77-80`):
     - Long: Bắt buộc $\text{Close} > \text{Open} \land CLV_i \ge \text{min\_clv} > 0$.
     - Short: Bắt buộc $\text{Close} < \text{Open} \land CLV_i \le -\text{min\_clv} < 0$.
  3. `AbsorptionScout` (`absorption.rs:95-98`):
     - Long: Bắt buộc $CLV_i \ge \text{min\_clv} > 0 \land \%B \le \text{max\_bound} \land \text{VWMA} < \text{BB}_{\text{mid}}$.
     - Short: Bắt buộc $CLV_i \le -\text{min\_clv} < 0 \land \%B \ge 1 - \text{max\_bound} \land \text{VWMA} > \text{BB}_{\text{mid}}$.
  4. `ClimaxReversal` (`reversal.rs:83-91`):
     - Long: Bắt buộc $Z < 0 \land \text{Close} < \text{KAMA}_{\text{mid}} \land CLV_i \ge \text{min\_clv} > 0$.
     - Short: Bắt buộc $Z > 0 \land \text{Close} > \text{KAMA}_{\text{mid}} \land CLV_i \le -\text{min\_clv} < 0$.
- **Hệ quả tất yếu:**
  - Nếu $CLV_i > 0$: Không có bất kỳ Scanner nào có thể sinh tín hiệu Short $\implies \text{best\_short} = \text{None}$.
  - Nếu $CLV_i < 0$: Không có bất kỳ Scanner nào có thể sinh tín hiệu Long $\implies \text{best\_long} = \text{None}$.
  - Nếu $|CLV_i| < \text{min\_clv}$ (Nến Doji): Cả Long và Short đều bị chặn ngay từ vòng gửi xe.
  $\implies$ **Cặp `(Some(best_long), Some(best_short))` KHÔNG BAO GIỜ XUẤT HIỆN TRÊN CÙNG MỘT CÂY NẾN ĐƠN.**
  $\implies$ Toàn bộ nhánh giải quyết xung đột liên chiều `|diff| <= margin -> Flat` là **MÃ CHẾT 100% (Dead Code)**, biến biến cấu hình `conflict_confidence_margin` thành biến bù nhìn!

---

### 2.2. Điểm Mù Bẻ Lái Liên Nến (`TODO-ARB-02` - Inter-bar Flip-Flop Whipsaw)
- **Thực trạng cấu trúc `HeliosArbiter` (`arbiter.rs:111-115`):**
  ```rust
  pub struct HeliosArbiter {
      pub scanners: Vec<Box<dyn TacticalScanner>>,
  }
  ```
  `HeliosArbiter` hoàn toàn **không lưu trữ trạng thái lịch sử (Stateless)**. Nó không biết cây nến $i-1$ vừa phát tín hiệu gì, có vừa bị dính Stop Loss hay không.
- **Kịch bản thực tế bào mòn vốn:**
  Xung đột giằng co Long vs Short thực sự trong thị trường tài chính không nằm trên 1 cây nến đơn, mà diễn ra **liên nến (Inter-bar)** tại các vùng biến động hỗn loạn:
  1. Cây nến $i-1$ đỏ mạnh: Hệ thống mở vị thế Short $\to$ Cuối nến giật râu quét trúng Hard SL của lệnh Short.
  2. Cây nến $i$ xanh rút chân: $CLV > 0$, `TrendPB` hoặc `Absorption` báo tín hiệu Long.
  3. Arbiter không có bộ nhớ liên nến, thấy nến $i$ đủ điều kiện liền **phê duyệt mở Long ngay lập tức**!
  4. Cây nến $i+1$ lại đảo chiều đỏ cắm xuống $\to$ Dính tiếp Hard SL của lệnh Long.
  $\implies$ Tài khoản bị "vả 2 đầu liên hoàn" (Flip-Flop Whipsaw), cào nát vốn trong vùng nhiễu khi Gatekeeper chưa kịp đạt trần bão (`chaos_index < 0.65`).

---

### 2.3. Bẫy Độc Quyền Playbook & Lệch Pha Phân Phối Điểm Số (`TODO-ARB-04`, `TODO-H-04`)
- **Khảo sát công thức tính `raw_score` của từng Scanner:**
  1. **`AbsorptionScout` (`absorption.rs:73-76`):**
     $$\text{raw\_score} = w_{\text{sqz}} \cdot \tanh\left(\frac{\text{sqz}}{\text{thresh}}\right) + (1 - w_{\text{sqz}}) \cdot \tanh\left(\frac{\sqrt{\text{vol}/\text{spread}}}{\text{thresh}}\right)$$
     - Dùng 2 hàm $\tanh$, không có thành phần tuyến tính trừ điểm.
     - Dải điểm thực tế: Thường xuyên đạt **$0.85 \sim 0.96$**.
  2. **`TrendPullback` (`trend_pb.rs:120`):**
     $$\text{score} = 0.40 \cdot \tanh\left(\frac{|\text{slope}|}{\text{thresh}}\right) + 0.30 \cdot \text{sat\_prox}(\text{stoch}) + 0.30 \cdot \frac{CLV - \text{min\_clv}}{1.0 - \text{min\_clv}}$$
     - Bị kẹp bởi thành phần $f_{\text{clv}}$ tuyến tính (thường chỉ đạt $0.30 \sim 0.60$).
     - Dải điểm thực tế: **$0.65 \sim 0.85$**.
  3. **`MomentumIgnition` (`momentum_ignition.rs:116, 132`):**
     $$\text{raw\_score} = w_{\text{brk}} \cdot f_{\text{break}} + w_{\text{vol}} \cdot f_{\text{vol}} + w_{\text{rsi}} \cdot \tanh\left(\frac{\text{rsi\_excess}}{\mathbf{15.0}}\right)$$
     - Mẫu số cứng **$15.0$** khiến $f_{\text{rsi}}$ cực kỳ hiếm khi vượt qua $0.50$.
     - Dải điểm thực tế: **$0.70 \sim 0.78$**.
  4. **`ClimaxReversal` (`reversal.rs:106, 113`):**
     $$\text{score} = w_z \cdot f_z + w_{\text{disp}} \cdot f_{\text{disp}} + w_{\text{vol}} \cdot \text{sat\_proximity}(\text{vol\_ratio}, \mathbf{0.30})$$
     - Hàm `sat_proximity(vol_ratio, 0.30)` triệt tiêu $f_{\text{vol}} = 0.0$ khi volume bán tháo bùng nổ.
     - Dải điểm thực tế: **$0.50 \sim 0.65$**.
- **Hệ quả của việc so sánh thô trong `compute_confidence`:**
  ```rust
  pub fn compute_confidence(raw_score: f64, _chaos_index: f64, _max_chaos_penalty: f64) -> f64 {
      if raw_score.is_nan() || raw_score <= 0.0 { 0.0 } else { raw_score.clamp(0.0, 1.0) }
  }
  ```
  - Arbiter so sánh trực tiếp điểm số tuyệt đối thô mà không qua chuẩn hóa phân vị xác suất (Quantile Normalization).
  - Kết quả: `Absorption` và `Trend_PB` **độc quyền thắng thầu tuyệt đối**, cướp sạch cơ hội của `Momentum` và `Reversal`. Hệ thống rơi vào bẫy **Nghiện một Playbook duy nhất (Mono-playbook Addiction)**, biến các Scanner khác thành bù nhìn!

---

### 2.4. Dọn Dẹp 8 Biến Ma & Trường Rác Trong `ArbiterConfig`
Đối chiếu giữa `config.rs:428-478` và `arbiter.rs`:

| Tên Biến Trong `ArbiterConfig` | Tình Trạng Thực Tế | Bản Chất & Đề Xuất Dọn Dẹp |
| :--- | :---: | :--- |
| `min_confidence_threshold` | Đang dùng (`arbiter.rs:162, 290`) | **HỢP LỆ (Giữ lại):** Ngưỡng sàn duyệt tín hiệu duy nhất của Arbiter (thuộc DNA). |
| `chaos_veto_threshold` | Đang dùng ở `gatekeeper.rs` | **LỆCH PHÂN TẦNG:** Cần chuyển sang struct `GatekeeperConfig`. |
| `min_natr_threshold` | Đang dùng ở `gatekeeper.rs` | **LỆCH PHÂN TẦNG:** Cần chuyển sang struct `GatekeeperConfig`. |
| `chaos_hysteresis_delta` | Đang dùng ở `gatekeeper.rs` | **LỆCH PHÂN TẦNG:** Cần chuyển sang struct `GatekeeperConfig`. |
| `max_chaos_penalty` | **BIẾN BÙ NHÌN (`arbiter.rs:150`)** | **XÓA BỎ:** Truyền vào `_max_chaos_penalty`, hoàn toàn không dùng do đã chuyển sang Pure Confidence. |
| `conflict_confidence_margin` | **BIẾN BÙ NHÌN (`arbiter.rs:527`)** | **XÓA BỎ:** Nằm trong nhánh đối kháng Long vs Short nến đơn (vốn là mã chết 100%). |
| `min_multiplier_veto` | **DEAD FIELD (`config.rs:461`)** | **XÓA BỎ:** Tàn dư cũ của mô hình nhân phi tuyến. |
| `scanner_weight` | **DEAD FIELD (`config.rs:464`)** | **XÓA BỎ:** Không có code nào đọc biến này. |
| `min_confidence_trending` | **DEAD FIELD (`config.rs:467`)** | **XÓA BỎ:** Tàn dư cũ khi phân chia ngưỡng theo regime. |
| `min_confidence_ranging` | **DEAD FIELD (`config.rs:470`)** | **XÓA BỎ:** Tàn dư cũ. |
| `min_confidence_compression`| **DEAD FIELD (`config.rs:473`)** | **XÓA BỎ:** Tàn dư cũ. |
| `min_confidence_chaos` | **DEAD FIELD (`config.rs:476`)** | **XÓA BỎ:** Tàn dư cũ. |

---

## ⚖️ 3. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành Phần & Đề Xuất Cải Tiến | Ưu Điểm (Pros) | Khuyết Điểm (Cons) | Edge Cases Nguy Hiểm & Giải Pháp |
| :--- | :--- | :--- | :--- |
| **Xóa Bỏ Nhánh Deadlock Nến Đơn** | • Tối ưu hóa code, giảm độ phức tạp tính toán.<br>• Chuyển `resolve_conflicts` về 1 dòng `proposals.into_iter().max_by(...)`. | Không có. | **Edge Case:** Xuất hiện Scanner lai vừa Long vừa Short.<br>$\implies$ Vi phạm nguyên lý cơ bản của giao dịch phái sinh (cấm tự hedging đối ứng). |
| **Bổ Sung Inter-Bar Flip-Flop Guard** | • Khử $100\%$ bẫy "vả 2 đầu" liên nến trong vùng Whipsaw.<br>• Bảo vệ tài khoản khỏi chuỗi cắt lỗ liên hoàn. | • Có thể bỏ lỡ nhịp đảo chiều chữ V cực mạnh (V-Shape Reversal). | **Edge Case:** Tin tức giật đảo chiều chữ V thật sự $> 2.0\text{x ATR}$.<br>$\implies$ Mở ngoại lệ: Nếu nến đảo chiều có biên độ bứt phá cực đại $> 2.0\text{ ATR}$ thì cho phép phá vỡ Flip-Flop Guard. |
| **Chuẩn Hóa Phân Vị Điểm Số (Score Calibration)** | • Thiết lập sân chơi bình đẳng tuyệt đối giữa 4 Tactical Scanners.<br>• Khử hoàn toàn hiện tượng nghiện một Playbook (`Absorption`). | • Cần duy trì bộ đệm phân vị trượt (Rolling Percentile) hoặc hiệu chuẩn lại công thức thành phần của từng scanner. | **Edge Case:** Giai đoạn warmup ban đầu chưa đủ mẫu phân vị.<br>$\implies$ Fallback về điểm thô có gắn trọng số bù trừ. |
| **Dọn Sạch 8 Biến Ma Trong ArbiterConfig** | • Làm sạch codebase chuẩn Clean Code / SOLID.<br>• Phân định ranh giới rõ ràng giữa Gatekeeper và Arbiter. | • Cần cập nhật cấu hình deserialization JSON để đảm bảo tính tương thích ngược. | **Edge Case:** File JSON cấu hình cũ chứa các trường bị xóa.<br>$\implies$ Sử dụng `#[serde(default)]` hoặc `ignore_unknown_fields`. |

---

## 📋 4. DANH MỤC CÔNG VIỆC CẦN TRIỂN KHAI (TODO ACTION ITEMS)

- [ ] **Task ARB-01:** Đơn giản hóa `resolve_conflicts` trong `arbiter.rs:494-539`: Xóa bỏ nhánh đối kháng nến đơn Long vs Short (mã chết 100%), chuyển sang tìm kiếm cực đại theo chiều đơn nhất `proposals.into_iter().max_by(...)`.
- [ ] **Task ARB-02:** Triển khai `InterBarFlipFlopGuard` vào `HeliosArbiter` hoặc `core_v3/mod.rs`: Ghi nhớ chiều lệnh và thời điểm dính Stop Loss gần nhất, cấm bẻ lái mở lệnh ngược chiều trong vòng $K=2$ nến trừ khi có nến xung lực bứt phá $> 2.0\text{ ATR}$.
- [ ] **Task ARB-03:** Hiệu chuẩn lại thang điểm `raw_score`: Sửa mẫu số cứng $15.0$ trong `momentum_ignition.rs:132` và sửa hàm volume bán tháo trong `reversal.rs:105-107` để dải điểm phân bố đồng đều trong $[0.65, 0.95]$.
- [ ] **Task ARB-04:** Dọn dẹp triệt để 8 trường ma và dead fields khỏi `ArbiterConfig` trong `config.rs:450-478`, chuyển các trường thuộc về Cầu Dao sang `GatekeeperConfig`.
