# Đề xuất 02: Chiến thuật Kích hoạt Nuke (The Nuke Deployment)

Dựa trên Q4, bạn chọn chiến thuật "Nuke" thay vì Pyramiding. Nuke nghĩa là khi hội tụ đủ điều kiện vàng, bot sẽ Đóng vị thế cũ, dồn lực mở lệnh đòn bẩy 100x, và kích hoạt Trailing Stop cực kỳ sát.

## Điều kiện Kích hoạt Nuke

Để mã hóa ý tưởng này vào `tactical.rs`, chúng ta cần tạo một Playbook mới hoặc cấu hình lại tính năng `Emergency Override` của đội Anomaly/Climax:

1. **Độ tin cậy Tuyệt đối (Absolute Confidence):** Điểm số (Score) của Regime hiện tại phải đạt `> 0.98` và Stability Index phải `> 0.9` (Môi trường cực kỳ trong sạch).
2. **Sự kiện Bóp cò:** Mọi chỉ số (Momentum, Z-Score, MACD) đều đồng thuận 100%.

## Quy trình Hành động Nuke

```rust
if is_nuke_conditions_met {
    // 1. Dọn dẹp chiến trường
    if tick.position().is_open() {
        tick.close(None); // Đóng ngay lập tức lệnh hiện tại dù lãi hay lỗ
    }
    
    // 2. Setup Trailing Stop cực sát (Tight SL)
    // Nuke không có chỗ cho sai lầm. Điểm SL chỉ cách Entry đúng 0.5 ATR.
    let nuke_sl_dist = atr_raw * 0.5; 
    let sl = current_close - nuke_sl_dist; // (Cho lệnh Long)
    
    // 3. Không đặt Take Profit cứng
    // Để cho sóng tự trôi, chỉ dùng Trailing Stop để chặn lãi.
    
    // 4. Vào lệnh
    tick.buy(nuke_size, Some(RiskPolicy::TrailingPrice { ... }), Some("NUKE_L"));
}
```

**Bảo vệ Thanh khoản:** Ngay cả khi đánh Nuke, việc tính toán Size hợp đồng vẫn phải tuân thủ Đề xuất 01 (Risk-Based Sizing) để đảm bảo nếu Nuke bị xịt (Miss), bạn chỉ mất đúng số tiền rủi ro được phép, không cháy cả tài khoản gốc.
