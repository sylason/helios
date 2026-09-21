# 📋 DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO): ARBITER CORE V3 (`arbiter.rs`)
> **Phân hệ:** Signal Arbiter & Two-Stage Auction Engine — Helios Trading System  
> **Tài liệu tham chiếu:** [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/04_arbiter/technical_audit.md), [`00_arbiter_rr_fallacy.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/00_arbiter_rr_fallacy.md)  
> **Trạng thái:** Sẵn sàng triển khai (Ready for Implementation)

---

## 🧭 1. MA TRẬN TỔNG HỢP LIÊN ĐỚI TỪ CÁC PHÂN HỆ TRƯỚC

Dưới đây là bảng tổng hợp toàn bộ các task liên quan đến Arbiter được phát hiện và chuyển giao từ các đợt kiểm toán `01_absorption`, `02_momentum_ignition`, `03_reversal`, `14_cross_audit` và `00_arbiter_rr_fallacy`:

| Nguồn Chuyển Giao | Mã Task Gốc | Mô Tả Vấn Đề Gốc | Vị Trí Dòng Code |
| :--- | :--- | :--- | :--- |
| `00_arbiter_rr_fallacy.md`<br>`01_absorption/TODO.md`<br>`02_momentum/TODO.md` | `TASK-ABS-03`<br>`TODO-P0-01` | **Bẫy cướp thầu ngược (Adverse Bid-Sniping):** Arbiter ưu tiên lệnh có SL gần hơn (`dist_new < dist_curr`), gạt bỏ Scanner có Confidence cao hơn ($0.90$ bị thua $0.50$ chỉ vì SL hẹp hơn \$1). | [`arbiter.rs:173–183`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L173-L183) |
| `00_arbiter_rr_fallacy.md`<br>`04_arbiter/TODO.md` | `TASK-ARB-02` | **Lỗi thoát sớm phụ thuộc thứ tự duyệt mảng (Loop Order Dependency):** Khi gặp 2 lệnh ngược chiều hòa nhau, Arbiter gọi `return` ngay lập tức, bỏ rơi toàn bộ các Scanner đứng phía sau trong mảng dù chúng có Confidence cực cao. | [`arbiter.rs:184–202`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L184-L202) |
| `03_reversal/TODO.md`<br>`technical_audit.md` | `REV-02`<br>`TASK-ARB-03` | **Nghịch lý triệt tiêu do Macro Veto:** Macro Veto cấm Long khi KAMA dốc xuống âm sâu ($\text{slope} < -0.10 \land \text{trend\_axis} \ge 0.40$). Khi thị trường sập hoảng loạn (Capitulation Dump), Reversal bị Macro Veto chặn đứng $100\%$ các điểm bắt đáy kiệt sức. | [`arbiter.rs:112–118`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L112-L118)<br>[`arbiter.rs:136–144`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L136-L144) |
| `14_cross_audit/TODO.md` | `ARB-HARDCODE`<br>`TASK-ARB-04` | **Hardcode hằng số ma thuật:** Ngưỡng dốc KAMA $0.10$, ngưỡng trend $0.40$, và ngưỡng chênh lệch phân xử ngược chiều $0.15$ bị gán cứng trong code. | [`arbiter.rs:115–116, 128, 188`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L115-L116) |
| `04_arbiter/TODO.md` | `TASK-ARB-05` | **Lãng phí cấp phát heap ở mỗi tick:** Khởi tạo `HeliosArbiter::new()` và 4 lần `Box::new(...)` lặp lại ở **mỗi cây nến 1 phút** trong hàm `next()`. | [`core_v3/mod.rs:338–346`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L338-L346) |

---

## 🚫 2. BẢNG PHÂN TÍCH VÀ VẠCH TRẦN CÁC BẪY OVER-ENGINEERING

Dưới đây là các đề xuất thiết kế ban đầu đã được bóc tách và loại bỏ theo nguyên lý gốc (**First-Principles** & **Occam's Razor**):

### ❌ 1. Bẫy nhét dải ATR của Stop Loss vào Arbiter (Trong TASK-ARB-01 cũ)
- **Ý tưởng bị vạch trần:** "Nếu Confidence bằng nhau, Arbiter xét tiếp xem khoảng cách SL có nằm trong dải $[1.0 \times \text{ATR}, 3.0 \times \text{ATR}]$ hay không..."
- **Dẫn chứng & Phản biện:**
  - Vi phạm nghiêm trọng nguyên tắc **Phân định ranh giới (Separation of Concerns)**. Bản thân từng Scanner (`trend_pb`, `momentum`, `absorption`, `reversal`) ĐÃ CÓ các gen `struct_sl_buffer_atr` hoặc `wick_sl_buffer_atr` để tính toán SL chuẩn cấu trúc của riêng nó.
  - Arbiter chỉ là người phân xử chất lượng tín hiệu (**Confidence-First**). Nhét thêm logic kiểm tra dải ATR của SL vào Arbiter sẽ đẻ thêm nhánh `if-else` phức tạp, thêm magic numbers và làm rối loạn ranh giới của Risk Manager.
- **Giải pháp tối giản:** Chỉ so sánh `confidence`. Nếu $\Delta \text{conf} \le \epsilon$ (tự tin tương đương), ưu tiên giữ nguyên đề xuất có `raw_score` cao hơn.

### ❌ 2. Bẫy đẻ thêm cờ Boolean & kiểm tra lặp lại Z-Score/Volume cho Reversal (Trong TASK-ARB-03 cũ)
- **Ý tưởng bị vạch trần:** "Bổ sung cờ cấu hình `exempt_reversal_from_veto: bool`, kiểm tra $|Z| \ge 2.5$ và kiểm tra Volume Exhaustion trong Arbiter..."
- **Dẫn chứng & Phản biện:**
  - Vi phạm nguyên tắc **DRY (Don't Repeat Yourself)**. Mô-đun `03_reversal` vừa được trang bị CLV Guard (chống nến Marubozu), MFI Filter (kiệt sức dòng tiền bán $\le 30$) và Remaining Displacement Guard $\implies$ Bản thân `ReversalScanner` **ĐÃ TỰ BẢO VỆ CHỐNG BẮT DAO RƠI**.
  - Nếu Arbiter bật Veto cho Reversal thì Reversal chết $100\%$ (minh chứng trong log `WFSS_39`: Bar 31142-31166 bị chặn đứng). Do đó thêm cờ boolean là thừa thãi.
- **Giải pháp tối giản:** Mặc định miễn trừ Macro Veto thông thường cho `ScannerId::ClimaxReversal`. Chỉ chặn khi thị trường rơi tự do ở cấp độ cuồng loạn cực đoan dị thường ($\text{slope} < -0.60 \land \text{trend\_axis} \ge 0.85$).

### ❌ 3. Bẫy bùng nổ số chiều DNA cho hằng số ma thuật (Trong TASK-ARB-04 cũ)
- **Ý tưởng bị vạch trần:** "Đưa tất cả các ngưỡng `macro_thresh`, `trend_axis_min`, `conflict_margin` vào DNA để GA tối ưu hóa..."
- **Dẫn chứng & Phản biện:**
  - Bùng nổ số chiều DNA làm không gian tìm kiếm phình to, thuật toán GA dễ bị rơi vào bẫy quá khớp (Overfitting) trên tập In-Sample.
- **Giải pháp tối giản:** Đưa các ngưỡng này vào `ArbiterConfig` dưới dạng trường cấu hình tĩnh có giá trị mặc định rõ ràng qua `serde(default = "...")`, không mở rộng thêm gen DNA không cần thiết.

### ❌ 4. Bẫy phình phạm vi: Phục hồi RangeOsc & AnomalyFade (TASK-ARB-07 cũ)
- **Ý tưởng bị vạch trần:** "Đánh giá phục hồi có kiểm soát cho `RangeOsc` & `AnomalyFade` ngay trong đợt kiểm toán Arbiter..."
- **Dẫn chứng & Phản biện:**
  - **Scope Creep (Phình phạm vi):** Việc kiểm toán và phục hồi `RangeOsc` và `AnomalyFade` thuộc về các đợt kiểm toán riêng của từng scanner đó (`05_range_osc`, `06_anomaly`). Arbiter chỉ cung cấp khung đấu thầu mở, không chịu trách nhiệm sửa chữa các scanner chưa được kiểm toán.
- **Giải pháp tối giản:** Loại bỏ task này khỏi phạm vi của `04_arbiter`.

---

## 🎯 3. DANH MỤC CÔNG VIỆC CHỐT LẠI (ACTIONABLE TODO)

| Thứ Tự | Mã Task | Tên Công Việc | Mức Độ | Trạng Thái | File Liên Quan |
| :---: | :--- | :--- | :---: | :---: | :--- |
| **1** | `TASK-ARB-01` | **Xóa bỏ R:R Fallacy, chuyển sang Confidence-First** | 🔴 **CRITICAL** | ✅ Hoàn thành | [`arbiter.rs:173–183`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L173-L183) |
| **2** | `TASK-ARB-02` | **Tái cấu trúc Two-Stage Auction (Đấu thầu 2 tầng)** | 🔴 **CRITICAL** | ✅ Hoàn thành | [`arbiter.rs:164–205`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L164-L205) |
| **3** | `TASK-ARB-03` | **Miễn trừ Macro Veto cho Climax Reversal (Giải quyết REV-02)** | 🔴 **CRITICAL** | ⏳ Chờ xử lý | [`arbiter.rs:108–118, 136–144`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L108-L118) |
| **4** | `TASK-ARB-04` | **Khử Magic Numbers, cấu hình hóa vào `ArbiterConfig`** | 🟡 **HIGH** | ⏳ Chờ xử lý | [`config.rs:300–327`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L300-L327), [`arbiter.rs:128`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L128) |
| **5** | `TASK-ARB-05` | **Tối ưu hóa Zero-Allocation cho Arbiter trong Strategy** | 🟡 **HIGH** | ⏳ Chờ xử lý | [`core_v3/mod.rs:338–346`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L338-L346) |
| **6** | `TASK-ARB-06` | **Vá an toàn `NaN` tường minh cho `compute_confidence`** | 🟢 **MEDIUM** | ⏳ Chờ xử lý | [`arbiter.rs:98–106`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L98-L106) |
| **7** | `TASK-ARB-07` | **Xây dựng Suite Unit Tests Toàn Diện Cho Arbiter** | 🔵 **TESTING** | ⏳ Chờ xử lý | [`arbiter.rs:230–298`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L230-L298) |

---

## 📋 4. CHI TIẾT KỸ THUẬT TỪNG ĐẦU VIỆC

### 🔴 TASK-ARB-01: Xóa Bỏ Quy Tắc Cướp Thầu SL Hẹp (`dist_new < dist_curr`)
- **Mã lỗi liên đới:** `VULN-ARB-01`, `TASK-ABS-03`, `TODO-P0-01`.
- **Hành động:**
  - Xóa bỏ hoàn toàn việc tính toán `dist_new < dist_curr`.
  - Thay thế bằng tiêu chí: Đề xuất có `confidence` cao hơn sẽ chiến thắng.
  - Nếu `(conf - current_conf).abs() <= 0.02` (tương đương), ưu tiên đề xuất có `raw_score` cao hơn.
- **Tiêu chuẩn nghiệm thu:**
  - [x] Proposal có `conf = 0.85` và `dist = 1.8 ATR` phải chiến thắng proposal có `conf = 0.40` và `dist = 0.5 ATR`.

---

### 🔴 TASK-ARB-02: Tái Cấu Trúc Two-Stage Auction (Đấu Thầu 2 Tầng)
- **Mã lỗi liên đới:** `VULN-ARB-02`, Loop Order Dependency.
- **Hành động:**
  - **Tầng 1 (Cùng chiều):** Tìm ra `best_long: Option<(Proposal, f64)>` và `best_short: Option<(Proposal, f64)>` tốt nhất từ toàn bộ danh sách Scanner.
  - **Tầng 2 (Đối đầu Long vs Short):** So sánh duy nhất giữa `best_long` và `best_short`:
    + Chỉ có Long $\implies$ Long thắng.
    + Chỉ có Short $\implies$ Short thắng.
    + Có cả hai: Nếu $|\text{conf}_{\text{long}} - \text{conf}_{\text{short}}| > \text{conflict\_margin}$ chọn bên mạnh hơn; nếu $\le \text{conflict\_margin}$ thì trả về Flat.
- **Tiêu chuẩn nghiệm thu:**
  - [x] Thứ tự đăng ký scanner không làm thay đổi quyết định của Arbiter (tính giao hoán $100\%$).

---

### 🔴 TASK-ARB-03: Miễn Trừ Macro Veto Cho Climax Reversal (Task REV-02)
- **Mã lỗi liên đới:** `VULN-ARB-03`, `FLAW-REV.02`, `REV-02`.
- **Hành động:**
  - Cập nhật logic trong `arbiter.rs`: Nếu `proposal.scanner_id == ScannerId::ClimaxReversal`, chỉ Veto khi xu hướng cuồng loạn cực đoan ($\text{slope}.abs() > 0.60 \land \text{trend\_axis} \ge 0.85$).
  - Các scanner khác (`TrendPullback`, `MomentumIgnition`, `Absorption`) vẫn tuân thủ Macro Veto nghiêm ngặt.
- **Tiêu chuẩn nghiệm thu:**
  - [ ] Tín hiệu ClimaxReversal không còn bị VETO trong các đợt sập/bơm có `kama_slope = 0.13 ~ 0.50` và `trend_axis = 0.62`.

---

### 🟡 TASK-ARB-04: Khử Magic Numbers, Cấu Hình Hóa Vào `ArbiterConfig`
- **Mã lỗi liên đới:** `VULN-ARB-04`, `ARB-HARDCODE`.
- **Hành động:**
  - Bổ sung vào `ArbiterConfig` trong `config.rs`:
    + `macro_veto_slope_threshold: f64` (mặc định $0.10$).
    + `macro_veto_trend_axis_min: f64` (mặc định $0.40$).
    + `conflict_confidence_margin: f64` (mặc định $0.15$).
  - Truyền các giá trị này vào `arbiter.process_tick(...)`.
- **Tiêu chuẩn nghiệm thu:**
  - [ ] Không còn bất kỳ hằng số ma thuật nào bị hardcode trong `arbiter.rs`.

---

### 🟡 TASK-ARB-05: Tối Ưu Hóa Zero-Allocation Cho Arbiter Trong Strategy
- **Mã lỗi liên đới:** `VULN-ARB-05`.
- **Hành động:**
  - Đưa `arbiter: HeliosArbiter` vào trong struct `HeliosStrategyV3`.
  - Khởi tạo và nạp scanners 1 lần duy nhất trong `HeliosStrategyV3::new(...)`.
  - Trong hàm `next()`, chỉ gọi `self.arbiter.process_tick(...)`.
- **Tiêu chuẩn nghiệm thu:**
  - [ ] Zero heap allocation ở mỗi tick cho Arbiter.

---

### 🟢 TASK-ARB-06: Vá An Toàn `NaN` Tường Minh Cho `compute_confidence`
- **Hành động:**
  ```rust
  #[inline]
  pub fn compute_confidence(raw_score: f64, chaos_index: f64) -> f64 {
      if raw_score.is_nan() || chaos_index.is_nan() || raw_score <= 0.0 {
          return 0.0;
      }
      let penalty = chaos_index.max(0.0).min(1.0);
      (raw_score * (1.0 - penalty)).max(0.0).min(1.0)
  }
  ```
- **Tiêu chuẩn nghiệm thu:**
  - [ ] `compute_confidence(f64::NAN, 0.2) == 0.0`
  - [ ] `compute_confidence(0.8, f64::NAN) == 0.0`

---

### 🔵 TASK-ARB-07: Xây Dựng Suite Unit Tests Toàn Diện Cho Arbiter
- **Danh mục test cases cần viết:**
  1. `test_confidence_first_beats_narrow_sl`: Confidence $0.85$ (SL $1.8$ ATR) thắng Confidence $0.40$ (SL $0.5$ ATR).
  2. `test_two_stage_auction_commutativity`: Hoán vị thứ tự scanner cho ra cùng kết quả.
  3. `test_macro_veto_reversal_exception`: Reversal được thông qua trong khi TrendPB bị chặn.
  4. `test_nan_safety_in_confidence`: Kiểm chứng an toàn tuyệt đối với NaN.
- **Tiêu chuẩn nghiệm thu:**
  - [ ] $100\%$ các test cases pass không có cảnh báo.
