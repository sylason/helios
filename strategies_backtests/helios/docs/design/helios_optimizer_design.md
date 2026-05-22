# Tài liệu Thiết kế: Cỗ máy Tối ưu hóa Helios
**Dự án:** Helios (Thế hệ tiếp theo của DyMATE)
**Phiên bản:** 1.1 (Cập nhật chiến lược "Vét cạn thông minh")
**Ngày:** 2026-01-19

---

## **Mục 1: Bối cảnh & Thách thức**

### **1.1. Hạn chế của Hệ thống Tiền nhiệm (Dymate)**
Hệ thống DyMATE, mặc dù mạnh mẽ về mặt logic, nhưng đối mặt với hai thách thức lớn trong quá trình tối ưu hóa tham số:
1.  **Tốc độ:** Được viết bằng Python, việc chạy một vòng lặp tối ưu hóa đầy đủ trên một không gian tham số lớn mất rất nhiều thời gian (ví dụ: 4 giờ), gây khó khăn cho việc tái tối ưu hóa thường xuyên.
2.  **"Lời nguyền của Số chiều":** Số lượng tham số có thể tối ưu hóa là rất lớn (40-50), dẫn đến một không gian tìm kiếm khổng lồ, khiến việc tìm kiếm toàn bộ trở nên bất khả thi về mặt toán học.
3.  **Bỏ sót tiềm năng:** Cơ chế trả về duy nhất một kết quả tốt nhất (Best-First) có thể dẫn đến việc bỏ sót các vùng cực trị địa phương khác (Local Optima) cũng có tiềm năng sinh lời.

### **1.2. Vấn đề Cốt lõi cần giải quyết**
Mục tiêu của Cỗ máy Tối ưu hóa Helios là giải quyết bài toán cân bằng giữa hai yếu tố xung đột:
*   **Sự Toàn diện (Thoroughness):** Mong muốn khám phá mọi ngóc ngách của không gian tham số để không bỏ lỡ các giải pháp tối ưu (Vét cạn thông minh).
*   **Tốc độ (Speed):** Nhu cầu tìm ra các giải pháp đủ tốt trong một khoảng thời gian hợp lý để chống lại **Sự lỗi thời của Tham số (Parameter Decay)** do thị trường liên tục biến đổi.

---

## **Mục 2: Thiết kế cho Helios v1.0 - Cỗ máy "Prospector" (Smart Exhaustive Search)**

Đây là thiết kế được thống nhất cho phiên bản đầu tiên của Helios, tập trung vào sự cân bằng, hiệu quả và tính thực tiễn.

### **2.1. Triết lý Thiết kế**
Thay vì một quy trình tuyến tính, cỗ máy hoạt động như một **vòng lặp vận hành liên tục**, được điều khiển bởi một hệ thống hàng đợi thông minh. Nó hoạt động như một **Cỗ máy Khai phá và Tinh chỉnh (Prospecting & Refining Engine)**, liên tục tìm kiếm và đào sâu vào các vùng tiềm năng.

### **2.2. Các Thành phần Chính**
1.  **Nhiệm vụ (Job):** Một đối tượng định nghĩa một vùng không gian tham số cần được khám phá, bao gồm các dải giá trị và bước nhảy (`step`) cho từng tham số.
2.  **🟢 Hàng đợi Ưu tiên (Hotspot Queue):** Chứa các "nhiệm vụ" hứa hẹn nhất (vùng nóng), được ưu tiên xử lý trước để nhanh chóng tìm ra kết quả tốt.
3.  **⚪ Hàng đợi Tồn đọng (Backlog Queue):** Chứa các "nhiệm vụ" kém hứa hẹn hơn hoặc các vùng chưa được khám phá (vùng nguội). Chúng sẽ được xử lý khi hàng đợi ưu tiên rỗng, đảm bảo tính toàn diện "không bỏ sót".
4.  **🏆 Danh sách Quán quân (Champions List):** Một bảng thành tích được cập nhật liên tục. Bất kỳ bộ tham số nào đạt tiêu chuẩn sẽ được lưu lại.

### **2.3. Luồng Hoạt động Chi tiết**

**1. Khởi tạo:**
*   Hệ thống bắt đầu với một `Job 0` duy nhất bao trùm toàn bộ không gian tham số với `step` lớn nhất.
*   `Job 0` được đưa vào `🟢 Hotspot Queue`.

**2. Vòng lặp Tối ưu hóa Chính:**
*   **BƯỚC A: Lấy Nhiệm vụ:**
    *   Ưu tiên lấy `Job` từ `🟢 Hotspot Queue`.
    *   Nếu `Hotspot Queue` rỗng, lấy `Job` từ `⚪ Backlog Queue`.
    *   Nếu cả hai đều rỗng, chuyển sang kiểm tra điều kiện dừng.
*   **BƯỚC B: Thực thi GA Thích ứng (Adaptive GA Worker):**
    *   Hệ thống chạy một phiên GA trên không gian của `Job` vừa lấy.
    *   **Cải tiến quan trọng:** Worker không chỉ trả về 1 kết quả tốt nhất, mà trả về **Top N Champions** đại diện cho các cụm (Clusters) khác nhau tìm thấy trong không gian đó.
*   **BƯỚC C: Phân tích & Tạo Nhiệm vụ Mới:**
    *   **Cập nhật Quán quân:** Lưu các Champions tìm được vào danh sách.
    *   **Phân nhánh logic dựa trên Step:**
        *   **Nếu `step` > `min_step` (Còn có thể đào sâu):**
            1.  Với mỗi Champion (đại diện cho một vùng nóng), tạo ra một **Job con** với `step` nhỏ hơn (Zoom-in) và đưa vào `🟢 Hotspot Queue`.
            2.  Phần không gian còn lại (chưa được khám phá kỹ) được gói thành các Job mới và đưa vào `⚪ Backlog Queue`.
        *   **Nếu `step` == `min_step` (Đã ở mức chi tiết nhất):**
            1.  Nhiệm vụ được xem là hoàn thành.
            2.  Có thể chạy Cross-Validation (CV) để xếp hạng (Rank) Champion này, nhưng **không loại bỏ** (Pruning) nó khỏi danh sách cuối cùng.
*   **BƯỚC D: Lặp lại:** Quay trở lại BƯỚC A.

### **2.4. Điều kiện Dừng (Strict Condition)**
Quá trình chỉ kết thúc khi thỏa mãn đồng thời cả hai điều kiện (AND):
1.  Cả hai hàng đợi (`Hotspot` và `Backlog`) đều **RỖNG**.
2.  Tất cả các Job đang xử lý đều đã đạt `step` tối thiểu (`min_step`).

Điều này đảm bảo nguyên tắc "Vét cạn thông minh": Không bỏ sót bất kỳ ngóc ngách nào, nhưng ưu tiên đào chỗ ngon trước.

---

## **Mục 3: Tầm nhìn cho Helios v2.0 - Các Hướng Nâng cấp Tương lai**

Sau khi phiên bản v1.0 được triển khai và hoạt động ổn định, đây là các hướng nghiên cứu và nâng cấp tiềm năng để đưa cỗ máy tối ưu hóa lên một tầm cao mới.

### **3.1. Tối ưu hóa Bầy đàn (Particle Swarm Optimization - PSO)**
*   **Ý tưởng:** Một thuật toán dựa trên hành vi bầy đàn, thường hội tụ nhanh hơn GA.
*   **Vai trò tiềm năng:** Có thể được dùng để thay thế hoặc chạy song song với GA trong giai đoạn khai thác "hạng nặng", giúp nhanh chóng tìm ra đỉnh của một "vùng nóng".

### **3.2. CMA-ES (Covariance Matrix Adaptation Evolution Strategy)**
*   **Ý tưởng:** Một thuật toán "tự hiệu chỉnh" cực kỳ mạnh mẽ cho các tham số dạng số thực.
*   **Vai trò tiềm năng:** Sử dụng ở giai đoạn cuối cùng để "đánh bóng" nhà vô địch. Sau khi GA/PSO đã tìm ra một bộ tham số tốt, CMA-ES có thể được dùng để tinh chỉnh các giá trị số thực xung quanh đó với độ chính xác rất cao.

### **3.3. Tối ưu hóa dựa trên Mô hình (Model-Based Optimization)**
*   **Ý tưởng:** Sử dụng các thuật toán như Tối ưu hóa Bayesian để xây dựng một "bản đồ xác suất" của không gian tham số và chọn điểm thử nghiệm tiếp theo một cách thông minh nhất.
*   **Vai trò tiềm năng:** Có thể thay thế hoàn toàn phiên quét "hạng trung" ở vòng đầu tiên. Thay vì quét bằng GA, ta dùng Tối ưu hóa Bayesian để tạo ra "bản đồ nhiệt" ban đầu với số lần chạy backtest ít hơn.

### **3.4. Mô hình "Tối thượng": Hệ thống Lai ghép (Hybrid Ensemble)**
Đây là tầm nhìn dài hạn, kết hợp sức mạnh của tất cả các phương pháp trên:
1.  **Quét Thông minh:** Dùng **Tối ưu hóa Bayesian** để thực hiện phiên quét ban đầu, tạo ra các "vùng nóng" chất lượng cao.
2.  **Khai thác Cạnh tranh:** Với mỗi vùng nóng, chạy song song cả **GA** và **PSO** để tìm ra nhà vô địch của vùng đó.
3.  **Tinh chỉnh Chính xác:** Lấy nhà vô địch từ bước 2 và dùng **CMA-ES** để "đánh bóng" các tham số số thực.
4.  **Kiểm định Cuối cùng:** Đưa tất cả các nhà vô địch đã được tinh chỉnh vào một cuộc kiểm định cuối cùng trên dữ liệu out-of-sample.

---
**Kết luận:** Tài liệu này vạch ra một lộ trình rõ ràng, bắt đầu với một hệ thống v1.0 mạnh mẽ và thực tế, đồng thời mở ra một tầm nhìn phát triển hấp dẫn và chuyên nghiệp cho tương lai của dự án Helios.