# 🛡️ HỆ THỐNG SINH TỒN & TIẾN HÓA (RISK & OBJECTIVE)

Tại sao lợi nhuận 600% biến mất? Tại sao bot chỉ trade 35 lệnh? Tài liệu này mổ xẻ "Trái tim" của hệ thống quản trị rủi ro.

---

## 1. CHIẾN BINH VIỄN CHINH VS. XẠ THỦ ĐƠN ĐỘC 🤺
Kết quả Optimize cho thấy bot đang quá "nhát". Chúng ta cần thay đổi tư tưởng thiết kế:

*   **Hiện tại (Sniper):** Đợi 100% tín hiệu -> Trade 1 lệnh lớn -> Thua 1 lệnh là mất cả tuần chờ đợi.
*   **V2 (Expeditionary Force):** 
    *   Chia vốn nhỏ hơn (Size 1/10 thay vì 1/5).
    *   Tham chiến liên tục trong mọi trạng thái (Trend, Range, Squeeze).
    *   **Mục tiêu:** Thắng nhờ số lượng cơ hội và khả năng "Gồng lời" (Let winners run).

---

## 2. ELASTIC TRAILING STOP (CO GIÃN THÍCH NGHI) 📏
Trailing Stop của Helios V2 không phải là một con số chết. Nó co giãn theo Playbook:

1.  **Tight (Siết chặt):** Dùng cho `Parabolic` và `Anomaly`. Khi thị trường điên cuồng, ta phải khóa lời ngay lập tức.
2.  **Normal (Bình thường):** Dùng cho `Trending`. Cho giá không gian để thở.
3.  **Loose (Thả lỏng):** Dùng cho `Squeeze`. Chấp nhận retest sâu để bắt được con sóng thần Breakout.

---

## 3. GIẢI MÃ HÀM MỤT TIÊU (OBJECTIVE FUNCTION) 🎯
Hàm mục tiêu trong `run_helios_optimize.rs` cần được refactor:

*   **Lỗi cũ:** Phạt nặng bot nếu đánh ít hơn 1 lệnh/ngày. Điều này ép bot trade ẩu.
*   **Công thức V2 đề xuất:** 
    *   Ưu tiên **Profit Factor > 2.0**.
    *   Thưởng cho **Sharpe Ratio** (độ mượt của đường cong vốn).
    *   Chấp nhận **Drawdown < 30%** (vì chúng ta trade Futures, không thể quá nhát).

---

## 4. HIỆN TƯỢNG "SORTINO ABNORMAL" (🚩 CẢNH BÁO)
Phát hiện Rank 1 có Sortino Ratio = 5477. 
*   **Nguyên nhân:** Có thể do dữ liệu 30 ngày quá ngắn hoặc lỗi chia cho số gần bằng 0 trong công thức tính toán độ lệch chuẩn âm.
*   **Hành động:** Cần kiểm tra lại hàm tính metric trong `ferro_trade` để tránh Optimizer bị đánh lừa bởi những DNA "may mắn ảo".
