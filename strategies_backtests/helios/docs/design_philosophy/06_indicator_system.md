# Helios Indicator System: The Sensory Architecture

Tài liệu này định nghĩa hệ thống giác quan của Helios, chuyển hóa dữ liệu toán học thành nhận thức chiến trường.

## I. Phân tầng Giác quan (Hierarchical Sensors)

### 1. Cấp độ 1: Bộ lọc Entropy (Environment Sensors)
Mục tiêu: Đảm bảo "tín hiệu sạch" trước khi phân tích sâu hơn.

*   **🛡️ KAMA Efficiency Ratio (ER)**
    *   **Toán học:** Tỷ lệ giữa đường thẳng và đường zíc-zắc của giá.
    *   **Thực chiến:** Đo lường "độ nhiễu". Nếu ER thấp, mọi chỉ báo khác đều mất hiệu lực. Helios sẽ chuyển sang trạng thái "Thủ thế" (Defense).
*   **📊 ATR Z-Score**
    *   **Toán học:** Khoảng cách của biến động hiện tại so với trung bình thống kê (Standard Deviation).
    *   **Thực chiến:** Nhận diện các pha hoảng loạn (Panic) hoặc hưng phấn (Euphoria). Khi giá nhảy quá tầm kiểm soát của xác suất, Helios sẽ "Thu quân" (Cash is King).
*   **⚡ Volume Ratio**
    *   **Toán học:** So sánh khối lượng hiện tại với mức trung bình trượt.
    *   **Thực chiến:** Lọc bỏ các chuyển động giá ảo (Low liquidity). Chỉ đánh khi có sự tham gia của đám đông.

### 2. Cấp độ 2: Nhận diện Thế trận (Regime Strategist)
Mục tiêu: Ánh xạ cấu trúc giá vào các kịch bản chiến tranh (Playbooks).

*   **🏹 ADX (Average Directional Index)**
    *   **Toán học:** Đo lường sự thay đổi của các cực trị (High/Low) theo thời gian.
    *   **Thực chiến:** Phân biệt thị trường có hướng (Trending) và không hướng (Ranging).
*   **🌀 Squeeze Metric (BBands + Keltner)**
    *   **Toán học:** Tương quan giữa độ lệch chuẩn (BB) và ATR (KC).
    *   **Thực chiến:** Phát hiện trạng thái "Lò xo nén". Đây là tiền đề cho đòn tấn công mạnh nhất của Helios.
*   **📈 Linear Regression Slope**
    *   **Toán học:** Hệ số góc của đường hồi quy tuyến tính tốt nhất.
    *   **Thực chiến:** Đo "Góc tấn công". Slope càng dốc, tốc độ tăng trưởng càng nhanh (Parabolic Phase).

### 3. Cấp độ 4: Vi lệnh bóp cò (Tactical Sniper)
Mục tiêu: Tìm điểm vào tối ưu để cực tiểu hóa khoảng cách Stop Loss.

*   **🎣 RSI (Relative Strength Index)**
    *   **Toán học:** So sánh cường độ tăng và giảm của giá.
    *   **Thực chiến:** Tìm điểm "kiệt sức" tạm thời của cú hồi (Pullback) để thuận thế đánh tiếp.
*   **🎯 Stochastic Oscillator**
    *   **Toán học:** Vị trí giá hiện tại trong biên độ High-Low của chu kỳ.
    *   **Thực chiến:** Xác nhận sự đồng thuận của dòng tiền ngắn hạn (Micro-momentum).

## II. Nguyên tắc "Thanh lọc dữ liệu" (Data Purity)

1.  **Tính toán Vector hóa:** Mọi chỉ báo được tính toán đồng loạt trong `init()` thông qua `resample_apply` để đạt hiệu năng tối đa.
2.  **Đồng bộ hóa (Consensus):** Tín hiệu từ khung 15m (Chiến lược) phải được "phủ sóng" (ffill) xuống khung 1m để vi lệnh bóp cò đúng nhịp.
3.  **Toán học thuần túy:** Tuyệt đối không sử dụng các khái niệm cảm tính. Mọi biến số đều có thể tối ưu hóa (Optimize) thông qua Backtest.
