# 📑 ĐẶC TẢ KỸ THUẬT: KHÔI PHỤC INITIAL SL `base_sl_atr` & KÍCH HOẠT UNDERWATER TSL

> **Mục tiêu:** 
> 1. Kết nối lại tham số DNA `base_sl_atr` cho Initial Stop Loss thay vì dùng nhầm `tsl_atr_normal`.
> 2. Khai thông cơ chế siết Stop Loss khi vị thế đang âm (`current_pnl <= 0`) theo Time Decay, kèm theo các chốt chặn chống Whipsaw và chống vượt giá hiện tại.

---

## 1. BỐI CẢNH & NGUYÊN NHÂN GỐC

### 1.1. Lỗi Initial SL dùng nhầm `tsl_atr_normal` (`mod.rs:302`)
- **Hiện trạng:**
  ```rust
  let raw_sl = decision.final_sl.unwrap_or_else(|| {
      RiskManager::get_initial_sl(
          tick,
          &self.sensors,
          is_long,
          self.config.risk_management.stop_management.tsl_atr_normal, // 🚨 SAI
          playbook_max_sl_pct,
          min_initial_sl_pct,
          leverage,
      )
  });
  ```
- **Hậu quả:** `base_sl_atr` ($0.5 \dots 6.0$ ATR) bị bỏ rơi. Các Scanner không có SL riêng phải dùng `tsl_atr_normal` ($3.0 \dots 7.5$ ATR), khiến khoảng cách SL ban đầu quá rộng ($1.5\% \dots 2.5\%$ giá ETH), gây lỗ $-30\% \dots -50\%$ ROE khi dính SL ở đòn bẩy $20x$.

### 1.2. Lỗi TSL bị đóng băng khi âm (`risk.rs:295`)
- **Hiện trạng:**
  ```rust
  if current_pnl > 0.0 {
      let should_update = ...
      if should_update {
          tick.update_trade_sl_tp(trade.id, Some(calculated_sl), trade.tp_price);
      }
  }
  ```
- **Hậu quả:** Khi giá đi ngược kỳ vọng hoặc chìm dưới giá vốn, `current_pnl <= 0.0`. Toàn bộ khối logic dời SL bị bỏ qua, dẫn đến việc vị thế ngâm hàng nghìn nến mà SL không thể siết lại gần giá vốn để cắt lỗ sớm bảo toàn vốn.

---

## 2. GIẢI PHÁP THIẾT KẾ QUANT

### 2.1. Nối lại `base_sl_atr` tại `mod.rs`
- Trong `HeliosStrategyV3::next`:
  ```rust
  let raw_sl = decision.final_sl.unwrap_or_else(|| {
      RiskManager::get_initial_sl(
          tick,
          &self.sensors,
          is_long,
          self.config.risk_management.stop_management.base_sl_atr, // ✅ CHUẨN DNA
          playbook_max_sl_pct,
          min_initial_sl_pct,
          leverage,
      )
  });
  ```

### 2.2. Khai thông TSL khi lệnh âm trong `risk.rs`
- Tách biệt rõ 2 luồng:
  1. **Khi có lãi (`current_pnl > 0.0`):** Áp dụng High Watermark Peak, Dynamic BE và Mega Trend Floor như hiện tại.
  2. **Khi đang âm (`current_pnl <= 0.0` và `decay_cycles > 0`):**
     - Cho phép Stagnant Time Decay thu hẹp khoảng cách SL từ giá vào lệnh (`entry_price`):
       - LONG: `tightened_sl = trade.entry_price - (current_atr * current_multiplier)`
       - SHORT: `tightened_sl = trade.entry_price + (current_atr * current_multiplier)`
     - **Chốt chặn 1 (Buffer Floor chống Whipsaw):** Không được siết vượt quá `current_close - 1.0 * current_atr` (với Long) hoặc `current_close + 1.0 * current_atr` (với Short) để giá luôn có không gian tối thiểu $1.0$ ATR để thở.
     - **Chốt chặn 2 (Chống nhảy qua đầu giá):** Nếu `tightened_sl >= current_close` (Long) hoặc `<= current_close` (Short) $\rightarrow$ Lệnh ngâm đã hết hạn, gọi trực tiếp `tick.close(None)` để thoát vị thế dứt khoát.
     - **Chốt chặn 3 (Monotonic Ratchet):** Bất kể âm hay dương, chỉ cho phép dời TIẾN (Long: `calculated_sl > sl_price`; Short: `calculated_sl < sl_price`), TUYỆT ĐỐI CẤM LÙI SL.

---

## 3. KẾ HOẠCH KIỂM THỬ
1. Viết Unit Test xác nhận `get_initial_sl` sử dụng đúng `base_sl_atr`.
2. Viết Unit Test xác nhận khi vị thế đang âm và số nến ngâm tăng lên (`decay_cycles > 0`), SL được dời siết lại gần giá vốn mà không vi phạm tính đơn điệu.
3. Chạy toàn bộ test suites (`ferro_trade`, `helios`, `bot_engine`).
