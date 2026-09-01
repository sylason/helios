# 🔬 Kiểm Chứng Chuyên Sâu: Bẫy Kế Toán `bars_held` & Ánh Xạ `mapped_entry_bar`

**Chủ đề:** Đồng bộ hóa Vị thế theo Timestamp Tuyệt đối (Absolute Timestamp Alignment)  
**Vị trí liên quan:**  
- [`trading_bot/bot_engine/src/execution/paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs)  
- [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs)  
- [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)  

---

## 📌 1. Triệu Chứng & Hiện Tượng Thực Tế

Trong các bài chạy thử nghiệm Forward Paper Trading trên tập dữ liệu dài ngày:
1. Số lượng nến ngâm vị thế (`bars_held`) trong bảng báo cáo lệnh đóng của `PaperTrader` thường xuyên bị gán bằng **`1`**, bất kể vị thế đã mở từ hàng trăm cây nến trước.
2. Hàm tính toán suy giảm động năng theo thời gian (`Stagnant Capital Release` / `decay_cycles`) trong `RiskManager` hoàn toàn bị tê liệt, không nhận diện được vị thế đang bị ngâm vốn (Zombie Trade).

---

## 🔬 2. Bằng Chứng Mã Nguồn & Cơ Chế Gây Lỗi

### 🔬 Bằng chứng 1: Bẫy Kế toán `bars_held` khi Rolling Buffer đạt giới hạn
Trong [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L330-L335):
```rust
// Khi Rolling Buffer đạt tối đa 43,200 nến (30 ngày 1m):
let current_bar_idx = full_buffer.len().saturating_sub(1);
```
- Khi bộ đệm nến đầy $43.200$ nến, mỗi khi nến mới được thêm vào (`push_candle`), cây nến cũ nhất ở đầu bị đẩy ra ngoài (`pop_front`).
- Tổng độ dài `full_buffer.len()` luôn cố định là $43.200$.
- Do đó, `current_bar_idx` luôn bị **ghim cứng tại giá trị 43.199**.
- Nếu tính toán `bars_held` theo công thức cũ:
  $$\text{bars\_held} = \text{exit\_bar\_idx} - \text{entry\_bar\_idx} = 43199 - 43199 = 0 \xrightarrow{\text{max}(1)} \mathbf{1}$$

### 🔬 Bằng chứng 2: Bẫy Ánh xạ `mapped_entry_bar` bằng phép trừ Offset tương đối
Trong [`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L280-L290):
```rust
// Mã nguồn cũ trước khi fix:
let mapped_entry_bar = pos.live_entry_bar.saturating_sub(slice_start);
```
- Khi cửa sổ trượt tịnh tiến theo thời gian (`slice_start` tăng dần theo từng nến), phép trừ `pos.live_entry_bar.saturating_sub(slice_start)` làm chỉ số `mapped_entry_bar` bị co ngắn lại hoặc nhảy sai lệch so với mảng nến đang được nạp vào `TickContext`.
- Trong [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L105-L115):
  ```rust
  let start_b = (entry_bar + 1).min(i);
  let effective_stagnant_bars = if i >= peak_bar { i - peak_bar } else { 0 };
  ```
- Nếu `mapped_entry_bar` bị gán nhầm bằng `i` (nến hiện tại), `effective_stagnant_bars` luôn bằng `0`, khiến `decay_cycles = 0` $\rightarrow$ Tính năng cắt lỗ vị thế ngâm vốn (`Stagnant Capital Release`) không bao giờ được kích hoạt.

---

## 🛠️ 3. Giải Pháp Kỹ Thuật Đã Xác Thực

### A. Chuẩn hóa Kế toán `bars_held` dựa trên Timestamp Tuyệt đối
Trong [`paper_trader.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/execution/paper_trader.rs#L214-L219):
```rust
let duration_ms = exit_timestamp.saturating_sub(self.entry_timestamp);
let bars_held = if self.entry_timestamp > 0 && duration_ms > 0 {
    (duration_ms / 60_000).max(1) as usize
} else {
    exit_bar_index.saturating_sub(self.entry_bar_index).max(1)
};
```

### B. Ánh xạ `mapped_entry_bar` bằng Binary Search trên Timestamp
Trong [`helios_bridge.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/helios_bridge.rs#L280-L287):
```rust
let mapped_entry_bar = if pos.entry_timestamp > 0 {
    match candles.binary_search_by_key(&pos.entry_timestamp, |c| c.timestamp) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
} else {
    pos.live_entry_bar.saturating_sub(slice_start)
};
```
- Vì `candles` là mảng nến đã được sắp xếp tăng dần theo thời gian, `binary_search_by_key` có độ phức tạp cực thấp $O(\log N)$ (khoảng 15 phép so sánh cho $43.200$ nến).
- Đảm bảo tìm chính xác $100\%$ vị trí index nến mở lệnh trong mảng `rolling_buffer`, bất chấp mọi thao tác cắt tỉa (slice) buffer.
