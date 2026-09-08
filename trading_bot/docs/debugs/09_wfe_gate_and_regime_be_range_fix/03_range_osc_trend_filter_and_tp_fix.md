# 🌊 TÀI LIỆU KỸ THUẬT: BỘ LỌC XU HƯỚNG & SỬA LỖI TAKE PROFIT CHO RANGE_OSC
> **Mã lỗi xử lý:** FLAW-4.2 (Take Profit đóng nhầm chiều) & FLAW-4.3 (Lỗi `.abs()` mở Cooldown bắt dao rơi)  
> **Vị trí áp dụng:**  
> - `strategies_backtests/helios/src/strategy/core_v3/config.rs`  
> - `strategies_backtests/helios/src/strategy/core_v3/scanners/range_osc.rs`  
> - `strategies_backtests/helios/src/strategy/core_v3/mod.rs`

---

## 1. Bối cảnh & Điểm nghẽn
- **Hiện trạng:** `RangeOscScanner` chỉ đo khoảng cách giá tới dải Bollinger Bands cục bộ. Khi thị trường xuất hiện cú sập dốc mạnh (Breakout / Strong Trend), giá liên tục ép đáy dải Bollinger.
- **Hậu quả từ Log 24:** `RANGE_OSC` tưởng nhầm là vùng quá bán (Oversold) và kích hoạt lệnh Long 4 lần liên tiếp (Trades #3, #4, #5, #6), gây thiệt hại $57.4\%$ tổng lỗ của bot.
- **FLAW-4.2:** Khi nến bứt phá vượt biên, `suggested_tp` của RangeOsc có thể nhỏ hơn Entry Price (đối với Long), dẫn đến đóng lệnh ngay nến kế tiếp với giá lỗ.
- **FLAW-4.3:** Trong `mod.rs:240`, `price_diff = (current_price - last_exit).abs()`. Khi Long bị cắt lỗ mà giá tiếp tục rơi tự do $\ge 1.5$ ATR, hàm `.abs()` mở khóa Cooldown sớm cho bot nhảy vào bắt dao rơi tiếp!

---

## 2. Thiết kế Giải pháp DNA-Driven

### 2.1. Cấu hình DNA trong `TacticalRangeParams` (`config.rs`)
Bổ sung 2 gen DNA phổ quát:
```rust
pub struct TacticalRangeParams {
    // ... các trường hiện có ...

    /// Ngưỡng sức mạnh Trend tối đa mà Range được phép hoạt động (0.10 -> 0.80, Step 0.05)
    /// Đọc từ matrix.trend_strength (vốn được tính động từ ADX theo Timeframe mà DNA chọn!)
    #[dna(min = 0.10, max = 0.80, step = 0.05)]
    pub max_trend_strength: f64,

    /// Ngưỡng dốc xu hướng KAMA tối đa cấm bắt dao rơi (0.05 -> 0.50, Step 0.05)
    #[dna(min = 0.05, max = 0.50, step = 0.05)]
    pub max_adverse_slope: f64,
}
```

### 2.2. Kiểm tra Bộ Lọc trong `range_osc.rs`
1. Nếu $\text{matrix.trend\_strength}[i] > \text{range\_cfg.max\_trend\_strength} \rightarrow$ Trả về `None` (thị trường đang có xu hướng mạnh, cấm đánh Range).
2. Nếu $\text{matrix.trend\_dir}[i] < -\text{range\_cfg.max\_adverse\_slope} \rightarrow$ Cấm phát tín hiệu Long (chống bắt dao rơi khi dốc xuống).
3. Nếu $\text{matrix.trend\_dir}[i] > +\text{range\_cfg.max\_adverse\_slope} \rightarrow$ Cấm phát tín hiệu Short (chống cản tàu khi dốc lên).
4. **Sửa FLAW-4.2:** Chuẩn hóa Take Profit:
   - Long: `suggested_tp = Some(bb_u.max(close * 1.002))` (đảm bảo TP luôn cao hơn giá vào ít nhất 0.2% để trừ phí sàn).
   - Short: `suggested_tp = Some(bb_l.min(close * 0.998))`.

### 2.3. Sửa FLAW-4.3 trong `mod.rs`
Thay thế biểu thức `.abs()` bằng kiểm tra định hướng:
- Với lệnh Long vừa đóng: Chỉ mở khóa Cooldown sớm nếu giá **tăng hồi phục ngược lên**:
  `current_price - last_exit >= 1.5 * current_atr`. Nếu giá tiếp tục rơi $\rightarrow$ Tiếp tục khóa Cooldown.
- Với lệnh Short vừa đóng: Chỉ mở khóa Cooldown sớm nếu giá **giảm sâu tiếp**:
  `last_exit - current_price >= 1.5 * current_atr`.
