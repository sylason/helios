# ⚠️ CÁC NÚT THẮT & LỖI TIỀM ẨN (CRITICAL FLAWS)

Danh sách các "bệnh trạng" đã phát hiện và cần phải xử lý để Helios đạt trạng thái Elite.

---

## 1. LỖI "OVER-CONFIRMATION" (ĐỢI QUÁ NHIỀU XÁC NHẬN) 🛑
*   **Mô tả:** Code `tactical.rs` yêu cầu quá nhiều điều kiện `AND` (Ví dụ: Trend + Momentum + Volume + KAMA).
*   **Hậu quả:** Robot vào lệnh khi sóng đã đi được 70% quãng đường.
*   **Giải pháp:** Chuyển sang mô hình **"Ready - Aim - Fire"**:
    *   **Ready:** Ma trận báo Regime (Não).
    *   **Aim:** Indicator đồng thuận (Mắt).
    *   **Fire:** Chỉ cần 1 ngòi nổ MACD hoặc Stoch nhích nhẹ là bắn ngay (Tay).

---

## 2. VẤN ĐỀ "THẤT BẠI ÂM THẦM" (SILENT FAILURES) 💰
*   **Phát hiện:** `Triggered` nhưng không có `Trade`.
*   **Lý do 1:** Lỗi làm tròn khối lượng (Fractional Unit).
*   **Lý do 2:** Ký quỹ (Margin) không đủ do lệnh cũ đang Drawdown âm.
*   **Giải pháp:** Cần in log cảnh báo từ Broker của `ferro_trade` khi lệnh `buy/sell` bị từ chối để trader biết bot đang "đói vốn" hay "lỗi logic".

---

## 3. LỖI "UNIT MISMATCH" (SAI LỆCH ĐƠN VỊ) 📐
*   **Bệnh:** So sánh Góc (Degrees 0-90) với Percentile (0.0-1.0).
*   **Hậu quả:** Các bộ lọc Parabolic (Góc dốc) không bao giờ kích hoạt hoặc kích hoạt sai thời điểm.
*   **Trạng thái:** Đã Fix trong code nhưng cần giám sát chặt chẽ các hàm `adj_slope`.

---

## 4. "ĐỘC QUYỀN VỊ THẾ" (THE ARBITRATION BLOCK) 🔒
*   **Bệnh:** Squad yếu (ví dụ Absorption) chiếm chỗ của Squad mạnh (Squeeze) suốt 2 ngày.
*   **Giải pháp:** Cần logic **"Sa thải quân yếu"**. Nếu đang có lệnh Long của Squad thấp điểm, nhưng Squad cao điểm hơn xuất hiện tín hiệu ngược lại -> Đóng lệnh cũ, thực hiện `Emergency Override` ngay.

---

## 5. ĐIỂM YẾU CỦA ROLLING PERCENTILE 💎
*   **Bệnh:** Trong thị trường đi ngang (Flat), một biến động siêu nhỏ (9 độ) cũng được Percentile đẩy lên thành 0.99 (Bullish cực mạnh).
*   **Hậu quả:** Bot bị FOMO trong vùng nhiễu.
*   **Giải pháp:** Phải có một ngưỡng **"Absolute Threshold"** (Ví dụ: Góc phải > 15 độ) trước khi cho phép Percentile hoạt động.
