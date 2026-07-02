# 1a. Sự thích nghi của Mahoraga (The Mahoraga Adaptation)

## Ý tưởng Cốt lõi
Trong Jujutsu Kaisen, Mahoraga là một thực thể có khả năng thích nghi với mọi đòn tấn công sau khi "xoay bánh xe". Helios được thiết kế với triết lý tương tự: **Thị trường luôn thay đổi, bot giao dịch không thể dùng một bộ thông số cố định.**

Thay vì Hard-code các giá trị như `RSI > 70 là quá mua`, Helios sử dụng **Genetic Algorithm (GA)** để liên tục quay bánh xe tìm ra bộ gen (DNA) phù hợp nhất với trạng thái thị trường hiện tại.

## Cách Helios "Xoay Bánh Xe"
1. **Perception Matrix:** Thay vì nhìn vào giá trị tuyệt đối, Helios dùng `Rolling Percentile` (thứ hạng phân vị) để tự động chuẩn hóa mọi chỉ báo về thang `[0,1]`. Điều này giống như Mahoraga đang phân tích cấu trúc đòn tấn công.
2. **Squad Arbitration:** Các Squad (Trend, Squeeze, Anomaly...) tranh giành quyền kiểm soát bằng điểm số. GA sẽ tự tối ưu trọng số để chọn ra Squad nào đánh tốt nhất ở thời điểm hiện tại.
3. **Continuous Mutation:** Optimizer chạy liên tục (Walk-forward analysis) để tạo ra các thế hệ tham số mới ngay khi thị trường chuyển pha.

## Điểm yếu cần khắc phục
Hiện tại, Helios chỉ đang tối ưu hóa thông số (Parameters), nhưng chưa thể tự thay đổi **Quy tắc (Rules)**. Để đạt tới cấp độ Mahoraga thực thụ, hệ thống GA cần được cấp quyền ghép nối các quy tắc (Rule-based Genetic Programming), ví dụ: tự học cách kết hợp `RSI` + `Volume` thay vì bị ép buộc bởi code cố định trong `playbook.rs`.
