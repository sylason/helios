# 🔬 Kiểm Chứng Chuyên Sâu: Bẫy Đóng Lệnh Đột Ngột Khi DNA Hot-Swapping (`decay_cycles` Jump)

**Chủ đề:** Xung đột trạng thái giữa tham số DNA cũ và DNA mới trong `RiskManager`  
**Vị trí liên quan:**  
- [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L133-L151)  
- [`trading_bot/bot_engine/src/controller/asset_actor.rs`](file:///E:/Projects/Automations/trading_backtest/trading_bot/bot_engine/src/controller/asset_actor.rs#L154-L183)  
- [`strategies_backtests/helios/src/strategy/core_v3/config.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/config.rs)  

---

## 📌 1. Triệu Chứng & Hiện Tượng Thực Tế

Trong bài test Walk-Forward kết hợp Re-Optimizer định kỳ ($N = 240$ phút):
- Tại đúng thời điểm một chu kỳ Re-Optimizer hoàn tất và nạp DNA mới vào bot, các vị thế đang mở có PnL âm nhẹ (ví dụ $-0.05\%$) **bị đóng cưỡng bức ngay lập tức** bằng `ExitReason::StrategySignalClose` dù giá chưa hề chạm Stop Loss hay Take Profit.

---

## 🔬 2. Bằng Chứng Mã Nguồn & Cơ Chế Gây Lỗi

### 🔬 Bằng chứng 1: Logic Cắt Lỗ Ngâm Vốn (Stagnant Capital Release)
Trong [`strategies_backtests/helios/src/strategy/core_v3/risk.rs`](file:///E:/Projects/Automations/trading_backtest/strategies_backtests/helios/src/strategy/core_v3/risk.rs#L133-L146):

```rust
let time_countdown_bars = risk_cfg.time_countdown_bars.max(1);
let decay_cycles = effective_stagnant_bars / time_countdown_bars;
let current_pnl = trade.pl(Some(tick.closes()[i]));

// ❌ BẪY CẮT LỖ NGÂM VỐN:
if decay_cycles > 0 && current_pnl <= 0.0 {
    tick.close(None);
    continue;
}
```

### 🔬 Bằng chứng 2: Bước Nhảy Đột Ngột của `decay_cycles` khi DNA Hot-Swap
1. Giả sử một vị thế đã được mở và giữ qua $20$ cây nến 1m (`effective_stagnant_bars = 20`).
2. Bộ DNA cũ (được tối ưu từ chu kỳ trước) có tham số:
   $$\text{time\_countdown\_bars} = 60 \implies \text{decay\_cycles} = \left\lfloor\frac{20}{60}\right\rfloor = 0$$
   $\rightarrow$ Vị thế vẫn đang trong thời gian an toàn cho phép rung lắc.
3. Khi chu kỳ Re-Optimize mới kết thúc, `ProspectorRunner` tìm ra bộ DNA Champion mới có:
   $$\text{time\_countdown\_bars} = 15$$
4. Ngay khi `AssetActor` nhận DNA mới qua kênh `watch::channel`, nó gọi `strategy.update_from_dna()`.
5. Tại cây nến tiếp theo ($i = 21$), `decay_cycles` lập tức nhảy vọt:
   $$\text{decay\_cycles} = \left\lfloor\frac{21}{15}\right\rfloor = 1$$
6. Nếu tại đúng thời điểm đó, giá nến đang biến động nhẹ khiến `current_pnl = -0.01$`, điều kiện `decay_cycles > 0 && current_pnl <= 0.0` **ngay lập tức thỏa mãn**.
7. Hàm `tick.close(None)` được kích hoạt và đóng vị thế ngay lập tức, tước đoạt toàn bộ cơ hội hồi phục và tăng trưởng của lệnh.

---

## 🛠️ 3. Giải Pháp Kỹ Thuật Đề Xuất

1. **Bảo Lưu Thông Số Risk Management Cho Vị Thế Đang Mở (Position Parameter Pinning):**
   - Khi vị thế được mở, các tham số quản trị rủi ro (`time_countdown_bars`, `initial_sl`, `tsl_atr_multiplier`) của DNA tại thời điểm mở lệnh nên được gắn kèm vào cấu trúc `LivePositionState`.
   - Khi có DNA mới nạp vào, DNA mới chỉ áp dụng cho **các lệnh mở mới trong tương lai**, tránh gây xung đột tham số cho lệnh đang chạy.
2. **Grace Period sau khi Hot-Swap DNA:**
   - Đặt thời gian ân hạn (ví dụ tối thiểu $K$ nến) trước khi cho phép tính toán `decay_cycles` theo bộ tham số mới.
