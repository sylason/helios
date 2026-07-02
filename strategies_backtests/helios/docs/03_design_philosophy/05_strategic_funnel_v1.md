# Helios Strategic Funnel: 4-Level Architecture

Sơ đồ này hệ thống hóa bộ não của Helios thành 4 cấp độ phân tích dữ liệu, từ phổ quát đến chi tiết.

## 1. Cấp độ Môi trường (Environment - The Fuse)
Quyết định: **Hệ thống Active hay Defense?**
*   **Entropy Index (Efficiency Ratio):** Đo độ sạch của giá. ER < 0.3 -> Hỗn loạn (Choppy) -> **Defense**.
*   **Volatility Z-Score:** Đo độ điên rồ. Z > 2.5 -> Black Swan -> **Defense**.
*   **Volume Ratio:** Đo năng lượng. Ratio < 0.5 -> Ngủ gật -> **Scouting**.

## 2. Cấp độ Thế trận (Regime - The Strategist)
Quyết định: **Địa hình đang đứng là gì?**
*   **Trending (Strong/Weak):** Dựa trên ADX và KAMA Cross.
*   **Ranging (Orderly):** Dựa trên BBands và Entropy trung bình.
*   **Squeeze (Nén):** Dải BBands chui vào Keltner Channels.
*   **Parabolic:** Dựa trên Linear Regression Slope (Góc tấn công).

## 3. Cấp độ Sách lược (Playbook - The Armory)
Quyết định: **Đòn đánh nào được phép dùng?**
*   **Playbook Attack:** [Trend Pullback, Breakout Retest, Pyramiding].
*   **Playbook Counter:** [Mean Reversion, False Breakout].
*   **Playbook Squeeze:** [Volatility Expansion].

## 4. Cấp độ Vi lệnh (Tactical Trigger - The Sniper)
Quyết định: **Bóp cò tại tọa độ nào?**
*   **Consensus (Đồng thuận):** Sự khớp pha giữa HTF (15m) và STF (1m).
*   **Price Logic (Standardized):** Pinbar, Engulfing, RSI Hooks.
*   **Execution:** Lệnh Market để đảm bảo thực chiến.

---

## Danh sách 7 Cảm biến Cốt lõi (The Elite 7)

| Cảm biến | Thư viện | Vai trò |
| :--- | :--- | :--- |
| **KAMA** | `yata` | Xu hướng & Lọc nhiễu (ER). |
| **ADX** | `kand` | Sức mạnh xu hướng. |
| **ATR** | `kand` | Biến động & Định cỡ vị thế. |
| **BBands** | `kand` | Biên độ thống kê (2 Standard Deviations). |
| **KC** | `yata` | Kênh giá động để phát hiện Squeeze. |
| **RSI** | `kand` | Động lượng & Quá mua/Quá bán. |
| **LinReg** | `yata` | Góc tấn công (Slope) cho Parabolic. |
