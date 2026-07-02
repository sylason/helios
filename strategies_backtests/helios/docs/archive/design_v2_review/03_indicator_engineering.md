# 🧬 KỸ THUẬT CHỈ BÁO PHÂN TẦNG (DNA ENGINEERING)

Tài liệu này phân tích cách Helios xử lý hàng loạt chỉ báo (Indicators) mà không bị rơi vào cái bẫy "Bùng nổ tham số".

---

## 1. TÁCH BIỆT DNA: NHẬN THỨC VS. HÀNH ĐỘNG 🏗️
Một lỗi thiết kế phổ biến là dùng chung tham số cho cả việc "nhìn" thị trường và "bóp cò". Helios V2 đề xuất tách biệt rõ:

*   **Perception DNA (The Eyes):** Các tham số `lookback` của Matrix (ví dụ: `trend_lookback: 200`). Những tham số này trả lời câu hỏi: "Thị trường là gì?".
*   **Tactical DNA (The Trigger):** Các tham số của Sniper (ví dụ: `rsi_buy_thresh: 30`). Những tham số này trả lời câu hỏi: "Khi nào vào lệnh?".

**Lợi ích:** Tránh việc Optimizer thay lời giải cho câu hỏi này bằng câu trả lời của câu hỏi kia (Decoupling).

---

## 2. CHUẨN HÓA TOÁN HỌC TUYỆT ĐỐI 📏
Để loại bỏ cảm tính, mọi chỉ báo trước khi đưa vào Não bộ (Arbitrator) phải đi qua bộ lọc **Mathematical Purity**:

1.  **Symmetry (Đối xứng):** Các chỉ báo hướng (Trend) phải ở dải `[-1.0, 1.0]`. 0.0 là trung tính tuyệt đối.
2.  **Probability (Xác suất):** Các chỉ báo cường độ (Squeeze, Volume) phải ở dải `[0.0, 1.0]`.
3.  **Non-Linearity (Phi tuyến tính):** Sử dụng hàm `tanh()` để nén các giá trị cực đoan (Outliers) về vùng giới hạn, giúp robot không bị "sốc" khi gặp râu nến dài.

---

## 3. ĐỒNG BỘ ĐA KHUNG THỜI GIAN (MTF SYNC) 🌍
Helios không chỉ nhìn 1m. Nó vận hành theo nguyên tắc **"Quyền lực tối thượng"**:
*   **1h/4h (Macro):** Có quyền "Cấm giao dịch" nếu xu hướng lớn không đồng thuận.
*   **15m (Squad Context):** Xác định Playbook nào được phép triển khai.
*   **1m (Execution):** Chỉ đóng vai trò tìm điểm vào lệnh chính xác (Sniper).

---

## 4. KIỂM SOÁT SỰ BÙNG NỔ THAM SỐ (PARAMETER EXPLOSION) 💥
Với 6 Squad, chúng ta có hơn 100 tham số. 
*   **Vấn đề:** Optimizer sẽ bị lạc (Lost in search space).
*   **Giải pháp Helios:** Sử dụng cơ chế **"Frozen DNA"**. Trong các đợt Optimize, chúng ta chỉ cho phép tiến hóa 1 hoặc 2 Squad cùng lúc, trong khi đóng băng các Squad khác để đảm bảo sự ổn định của hệ thống.
