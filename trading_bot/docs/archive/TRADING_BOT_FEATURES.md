# 🤖 Trading Bot - Đặc tả Hành vi & Logic (Engine)

Tài liệu này tập trung vào cách Engine hoạt động, mục tiêu và hành vi của từng module. Không đề cập đến kỹ thuật code, chỉ tập trung vào Logic Nghiệp vụ.

---

## 1. Hệ thống Dữ liệu (Data Engine)
*   **Mục tiêu:** Cung cấp "cửa sổ" dữ liệu 30 ngày sạch, chính xác và liên tục để phân tích.
*   **Hành vi:**
    *   **Bù đắp (Backfill):** Khi khởi động, tự động quét Database. Nếu thiếu dữ liệu, gọi API sàn để tải bù cho đủ 30 ngày.
    *   **Tổng hợp (Aggregation):** Không đợi sàn gửi nến. Engine nhận dữ liệu giao dịch thô (Ticks) và tự đóng nến 1m ngay tại giây thứ 0 để phản ứng nhanh hơn.
    *   **Hợp nhất (Stitching):** Đảm bảo nến tải về từ API và nến nhận từ WebSocket khớp nhau 100%, không trùng lặp, không có khoảng trống.
    *   **Lưu trữ:** Ghi dữ liệu vào Database cục bộ ngay lập tức để phục hồi khi App bị tắt đột ngột.

## 2. Trí tuệ & Tối ưu (Intelligence & Optimization)
*   **Mục tiêu:** Tự thích nghi với mọi điều kiện thị trường (Trending, Sideway, Volatile).
*   **Hành vi:**
    *   **Tối ưu liên tục (Self-Optimize):** Chạy Backtest ngầm trên 30 ngày dữ liệu để tìm bộ tham số (Params) sinh lời cao nhất cho hiện tại.
    *   **Thay thế nóng (Hot-swap):** Khi tìm thấy Params tốt hơn, Engine tự động cập nhật vào luồng Trading mà không làm gián đoạn vị thế đang mở.
    *   **Giám sát hiệu quả:** Nếu Params hiện tại bắt đầu gây lỗ vượt mức cho phép của Backtest, Engine tự động kích hoạt chu kỳ Optimize khẩn cấp.

## 3. Thực thi & Quản lý Asset (Asset Actor)
*   **Mục tiêu:** Quản lý độc lập từng cặp tiền, đảm bảo tính biệt lập và ổn định.
*   **Hành vi:**
    *   **Vòng đời độc lập:** Mỗi cặp tiền (như ETHUSDT) là một "nhân viên" riêng. Asset A lỗi không làm Asset B dừng chạy.
    *   **Quản lý vị thế:** Theo dõi sát sao giá vào (Entry), khối lượng và đòn bẩy. Tự động tính toán PnL (lời/lỗ) từng giây.
    *   **Ra quyết định:** Chỉ nhận tín hiệu khi nến đã đóng để tránh tín hiệu giả (Repainting).

## 4. Bảo vệ & Quản trị rủi ro (Risk & Guardian)
*   **Mục tiêu:** Bảo vệ tài khoản và duy trì kết nối an toàn với sàn.
*   **Hành vi:**
    *   **Bám đuổi lợi nhuận (Trailing Stoploss):** Theo dõi giá thị trường (Mark Price) liên tục. Nếu giá đi đúng hướng, Stoploss sẽ tự động dời lên để khóa lợi nhuận. Hành vi này chạy ở mức độ "giây", không đợi nến đóng.
    *   **Giám sát API (Weight Monitor):** Theo dõi mức độ sử dụng API của sàn. Nếu sắp chạm ngưỡng bị khóa IP, Engine chủ động giảm tần suất gọi lệnh.
    *   **Cơ chế Dừng (Termination):**
        *   **Dừng mềm (Soft):** Ngừng vào lệnh mới, chỉ tập trung quản lý và thoát các lệnh cũ một cách có lợi nhất.
        *   **Dừng khẩn cấp (Panic):** Hủy toàn bộ lệnh chờ, đóng mọi vị thế đang mở ngay lập tức và dừng hệ thống.
    *   **Ngưỡng an toàn:** Tự động ngừng giao dịch một Asset nếu Asset đó chạm mức lỗ tối đa trong ngày (Daily Loss Limit).
