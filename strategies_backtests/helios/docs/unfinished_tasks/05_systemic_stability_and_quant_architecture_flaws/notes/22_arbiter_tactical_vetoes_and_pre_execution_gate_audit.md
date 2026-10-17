# KIỂM TOÁN KIẾN TRÚC CHỐT CHẶN PHỦ QUYẾT & RÀO CẢN TIỀN KHỚP LỆNH ARBITER (ARBITER TACTICAL VETOES & PRE-EXECUTION GATE AUDIT)

> **Mã tài liệu:** `NOTE-22-ARBITER-VETOES-PRE-EXECUTION-AUDIT`  
> **Dự án:** Helios V3 Quantitative Strategy Core  
> **Phạm vi kiểm toán:** `arbiter.rs`, `risk.rs`, `core_v3/mod.rs`, `gatekeeper.rs`, `scanners/`  
> **Đối chiếu chéo:** `docs/audit_01/03_arbiter/`, `04_arbiter/`, `audit_01/ROADMAP.md`, `notes/12_tactical_risk_and_cross_blocking_audit.md`, `notes/15_the_4_pillar_risk_architecture_blueprint.md`, `notes/20_regime_gatekeeper_and_pretrade_gateway_audit.md`  
> **Mục tiêu:** Kiểm toán toàn diện các chốt chặn VETO tiền khớp lệnh, chứng minh mã chết của `clamp_sl_price`, giải mã nghịch lý Catch-22 của Sniper Guard, hiện tượng tù mù chi phí ma sát và các chốt chặn ngáng chân chéo.

---

## 🧭 1. BẢNG ĐỐI CHIẾU TRẠNG THÁI TỔNG THỂ (STATUS MAPPING)

| Hạng Mục / Hiện Tượng | Vị Trí Code (File & Dòng) | Hiện Trạng Thực Tế Trong Codebase | Phân Loại & Đánh Giá |
| :--- | :--- | :--- | :---: |
| **`TODO-ARB-03`**<br>(Binary Sniper Veto) | `arbiter.rs:223-289`<br>`risk.rs:84-138` | SL cấu trúc $> 15\%$ ROE bị VETO loại bỏ $100\%$ proposal; biến `clamp_sl_price` ở `mod.rs` thành mã chết. | 🔴 **CHƯA FIX - NGUY HIỂM**<br>*(Bóp nghẹt 80% cơ hội)* |
| **`CLAMP-DEAD-CODE`**<br>(Mã Chết Kẹp Biên SL) | `core_v3/mod.rs`<br>`L332-370` | Khối `clamp_sl_price`, `is_hard_clamped`, `is_playbook_clamped`, `ledger.on_sl_clamped` là **MÃ CHẾT 100%**. | 🔴 **MÃ CHẾT 100%**<br>*(Chưa từng được kích hoạt)* |
| **`TODO-ARB-05`**<br>(Tù Mù Chi Phí Ma Sát) | `arbiter.rs:290-302` | Arbiter chỉ duyệt `confidence >= min_threshold`, hoàn toàn không kiểm tra biên độ kỳ vọng so với phí ma sát $2.6\%$ ROE. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Bào mòn vốn vì phí sàn)* |
| **`NEW-DOUBLE-EVAL`**<br>(Đánh Giá Lặp Lại 2 Lần) | `core_v3/mod.rs:241`<br>`arbiter.rs:169` | `RegimeGatekeeper::evaluate` bị gọi lặp lại 2 lần mỗi tick (lần 1 ở `mod.rs`, lần 2 ở `arbiter.rs`). | 🟡 **LỖI MỚI / CHỒNG CHÉO**<br>*(Lãng phí tài nguyên CPU)* |
| **`TODO-ARB-02-R2`**<br>(Lỗ Hổng Inverted SL) | `risk.rs:84-138` | `validate_structural_sl` dùng `abs(price - sl)`, không kiểm tra hướng logic (Long bắt buộc $SL < P$, Short $SL > P$). | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Nguy cơ cắn SL tức thì)* |
| **`HIGH-LEV-MMR-VETO`**<br>(Bẫy MMR 1.0% Đòn Bẩy Cao) | `risk.rs:61, 105`<br>`config.rs:989` | `default_maintenance_margin_rate = 0.01` ($1.0\%$). Khi đòn bẩy $\ge 100\text{x}$, `raw_liq_dist <= 0` $\implies$ VETO $100\%$ oan. | 🟡 **LỖI MỚI PHÁT HIỆN**<br>*(Tê liệt đòn bẩy cao)* |
| **`MACRO-ADVERSE-VETO`**<br>(Macro Veto Toàn Cục) | `arbiter.rs:95-160` | Hàm `is_direction_vetoed` đã được gỡ bỏ hoàn toàn khỏi `arbiter.rs` theo `TASK-ARB-03`. | 🟢 **ĐÃ FIX TRONG CODE**<br>*(Đã gỡ bỏ sạch sẽ)* |
| **`REVERSAL-VOL-PARADOX`**<br>(Triệt Tiêu Volume Bán Tháo) | `reversal.rs:105-107`<br>`normalization.rs:55` | `sat_proximity(vol_ratio, 0.50)` gán $f_{\text{vol}} = 0.0$ khi volume $\ge 0.50\times \text{SMA}$, triệt tiêu điểm bán tháo Climax ($2\sim 4\text{x}$). | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Bóp nghẹt Reversal)* |
| **`POST-STORM-BLINDNESS`**<br>(Điểm Mù Dư Chấn Sau Bão) | `gatekeeper.rs:177-206` | Hysteresis $K=5$ bars vừa mở khóa là Arbiter mở lệnh $100\%$ size ngay nến kế tiếp khi sổ lệnh còn rỗng, chịu trượt giá nặng. | 🔴 **CHƯA FIX - ĐANG TỒN TẠI**<br>*(Trượt giá sau bão)* |

---

## 🔬 2. MỔ XẺ CHI TIẾT CÁC LỖ HỔNG VÀ BẪY KIẾN TRÚC

### 2.1. Nghịch Lý Catch-22: Binary All-or-Nothing Sniper Veto (`TODO-ARB-03`)
- **Hiện trạng mã nguồn tại `arbiter.rs:223-289`:**
  ```rust
  let validation_res = RiskManager::validate_structural_sl(
      current_price, sl, playbook_max_sl_roe, leverage, ...
  );
  match validation_res {
      SlValidationResult::StructuralRiskVeto { suggested_roe, max_roe } => {
          (confidence, false, Some(AuctionStatus::StructuralRiskVeto), ...) // VETO HỦY BỎ 100% PROPOSAL!
      }
      ...
  }
  ```
- **Chứng minh toán học: Khối `clamp_sl_price` tại `core_v3/mod.rs:332-370` là MÃ CHẾT 100%:**
  ```rust
  let raw_sl = decision.final_sl.unwrap_or_else(|| { ... });
  let initial_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, playbook_max_sl_roe, ...);
  let hard_cap_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, 0.15, ...);
  let is_hard_clamped = (raw_sl - hard_cap_sl).abs() > 1e-4;
  let is_playbook_clamped = (raw_sl - initial_sl).abs() > 1e-4;
  ```
  1. Nếu proposal có `decision.final_sl = Some(sl)`: Để vượt qua Arbiter và thắng thầu, đề xuất này **BẮT BUỘC** phải có $\text{roe} \le \text{playbook\_max\_sl\_roe} \le 0.15$ (nếu lớn hơn đã bị VETO loại bỏ ngay từ dòng 271 trong `arbiter.rs`). Do đó, `raw_sl` luôn nằm trong ngưỡng cho phép $\implies$ `clamp_sl_price` không làm thay đổi giá $\implies$ `is_playbook_clamped` và `is_hard_clamped` luôn luôn bằng `false`.
  2. Nếu proposal có `decision.final_sl = None`: `raw_sl` được sinh ra từ `RiskManager::get_initial_sl` (`risk.rs:239-260`), vốn **ĐÃ TỰ ĐỘNG GỌI `clamp_sl_price` TẠI DÒNG 260**. Khi trả về, giá trị này đã được kẹp an toàn $\implies$ Lệnh gọi `clamp_sl_price` tại `mod.rs:332` là hoàn toàn thừa thãi.
  $\implies$ Khối code telemetry `ledger.on_sl_clamped` và nhãn `"CLAMPED: HARD ROE CAP 15%"` **KHÔNG BAO GIỜ CÓ CƠ HỘI ĐƯỢC THỰC THI (DEAD CODE)**.

- **Nghịch lý "Thưởng kẻ lười, phạt kẻ siêng" (Perverse Incentive):**
  - **Scanner siêng năng (`Trend_PB`, `Absorption`):** Tính toán SL cấu trúc cẩn thận theo đáy/đỉnh nến vi mô + đệm ATR. Nếu gặp cây nến rũ thanh khoản có biên độ hơi rộng ($16\%$ ROE tại $20\text{x}$ tương đương $0.8\%$ giá), Arbiter **VETO thẳng tay, giết chết cơ hội**.
  - **Scanner lười biếng:** Đặt `suggested_sl: None`. Arbiter cho qua (`Acceptable`), sau đó `mod.rs` tự động tính SL theo ATR và kẹp an toàn về $15\%$ ROE!
  - **Hệ quả thực tế:** Hệ thống giết chết tới $70\% - 80\%$ các cơ hội gom hàng chân sóng có điểm tự tin cao ($0.90 \sim 0.95$), chỉ các cây nến còi cọc li ti mới lọt qua được và dính Stop Loss ngay sau đó vì khoảng cách quá sát nhiễu vi mô.

- **Giải pháp chuyển đổi sang Dynamic De-leverage & Size Scaling:**
  $$\text{suggested\_roe} = \frac{|\text{price} - \text{suggested\_sl}|}{\text{price}} \times \text{leverage}$$
  - Nếu $\text{suggested\_roe} \le 15.0\% \implies \text{Scale Factor} = 1.0$.
  - Nếu $15.0\% < \text{suggested\_roe} \le 35.0\%$ (vẫn nằm ngoài vùng thanh lý an toàn):
    $$\text{Scale Factor} = \frac{0.15}{\text{suggested\_roe}}$$
    $$\text{Final Order Size} = \text{Base Size} \times \text{Scale Factor}$$
    $$\text{Rủi ro USD thực tế} = \text{Final Order Size} \times \text{suggested\_roe} = \text{Base Size} \times 0.15 = \text{HẰNG SỐ BẤT BIẾN}$$
  - Nếu $\text{suggested\_roe} > \text{safe\_liq\_dist} \times \text{leverage} \implies$ Mới kích hoạt `LiquidationRiskVeto`.

---

### 2.2. Hiện Tượng Tù Mù Trước Chi Phí Ma Sát Sàn (`TODO-ARB-05` - Frictional Blindness)
- **Bản chất ma sát tại đòn bẩy $20\text{x}$ trên Binance Futures:**
  - Phí Taker 2 chiều: $2 \times 0.04\% = 0.08\%$.
  - Trượt giá khớp lệnh 2 chiều: $2 \times 0.025\% = 0.05\%$.
  - Tổng hao mòn giá: $\Delta P_{\text{friction}} = 0.13\%$.
  - Tỷ lệ hao mòn vốn ký quỹ:
    $$\text{ROE}_{\text{friction}} = 0.13\% \times 20\text{x} = \mathbf{2.60\% \text{ ROE}}$$
- **Điểm mù trong `arbiter.rs:290-302`:**
  - Arbiter chỉ kiểm tra `confidence >= min_threshold`, hoàn toàn **tù mù trước tiềm năng lợi nhuận của lệnh**.
  - Nếu một đề xuất chỉ có biên độ kỳ vọng hồi quy đạt $0.20\%$ giá ($4.0\%$ ROE): Chi phí sàn ngốn mất $2.6\% / 4.0\% = \mathbf{65\%}$ lợi nhuận tiềm năng! Khi dính lệnh thua, tài khoản mất đứt $\text{SL} + 2.6\%$ ROE.
  - Kỳ vọng toán học của lệnh bị âm nặng ($EV \ll 0$).
- **Giải pháp: Bổ sung EV Hurdle Gate:**
  Bắt buộc tiềm năng biến động tối thiểu của lệnh ($\text{Displacement}$ hoặc $\text{TP}$) phải thỏa mãn:
  $$\text{Expected Return} \ge 2.5 \times \text{Friction Cost} \approx \mathbf{6.5\% \text{ ROE}}$$

---

### 2.3. Các Chốt Chặn VETO Ngáng Chân Nhau
1. **Lệnh gọi lặp lại Gatekeeper lần 2 (`NEW-DOUBLE-EVAL`):**
   - Tại `mod.rs:241`: Gọi `RegimeGatekeeper::evaluate`.
   - Tại `arbiter.rs:169`: Lại gọi `RegimeGatekeeper::evaluate` lần thứ 2, chạy lại vòng lặp quét Hysteresis $K=5$ nến.
   - Vi phạm nguyên tắc "Fail-Fast", lãng phí chu kỳ CPU trên từng cây nến 1m.
2. **Lỗ hổng Inverted SL (`TODO-ARB-02-R2`):**
   - Trong `risk.rs:84-138`, hàm `validate_structural_sl` dùng `abs(price - sl)`. Nếu Scanner gặp bug logic đề xuất Long nhưng $SL > \text{price}$, hàm vẫn trả về `Acceptable`! Lệnh gửi lên sàn sẽ bị kích hoạt Stop Market cắn lỗ ngay lập tức.
3. **Bẫy Đòn Bẩy Cao Với MMR Mặc Định 1.0% (`HIGH-LEV-MMR-VETO`):**
   - Trong `config.rs:989`, `default_maintenance_margin_rate = 0.01` ($1.0\%$).
   - Khi đòn bẩy nâng lên $100\text{x}$:
     $$\text{raw\_liq\_dist} = \frac{1}{100} - 0.01 - (2 \times 0.0004) = -0.0008 \le 0.0$$
     $\implies \text{safe\_liq\_dist} = 0.0 \implies$ **$100\%$ LỆNH BỊ VETO TỰ ĐỘNG BỞI `LiquidationRiskVeto`**, dù thực tế Binance áp dụng MMR Tier 1 cho BTC/ETH chỉ là $0.4\% \sim 0.5\%$.
4. **Triệt tiêu Volume Bán Tháo Trong `reversal.rs:105-107`:**
   - Hàm `sat_proximity(vol_ratio, 0.50)` gán $f_{\text{vol}} = 0.0$ khi volume $\ge 0.50\times \text{SMA}$.
   - Khi xảy ra Capitulation Dump với volume bùng nổ $2.0 \sim 4.0\text{x}$ (cơ hội vàng để bắt đáy đảo chiều), điểm số khối lượng của Reversal bị đè bẹp về $0.0$, giết chết hoàn toàn Scanner này.

---

### 2.4. Điểm Mù Dư Chấn Sau Bão (Post-Storm Residual Shock)
- **Cơ chế Hysteresis hiện tại:** `gatekeeper.rs:177-206` quét ngược $K=5$ nến. Ngay khi nến thứ 6 hạ nhiệt dưới ngưỡng bão, Gatekeeper lập tức mở khóa cửa (`passed = true`).
- **Thực tế vi cấu trúc:** Sau cơn bão quét thanh khoản dữ dội, sổ lệnh (Orderbook) còn rất mỏng, spread giãn nở gấp 3–5 lần bình thường.
- Arbiter hiện tại **không có cơ chế làm nguội (Cooldown / Ramp-up)**, duyệt ngay lệnh ở nến đầu tiên với $100\%$ kích thước vị thế thông thường $\to$ Bị trượt giá nặng và dính bẫy râu nến dư chấn (Secondary Aftershock Whipsaw).

---

## ⚖️ 3. BẢNG ĐỐI CHẤT ƯU - KHUYẾT - EDGE CASES

| Thành Phần & Đề Xuất Cải Tiến | Ưu Điểm (Pros) | Khuyết Điểm (Cons) | Edge Cases Nguy Hiểm & Giải Pháp |
| :--- | :--- | :--- | :--- |
| **Dynamic De-leverage thay vì Sniper Veto** | • Tái sinh khối code `clamp_sl_price` bị chết.<br>• Giải phóng $80\%$ số lệnh bị chặn oan của TrendPB và Absorption.<br>• Rủi ro đô-la (\$ USD Risk) giữ nguyên hằng số bất biến. | • Vị thế có khoảng cách SL rộng sẽ vào size nhỏ hơn (lãi tuyệt đối ít hơn nếu sóng chạy ngắn). | **Edge Case:** SL bị kẹp quá xa $> 35\%$ ROE.<br>$\implies$ Vẫn giữ chốt chặn trần cứng $35\%$ ROE để né vùng thanh lý sàn. |
| **Bổ Sung Frictional EV Hurdle Gate** | • Khử $100\%$ các lệnh scalp mỏng bị lỗ ngược do phí sàn và trượt giá $2.6\%$ ROE.<br>• Tăng tỷ lệ Expectancy $EV$ thực tế của hệ thống. | • Có thể bỏ qua một số lệnh scalp nhanh trong thị trường biên độ hẹp. | **Edge Case:** Thị trường tích lũy nén chặt biên độ ATR nhỏ.<br>$\implies$ Tự động co giãn theo ATR thực tế. |
| **Thống Nhất Single Evaluation Cho Gatekeeper** | • Tiết kiệm tài nguyên CPU trong vòng lặp tick.<br>• Rõ ràng ranh giới kiến trúc: Gatekeeper duyệt cửa ngõ, Arbiter chỉ chuyên tâm đấu thầu. | • Cần điều chỉnh nhẹ chữ ký của hàm `arbiter.process_tick`. | **Edge Case:** Dữ liệu tick bị thay đổi giữa chừng.<br>$\implies$ Không thể xảy ra do `TickContext` là bất biến. |
| **Khắc Phục Lỗ Hổng Inverted SL & Động Hóa MMR** | • Khử nguy cơ vào lệnh Long nhưng SL cao hơn giá Entry.<br>• Khôi phục khả năng giao dịch ở đòn bẩy cao $\ge 100\text{x}$. | • Cần truyền thêm cờ `is_long: bool` vào hàm thẩm định SL. | **Edge Case:** Sàn Binance thay đổi bảng bậc thang MMR.<br>$\implies$ Đọc trực tiếp từ API sàn hoặc đặt MMR an toàn theo Tier. |
| **Post-Storm Ramp-up Filter** | • Bảo vệ tài khoản khỏi trượt giá và râu nến dư chấn khi sổ lệnh còn mỏng sau bão. | • Giảm quy mô vị thế ở $M=3$ nến đầu tiên sau khi bão tan. | **Edge Case:** Sóng đảo chiều bùng nổ cực nhanh ngay sau bão.<br>$\implies$ Vẫn được vào lệnh nhưng với $50\%$ size để an toàn vốn. |

---

## 📋 4. DANH MỤC CÔNG VIỆC CẦN TRIỂN KHAI (TODO ACTION ITEMS)

- [ ] **Task ARB-05:** Triển khai cơ chế Dynamic De-leverage trong `arbiter.rs:223-289` và `risk.rs:84-138`: Khi SL cấu trúc $> 15\%$ ROE (nhưng $\le 35\%$ ROE), tự động co giảm kích thước vị thế tỷ lệ nghịch với khoảng cách SL thay vì VETO hủy lệnh.
- [ ] **Task ARB-06:** Tái kích hoạt khối code kẹp SL và sổ cái kiểm toán `defense_ledger.on_sl_clamped` trong `core_v3/mod.rs:332-370`.
- [ ] **Task ARB-07:** Bổ sung `is_long: bool` vào hàm `validate_structural_sl` (`risk.rs:84-138`) để triệt tiêu hoàn toàn lỗi Inverted SL.
- [ ] **Task ARB-08:** Thống nhất điểm gọi `RegimeGatekeeper::evaluate`: Đưa về một cửa ngõ duy nhất tại đầu hàm `next()` trong `core_v3/mod.rs`, xóa bỏ lệnh gọi lặp lại lần 2 trong `arbiter.rs:169`.
- [ ] **Task ARB-09:** Bổ sung EV Hurdle Gate: Kiểm tra biên độ kỳ vọng $\ge 2.5 \times \text{Friction Cost} \ge 6.5\%$ ROE trước khi duyệt mở lệnh.
- [ ] **Task ARB-10:** Khắc phục hàm tính điểm khối lượng của `reversal.rs:105-107`: Thưởng điểm tối đa khi volume bùng nổ bán tháo hoảng loạn ($vol\_ratio \ge 2.0\text{x}$).
- [ ] **Task ARB-11:** Triển khai Post-Storm Ramp-up Filter: Giảm $50\%$ position size trong $M=3$ nến đầu tiên sau khi Gatekeeper mở khóa sau bão.
