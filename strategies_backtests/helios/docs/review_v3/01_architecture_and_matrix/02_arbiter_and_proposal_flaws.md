# Cấu Trúc Proposal Yếu & Vi Phạm Triết Lý Arbiter

## 1. Điểm Mù Của Cấu Trúc `Proposal`
Cấu trúc trung tâm giao tiếp giữa Scanner và Arbiter hiện tại nằm ở `core_v3/arbiter.rs`:

```rust
pub struct Proposal {
    pub signal_type: SignalType,
    pub raw_score: f64,
    pub scanner_id: &'static str,
}
```

### 🚨 Lỗ Hổng
*   **Thiếu Thông Số Sinh Tồn:** Scanner chỉ có thể hô "Mua" hoặc "Bán", nhưng lại **không thể đưa ra đề xuất điểm Chốt Lời (Take Profit) và Cắt Lỗ (Stop Loss)**.
*   **Mất Khả Năng Tối Ưu Chiến Thuật (Tactical Flexibility):** Ở V2, chiến thuật Range chốt lời tại Band đối diện (Ping-Pong TP), chiến thuật Squeeze có thể cần chốt lời rộng hơn. Với cấu trúc `Proposal` nghèo nàn này, tầng thực thi (Execution) ở `mod.rs` không biết phải set Take Profit ở đâu, dẫn đến việc phải set thành `None` toàn bộ!

### 🛠️ Giải Pháp
Bổ sung tính năng tự chủ cho Scanner:
```rust
pub struct Proposal {
    pub signal_type: SignalType,
    pub raw_score: f64,
    pub scanner_id: &'static str,
    pub suggested_sl: Option<f64>, // Scanner tự tính SL
    pub suggested_tp: Option<f64>, // Scanner tự tính TP (Cực kỳ quan trọng cho Range)
}
```

---

## 2. Scanner Đang Làm Hộ Việc Của Arbiter
Triết lý của kiến trúc Signal-First là: **"Scanner chỉ lo tìm Pattern đẹp. Arbiter lo duyệt xem hoàn cảnh thị trường có phù hợp không"**. 

Nhưng hiện tại trong tệp `range_osc.rs`, Scanner lại đang tự kiểm duyệt chính mình:
```rust
let trend_strength = matrix.trend_strength[i];
if trend_strength > 0.4 {
    return None; // Xu hướng quá mạnh, không đánh Range
}
```

### 🚨 Lỗ Hổng
*   **Chống Lại Hệ Sinh Thái Multiplier:** Arbiter đã có sẵn `range_multiplier` (Được tính từ `1.0 - trend_strength`). Nếu xu hướng mạnh, `range_multiplier` tự động thấp, và Arbiter sẽ tự động loại Proposal của Range vì `confidence` thấp. 
*   **Hậu Quả:** Việc Scanner tự dùng một con số chết `0.4` để "im mồm" là dư thừa, làm giảm tính năng động của mô hình Arbiter, và lại đưa ta về cái máng lợn của cấu trúc `if-else` cứng nhắc của V1/V2.

### 🛠️ Giải Pháp
Phải loại bỏ toàn bộ các điều kiện lọc môi trường/Regime bên trong thân của các Scanner. Hãy để Scanner thuần túy tập trung chấm điểm (Score) mức độ hoàn hảo của Pattern, mọi việc cọ xát với môi trường phải để cho Arbiter cầm cân nảy mực.
