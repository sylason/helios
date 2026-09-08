# Product Requirements Document (PRD) - Production Phase

## 1. Mục tiêu (Objective)
Mục tiêu của giai đoạn Production là xây dựng một cỗ máy giao dịch hoàn chỉnh, có khả năng tự động hóa việc kiếm tiền (Real Money) với độ ổn định tuyệt đối (99.9% uptime), quản trị rủi ro chặt chẽ và khả năng tự thích nghi với thị trường.

## 2. Đối tượng mục tiêu (Target Audience)
- Bot giao dịch Live cho quỹ cá nhân.
- Hoạt động 24/7 (Có thể trên máy chủ Local hoặc VPS chuyên dụng).

## 3. Scope of Work (Phạm vi công việc)

### 3.1. Các Tính Năng Cốt Lõi (Core Features)
- **Đa tài sản & Đa sàn (Multi-asset & Multi-exchange)**:
  - Hỗ trợ giao dịch nhiều cặp tiền tệ cùng lúc (BTCUSDT, ETHUSDT, SOLUSDT, v.v.).
  - Sẵn sàng mở rộng sang Bybit, OKX bên cạnh Binance (thông qua interface trừu tượng).
- **Self-Adaptive Strategy (Tự thích nghi)**:
  - Bot liên tục chạy backtest ngầm (Background Task) dựa trên dữ liệu 15-30 ngày qua.
  - Tự động thay đổi tham số (ví dụ: RSI length, Take Profit %) dựa trên kết quả tối ưu nhất.
- **Advanced Execution & Risk (Thực thi & Rủi ro nâng cao)**:
  - **Hedge Mode**: Tự động mở lệnh đối ứng khi rủi ro danh mục quá cao.
  - **Auto-Panic**: Dừng mọi hoạt động khi gặp lỗi không mong muốn quá N lần/phút.
  - **Observer Tasks**: Các luồng theo dõi chốt lời/cắt lỗ độc lập, chạy song song với Strategy Engine để phản ứng mili-giây.
- **Recovery Management (Quản lý phục hồi)**:
  - Bot "Crash" hoặc mất điện. Khi bật lại, Bot quét tài khoản Binance để **Take-over (Tiếp quản)** các vị thế đang mở và gắn lại các logic quản trị rủi ro.

### 3.2. User Interface (UI - Tauri)
- Giao diện Darkmode/Premium Aesthetics chuyên nghiệp.
- **Charts**: Tích hợp đồ thị nến (TradingView Lightweight Charts) để hiển thị lịch sử và vị trí đặt lệnh.
- Cấu hình quản trị vốn trực quan (Kéo thả, Input trực tiếp).
- Các nút Emergency Stop (Kill switch) rõ ràng.

## 4. Yêu Cầu Phi Chức Năng (Non-functional Requirements)
- **Performance**: Phản hồi sự kiện WebSocket (Tạo tín hiệu -> Gửi lệnh) dưới 10ms.
- **Bảo mật**: Các khóa API phải được mã hóa tại Client (Không lưu plaintext).
- **Tính trọn vẹn dữ liệu**: Gap dữ liệu phải tự động được "vá" (backfill) thông qua API REST.

## 5. Rủi ro (Risks)
- Kết nối internet thiếu ổn định ảnh hưởng tới việc cắt lỗ. -> **Giải pháp**: Phải đẩy cả lệnh Stop Loss/Take Profit cứng lên sàn (Hard SL/TP) chứ không chỉ Soft SL/TP trên RAM.
