# 2a. Lỗ hổng Chí mạng: Quản lý Vốn đòn bẩy cao (Position Sizing)

> [!CAUTION]
> Mã nguồn hiện tại trong `tactical.rs` chứa một lỗ hổng có thể làm cháy tài khoản ngay lập tức khi dùng đòn bẩy 100x.

## Hiện trạng Bug trong `tactical.rs`
Code đang chia vốn một cách cố định bằng `size_fraction` (Margin Fraction):
```rust
let size_fraction = if equity < 100.0 { Some(0.2) } else { Some(0.1) };
tick.buy(size_fraction, ...)
```

**Tại sao đây là tự sát?**
Nếu bạn có 1000$ và dùng `size_fraction = 0.1` (10%), bạn ký quỹ 100$.
Với đòn bẩy 100x, vị thế danh nghĩa (Notional Position) của bạn là `100$ * 100 = 10,000$`.
Nếu giá đi ngược 1%, bạn lỗ `10,000$ * 1% = 100$`. Tài khoản ký quỹ của bạn bốc hơi hoàn toàn (Bị thanh lý). Trong chế độ Cross-margin, nó có thể ăn cả 900$ còn lại.

Hệ thống đặt Stop Loss dựa trên ATR (`atr_raw * sl_factor`). Nếu khoảng cách SL này lớn hơn 1%, sàn sẽ thanh lý lệnh trước khi giá kịp chạm Stop Loss của bạn!

## Giải pháp: Sizing theo Rủi ro (Risk-Based Sizing)

Phải chuyển tư duy từ "Dùng bao nhiêu tiền để đánh?" sang "Chấp nhận mất bao nhiêu tiền cho lệnh này?".

### Mã giả đề xuất sửa chữa (Pseudo-code)

```rust
// 1. Xác định số tiền sẵn sàng mất (Ví dụ: 2% tổng vốn)
let risk_per_trade_pct = 0.02; 
let max_loss_usd = equity * risk_per_trade_pct;

// 2. Tính khoảng cách Stop Loss tuyệt đối (Dollar Distance)
// Ví dụ: Entry = 50000, SL = 49500 -> Khoảng cách = 500
let sl_dist_usd = current_close - sl_price;

// 3. Tính toán Khối lượng vị thế Danh nghĩa (Notional Size)
// Số lượng Coin/Hợp đồng = Số tiền rủi ro / Khoảng cách giá SL
let contract_size = max_loss_usd / sl_dist_usd;

// 4. Tính toán số Margin cần thiết (ký quỹ) dựa trên Đòn bẩy
let leverage = 100.0;
let required_margin = (contract_size * current_close) / leverage;

// 5. Cầu chì chống thanh lý: Đảm bảo Margin không vượt quá Equity
if required_margin > equity * 0.5 {
    // Rủi ro thanh lý quá cao, không vào lệnh!
    return TacticalPlaybook::Wait;
}

// Thực thi lệnh với số hợp đồng chính xác
tick.buy(Some(required_margin / equity), ...)
```

Với công thức này, dù SL ở xa (ATR lớn) hay ở gần, nếu chạm SL, bạn **luôn luôn chỉ mất đúng 2% vốn**. Đây là Chén Thánh của quản lý rủi ro đòn bẩy cao.
