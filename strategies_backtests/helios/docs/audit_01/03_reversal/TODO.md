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
  - [x] Thêm test case truyền vào dữ liệu chứa `NaN`, hàm `scan()` phải trả về `None`.
  - [x] Không có bất kỳ `Proposal` nào có `raw_score` là `NaN` lọt qua tầng Tactical Scanner.

---

### Task REV-02: Giải Quyết Triệt Để Nghịch Lý Arbiter Macro Veto (WFSS_37.log L.25806-25840)
- **Độ ưu tiên:** 🔴 **P0 (Khẩn cấp)**
- **Phân hệ phụ trách:** 🏛️ **`04_arbiter`** ([`arbiter.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/arbiter.rs)) — *Sẽ được xử lý trong đợt kiểm toán phân hệ Arbiter*
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

### Task REV-03: Tích Hợp Chốt Chặn Nến Xác Nhận Đảo Chiều (Anti-Falling Knife Guard) [ĐÃ HOÀN THÀNH]
- **Độ ưu tiên:** 🔴 **P0 (Khẩn cấp)**
- **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH: CLV Guard qua DNA `min_reversal_clv`**.
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 70 - 87)
- **Mô tả vấn đề:**
  - Hiện tại chỉ kiểm tra $Z < 0$ và $\text{close} < \text{kama\_mid}$.
  - Nến hiện tại có thể là nến Marubozu cắm đầu rơi tự do không có râu, vào lệnh ngay lập tức bị quán tính quét SL ở nến sau.
- **Giải pháp kỹ thuật đã chốt (Phương án 2 - CLV Guard):**
  - Trích xuất: `open = tick.opens()[i]`, `high = tick.highs()[i]`, `low = tick.lows()[i]`, `close = tick.closes()[i]`.
  - Tính toán độ lệch đóng nến phi thứ nguyên **CLV (Close Location Value)**:
    $$\text{CLV} = \frac{2 \cdot \text{Close} - \text{High} - \text{Low}}{\max(\text{High} - \text{Low}, 1e-5)} \in [-1.0, +1.0]$$
  - Bổ sung tham số DNA `min_reversal_clv` $[0.05, 0.50]$, step $0.05$, default $0.20$ vào `TacticalReversalParams`.
  - **1. Long Reversal:** Bắt buộc $\text{CLV} \ge \text{min\_reversal\_clv}$ (giá đóng cửa ở nửa trên của nến, xác nhận nến rút chân Hammer hoặc nến xanh có lực hấp thụ mua lên; cấm tuyệt đối nến Marubozu đỏ $\text{CLV} \le -0.50$).
  - **2. Short Reversal:** Bắt buộc $\text{CLV} \le -\text{min\_reversal\_clv}$ (giá đóng cửa ở nửa dưới của nến, xác nhận nến rút râu đỉnh Shooting Star hoặc nến đỏ có lực xả đè xuống; cấm tuyệt đối nến Marubozu xanh $\text{CLV} \ge 0.50$).
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] Khi nến $i$ là nến Marubozu đỏ đặc không râu ($\text{low} == \text{close}$), ReversalScanner phải trả về `None`.
  - [x] Khi nến $i$ là nến rút chân với $\text{CLV} \ge \text{min\_reversal\_clv}$, ReversalScanner tạo `Proposal` Long thành công.

---

## 🟡 GIAI ĐOẠN 2: NÂNG CẤP THUẬT TOÁN ĐẢO CHIỀU & BỘ LỌC NẾN (P1)

### Task REV-04: Tái Cấu Trúc Thành Phần Khối Lượng $f_{\text{vol}}$ & Đưa MFI Về Đúng Thiên Chức [ĐÃ HOÀN THÀNH]
- **Độ ưu tiên:** 🟡 **P1 (Quan trọng)**
- **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH: Tách rời Volume thực & Dùng MFI làm Bộ lọc Dòng tiền Quá bán/Quá mua**.
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 41, Dòng 55)
- **Mô tả vấn đề:**
  - `vol_force` trong `matrix.rs` cộng gộp $50\%$ MFI. Vì MFI luôn duy trì quanh $40-60$, `vol_force` không bao giờ tiệm cận 0, làm $f_{\text{vol}}$ luôn bị đè chết sát 0.
  - Hơn nữa, dùng MFI chia cho 100 làm thước đo khối lượng gây nghịch lý hướng: khi giá bơm dựng cột để Short Reversal, MFI vọt lên $80-90$ làm $f_{\text{vol}}$ triệt tiêu về 0!
- **Giải pháp kỹ thuật đã chốt (Phương án 1 - Separation of Concerns):**
  1. **Thành phần Khối lượng $f_{\text{vol}}$ thuần túy:** Đo bằng tỷ lệ khối lượng thực so với SMA Volume 20 phiên từ `sensors.vol_sma`:
     ```rust
     let vol_current = tick.volumes()[i];
     let vol_sma = sensors.vol_sma.get(i).copied().unwrap_or(vol_current);
     let vol_ratio = if vol_sma > 0.0 { vol_current / vol_sma } else { 1.0 };
     let f_vol = (1.0 - (vol_ratio / exhaust_drop).tanh()).max(0.0).min(1.0);
     ```
  2. **Đưa MFI về đúng thiên chức (Money Flow Exhaustion Filter):**
     - Bổ sung tham số DNA `mfi_reversal_thresh` $[20.0, 45.0]$, step $5.0$, default $30.0$ vào `TacticalReversalParams`.
     - **Long Reversal:** Đòi hỏi dòng tiền bán cạn kiệt: $\text{MFI} \le \text{mfi\_reversal\_thresh}$ (ví dụ $\le 30.0$).
     - **Short Reversal:** Đòi hỏi dòng tiền mua quá tải: $\text{MFI} \ge 100.0 - \text{mfi\_reversal\_thresh}$ (ví dụ $\ge 70.0$).
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] Khi volume nến hiện tại cạn kiệt bằng $30\%$ mức trung bình (`vol_ratio = 0.3`), $f_{\text{vol}}$ đạt điểm cao ($> 0.60$).
  - [x] MFI đóng vai trò bộ lọc định hướng đảo chiều chính xác cho cả 2 chiều Long ($\le 30$) và Short ($\ge 70$), không còn bị bóp méo ngược chiều.

---

### Task REV-05: Chốt Chặn Khoảng Cách Hồi Quy Tối Thiểu (Remaining Displacement Filter) [ĐÃ HOÀN THÀNH]
- **Độ ưu tiên:** 🟡 **P1 (Quan trọng)**
- **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH: Khoảng cách hồi quy tối thiểu qua DNA `min_reversal_displacement`**.
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs) (Dòng 48 - 56)
- **Mô tả vấn đề:**
  - Khi giá nến đóng cửa đã hồi phục quá gần `kama_mid`, miếng bánh tiềm năng không còn đủ lớn trong khi rủi ro vẫn hiện hữu.
  - Sử dụng R:R cố định (Fixed R:R Cutoff) là sai lầm vì bị râu nến biến dạng và triệt tiêu các cơ hội có xác suất thắng cao.
- **Giải pháp kỹ thuật đã chốt (Phương án 1 - Remaining Displacement Pre-condition):**
  - Tận dụng trực tiếp gen DNA sẵn có `min_reversal_displacement` $[0.50, 3.00] \times ATR$ trong `TacticalReversalParams`.
  - Nâng cấp từ chấm điểm mềm sang **Điều kiện Tiên Quyết Cứng (Early Exit Guard)**:
    ```rust
    let displacement = (close_price - kama_mid).abs();
    let min_disp_dist = rev_cfg.min_reversal_displacement.max(0.50) * current_atr;

    // Nếu khoảng cách tới KAMA Midline không đủ lớn -> Giá đã hồi gần xong, từ chối ngay lập tức!
    if displacement < min_disp_dist {
        return None;
    }
    ```
  - Đảm bảo tiềm năng sóng hồi luôn $\ge 0.50 \sim 3.00 \times ATR$, triệt tiêu 100% các lệnh vào khi giá đã áp sát KAMA.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] Khi $Close$ chỉ cách $KAMA$ một khoảng $< min\_reversal\_displacement \times ATR$, Scanner phải trả về `None`.
  - [x] Khi $Close$ còn cách $KAMA \ge min\_reversal\_displacement \times ATR$, Scanner tiếp tục đánh giá các bước kế tiếp.

---

### Task REV-06: Chống Bẫy Phục Hồi Giả (Dead Cat Bounce Protection) Trong `risk.rs`
- **Độ ưu tiên:** 🟡 **P1 (Quan trọng)**
- **Phân hệ phụ trách:** 🛡️ **`06_risk`** ([`risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)) — *Sẽ được xử lý trong đợt kiểm toán phân hệ Risk Manager*
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

### Task REV-07: Tinh Chỉnh Dải Gen DNA & Khử Hardcode Tỷ Trọng Trong `config.rs` [ĐÃ HOÀN THÀNH]
- **Độ ưu tiên:** 🟢 **P2 (Trung bình)**
- **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH: Bổ sung 4 gen mới, nâng `DNA_WIDTH` từ 103 lên 107 chiều, giữ nguyên dải rộng phổ quát**.
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs) (Dòng 505 - 530)
- **Mô tả vấn đề:**
  - Điểm số hiện tại bị hardcode cố định tỷ trọng $0.40 / 0.40 / 0.20$.
  - Cần đưa các ngưỡng CLV và MFI ra ngoài DNA để GA tự thích ứng.
  - Tuân thủ nguyên tắc Universal Adaptability: Giữ nguyên dải rộng phổ quát cho `z_score_threshold` $[0.5, 5.0]$ và `wick_sl_buffer_atr` $[0.20, 6.00]$.
- **Giải pháp kỹ thuật đã chốt:**
  1. Bổ sung 4 gen mới vào `TacticalReversalParams`:
     - `min_reversal_clv`: $[0.05, 0.60]$, step $0.05$, default $0.20$
     - `mfi_reversal_thresh`: $[15.0, 45.0]$, step $5.0$, default $30.0$
     - `w_z_score`: $[0.10, 0.80]$, step $0.05$, default $0.40$
     - `w_displacement`: $[0.10, 0.80]$, step $0.05$, default $0.40$
  2. Nâng `DNA_WIDTH` của `HeliosConfigV3` từ 103 lên 107 chiều.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] `DNA_WIDTH` nâng thành công lên 107 và test `test_dna_width_and_parameter_defs_integrity` pass 100%.

---

### Task REV-08: Chuẩn Hóa Cập Nhật Dynamic TP Cho Reversal Khi KAMA Trôi Dốc
- **Độ ưu tiên:** 🟢 **P2 (Trung bình)**
- **Phân hệ phụ trách:** 🛡️ **`06_risk`** ([`risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)) — *Sẽ được xử lý trong đợt kiểm toán phân hệ Risk Manager*
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs) (Dòng 372 - 390)
- **Mô tả vấn đề:**
  - Khi KAMA Mid trôi dốc quá nhanh, TP bị kéo lùi lại sát entry price, vô hiệu hóa tiềm năng lợi nhuận.
- **Giải pháp kỹ thuật:**
  Thiết lập chốt chặn đơn điệu cho TP (Monotonic TP Floor Guard): Mức TP của lệnh Reversal không được phép dời lùi quá $30\%$ so với mức TP ban đầu được thiết lập tại nến entry.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [ ] TP không bị tụt dốc thê thảm theo KAMA khi KAMA phản ứng trễ với đợt sập.

---

## 🔵 GIAI ĐOẠN 4: KIỂM THỬ & ĐÁNH GIÁ THỰC NGHIỆM (P3)

### Task REV-09: Xây Dựng Suite Unit Tests Toàn Diện Cho `reversal.rs` [ĐÃ HOÀN THÀNH]
- **Độ ưu tiên:** 🔵 **P3 (Kiểm thử)**
- **Trạng thái:** ✅ **ĐÃ HOÀN THÀNH: 4 unit test cases toàn diện trong `core_v3/mod.rs` pass 100%**.
- **Tệp tác động:** [`strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/scanners/reversal.rs)
- **Danh mục Test Cases đã viết:**
  1. `test_reversal_clv_guard_rejects_marubozu_and_accepts_hammer`: Kiểm chứng từ chối nến Marubozu đỏ đặc và phê duyệt nến rút chân Hammer.
  2. `test_reversal_remaining_displacement_guard`: Kiểm chứng hủy lệnh khi khoảng cách tới KAMA không đủ lớn.
  3. `test_reversal_mfi_exhaustion_filter`: Kiểm chứng lọc chính xác quá bán cho Long và quá mua cho Short.
  4. `test_reversal_nan_safety`: Kiểm chứng an toàn tuyệt đối khi gặp dữ liệu NaN.
- **Tiêu chí nghiệm thu (Acceptance Criteria):**
  - [x] Tất cả 4 test cases chạy `cargo test` vượt qua 100% không có cảnh báo.

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
