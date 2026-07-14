# 🤖 Trading Bot - Final Design Overview (Thuần Rust)

Tài liệu này là "Nguồn chân lý" (Source of Truth) tóm tắt các quyết định kiến trúc cốt lõi cho hệ thống giao dịch tự thích nghi.

---

## 🏗️ 1. Mô hình vận hành (Execution Model)
- **Ngôn ngữ:** 100% Rust. Tận dụng tối đa hiệu năng phần cứng cá nhân, đảm bảo an toàn bộ nhớ và xử lý đa luồng vượt trội.
- **Kiến trúc:** **Actor Model** (dùng `tokio` tasks). Mỗi tài sản (Asset) là một thực thể độc lập, tự quản lý dữ liệu, logic và trạng thái riêng.
- **Triết lý Resilience (Khả năng phục hồi):** App được thiết kế để "tự nhận thức" trạng thái trên sàn. Trong trường hợp mất điện hoặc crash, khi khởi động lại, Bot sẽ tự động tiếp quản (Takeover) các vị thế đang mở, thiết lập bảo vệ ngay lập tức và tái phân tích để thích nghi với tình hình mới.
- **Tích hợp:** `ferro_trade` (Engine tính toán) và `helios` (Logic chiến lược) được tích hợp trực tiếp dưới dạng thư viện nội bộ (Crates).

## 💡 2. Các cột trụ chức năng
1. **Self-Adaptive:** Loại bỏ các thông số tĩnh. Hệ thống liên tục tự tối ưu hóa (Re-optimize) thông qua backtest thời gian thực trên cửa sổ dữ liệu 15-30 ngày.
2. **Local-First & Privacy:** Toàn bộ dữ liệu phân tích và khóa API được lưu trữ và xử lý cục bộ. Không sử dụng nền tảng đám mây để đảm bảo an toàn tài chính.
3. **High-Frequency Ready:** Tự tổng hợp nến từ dữ liệu giao dịch thô (`aggTrade`) để ra quyết định sớm hơn sàn 1-2 giây.
4. **Safety First:** Cơ chế bảo vệ đa tầng: Dedicated Observer chốt lỗ siêu tốc, Auto-Panic khi hệ thống lỗi, và Hedge khi thị trường sụp đổ.

## 🖥️ 3. Lộ trình Phát triển (Roadmap)
- **Giai đoạn 1 (Demo):** Tập trung vào cặp ETHUSDT trên Binance Futures. Chạy trên môi trường Testnet để kiểm chứng logic và UI tối giản.
- **Giai đoạn 2 (Production):** Chạy Mainnet với tiền thật. Bổ sung Notification, Quản lý đa tài sản/đa sàn và cơ chế phục hồi nâng cao.

---
*Chi tiết kỹ thuật từng phần:*
- [Thiết kế Data Engine](./ARCH_DATA_ENGINE.md)
- [Thiết kế Adaptive Loop & Actor](./ARCH_ADAPTIVE_LOOP.md)
- [Thiết kế Execution & Risk](./ARCH_EXECUTION_RISK.md)
- [Thiết kế Phục hồi & Quản lý vị thế](./ARCH_RECOVERY_MANAGEMENT.md)
