# 🏛️ ĐẶC TẢ KỸ THUẬT: SỬA LỖI FLAW-4.4 (FALSE WIN KHI NẾN RÚT CHÂN & TỐI ƯU HÓA TICKCONTEXT)

> **Mã lỗi kiểm toán:** `FLAW-4.4`  
> **Mức độ rủi ro:** **CRITICAL** (Nhận diện sai lệnh thua thành lệnh thắng khi nến rút chân, vô hiệu hóa Cooldown và đu đỉnh/bắt dao rơi).  
> **Thư mục lưu trữ:** `trading_bot/docs/debugs/11_simplex_nan_and_false_win_fix/02_flaw_4_4_last_closed_trade_spec.md`  

---

## 1. Bản Chất Vấn Đề & Bằng Chứng Mã Nguồn

### 1.1. Hiện trạng trong `strategies_backtests/helios/src/strategy/core_v3/mod.rs`
Tại dòng 183-208:
```rust
    let current_close = tick.closes()[i];
    let is_long = last_trade.is_long();
    let trade_pnl = if is_long {
        current_close - last_trade.entry_price
    } else {
        last_trade.entry_price - current_close
    };

    // Nếu lệnh vừa đóng hòa vốn hoặc lỗ danh nghĩa (<= 0.001)
    if trade_pnl <= 0.001 {
        self.cooldown_tracker.locked_until_bar = i + stop_cfg.re_entry_cooldown_bars;
        self.cooldown_tracker.locked_side = if is_long { Some(SignalType::Long) } else { Some(SignalType::Short) };
        self.cooldown_tracker.last_exit_price = current_close;
        ...
    } else if self.debug_mode {
        warn!(
            "ℹ️ [COOLDOWN SKIPPED - FALSE WIN?] Bar: {} | Side: {:?} | Entry: {:.2} | Close: {:.2} | PnL: {:.4} > 0.001 -> Cooldown NOT triggered",
            i, if is_long { SignalType::Long } else { SignalType::Short }, last_trade.entry_price, current_close, trade_pnl
        );
    }
```

### 1.2. Kịch bản Bẫy Đu Đỉnh "False Win"
1. **Lệnh Long mở tại:** $\$2500.00$. Stop Loss đặt tại $\$2485.00$.
2. **Trong nến tiếp theo:** Giá sập mạnh xuống $\$2480.00$. Lệnh bị cắt lỗ thực tế tại mức SL $\$2485.00$ (**Lỗ thực tế: $-\$15.00$**). Vị thế trên sàn đã bị đóng (`is_pos_open = false`).
3. **Cuối nến đó:** Phe bắt đáy nhảy vào tạo râu nến dài (Hammer/Rút chân), kéo giá đóng cửa `Close` bật vọt lên **$\$2505.00$**.
4. **Hàm `next()` chạy khi nến đóng:**
   - Đọc `current_close = 2505.00`.
   - Tính toán PnL: `current_close - entry_price = 2505.00 - 2500.00 = +$5.00`!
   - Kiểm tra: `5.00 <= 0.001` $\implies$ **`false`**!
   - Code nhận định: *"Đây là một lệnh THẮNG ăn +$5! Không cần kích hoạt Cooldown!"* (Chính comment trong code ghi: `COOLDOWN SKIPPED - FALSE WIN?`).
5. **Hậu quả:** Cooldown Tracker bị vô hiệu hóa hoàn toàn. Arbiter nhìn thấy nến xanh rút chân bứt phá mạnh, liền **phát tín hiệu MUA NGAY ĐỈNH CÂY NẾN VỪA RÚT CHÂN ($\$2505.00$)** $\implies$ Bot vừa dính cắt lỗ $-\$15$ xong lại đu ngay đỉnh mới, tiếp tục bị cắt lỗ lần thứ 2!

---

## 2. Thiết Kế Kỹ Thuật: Phương Án 3 (Expose `last_closed_trade` Trong `TickContext`)

### 2.1. Cập nhật `TickContext` trong `ferro_trade/src/backtest/context.rs`
Bổ sung một trường tham chiếu mượn Zero-Cost:
```rust
pub struct TickContext<'a> {
    pub i: usize,
    pub equity: f64,
    pub data_view: &'a DataView,
    pub indicator_manager: &'a IndicatorManager,
    pub trades: &'a [Trade],
    pub last_closed_trade: Option<&'a Trade>, // [ZERO-COST REFERENCE] Lệnh vừa đóng gần nhất trong Broker
    pub actions: &'a mut SmallVec<[Action; 4]>,
    pub position: Position,
}
```

### 2.2. Nạp dữ liệu trong `ferro_trade/src/backtest/engine.rs` & `bot_engine/helios_bridge.rs`
Khi khởi tạo `TickContext`:
```rust
let mut tick_context = TickContext {
    i,
    equity: current_equity,
    data_view: &broker.data,
    indicator_manager: &indicator_manager,
    trades: &broker.trades,
    last_closed_trade: broker.closed_trades.last(),
    actions: &mut actions_buffer,
    position: position_snapshot,
};
```

### 2.3. Cập nhật logic Cooldown trong `strategies_backtests/helios/src/strategy/core_v3/mod.rs`
Xóa bỏ hoàn toàn việc tính PnL theo `current_close`. Thay bằng đọc trực tiếp từ `tick.last_closed_trade`:
```rust
        // 2. Theo dõi trạng thái đóng lệnh để kích hoạt Cooldown chống cào lệnh
        if !is_pos_open {
            let stop_cfg = &self.config.risk_management.stop_management;
            if stop_cfg.enable_re_entry_cooldown && stop_cfg.re_entry_cooldown_bars > 0 {
                if let Some(closed) = tick.last_closed_trade {
                    // Kiểm tra xem lệnh này có vừa đóng trên cây nến hiện tại (i) hoặc nến liền trước (i - 1)
                    let exit_bar = closed.exit_bar.unwrap_or(0);
                    if exit_bar == i || exit_bar == i.saturating_sub(1) {
                        let is_long = closed.is_long();
                        let real_pnl = closed.pl(None); // PnL thực tế 100% từ Broker khớp lệnh
                        let exit_price = closed.exit_price.unwrap_or_else(|| tick.closes()[i]);

                        // Nếu lệnh vừa đóng hòa vốn hoặc lỗ danh nghĩa (<= 0.001)
                        if real_pnl <= 0.001 {
                            self.cooldown_tracker.locked_until_bar = i + stop_cfg.re_entry_cooldown_bars;
                            self.cooldown_tracker.locked_side = if is_long { Some(SignalType::Long) } else { Some(SignalType::Short) };
                            self.cooldown_tracker.last_exit_price = exit_price;

                            if self.debug_mode {
                                warn!(
                                    "🛑 [COOLDOWN TRIGGERED] Bar: {} | Side: {:?} | Entry: {:.2} | Exit: {:.2} | Realized PnL: {:.4} <= 0.001 | Locked until Bar: {} ({} bars)",
                                    i, self.cooldown_tracker.locked_side, closed.entry_price, exit_price, real_pnl, self.cooldown_tracker.locked_until_bar, stop_cfg.re_entry_cooldown_bars
                                );
                            }
                        }
                    }
                }
            }
            self.prev_open_trade = None;
        }
```

---

## 3. Khảo Sát Tính Toàn Vẹn & Edge Cases

1. **Nến rút chân râu dài (Hammer/Inverted Hammer):** PnL được tính từ `closed.exit_price` (mức Stop Loss thực tế), miễn nhiễm 100% với giá `Close` của nến rút chân.
2. **Lệnh Take Profit:** `closed.pl(None) > 0.001` chính xác, không bị nhận diện nhầm thành lỗ.
3. **Lệnh đóng bằng Stagnant Decay (Action::ClosePosition):** `closed.exit_price` được gán chính xác giá thị trường khi đóng, phản ánh đúng lãi/lỗ thực tế.
4. **Hiệu năng:** `Option<&'a Trade>` là một con trỏ tham chiếu trên stack, Zero Heap Allocation, chi phí $O(1)$.
