# 🔬 Kiểm Chứng Chuyên Sâu: Bẫy Bóp Nghẹt Stop Loss Khi Vị Thế Đang Âm (Initial SL Invariance)

**Chủ đề:** Bảo toàn Initial Stop Loss khi Vị thế Đang Âm Danh nghĩa (`current_pnl <= 0.0`)  
**Vị trí liên quan:**  
- [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs)  
- [`trading_bot/bot_engine/src/optimizer/prospector_runner.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/optimizer/prospector_runner.rs)  
- [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs)  

---

## 📌 1. Triệu Chứng & Hiện Tượng Thực Tế

1. Trong bài test Walk-Forward kết hợp Re-Optimizer định kỳ: Khi DNA mới được nạp vào lúc một vị thế đang mở và có PnL âm nhẹ (ví dụ $-0.2\%$), vị thế gần như **lập tức bị quét Stop Loss ở ngay nến tiếp theo**.
2. Stop Loss bị co lại sát giá hiện tại thay vì giữ khoảng đệm an toàn của `initial_sl` ban đầu, tước đoạt hoàn toàn không gian thở (wiggle room) của thị trường.

---

## 🔬 2. Bằng Chứng Mã Nguồn & Cơ Chế Gây Lỗi

### 🔬 Bằng chứng 1: Thiếu điều kiện ràng buộc PnL khi dời Trailing Stop Loss
Trong [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L190-L210):
- Trước khi chuẩn hóa, logic dời Trailing Stop Loss chỉ kiểm tra điều kiện so sánh đơn thuần:
  $$\text{should\_update} = (\text{new\_sl} > \text{old\_sl})$$
- Khi thị trường biến động và nến giá tạo đỉnh nhỏ, công thức tính toán khoảng cách SL theo ATR (`new_sl = peak_val - current_atr * multiplier`) có thể trả về một mức giá cao hơn mức Initial SL ban đầu, dù vị thế tổng thể vẫn đang bị **LỖ DANH NGHĨA** (`current_pnl <= 0.0`).
- Mức SL mới bị kéo sát vào giá vào lệnh trong khi giá đang thoái lui, khiến lệnh bị quét Stop Loss non.

### 🔬 Bằng chứng 2: Tác động kép khi DNA Hot-Swapping
Trong [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L160-L175):
- Khi nạp DNA mới, các hệ số ATR (`tsl_atr_normal`, `tsl_atr_tight`) có thể nhỏ hơn DNA cũ.
- Khi `manage_trailing_stop` chạy lại ở nến tiếp theo, nó tính `new_sl` theo hệ số mới siết chặt hơn và phát ra `Action::UpdateTradeSlTp`.
- `PaperTrader` nhận action này và cập nhật ngay `current_sl_price = new_sl`. Lệnh đang âm bị "bóp nghẹt" ngay tại chỗ.

---

## 🛠️ 3. Giải Pháp Kỹ Thuật Đã Xác Thực (Initial SL Invariance)

Trong [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L199-L212):

```rust
// BẢO TOÀN STOP LOSS BAN ĐẦU (INITIAL SL INVARIANCE):
// Chỉ kích hoạt dời Trailing Stop Loss khi vị thế ĐÃ CÓ LÃI DANH NGHĨA (current_pnl > 0.0).
// Khi current_pnl <= 0.0, tuyệt đối giữ nguyên Initial Stop Loss ban đầu, không cho phép dời SL bóp nghẹt vị thế.
if current_pnl > 0.0 {
    let should_update = if trade.is_long() {
        new_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
    } else {
        new_sl < trade.sl_price.unwrap_or(f64::INFINITY)
    };

    if should_update {
        tick.update_trade_sl_tp(trade.id, Some(new_sl), trade.tp_price);
    }
}
```

### Lợi ích đã chứng minh:
1. **Kháng 100% việc co hẹp SL sớm**: Vị thế khi vừa mở hoặc đang trong giai đoạn rung lắc âm nhẹ luôn được bảo vệ bởi toàn bộ biên độ Stop Loss ban đầu (`playbook_max_sl_pct` & `min_initial_sl_pct`).
2. **Chỉ khóa lãi khi đã có lợi nhuận**: Trailing Stop chỉ bắt đầu hoạt động nâng dần đáy bảo vệ khi giá đã đi đúng hướng và sinh lãi thực sự.
