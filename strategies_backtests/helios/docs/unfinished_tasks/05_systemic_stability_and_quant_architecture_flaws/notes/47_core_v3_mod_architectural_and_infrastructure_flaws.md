# BÁO CÁO KIỂM TOÁN HẠ TẦNG: GIẢI PHẪU MÃ NGUỒN `core_v3/mod.rs` & ĐỐI CHIẾU HỆ THỐNG
**File tài liệu:** `47_core_v3_mod_architectural_and_infrastructure_flaws.md`  
**Chuyên đề:** Kiểm toán kiến trúc hạ tầng và luồng điều phối trung tâm của `strategies_backtests/helios/src/strategy/core_v3/mod.rs`; Đối chiếu với các báo cáo kiểm toán cũ trong `audit_01`, `ROADMAP.md` và `unfinished_tasks/05`.  
**Nguyên tắc:** Tiếp cận từ First-Principles, Ranh giới Trách nhiệm (Separation of Concerns), Tính toàn vẹn Dữ liệu (Data Lineage) và Quản trị Rủi ro Định lượng Chuẩn mực.  

---

## 🏛️ PHẦN 1: TỔNG QUAN VAI TRÒ CỦA `core_v3/mod.rs`

Trong hệ sinh thái Helios V3, `HeliosStrategyV3` (nằm trong `core_v3/mod.rs`) đóng vai trò là **Trái Tim Điều Phối Trung Tâm (Strategy Orchestrator)**:
1. **Khởi tạo & Nạp Cảm Biến (`init`):** Đăng ký chuỗi chỉ báo với `ferro_trade::InitContext` và nạp dữ liệu vào struct `HeliosSensors`.
2. **Cập nhật Tham số Di truyền (`update_from_dna`):** Ánh xạ mảng gen DNA từ thuật toán tối ưu `Prospector` vào `HeliosConfigV3` và chuyển hóa thành `MarketMatrixV3`.
3. **Vòng lặp Nến (`next`):** 
   - Kiểm tra và quản lý rủi ro vị thế đang mở qua `RiskManager::manage_trailing_stop`.
   - Nếu chưa có vị thế: Kích hoạt sàn đấu thầu `self.arbiter.process_tick`.
   - Định cỡ vị thế (`RiskManager::calculate_size`) và phát lệnh `tick.buy` / `tick.sell` với chính sách rủi ro `RiskPolicy::FixedPrice`.
   - Ghi nhận viễn trắc (Decision Telemetry & Defense Ledger).

---

## 🔬 PHẦN 2: BẢNG KIỂM TOÁN ĐỐI SOÁT CÁC LỖI TRONG `core_v3/mod.rs`

| Mã Lỗi | Tên Lỗ Hổng Kiến Trúc Hạ Tầng | Tình Trạng Trong `audit_01` & `ROADMAP.md` | Tình Trạng Trong `unfinished_tasks/05` | Hiện Trạng Thực Tế Trong Mã Nguồn `mod.rs` |
| :--- | :--- | :--- | :--- | :--- |
| **FLAW-MOD-01** | **Tử Huyệt Lệch Pha DNA Cảm Biến Khi Hot-Swap (Sensors Desynchronization):** `update_from_dna` chỉ cập nhật `config` và `matrix`, KHÔNG HỀ tính lại `sensors`. Các gen chu kỳ chỉ báo (KAMA, ADX, ATR, BB) bị vô hiệu hóa khi bot chạy thật. | ❌ Chưa từng phát hiện | ❌ Chưa từng phát hiện | 🔴 **TỒN TẠI NGHIÊM TRỌNG:** Gây sai lệch 1:1 giữa kết quả Backtest GA và Live Execution. |
| **FLAW-MOD-02** | **Lãng Phí Đánh Giá Cầu Dao Kép (Double Gatekeeper Evaluation):** `RegimeGatekeeper::evaluate` bị gọi lặp lại 2 lần mỗi tick (ở `mod.rs:241` và `arbiter.rs:169`). | ❌ Chưa từng phát hiện | ⚠️ Nhắc thoáng qua ở Note 20 | 🔴 **TỒN TẠI:** Lãng phí chu kỳ CPU trên từng nến. |
| **FLAW-MOD-03** | **Mã Chết Kẹp SL & Nghịch Lý Catch-22 Với Arbiter:** `mod.rs:345-368` kẹp SL về 15% ROE là mã chết 100% vì Arbiter đã VETO hủy lệnh trước đó nếu SL > 15% ROE. | ⚠️ Có `TODO-MOD-03` ghi nhận kẹp SL kép | ⚠️ Có Note 15 & 22 vạch trần bẫy VETO nhị phân | 🔴 **TỒN TẠI:** Code kiểm tra `is_hard_clamped` không bao giờ được kích hoạt. |
| **FLAW-MOD-04** | **Định Cỡ Vị Thế Mù Lòa Khoảng Cách Cắt Lỗ (Fixed Margin Sizing):** `RiskManager::calculate_size` nhận vốn cố định nhân đòn bẩy, hoàn toàn không nhận `initial_sl` hoặc khoảng cách SL. | ❌ Bị bỏ sót | ⚠️ Nhắc ở Note 27 | 🔴 **TỬ HUYỆT ĐỊNH CỠ:** Vi phạm công thức Quant $\text{Size} = \frac{\text{Risk USD}}{\text{Dist SL}}$, rủi ro USD trồi sụt thất thường. |
| **FLAW-MOD-05** | **Pha Tạp Chỉ Báo & Lệch Pha Đa Khung Trong `init`:** `init()` tự viết vòng lặp tính `bb_middle_slope_adj` thay vì đưa vào `sensors/matrix`; lấy BB MTF chia cho 1m ATR. | ❌ Chưa từng phát hiện | ❌ Bị bỏ sót | 🔴 **TỒN TẠI:** Vi phạm ranh giới trách nhiệm, tạo bậc thang giả khi BB là khung lớn. |
| **FLAW-MOD-06** | **Biến Trạng Thái Ma Ghi Đè (Zombie Field `prev_open_trade`):** Khởi tạo, gán dữ liệu nhưng bị xóa về `None` ngay khi đóng lệnh, không hàm nào đọc. | ❌ Bị bỏ sót | ❌ Bị bỏ sót | 🟡 **TỒN TẠI:** Rác bộ nhớ và gây hiểu lầm logic. |
| **FLAW-MOD-07** | **Comment-out Chỉ Báo Gây Panic Crash:** Tắt `stoch_range` và `rsi_anomaly` trong `init()` khiến các scanner tương ứng truy cập `[i]` bị văng chương trình. | ⚠️ Có task `TODO-SEN-08` | ⚠️ Có nhắc ở Note 45 | 🔴 **TỒN TẠI:** Chưa được dọn dẹp triệt để. |

---

## 💥 PHẦN 3: GIẢI PHẪU CHI TIẾT 5 TỬ HUYỆT HẠ TẦNG TRONG `mod.rs`

### 1. FLAW-MOD-01: Tử Huyệt Lệch Pha Cảm Biến Khi DNA Hot-Swapping
* **Vị trí:** `mod.rs:67-79`
  ```rust
  fn update_from_dna(&mut self, dna: &[f64]) {
      self.config.update_from_dna(dna);
      self.config.indicators.keltner.timeframe = self.config.indicators.bbands.timeframe;
      self.matrix = MarketMatrixV3::from_sensors(&self.config, &self.sensors);
  }
  ```
* **Bản chất kỹ thuật:**
  - `HeliosConfigV3` chứa hàng chục gen quy định chu kỳ chỉ báo (`ind.kama.short_period`, `ind.bbands.period`, `ind.atr.period`, `ind.adx.period`).
  - Trong **Backtest/GA**, mỗi lần đánh giá một cá thể, engine tạo mới Strategy hoặc gọi `init()` $\implies$ Các chỉ báo được tính toán lại theo chu kỳ mới.
  - Nhưng trong **Live Trading / Paper Trading** (`trading_bot`), khi `ProspectorRunner` tìm ra DNA tối ưu mới và bắn qua kênh `watch` để hot-swap vào `AssetActor`, bot gọi `strategy.update_from_dna(dna)`.
  - Hàm này chỉ cập nhật struct cấu hình `self.config` và ma trận `self.matrix`. Struct `self.sensors` **KHÔNG ĐƯỢC TÍNH TOÁN LẠI**!
* **Hậu quả:** Toàn bộ các vector chỉ báo trong `self.sensors` vẫn giữ nguyên giá trị được tính từ ngày đầu tiên bot khởi động! DNA mới thay đổi chu kỳ nhưng cảm biến vẫn chạy chu kỳ cũ $\implies$ **Lệch pha hoàn toàn giữa Backtest và Thực chiến**.

---

### 2. FLAW-MOD-02: Lãng Phí Đánh Giá Cầu Dao Kép (Double Gatekeeper Evaluation)
* **Vị trí:** `mod.rs:241` và `arbiter.rs:169`
  - Tại `mod.rs:241`:
    ```rust
    let gate_eval = crate::strategy::core_v3::gatekeeper::RegimeGatekeeper::evaluate(tick, &self.sensors, &self.matrix, &self.config);
    ```
  - Sau đó đến `mod.rs:268`, gọi `self.arbiter.process_tick(...)`, và bên trong `arbiter.rs:169` lại gọi:
    ```rust
    let gate_eval = RegimeGatekeeper::evaluate(tick, sensors, matrix, config);
    ```
* **Bản chất:** `RegimeGatekeeper::evaluate` chứa nhiều phép so sánh phân vị, kiểm tra Hysteresis và điều kiện trôi dạt. Việc gọi 2 lần trên cùng 1 cây nến là dư thừa $100\%$, gây lãng phí chu kỳ CPU trên hàng triệu nến backtest.

---

### 3. FLAW-MOD-03: Mã Chết Kẹp Stop Loss & Nghịch Lý Catch-22 Với Arbiter
* **Vị trí:** `mod.rs:345-368` so với `arbiter.rs:223-289`
  - Tại `arbiter.rs:244-250`:
    ```rust
    let effective_max_sl_roe = playbook_max_sl_roe.min(0.15);
    // Nếu SL > effective_max_sl_roe -> VETO và trả về is_approved = false!
    ```
  - Tại `mod.rs:345-368`:
    ```rust
    let hard_cap_sl = RiskManager::clamp_sl_price(price, raw_sl, is_long, 0.15, ...);
    let is_hard_clamped = (raw_sl - hard_cap_sl).abs() > 1e-4;
    // ...
    ```
* **Bản chất:** Nếu `raw_sl` vượt quá $15\%$ ROE, Arbiter đã VETO và hủy lệnh ngay từ dòng 268. Lệnh không bao giờ có `is_approved == true`. Do đó, khối code kiểm tra `is_hard_clamped` trong `mod.rs` là **MÃ CHẾT (Luôn là false)**!
* **Hậu quả kiến trúc:** Tạo ra tư duy chắp vá. Đáng lẽ khi SL vượt trần an toàn, hệ thống phải tự động kẹp SL về trần an toàn và co cụm khối lượng (Dynamic De-leverage), thì Arbiter lại hủy lệnh, còn `mod.rs` lại viết code kẹp SL thừa thãi.

---

### 4. FLAW-MOD-04: Tử Huyệt Định Cỡ Vị Thế Mù Lòa Khoảng Cách Cắt Lỗ
* **Vị trí:** `mod.rs:277-283` và `risk.rs:158-189`
  ```rust
  let size = RiskManager::calculate_size(
      tick.equity, 
      self.config.risk_management.risk_base.percent_per_trade, 
      self.config.risk_management.risk_base.min_order_usd, 
      leverage,
      self.config.risk_management.risk_base.max_cap_fraction,
  );
  ```
* **Bản chất toán học:** Hàm `calculate_size` nhận vốn cố định (`target_margin = equity * risk_pct`). Nó hoàn toàn **không quan tâm khoảng cách Stop Loss là $0.3\%$ hay $3.0\%$**.
* **Hậu quả tài chính:**
  - Lệnh SL hẹp ($0.3\%$): Bị lỗ rất ít USD, không tối ưu hóa được đòn bẩy.
  - Lệnh SL rộng ($2.0\%$): Bị lỗ gấp 7 lần USD so với lệnh SL hẹp!
  - Vi phạm nguyên tắc bảo toàn rủi ro danh mục ($1\text{R}$ cố định).

---

### 5. FLAW-MOD-05: Pha Tạp Chỉ Báo & Lệch Pha Đa Khung Thời Gian Trong `init`
* **Vị trí:** `mod.rs:181-200`
  ```rust
  let k_bars = 5;
  for idx in 0..bb_m_len {
      let raw_slope = (m_curr - m_prev) / (k_bars as f64 * atr);
      bb_slope.push(raw_slope.clamp(-2.0, 2.0));
  }
  s.bb_middle_slope_adj = std::sync::Arc::new(bb_slope);
  ```
* **Bản chất:**
  - `mod.rs` là tầng Orchestrator nhưng lại đi tính toán logic chỉ báo thủ công.
  - `m_curr` là đường giữa Bollinger Bands (thường ở khung 15m/1h forward-fill), trong khi `atr` lấy từ `atr_sl` (khung 1m).
  - Phép chia này gây lệch pha đơn vị (Unit Mismatch), tạo ra các bước nhảy bậc thang giả tạo trong độ dốc của trục giữa.

---

## 🎯 PHẦN 4: ĐỀ XUẤT BẢN THIẾT KẾ CẢI TIẾN CHO `core_v3/mod.rs`

1. **Khử Bỏ Lệch Pha DNA (DNA Hot-Swap Harmonization):**
   - Chỉ cho phép DNA tối ưu các tham số trọng số, ngưỡng kích hoạt và tỷ lệ rủi ro (Weights, Thresholds, Risk Multipliers).
   - Khóa cố định các chu kỳ chỉ báo cơ sở (Base Indicator Periods) hoặc cung cấp hàm `recompute_sensors(&mut self, ctx)` khi hot-swap trong Live.
2. **Loại Bỏ Hoàn Toàn Mã Chết & Đánh Giá Kép:**
   - Xóa bỏ lượt gọi `RegimeGatekeeper::evaluate` thừa thãi tại dòng 241; tái sử dụng kết quả đánh giá từ `decision` của Arbiter.
   - Xóa bỏ trường ma `prev_open_trade`.
3. **Chuyển Đổi Sang Dynamic Position Sizing (Khử Bẫy VETO Nhị Phân):**
   - Chuyển `calculate_size` sang nhận tham số `initial_sl` để tính kích thước lệnh theo công thức chuẩn:
     $$\text{Position Size} = \min\left(\frac{\text{Risk USD}}{|Price - SL|},\ \text{Max Capital Fraction} \times \text{Equity}\right)$$
   - Xóa bỏ việc Arbiter VETO khi SL rộng; thay vào đó, hạ quy mô vị thế để bảo toàn số tiền rủi ro tối đa cố định.
4. **Di Dời `bb_middle_slope_adj` Về `matrix.rs`:**
   - Đưa logic tính độ dốc về đúng vị trí trong `MarketMatrixV3`, trả lại sự trong sạch cho hàm `init()` của `mod.rs`.
