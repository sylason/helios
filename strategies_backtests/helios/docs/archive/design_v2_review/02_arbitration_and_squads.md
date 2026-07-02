# 🧠 PHÂN XỬ & ĐỘI ĐẶC NHIỆM SWAT (TACTICS)

Tầng này trả lời câu hỏi: **"Dùng sách lược gì để chiến thắng trong thế trận này?"**.

---

## 1. PHÁ BỎ SỰ ĐỘC TÀI CỦA ARBITRATOR ⚖️
*   **Vấn đề cũ:** Arbitrator chỉ chọn 1 Squad cao điểm nhất (`candidates[0]`). Các Squad khác dù điểm cao cũng bị gạt bỏ.
*   **Thiết kế mới (Proposed):**
    *   **Convergence Gatekeeper (Cổng hội tụ) ⛩️:** Một Playbook chỉ được phép chuyển từ trạng thái `Wait` sang `Ready` nếu có sự hội tụ của ít nhất 2/3 Trụ cột Sự thật (Angle, ER, Z-Score).
    *   **Convergence Score:** Nếu nhiều Squad cùng báo điểm cao (ví dụ Trend + Squeeze), chúng ta sẽ cộng hợp điểm số để tăng độ tin cậy.
    *   **Priority Bias:** DNA sẽ quyết định trọng số ưu tiên của từng Squad (ví dụ: Bạn thích Squeeze hơn Trend).

---

## 2. REVIEW 6 ĐỘI SWAT (PLAYBOOKS)

| Squad | Chiến thuật | Logic "Bóp cò" (Sniper) | Đánh giá |
| :--- | :--- | :--- | :--- |
| **Squeeze** | Breakout | Đợi nén lò xo rồi đánh bung biên. | 🚀 Đòn đánh mạnh nhất. |
| **Trend** | Pullback | Đợi giá hồi về trục KAMA mới vào. | ✅ Tránh mua đuổi đỉnh. |
| **Momentum** | Ignition | Đuổi theo khi dòng tiền bùng nổ. | 🔥 Không để bị bỏ rơi. |
| **Range** | Oscillator | Đánh bật lại từ biên hộp (Mean Reversion). | 🏓 Tích tiểu thành đại. |
| **Reversal** | Fading | Bắt đỉnh/đáy khi Parabolic cạn kiệt. | 🌋 R/R cực cao. |
| **Anomaly** | Fade | Phản ứng nhanh khi có Flash Crash. | 🚑 "Nhặt tiền" tai nạn. |

---

## 3. LOGIC "LẬT VỊ THẾ" (THE POSITION FLIP) 🔄
Đây là chìa khóa của sự **Thích nghi khôn lường**.
*   **Protocol:** Đang có lệnh Long, nhưng nếu Squad **Anomaly** hoặc **Reversal** đạt điểm tuyệt đối (> 0.98) theo hướng Short.
*   **Hành động:** Đóng lệnh Long cũ ngay lập tức và vào lệnh Short mới. 
*   **Mục tiêu:** Không để bot bị "kẹt" trong một vị thế sai khi thị trường đảo chiều đột ngột (V-shape).

---

## 4. LỖI "OVER-CONFIRMATION" 🛑
*   **Phát hiện:** Code hiện tại yêu cầu quá nhiều điều kiện đồng thời. 
*   **Hậu quả:** Khi đủ 4 điều kiện thì giá đã chạy mất 50%.
*   **Giải pháp:** Chia thành trạng thái **Ready** (3/4 điều kiện) và **Trigger** (1 điều kiện bóp cò duy nhất).
