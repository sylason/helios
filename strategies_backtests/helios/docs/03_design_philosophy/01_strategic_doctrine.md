# Helios Strategic Doctrine: The "Adaptive Warrior"

Tài liệu này định nghĩa triết lý lõi của Helios, chuyển hóa các tư duy chiến thuật từ Gaming, Võ thuật và Quân sự vào Algorithmic Trading.

## 1. Triết lý Quyền Anh (Boxing): Phòng thủ là Tiên quyết
> *"Protect yourself at all times."*

Trong thị trường Futures đòn bẩy cao, việc sống sót quan trọng hơn kiếm tiền.
*   **Risk Floor (Sàn rủi ro):** Luôn đánh giá độ biến động (Volatility) trước khi ra đòn. Nếu thị trường quá hỗn loạn (High Chaos/Noise), **từ chối giao dịch** hoặc giảm Size về tối thiểu.
*   **Hard Guard (Thủ cứng):** Mọi lệnh mở ra bắt buộc phải có Initial Stop Loss ngay lập tức. Không có ngoại lệ.
*   **Clinch (Ôm ghì):** Khi tình thế bất lợi (Market Regime đổi chiều đột ngột), đóng vị thế ngay lập tức để bảo toàn vốn, không chờ hồi.

## 2. Triết lý StarCraft: Trinh sát & Macro
> *"Macro wins games. Micro wins fights."*

Helios không đánh mù quáng. Nó phải biết mình đang ở đâu trên bản đồ.
*   **Scouting (Trinh sát đa khung):**
    *   **15m (Macro):** Xác định xu hướng chủ đạo (Kinh tế đối phương).
    *   **5m (Mid-game):** Xác định cấu trúc sóng (Đội hình).
    *   **1m (Micro):** Tìm điểm vào lệnh chính xác (Micro-management).
*   **State Switching (Chuyển trạng thái):**
    *   *Accumulation (Ranging):* Đánh du kích (Mean Reversion), chốt lời ngắn.
    *   *Expansion (Trending):* Tấn công tổng lực (Trend Following, Pyramiding).
    *   *Chaos (Choppy):* Phòng thủ (Cash is King).

## 3. Triết lý Thái Cực Quyền (Tai Chi): Nương theo lực
> *"Mượn lực đả lực."*

*   **Flow with the Market:** Không đoán đỉnh đoán đáy. Chỉ vào lệnh khi đà (Momentum) đã xác nhận.
*   **Tiered Trailing (Thả lỏng dần):**
    *   *Giai đoạn đầu:* SL chặt (Tight) để giảm rủi ro.
    *   *Giai đoạn có lãi:* Nới lỏng SL (Loose) để "nuôi" con sóng lớn (Let profits run). Chuyển hóa từ Day Trading sang Swing Trading một cách tự nhiên nếu thị trường ủng hộ.

## 4. Triết lý Mahoraga (Jujutsu Kaisen): Thích ứng Vô hạn
> *"Bánh xe pháp bảo quay, đòn đánh cũ trở nên vô hiệu."*

Thị trường luôn thay đổi (Meta change). Chiến lược tĩnh sẽ chết.
*   **Dynamic Parameters:** Các tham số (RSI Period, Lookback) không được cố định cứng nhắc. Chúng phải co giãn theo nhịp điệu thị trường (Volatility-adjusted parameters).
*   **Re-learning:** Hệ thống Optimizer (`prospector`) sẽ chạy định kỳ trên dữ liệu gần nhất để tìm ra bộ tham số "Meta" hiện tại.

---

## Nguyên tắc Thực thi (Execution Rules)

1.  **Entry:** Sử dụng **Market Order** (khi đóng nến) hoặc **Stop-Market Order** (khi Breakout). **KHÔNG dùng Limit Order** để bắt đáy/đỉnh (tránh adverse selection).
2.  **Validation:** Mọi tín hiệu vào lệnh phải được thông qua bộ lọc "Confidence Score" (Điểm tự tin). Điểm thấp -> Đứng ngoài.
