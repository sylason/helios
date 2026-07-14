# Product Requirements Document (PRD) - Demo Phase

## 1. Mục tiêu (Objective)
Mục tiêu của giai đoạn Demo (MVP - Minimum Viable Product) là nhanh chóng xây dựng một phiên bản chạy được để kiểm chứng toàn bộ luồng dữ liệu và quan trọng nhất là **đánh giá tính hiệu quả của thuật toán Tối ưu hóa (Optimizer)** trên môi trường thực tế (Forward Backtest). Nếu Forward Backtest trả về lợi nhuận dương, đây sẽ là minh chứng sống (Proof of Concept) cho dự án.

## 2. Đối tượng mục tiêu (Target Audience)
- Nhà phát triển nội bộ.
- Phục vụ mục đích Nghiên cứu (Research), Debugging và Tối ưu hóa liên tục.

## 3. Scope of Work (Phạm vi công việc) & Các Giai Đoạn (Phases)

Bản Demo được chia làm 6 giai đoạn kiểm thử rõ ràng:

### Phase 1: Mở/Tắt Asset Bot & Lấy Dữ liệu
- Bật/tắt bot cho một cặp giao dịch (VD: ETHUSDT).
- Giao diện UI cực giản lược: Chỉ có thông tin vị thế (Position hiện tại), không cần đồ thị (Chart) hay Terminal/Logs (mọi logs in ra Console của IDE bằng `println!`).

### Phase 2: Xác thực Tính toàn vẹn Dữ liệu (No Gaps)
- Bot phải kết nối WebSocket trực tiếp kênh **K-line 1m** (bỏ qua `aggTrade` để tiết kiệm tài nguyên CPU/RAM).
- Bot phải luôn mở WebSocket **trước**, sau đó mới gọi REST API Historical K-line 1m để lấp khoảng trống (Gap). Đảm bảo 100% không bị hụt nến.

### Phase 3: Test Optimizer Liên tục (Trái tim của hệ thống)
- Tích hợp bộ Optimizer từ project `ferro_trade` (mô phỏng `run_helios_optimize.rs` dùng thuật toán Genetic Algorithm).
- Optimizer phải chạy **liên tục**, cứ xong 1 vòng lại tự động chạy lại trên 30 ngày dữ liệu gần nhất.
- Yêu cầu in ra Console kết quả bộ Tham số (Params) mới nhất và các chỉ số tối ưu (Stats).
- **Mục tiêu tối thượng**: Đánh giá Forward Backtest qua thời gian có đạt lợi nhuận dương không.

### Phase 4: Paper Trading Local (Mô phỏng cục bộ)
- Khi có tín hiệu giao dịch từ chiến lược, không gửi lệnh lên mạng.
- Tự tính Lãi/Lỗ ảo cục bộ trong Rust dựa vào `markPrice`. In lệnh Buy/Sell ra Console.

### Phase 5: Testnet Execution (Đánh trên sàn ảo)
- Khi Paper Trading đã ổn định, tích hợp API Key Testnet.
- Đẩy lệnh thật lên Binance Testnet để trải nghiệm trượt giá (Slippage) và độ trễ (Latency).

### Phase 6: Live Trading Demo (Đánh trên sàn thật)
- Khi mọi điều kiện trên Testnet hoàn hảo, bắt đầu Live Trading với vốn nhỏ ngay trong bản Demo.

## 4. Quản trị Dữ liệu (Data Retention)
- Chỉ giữ lại **1 tháng 3 ngày** dữ liệu K-line 1m gần nhất. 
- Backtest/Optimize chỉ chạy trên dữ liệu 1 tháng gần nhất.
- Hàng ngày khi mở app, tự động xóa data cũ trong Local DB (DuckDB) để giải phóng ổ cứng.

## 5. Rủi ro & Giả định (Risks & Assumptions)
- Optimizer tốn quá nhiều CPU/RAM làm chậm việc nhận K-line. (Cần cấp luồng riêng biệt).
