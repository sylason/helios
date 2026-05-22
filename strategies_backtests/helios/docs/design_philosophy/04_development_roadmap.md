# Helios Development Roadmap: The "Mahoraga" Path

Lộ trình này chia quá trình phát triển Helios thành 4 giai đoạn thích ứng, đi từ nền tảng vững chắc đến khả năng tấn công tột độ.

## Giai đoạn 1: Thiết lập Hệ thống Gốc (The Framework) - [HIỆN TẠI]
*   **Mục tiêu:** Xây dựng "hệ thần kinh" cho Helios trên nền tảng `ferro_trade`.
*   **Nhiệm vụ:**
    *   Hoàn thiện `types.rs` với các Enum và Struct chuẩn hóa.
    *   Thiết lập API liên kết giữa Strategy (Helios) và Engine (Ferro-Trade).
    *   Đảm bảo khả năng load tham số từ `optimizer_config.json`.
*   **Kiểm chứng:** Chạy được một loop backtest SMA Cross đơn giản mà không gặp lỗi kỹ thuật.

## Giai đoạn 2: Cài đặt Giác quan (Sensory Implementation)
*   **Mục tiêu:** Xây dựng khả năng nhận diện môi trường toán học.
*   **Nhiệm vụ:**
    *   Triển khai logic `MarketAnalyzer` với **7 cảm biến cốt lõi** (KAMA, ADX, ATR, BBands, KC, RSI, LinReg).
    *   Viết strategy `HeliosBase`: Chỉ làm nhiệm vụ **phân loại và in log** trạng thái thị trường (`Trending`, `Ranging`, `Squeeze`, `Choppy`).
*   **Kiểm chứng:** Chạy trên dữ liệu thực tế (ETHUSDT 1m/3m). Đối chiếu log trạng thái với biểu đồ giá để xác nhận độ chính xác của cảm biến.

## Giai đoạn 3: Triển khai Sách lược (Tactical Playbooks)
*   **Mục tiêu:** Bắt đầu giao tranh thực tế với mức độ an toàn cao.
*   **Nhiệm vụ:**
    *   Cài đặt Playbook Tấn công (Trend Pullback, Breakout Retest).
    *   Cài đặt Playbook Phản công (Mean Reversion, False Breakout).
    *   Triển khai **Stop Loss Tiến hóa** (Evolving SL) từ Day sang Swing.
*   **Kiểm chứng:** Chạy Backtest với Leverage 10x - 20x. Mục tiêu: Lợi nhuận ổn định, rủi ro sụt giảm (Drawdown) thấp.

## Giai đoạn 4: Tấn công Tột độ & Thích nghi (Extreme Assault)
*   **Mục tiêu:** Kích hoạt đòn "Nuke" và tối ưu hóa không ngừng.
*   **Nhiệm vụ:**
    *   Cài đặt đòn **Nuke 50x - 100x** với các điều kiện ngặt nghèo (Isolated Margin, Perfect Confluence).
    *   Triển khai **Nhồi lệnh động** (Dynamic Pyramiding).
    *   Kết nối với Genetic Algorithm của `prospector` để tự động tiến hóa tham số mỗi tuần.
*   **Kiểm chứng:** Đạt tỷ lệ Profit Factor (Lợi nhuận/Thua lỗ) tối ưu.
