# Chặn Nút Thắt Trong Thực Thi Lệnh (Execution Bottlenecks)

Tầng thực thi (Execution layer) nằm ở cuối phương thức `next()` trong tệp `mod.rs`. Tuy nhiên, đang có hai sai lầm lớn chặn đứng hoàn toàn khả năng sinh lời của Bot.

## 🚨 1. Lỗi Cố định Khối lượng Giao dịch (Hardcoded Order Size)

```rust
// Tính Size (% Initial Margin)
let _size = RiskManager::calculate_size(tick.equity, 0.10, 21.0, 20.0); // Tính toán nhưng BỎ ĐI

if is_long {
    tick.buy(Some(0.2), None, Some(initial_sl), None, None); // Điền cứng 0.2
}
```

### Phân Tích:
Việc truyền cứng giá trị `0.2` (20% số vốn) vào hàm `buy` và `sell` đã vứt bỏ toàn bộ công sức của hàm `RiskManager::calculate_size`. 
Trong quá trình Backtest dài hạn (Compounding), nếu hệ thống không sử dụng hàm quản trị vốn mà cứ Fix cứng số lượng, kết quả PnL sẽ sai lệch nghiêm trọng, đồng thời gây rủi ro cực cao nếu Equity (Vốn) bị sụt giảm.

### Giải pháp:
Khôi phục việc sử dụng giá trị `_size` đã được tính toán bởi Risk Manager (Sau khi đã update cho nó ăn khớp với thông số DNA `percent_per_trade` thay vì số cứng `0.10`).

---

## 🚨 2. Giao Dịch Không Có Mục Tiêu (Missing Take Profit)

```rust
tick.buy(Some(0.2), None, Some(initial_sl), None, None);
// Vị trí None thứ 2 chính là tp_price (Take Profit)
```

### Phân Tích:
Bài học xương máu từ phiên bản `v2` cho thấy: Chiến thuật Range Ping-Pong nếu không có Take Profit cứng tại các đường Band (Bollinger) thì sẽ vô tình biến thành "Cưỡi Sóng Tình Cờ" (Lỗi lợi nhuận Ảo).

Trong bản `v3`, toàn bộ Take Profit đang bị để trống (`None`). Nghĩa là mọi chiến thuật (Từ Range, Momentum cho tới Trend) đều phó mặc hoàn toàn việc Chốt lời cho cơ chế Trailing Stop Loss. Đối với đánh Trend thì điều này hợp lý, nhưng đối với đánh Range (Biên độ dao động hẹp), nếu không Take Profit chủ động, giá sẽ quay đầu đâm thủng SL ngay lập tức.

### Giải pháp:
Kết hợp với việc nâng cấp cấu trúc `Proposal` ở tệp [02_arbiter_and_proposal_flaws.md](../01_architecture_and_matrix/02_arbiter_and_proposal_flaws.md), tầng thực thi cần hứng giá trị `suggested_tp` từ Scanner trả về và gán nó vào tham số `tp_price` của hàm `buy`/`sell`.
