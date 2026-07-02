# 3b. Động cơ Chống Thanh lý (Liquidation Protector)

> [!IMPORTANT]
> Đây là chốt chặn cuối cùng ngăn chặn tài khoản của bạn bị bốc hơi do sự chênh lệch giữa giá Dừng lỗ (SL) logic và giá Thanh lý (Liquidation) của sàn.

## Bài toán Thanh lý Toán học
Khi sử dụng đòn bẩy $L$, mức giá thanh lý xấp xỉ nằm cách giá vào lệnh một khoảng:
$\text{Liq\_Dist} \approx \frac{1}{L} \times \text{Entry\_Price}$

Ví dụ: Entry = 50,000, L = 100x -> Khoảng cách thanh lý $\approx 500$ giá.
Nếu hệ thống tính toán Dừng lỗ theo ATR (ví dụ ATR đang cao, SL = 600 giá).
Điều gì sẽ xảy ra?
Lệnh của bạn bị SÀN THANH LÝ ở mức -500 giá, khiến bạn mất toàn bộ Margin trước khi bot kịp kích hoạt Dừng lỗ ở mức -600 giá.

## Đề xuất: Tích hợp Liquidation Engine

Thêm một module kiểm tra chéo vào trước khi hàm `execute_tactical` bắn lệnh:

```rust
fn is_safe_from_liquidation(
    entry: f64, 
    sl: f64, 
    leverage: f64, 
    maintenance_margin_pct: f64
) -> bool {
    // Tính khoảng cách SL tuyệt đối
    let sl_distance_pct = (entry - sl).abs() / entry;
    
    // Tính khoảng cách thanh lý ước tính
    let bankruptcy_distance_pct = 1.0 / leverage;
    let liq_distance_pct = bankruptcy_distance_pct - maintenance_margin_pct;
    
    // Đảm bảo SL phải nằm TRƯỚC mức giá bị thanh lý một khoảng đệm an toàn
    let safety_buffer = 0.001; // 0.1% buffer
    
    sl_distance_pct < (liq_distance_pct - safety_buffer)
}
```

Nếu hàm này trả về `false`, hệ thống có 2 lựa chọn:
1. **Từ chối vào lệnh:** Bỏ qua cơ hội này vì rủi ro quá cao.
2. **Kéo SL lại gần (Tighten SL):** Ép SL về mức an toàn tối đa trước thanh lý, dù điều này có nghĩa là dễ bị quét (Whip-saw) hơn. Đối với 100x, lựa chọn 1 (Từ chối) là thông minh hơn.
