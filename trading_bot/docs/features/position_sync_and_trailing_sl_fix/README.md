# Technical Specification: Position Synchronization & Trailing Stop Loss Invariance

## 1. Bối cảnh & Vấn đề Kỹ thuật
Khi chạy hệ thống Backtest tịnh tiến (Walk-Forward) hoặc Live Paper Trading, có sự sai lệch lớn giữa kết quả Single Backtest Eval (In-Sample / Rolling 30D) và Paper Trading thực tế:
1. **Lỗi Kế toán `bars_held`:** Khi Rolling Buffer đạt tối đa 43,200 nến, mỗi nến mới vào làm nến cũ nhất bị pop ra (`ohlcv.pop_front()`). Việc lấy chỉ số nến tương đối `current_bar_idx = full_buffer.len().saturating_sub(1)` khiến `current_bar_idx` bị ghim cố định ở 43199. Dẫn đến `bars_held` luôn tính ra 1 bar.
2. **Lỗi Ánh xạ `mapped_entry_bar` trong HeliosBridge:** `pos.live_entry_bar.saturating_sub(slice_start)` bị ghim ở nến hiện tại ($i = \text{entry\_bar}$), khiến Strategy và `RiskManager` mất khả năng nhận diện thời gian ngâm lệnh thực tế (`effective_stagnant_bars = 0`, `decay_cycles = 0`).
3. **Hiện tượng Bóp nghẹt Stop Loss khi Lệnh Đang Âm:** Khi Re-Optimizer nạp DNA mới lúc vị thế đang âm danh nghĩa (`current_pnl <= 0.0`), `RiskManager` tính toán lại SL theo tham số mới và kéo SL sát giá hiện tại, khiến lệnh bị quét Stop Loss cưỡng bức ở ngay nến tiếp theo.

## 2. Giải pháp Kiến trúc

### A. Đồng bộ hóa Vị thế theo Timestamp Tuyệt đối (Absolute Timestamp Alignment)
- Lưu trữ `entry_timestamp: i64` cho mọi vị thế.
- `bars_held = ((exit_timestamp - entry_timestamp) / 60_000).max(1)`.
- Trong `HeliosBridge::evaluate_live_tick`, tìm `mapped_entry_bar` trong `rolling_buffer` bằng tìm kiếm nhị phân $O(\log N)$:
  ```rust
  let mapped_entry_bar = match rolling_buffer.binary_search_by_key(&pos.entry_timestamp, |c| c.timestamp) {
      Ok(idx) => idx,
      Err(idx) => idx.saturating_sub(1),
  };
  ```

### B. Bảo toàn Initial Stop Loss khi Vị thế Đang Âm (Initial SL Invariance)
- Khi `current_pnl <= 0.0`, không cho phép Trailing Stop dời mức SL lại gần giá hơn Initial SL ban đầu.
- Chỉ kích hoạt cơ chế Trailing Stop bám đỉnh (`peak_val`) khi vị thế đã có lãi danh nghĩa (`current_pnl > 0.0`).
