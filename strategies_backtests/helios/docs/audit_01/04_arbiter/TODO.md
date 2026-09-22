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
| **3** | `TASK-ARB-03` | **Xóa bỏ Macro Veto trùng lặp khỏi Arbiter (Giải quyết REV-02)** | 🔴 **CRITICAL** | ✅ Hoàn thành | [`arbiter.rs:95–160`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L95-L160) |
| **4** | `TASK-ARB-04` | **Khử Magic Numbers, cấu hình hóa vào `ArbiterConfig`** | 🟡 **HIGH** | ✅ Hoàn thành | [`config.rs:300–335`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs#L300-L335), [`arbiter.rs:122`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L122) |
| **5** | `TASK-ARB-05` | **Tối ưu hóa Zero-Allocation cho Arbiter trong Strategy** | 🟡 **HIGH** | ✅ Hoàn thành | [`core_v3/mod.rs:338–346`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/mod.rs#L338-L346) |
| **6** | `TASK-ARB-06` | **Vá an toàn `NaN` tường minh cho `compute_confidence`** | 🟢 **MEDIUM** | ✅ Hoàn thành | [`arbiter.rs:99–108`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L99-L108) |
| **7** | `TASK-ARB-07` | **Xây dựng Suite Unit Tests Toàn Diện Cho Arbiter** | 🔵 **TESTING** | ✅ Hoàn thành | [`arbiter.rs:220–360`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs#L220-L360) |

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

### 🔴 TASK-ARB-03: Xóa Bỏ Macro Veto Trùng Lặp Khỏi Arbiter (Giải Quyết REV-02)
- **Mã lỗi liên đới:** `VULN-ARB-03`, `FLAW-REV.02`, `REV-02`.
- **Hành động:**
  - Xóa bỏ hoàn toàn hàm `is_direction_vetoed` và lời gọi kiểm tra trong `process_tick` của `arbiter.rs`.
  - Trả lại quyền kiểm soát xu hướng cho từng Scanner: `TrendPB`, `Momentum`, `Absorption` đều đã có trend guard nội tại, trong khi `Reversal` được bảo vệ bởi CLV Guard và MFI Filter.
  - Lưu ý liên đới: Dị tật `chaos_index` (xem Mục 5) cần được làm mềm để không bóp chết `ClimaxReversal` khi thị trường có biến động mạnh.
- **Tiêu chuẩn nghiệm thu:**
  - [x] `arbiter.rs` không còn bất kỳ dòng code nào cấm mở Long/Short theo hướng vĩ mô (thuần túy làm nhiệm vụ đấu thầu Two-Stage Auction).
  - [x] Tín hiệu ClimaxReversal không còn bị VETO trong các đợt sập/bơm có `kama_slope = 0.13 ~ 0.50` và `trend_axis = 0.62`.

---

### 🟡 TASK-ARB-04: Khử Magic Numbers, Cấu Hình Hóa Vào `ArbiterConfig`
- **Mã lỗi liên đới:** `VULN-ARB-04`, `ARB-HARDCODE`.
- **Hành động:**
  - Bổ sung vào `ArbiterConfig` trong `config.rs`:
    + `conflict_confidence_margin: f64` (mặc định $0.15$, biên độ phân xử Long vs Short).
  - Loại bỏ các magic numbers `macro_thresh = 0.10` và `trend_axis >= 0.40` không còn sử dụng sau khi gỡ bỏ `is_direction_vetoed`.
- **Tiêu chuẩn nghiệm thu:**
  - [x] Không còn bất kỳ hằng số ma thuật nào bị hardcode trong `arbiter.rs`.

---

### 🟡 TASK-ARB-05: Tối Ưu Hóa Zero-Allocation Cho Arbiter Trong Strategy
- **Mã lỗi liên đới:** `VULN-ARB-05`.
- **Hành động:**
  - Đưa `arbiter: HeliosArbiter` vào trong struct `HeliosStrategyV3`.
  - Khởi tạo và nạp scanners 1 lần duy nhất trong `HeliosStrategyV3::new(...)`.
  - Trong hàm `next()`, chỉ gọi `self.arbiter.process_tick(...)`.
- **Tiêu chuẩn nghiệm thu:**
  - [x] Zero heap allocation ở mỗi tick cho Arbiter.

---

### 🟢 TASK-ARB-06: Vá An Toàn `NaN` Tường Minh Cho `compute_confidence`
- **Hành động:**
  ```rust
  #[inline]
  pub fn compute_confidence(raw_score: f64, chaos_index: f64) -> f64 {
      if raw_score.is_nan() || chaos_index.is_nan() || raw_score <= 0.0 {
          return 0.0;
      }
      let penalty = (chaos_index.max(0.0) * 0.50).min(0.50);
      (raw_score * (1.0 - penalty)).max(0.0).min(1.0)
  }
  ```
- **Tiêu chuẩn nghiệm thu:**
  - [x] `compute_confidence(f64::NAN, 0.2) == 0.0`
  - [x] `compute_confidence(0.8, f64::NAN) == 0.0`

---

### 🔵 TASK-ARB-07: Xây Dựng Suite Unit Tests Toàn Diện Cho Arbiter
- **Danh mục test cases cần viết:**
  1. `test_conflict_resolution_higher_confidence_wins`: Đã hoàn thành (Khối 1).
  2. `test_two_stage_auction_commutativity`: Đã hoàn thành (Khối 1).
  3. `test_conflict_resolution_tie_breaker_uses_raw_score`: Đã hoàn thành (Khối 1).
  4. `test_conflict_resolution_opposite_signals_flat`: Đã hoàn thành (Khối 1).
  5. `test_conflict_resolution_opposite_signals_decisive_win`: Đã hoàn thành (Khối 1).
  6. `test_nan_safety_in_confidence`: Đã hoàn thành (Khối 2).
  7. `test_soft_chaos_penalty_preserves_strong_signals`: Đã hoàn thành (Khối 2).
- **Tiêu chuẩn nghiệm thu:**
  - [x] $100\%$ các test cases pass không có cảnh báo.

---

## 🧭 5. BẢNG TỔNG KẾT 5 ĐIỂM YẾU CỐT TỬ CỦA ARBITER (PHÁT HIỆN BỔ SUNG)

| STT | Điểm Yếu Cốt Tử | Bản Chất Gốc Rễ | Mức Độ | Trạng Thái |
| :---: | :--- | :--- | :---: | :---: |
| **1** | **Ảo tưởng Trọng số Playbook (The Phantom DNA Trap)** | Arbiter đấu thầu 1:1 thô, không có DNA `playbook_weight` điều tiết; Scanner chấm dễ dãi sẽ luôn đè bẹp Scanner khắt khe. | 🔴 **CRITICAL** | ✅ Hoàn thành (Unified Normalization Engine) |
| **2** | **Dị tật `chaos_index`** | Đánh đồng biến động mạnh (NATR cao, Z cao) với bão Whipsaw; bóp chết cả Momentum và Reversal khi có sóng mạnh. | 🔴 **CRITICAL** | ✅ Hoàn thành (`TASK-MAT-10`: ER-Based `chaos_index`) |
| **3** | **Phạt Tuyến Tính Bờ Vực (Linear Penalty Cliff)** | $(1 - \text{chaos})$ dìm chết $100\%$ tín hiệu khi chaos $\ge 0.80$ dù setup hoàn hảo. | 🟡 **HIGH** | ⏳ Lên kế hoạch xử lý sau |
| **4** | **Trùng lặp 2 tầng cổng lọc** | `min_activation_score` nội tại của Scanner dẫm chân lên `min_confidence_threshold` tĩnh của Arbiter. | 🟡 **HIGH** | ⏳ Lên kế hoạch xử lý sau |
| **5** | **Lãng phí Heap Allocation mỗi tick** | `Box::new()` 4 lần/nến tại `mod.rs:338–345` gây lãng phí CPU và RAM. | 🟡 **HIGH** | ✅ Hoàn thành (`TASK-ARB-05` Khối 3) |

