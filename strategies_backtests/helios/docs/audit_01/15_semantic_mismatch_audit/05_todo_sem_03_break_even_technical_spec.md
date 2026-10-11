# ĐẶC TẢ THIẾT KẾ KỸ THUẬT: TÁI CẤU TRÚC ĐỘNG CƠ BREAK-EVEN (TODO-SEM-03)

> **Mã Nhiệm Vụ:** `TODO-SEM-03`  
> **Phân Hệ:** Quản Trị Rủi Ro & Phòng Vệ Vốn (`risk.rs` & `config.rs`)  
> **Độ Ưu Tiên:** 🔴 P0 (Cốt Tử / Ngăn Chặn Bào Mòn Vốn & Tự Cắt Lỗ Oan)  
> **Nguyên Tắc Thiết Kế:** First-Principles, Occam's Razor, Zero Magic Numbers, Separation of Concerns.

---

## 1. BỐI CẢNH & HIỆN TRẠNG LỖ HỔNG (THE CRITICAL FLAW)

Trong kiến trúc Quản trị Rủi ro của Helios V3 tại [`risk.rs`](file:///e:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs), cơ chế Break-Even và xử lý lệnh đóng khẩn cấp tồn tại hai dị tật toán học và nghiệp vụ nghiêm trọng:

### 1.1. Lỗ Hổng 1: Kích Hoạt Hòa Vốn Sớm Khi Chưa Đủ Bù Phí Sàn (Dòng 436–441)
* **Code hiện tại:**
  ```rust
  let min_safe_be_roe = (fee_buffer_pct * leverage * 1.5).max(risk_cfg.base_be_roe_trigger.max(0.06));
  let required_profit = current_atr * be_trigger_mult * trade.size.abs();
  let is_be_triggered = peak_roe >= min_safe_be_roe || peak_profit >= required_profit;
  ```
* **Bản chất toán học:**
  - Vế `peak_profit >= required_profit` tương đương $\Delta P_{\text{peak}} \ge \text{current\_atr} \times \text{be\_trigger\_mult}$.
  - Trên khung nến vi mô 1m/3m/5m (`sensors.atr_sl`), giá trị `current_atr` lúc thị trường êm đềm rất nhỏ (ví dụ ETHUSDT chỉ $\$0.50 \sim \$1.00$, tương đương $0.02\% \sim 0.04\%$ giá).
  - Với `be_trigger_mult = 1.0`, chỉ cần giá nhích nhẹ $+0.02\%$ (khoảng $+0.40\%$ ROE tại 20x), điều kiện đã thỏa mãn do toán tử `||` (OR).
  - Trong khi đó, phí sàn 2 chiều + trượt giá là $fee\_buffer\_pct = 0.10\%$ ($2.0\%$ ROE tại 20x). Mức tăng $+0.02\%$ chưa đủ trả $1/5$ tiền phí sàn!

### 1.2. Lỗ Hổng 2: Dị Tật Bán Tháo Bằng Lệnh Market (`Emergency BE Rejection Close`) (Dòng 534–560)
* **Code hiện tại:**
  ```rust
  let min_be = trade.entry_price * (1.0 + fee_buffer_pct);
  if current_close <= min_be {
      tick.close(None); // 🛑 Bán tháo cắt lỗ khẩn cấp bằng Market!
      continue;
  }
  ```
* **Hậu quả thực tế:**
  - Ngay sau khi `is_be_triggered = true` oan uổng ở mốc $+0.02\%$, nến 1m đóng cửa ở $+0.02\%$ hiển nhiên nhỏ hơn `min_be` ($+0.10\%$).
  - Khối code này lập tức kích hoạt, ép bot bán tháo cắt lỗ bằng lệnh Market ngay tại nến đóng, chịu trọn vẹn khoản lỗ phí sàn $-0.08\%$ giá ($-1.6\%$ ROE).
  - Thay vì bảo vệ vốn, cơ chế này biến thành cỗ máy tự động bào mòn tài khoản.

### 1.3. Các Xung Đột Logic Ngầm (Hidden Structural Conflicts)
1. **Xung đột Trailing Take Profit (TTP) dòng 456:** `is_ttp_active = (peak_roe >= ttp_act_roe) || is_be_triggered;` khiến TTP bị kích hoạt non cùng lúc với BE, làm chốt lời non sóng lớn.
2. **Xung đột Monotonic Ratchet Guard dòng 652:** `if current_pnl > 0.0` chặn không cho đồng bộ SL mới khi giá đang retest quanh Entry.

---

## 2. NGUYÊN LÝ THIẾT KẾ TỐI ƯU (FIRST-PRINCIPLES SPECIFICATION)

### 2.1. Trụ Cột 1: Sàn Kinh Tế Kép (Economic Floor Engine)
Điều kiện kích hoạt Break-Even bắt buộc phải vượt qua Sàn Kinh Tế cả về tỷ lệ ROE lẫn biên độ giá thực tế:

$$\text{min\_safe\_be\_roe} = \max\Big(\text{fee\_buffer\_pct} \times \text{leverage} \times 2.0,\ \max(\text{risk\_cfg.base\_be\_roe\_trigger},\ 0.06)\Big)$$

$$\text{required\_price\_gain} = \max\Big(\text{trade.entry\_price} \times \text{fee\_buffer\_pct} \times 2.0,\ \text{current\_atr} \times \text{be\_trigger\_mult}\Big)$$

$$\text{is\_be\_triggered} = (\text{peak\_dist} \ge \text{required\_price\_gain}) \land (\text{peak\_roe} \ge \text{min\_safe\_be\_roe})$$

* **Ý nghĩa:**
  - Luôn đảm bảo lợi nhuận đỉnh tối thiểu gấp ít nhất **$2.0\text{x}$ tổng chi phí sàn 2 chiều** trước khi nghĩ đến việc dời Stop Loss.
  - Tự động co giãn theo đòn bẩy thực tế ($20\text{x}, 50\text{x}, 100\text{x}$).

### 2.2. Trụ Cột 2: Mức Dời Stop Loss Hòa Vốn Có Đệm Lãi Nhẹ (True Net BE + Micro Cushion)
Khoảng đệm hòa vốn sử dụng trực tiếp cấu hình `true_be_roundtrip_fee_pct` (hoặc `fee_buffer_pct + 0.0003` = $0.13\%$ giá):

$$\text{min\_be (Long)} = \text{Entry} \times \big(1.0 + \text{fee\_buffer\_pct} + 0.0003\big)$$
$$\text{max\_be (Short)} = \text{Entry} \times \big(1.0 - \text{fee\_buffer\_pct} - 0.0003\big)$$

$$\text{target\_be\_sl (Long)} = \max\Big(\text{peak\_val} - \text{current\_atr} \times \text{be\_buffer\_mult},\ \text{min\_be}\Big)$$
$$\text{target\_be\_sl (Short)} = \min\Big(\text{peak\_val} + \text{current\_atr} \times \text{be\_buffer\_mult},\ \text{max\_be}\Big)$$

* **Ý nghĩa:**
  - Nếu giá tiếp tục chạy xa, Stop Loss được neo lùi dưới đỉnh một khoảng `be_buffer_mult` nến ATR.
  - Sàn chặn dưới của Stop Loss luôn là `min_be` (đã có đệm lãi nhẹ $+0.03\%$ giá), đảm bảo nếu bị cắn SL thì PnL thực tế sau phí luôn $\ge \$0.00$.

### 2.3. Trụ Cột 3: Triệt Tiêu Lệnh Bán Tháo Market (`Emergency BE Rejection Close`)
* **Hành động:** Xóa bỏ hoàn toàn khối `if current_close <= min_be { tick.close(None); continue; }` tại cả 2 chiều Long và Short.
* **Nguyên tắc:** Mức `target_be_sl` được gửi sang Broker/Sàn. Nếu giá đảo chiều, cơ chế khớp Stop Loss tự nhiên của sàn sẽ thực hiện nhiệm vụ của nó. Tuyệt đối không can thiệp bằng lệnh Market đóng non.

### 2.4. Trụ Cột 4: Hóa Giải Toàn Diện Các Xung Đột Ngầm
1. **Tách biệt TTP khỏi BE:**
   ```rust
   // Dòng 456:
   let is_ttp_active = peak_roe >= ttp_act_roe;
   ```
2. **Khai thông Monotonic Ratchet Guard:**
   Cho phép cập nhật mức `calculated_sl` mới bất cứ khi nào mức SL này tiến lên theo hướng có lợi cho tài khoản (`sl_advances`), không bị chặn bởi điều kiện `current_pnl > 0.0`:
   ```rust
   let sl_advances = if trade.is_long() {
       calculated_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
   } else {
       calculated_sl < trade.sl_price.unwrap_or(f64::INFINITY)
   };

   if sl_advances || tp_changed {
       let sl_to_send = if sl_advances { Some(calculated_sl) } else { trade.sl_price };
       tick.update_trade_sl_tp(trade.id, sl_to_send, new_tp_to_update);
   }
   ```

---

## 3. DANH MỤC TEST CASES KIỂM CHỨNG (VERIFICATION MATRIX)

| Mã Test | Kịch Bản Kiểm Thử | Điều Kiện Đầu Vào | Hành Vi Kỳ Vọng |
| :---: | :--- | :--- | :--- |
| **TC-01** | **Chặn kích hoạt sớm nến vi mô** | ETH \$2500, ATR 1m = \$0.50, Râu nến tăng +\$0.60 ($+0.024\%$, ROE $+0.48\%$) | `is_be_triggered == false`. SL giữ nguyên mức Initial SL ban đầu. |
| **TC-02** | **Kích hoạt chuẩn khi vượt Sàn Kinh Tế** | ETH \$2500, Râu nến tăng +\$7.50 ($+0.30\%$, ROE $+6.0\%$) | `is_be_triggered == true`. `calculated_sl` nâng lên $\ge \$2503.25$ (Entry + $0.13\%$). |
| **TC-03** | **Miễn nhiễm Retest nến đóng** | Râu nến đạt +\$8.0 (đã kích hoạt BE), nhưng cuối nến đóng ở \$2501.0 ($< min\_be$) | **KHÔNG** bị gọi `tick.close()`. Lệnh tiếp tục sống với SL đặt tại `min_be`. |
| **TC-04** | **Độc lập TTP khỏi BE** | Lệnh kích hoạt BE (ROE $+6.0\% < ttp\_act\_roe$ $15\%$) | `is_ttp_active == false`. Không bị chốt non bởi logic Callback. |
| **TC-05** | **Ratchet Guard hoạt động khi retest** | Đã thỏa mãn BE, nến hiện tại đóng ở \$2499.0 (`current_pnl <= 0.0`) | `sl_advances == true`, `tick.update_trade_sl_tp` vẫn được gọi gửi mức SL hòa vốn. |
