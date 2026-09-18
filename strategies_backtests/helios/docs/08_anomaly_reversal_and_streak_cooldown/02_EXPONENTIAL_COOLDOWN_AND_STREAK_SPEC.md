# 📐 ĐẶC TẢ KỸ THUẬT: EXPONENTIAL COOLDOWN & LOSING STREAK MEMORY

---

## 🎯 1. Mục Tiêu Thiết Kế
- Chấm dứt triệt để tình trạng "say đòn", mở lệnh trả thù (Revenge Trading) hoặc liên tục bắt dao rơi sau khi vừa bị cắt lỗ.
- Ngăn chặn tình trạng Cooldown chỉ khóa cố định 30 nến rồi để Scanner nhảy vào tự sát tiếp.
- **Đảm bảo tính linh hoạt:** Không bao giờ khóa vô hạn tài khoản bằng cơ chế **Reset khi thắng** và **Trần thời gian tối đa (Max Cap)**.

---

## 🧠 2. Cấu Trúc Dữ Liệu `CooldownTracker` Mở Rộng

```rust
#[derive(Clone, Debug, Default)]
pub struct CooldownTracker {
    /// Nến bị khóa tới thời điểm hiện tại
    pub locked_until_bar: usize,
    /// Chiều lệnh vừa bị phạt (Long hoặc Short)
    pub locked_side: Option<SignalType>,
    /// Giá thoát lệnh gần nhất
    pub last_exit_price: f64,
    /// Đếm số lần thua liên tiếp cùng chiều
    pub consecutive_losses: usize,
    /// Scanner gần nhất bị cắt lỗ
    pub last_losing_scanner: Option<ScannerId>,
}
```

---

## ⚙️ 3. Cơ Chế Vận Hành Toán Học

### 1. Khi một giao dịch đóng lại (Trade Exit Event):
- **Trường hợp 1: Lệnh CÓ LÃI THỰC TẾ ($\text{Realized PnL} > 0.001$):**
  - **RESET HOÀN TOÀN:**
    $$\text{consecutive\_losses} = 0$$
    $$\text{locked\_until\_bar} = 0$$
    $$\text{locked\_side} = \text{None}$$
  - Hệ thống trở lại trạng thái sẵn sàng giao dịch 100% ngay lập tức. Không có bất kỳ hình phạt nào tồn đọng!

- **Trường hợp 2: Lệnh THUA hoặc HÒA VỐN ($\text{Realized PnL} \le 0.001$):**
  - Kiểm tra xem lệnh này có cùng chiều với lệnh thua trước đó hay không:
    - Nếu cùng chiều (Long nối tiếp Long, hoặc Short nối tiếp Short):
      $$\text{consecutive\_losses} = \text{consecutive\_losses} + 1$$
    - Nếu đổi chiều (trước đó thua Long, nay thua Short):
      $$\text{consecutive\_losses} = 1$$
  - **Công thức phạt Exponential Backoff:**
    $$\text{penalty\_bars} = \text{base\_cooldown} \times 2^{\min(\text{consecutive\_losses} - 1,\ 3)}$$
  - **Trần Khóa Tối Đa (Max Cap Guard - Chống khóa vô hạn):**
    $$\text{effective\_cooldown} = \min(\text{penalty\_bars},\ \text{max\_cooldown\_cap})$$
    $$\text{locked\_until\_bar} = i + \text{effective\_cooldown}$$

### 2. Ví dụ cụ thể với $\text{base} = 20$ bars, $\text{max\_cap} = 160$ bars:
- **Thua lần 1:** $20 \times 2^0 = 20$ nến (~20 phút).
- **Thua lần 2 liên tiếp:** $20 \times 2^1 = 40$ nến (~40 phút).
- **Thua lần 3 liên tiếp:** $20 \times 2^2 = 80$ nến (~1.3 giờ).
- **Thua lần 4+ liên tiếp:** $20 \times 2^3 = 160$ nến (~2.6 giờ - Chạm trần an toàn, **KHÔNG BAO GIỜ KHÓA VÔ HẠN**).
- **Chỉ cần có 1 lệnh thắng:** Lập tức quay về $0$ nến!

---

## 🚀 4. Cơ Chế Mở Khóa Sớm (Directional Early Release)

Dù đang trong thời gian bị phạt Cooldown, hệ thống **VẪN CHO PHÉP MỞ LỆNH** nếu thị trường chứng minh phe ta đã đúng bằng hành động giá bứt phá mạnh mẽ:
- **Với chiều Long:** Giá hiện tại tăng vượt xa giá cắt lỗ cũ một khoảng lớn:
  $$\text{CurrentPrice} - \text{LastExitPrice} \ge \text{mult} \times \text{ATR}$$
- **Với chiều Short:** Giá hiện tại sập sâu xuống dưới giá cắt lỗ cũ:
  $$\text{LastExitPrice} - \text{CurrentPrice} \ge \text{mult} \times \text{ATR}$$
- **Chiều ngược lại (Opposite Side):** Không bị ảnh hưởng nếu bối cảnh xu hướng phù hợp.
