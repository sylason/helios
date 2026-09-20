# 📋 DANH MỤC CÔNG VIỆC CẦN XỬ LÝ (TODO): CLIMAX REVERSAL SCANNER (`reversal.rs`)
> **Phân hệ:** Tactical Sniper Core V3.3 — Helios Trading System  
> **Tài liệu tham chiếu:** [`technical_audit.md`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/docs/audit_01/03_reversal/technical_audit.md)  
> **Trạng thái:** Sẵn sàng triển khai (Ready for Implementation)

---

## 🎯 MA TRẬN PHÂN LOẠI ƯU TIÊN

| Cấp độ | Nhãn | Số lượng Task | Mục tiêu chính |
| :---: | :---: | :---: | :--- |
| **P0** | 🔴 **CRITICAL** | 3 Tasks | Khắc phục lỗ hổng bảo vệ `NaN`, gỡ bỏ nghịch lý Arbiter Macro Veto, chặn bẫy bắt dao rơi. |
| **P1** | 🟡 **HIGH** | 3 Tasks | Tái cấu trúc công thức $f_{\text{vol}}$, bổ sung bộ lọc nến xác nhận rút chân, kiểm soát R:R. |
| **P2** | 🟢 **MEDIUM** | 2 Tasks | Khóa sàn Wick SL Buffer trong `config.rs`, chuẩn hóa Dynamic Structural TP KAMA. |
| **P3** | 🔵 **TESTING** | 2 Tasks | Viết Unit Test cô lập, chạy Walk-Forward Scenario Search kiểm chứng phục hồi PnL. |

---

## 🔴 GIAI ĐOẠN 1: KHẮC PHỤC LỖ HỔNG CRITICAL & NGHỊCH LÝ KIẾN TRÚC (P0)

### Task REV-01: Vá Lỗ Hổng Bảo Vệ `NaN` & Bypass Float Comparison
- **Độ ưu tiên:** 🔴 **P0 (Khẩn cấp)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 41 - 45, Dòng 60)
- **Mô tả vấn đề:**
  1. Dòng 41 đọc `vol_force = matrix.volume_force[i]`, nhưng dòng 43 không kiểm tra `vol_force.is_nan()`.
  2. Dòng 60: Biểu thức `if score < min_score` trả về `false` khi `score` là `NaN`. Code tiếp tục tạo `Proposal` chứa điểm `NaN`.
- **Giải pháp kỹ thuật:**
  ```rust
  // Bổ sung kiểm tra vol_force tại dòng 43:
  if z_score.is_nan() || kama_mid.is_nan() || current_atr.is_nan() || current_atr <= 0.0 || vol_force.is_nan() {
      return None;
  }

  // Sửa dòng 60 thành kiểm tra kép:
  if score.is_nan() || score < min_score {
      return None;
  }
  ```
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Thêm test case truyền vào `vol_force = f64::NAN`, hàm `scan()` phải trả về `None`.
  - [ ] Không có bất kỳ `Proposal` nào có `raw_score` là `NaN` lọt qua tầng Tactical Scanner.

---

### Task REV-02: Giải Quyết Triệt Để Nghịch Lý Arbiter Macro Veto (WFSS_37.log L.25806-25840)
- **Độ ưu tiên:** 🔴 **P0 (Khẩn cấp)**
- **Tệp tác động:** 
  - [`strategies_backtests/helios/src/strategy/core_v3/arbiter.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs) (Dòng 111 - 118, Dòng 136 - 144)
  - [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs)
- **Mô tả vấn đề:**
  - Macro Veto cấm tuyệt đối Long khi KAMA dốc xuống ($\text{kama\_slope} < -0.10 \land \text{trend\_axis} \ge 0.40$).
  - Trong các cú sập thác đổ (Capitulation Dump), KAMA luôn dốc âm sâu và Trend Axis luôn cao.
  - Kết quả: `Reversal` bị Macro Veto chặn đứng $100\%$ các điểm bắt đáy sau đợt bán tháo hoảng loạn.
- **Giải pháp kỹ thuật:**
  Thiết kế cơ chế **Ngoại Lệ Veto Có Điều Kiện (Conditional Macro Veto Bypass)**:
  Nếu `Proposal` xuất phát từ `ScannerId::ClimaxReversal`, chỉ bị Veto khi thị trường đang rơi tự do chưa có dấu hiệu kiệt sức. Nếu thỏa mãn điều kiện kiệt sức cực đại (Extreme Climax Exhaustion) + Phân kỳ RSI hoặc có nến rút chân xác nhận, cho phép Bypass Veto:
  ```rust
  // arbiter.rs
  let is_vetoed = match proposal.scanner_id {
      ScannerId::ClimaxReversal => {
          // Reversal có ngưỡng Veto riêng: Chỉ cấm khi đà rơi/bơm đang ở đỉnh điểm chưa có phân kỳ
          let extreme_macro = kama_slope.abs() > 0.60 && trend_axis >= 0.80;
          extreme_macro // Chỉ chặn khi xu hướng vượt quá cấp độ cuồng loạn
      }
      _ => Self::is_direction_vetoed(proposal.signal_type, kama_slope, trend_axis, macro_thresh),
  };
  ```
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Tín hiệu Reversal hợp lệ tại các điểm kiệt sức không còn bị Macro Veto chặn đứng trong backtest WFSS.
  - [ ] Vẫn giữ vững khả năng phòng thủ chống bắt dao rơi giữa chừng của các Playbook khác (`Anomaly`, `TrendPB`).

---

### Task REV-03: Tích Hợp Chốt Chặn Nến Xác Nhận Đảo Chiều (Anti-Falling Knife Guard)
- **Độ ưu tiên:** 🔴 **P0 (Khẩn cấp)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 70 - 87)
- **Mô tả vấn đề:**
  - Hiện tại chỉ kiểm tra $Z < 0$ và $\text{close} < \text{kama\_mid}$.
  - Nến hiện tại có thể là nến Marubozu cắm đầu rơi tự do không có râu, vào lệnh ngay lập tức bị quán tính quét SL ở nến sau.
- **Giải pháp kỹ thuật:**
  Bắt buộc nến kích hoạt phải là nến từ chối giá (Price Rejection Candle):
  ```rust
  let open = tick.opens()[i];
  let high = tick.highs()[i];
  let low = tick.lows()[i];

  // 1. Long Reversal: Bắt buộc có râu nến dưới thể hiện lực hấp thụ mua lên
  // HOẶC nến đóng cửa xanh và nằm ở nửa trên biên độ nến
  if z_score < 0.0 && close_price < kama_mid {
      let lower_wick = close_price.min(open) - low;
      let is_hammer = lower_wick >= (current_atr * 0.40);
      let is_bullish_close = close_price > open && close_price >= (high + low) * 0.5;

      if !is_hammer && !is_bullish_close {
          return None; // Từ chối nến Marubozu đỏ lòm đang rơi tự do!
      }
      ...
  }
  // 2. Short Reversal: Bắt buộc có râu trên HOẶC nến đóng đỏ ở nửa dưới
  else if z_score > 0.0 && close_price > kama_mid {
      let upper_wick = high - close_price.max(open);
      let is_shooting_star = upper_wick >= (current_atr * 0.40);
      let is_bearish_close = close_price < open && close_price <= (high + low) * 0.5;

      if !is_shooting_star && !is_bearish_close {
          return None; // Từ chối nến Marubozu xanh ngắt đang bơm dựng cột!
      }
      ...
  }
  ```
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Khi nến $i$ là nến Marubozu đỏ đặc không râu ($\text{low} == \text{close}$), ReversalScanner phải trả về `None`.
  - [ ] Khi nến $i$ là nến Hammer rút chân với râu dưới $\ge 0.4 \times \text{ATR}$, ReversalScanner tạo `Proposal` Long thành công.

---

## 🟡 GIAI ĐOẠN 2: NÂNG CẤP THUẬT TOÁN ĐẢO CHIỀU & BỘ LỌC NẾN (P1)

### Task REV-04: Tái Cấu Trúc Thành Phần Khối Lượng $f_{\text{vol}}$ (Thoát Khỏi Bẫy MFI)
- **Độ ưu tiên:** 🟡 **P1 (Quan trọng)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 41, Dòng 55)
- **Mô tả vấn đề:**
  - `vol_force` trong `matrix.rs` cộng gộp $50\%$ MFI. Vì MFI luôn duy trì quanh $40-60$, `vol_force` không bao giờ tiệm cận 0.
  - Phép tính $1.0 - \tanh(\text{vol\_force} / \text{drop})$ luôn bị dìm điểm sát 0.
- **Giải pháp kỹ thuật:**
  Sử dụng trực tiếp tỷ lệ khối lượng so với SMA Volume 20 phiên từ `sensors.vol_sma`:
  ```rust
  let vol_current = tick.volumes()[i];
  let vol_sma = sensors.vol_sma.get(i).copied().unwrap_or(vol_current);
  let vol_ratio = if vol_sma > 0.0 { vol_current / vol_sma } else { 1.0 };

  // Khối lượng cạn kiệt: vol_ratio < 1.0 -> f_vol tăng cao
  let f_vol = (1.0 - (vol_ratio / exhaust_drop).tanh()).max(0.0).min(1.0);
  ```
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Khi volume nến hiện tại cạn kiệt bằng $30\%$ mức trung bình (`vol_ratio = 0.3`), $f_{\text{vol}}$ phải đạt điểm cao ($> 0.60$).
  - [ ] Điểm số $f_{\text{vol}}$ phân phối đều trong dải $[0.0, 1.0]$ thay vì bị kẹt dưới $0.15$.

---

### Task REV-05: Kiểm Soát & Bảo Vệ Tỷ Lệ Risk/Reward Khởi Tạo
- **Độ ưu tiên:** 🟡 **P1 (Quan trọng)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 68 - 88)
- **Mô tả vấn đề:**
  - Khi giá nến đóng cửa đã hồi phục quá gần `kama_mid` trong khi râu nến sâu khiến SL ở rất xa, tỷ lệ R:R bị đảo ngược nghiêm trọng (ví dụ mạo hiểm 20 giá để ăn 3 giá).
- **Giải pháp kỹ thuật:**
  Bổ sung kiểm tra tỷ lệ R:R tối thiểu $\ge 1.0$ (hoặc tham số hóa):
  ```rust
  let sl_dist = (close_price - suggested_sl.unwrap()).abs();
  let tp_dist = (close_price - kama_mid).abs();

  if tp_dist < sl_dist * 0.80 {
      return None; // Hủy bỏ cơ hội nếu tiềm năng lợi nhuận không xứng đáng với rủi ro dừng lỗ!
  }
  ```
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Từ chối phát tín hiệu nếu khoảng cách TP nhỏ hơn $80\%$ khoảng cách SL.

---

### Task REV-06: Chống Bẫy Phục Hồi Giả (Dead Cat Bounce Protection) Trong `risk.rs`
- **Độ ưu tiên:** 🟡 **P1 (Quan trọng)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) (Dòng 361 - 410)
- **Mô tả vấn đề:**
  - Lệnh Reversal hướng về `kama_mid`. Trong nhiều trường hợp sóng chỉ hồi $50\%$ quãng đường rồi sập tiếp cán qua entry, biến một lệnh đang lãi đẹp thành lệnh thua.
- **Giải pháp kỹ thuật:**
  Khi lệnh Reversal đạt được $\ge 50\%$ quãng đường hồi quy về `kama_mid`:
  Kích hoạt dời Stop Loss về vùng hòa vốn dương (+phí sàn) ngay lập tức mà không cần đợi `be_atr_trigger_range`.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Lệnh Reversal có lãi $\ge 50\%$ khoảng cách đến KAMA Midline phải tự động khóa Break-even.

---

## 🟢 GIAI ĐOẠN 3: ĐỒNG BỘ DNA CONFIG & TINH CHỈNH KHÔNG GIAN TÌM KIẾM (P2)

### Task REV-07: Tối Ưu Hóa Dải Gen DNA Trong `config.rs`
- **Độ ưu tiên:** 🟢 **P2 (Trung bình)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs) (Dòng 485 - 509)
- **Mô tả vấn đề:**
  - `z_score_threshold`: Cận dưới $0.5$ quá nhỏ, tạo nhiễu. Cần nâng lên tối thiểu $1.5$ sigma.
  - `wick_sl_buffer_atr`: Cận dưới $0.20$ quá mỏng. Cần nâng lên tối thiểu $0.40$ hoặc $0.50$ ATR.
- **Giải pháp kỹ thuật:**
  ```rust
  pub struct TacticalReversalParams {
      ...
      /// Nâng cận dưới Z-Score để đảm bảo thực sự là Climax (1.5 -> 4.5 sigma)
      #[dna(min = 1.5, max = 4.5, step = 0.2)]
      pub z_score_threshold: f64,

      /// Nâng cận dưới Wick Buffer chống vi nhiễu sàn (0.4 -> 4.0 ATR)
      #[dna(min = 0.40, max = 4.00, step = 0.20)]
      pub wick_sl_buffer_atr: f64,
      ...
  }
  ```
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Bộ gen sau khi sửa compile thành công và giảm số lượng trạng thái rác trong thuật toán di truyền.

---

### Task REV-08: Chuẩn Hóa Cập Nhật Dynamic TP Cho Reversal Khi KAMA Trôi Dốc
- **Độ ưu tiên:** 🟢 **P2 (Trung bình)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) (Dòng 372 - 390)
- **Mô tả vấn đề:**
  - Khi KAMA Mid trôi dốc quá nhanh, TP bị kéo lùi lại sát entry price, vô hiệu hóa tiềm năng lợi nhuận.
- **Giải pháp kỹ thuật:**
  Thiết lập chốt chặn đơn điệu cho TP (Monotonic TP Floor Guard): Mức TP của lệnh Reversal không được phép dời lùi quá $30\%$ so với mức TP ban đầu được thiết lập tại nến entry.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] TP không bị tụt dốc thê thảm theo KAMA khi KAMA phản ứng trễ với đợt sập.

---

## 🔵 GIAI ĐOẠN 4: KIỂM THỬ & ĐÁNH GIÁ THỰC NGHIỆM (P3)

### Task REV-09: Xây Dựng Suite Unit Tests Toàn Diện Cho `reversal.rs`
- **Độ ưu tiên:** 🔵 **P3 (Kiểm thử)**
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs)
- **Danh mục Test Cases cần viết:**
  1. `test_nan_and_div_by_zero_safety`: Giả lập dữ liệu chứa NaN trong sensors và matrix.
  2. `test_rejection_candle_filter`: Kiểm chứng từ chối nến Marubozu và phê duyệt nến Hammer/Engulfing.
  3. `test_risk_reward_guard`: Kiểm chứng hủy lệnh khi khoảng cách TP < khoảng cách SL.
  4. `test_directional_macro_veto_bypass_exception`: Kiểm chứng ngoại lệ Veto cho Reversal khi thỏa mãn điều kiện kiệt sức.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Tất cả 4 test cases chạy `cargo test` vượt qua 100% không có cảnh báo.

---

### Task REV-10: Chạy Walk-Forward Scenario Search Đối Chiếu Trước & Sau
- **Độ ưu tiên:** 🔵 **P3 (Kiểm thử)**
- **Tệp tác động:** `strategies_backtests/helios`
- **Mục tiêu:**
  - Chạy lại kịch bản tương đương `WFSS_37` trên dữ liệu ETH/USDT 1m.
  - So sánh:
    + Tỷ lệ lệnh của Reversal được thông qua tầng Arbiter.
    + Win Rate của Reversal (Kỳ vọng: $> 55\%$).
    + Profit Factor tổng thể của hệ thống Helios V3.3.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] Reversal không còn 0 lệnh trong các chu kỳ biến động lớn.
  - [ ] Max Drawdown của hệ thống không bị tăng thêm khi mở khóa Reversal.
