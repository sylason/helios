# Specification: Dymate Strategy Logic Review & Research

## 1. Overview
Thực hiện đánh giá toàn diện và nghiên cứu cải tiến logic chiến lược Dymate. Mục tiêu là phát hiện lỗ hổng, tối ưu hóa các trạng thái thị trường (market states), và tích hợp các tư duy giao dịch hiện đại (Breakout, Smart Money, Momentum) để chuẩn bị cho việc port sang dự án Rust (Helios).

## 2. Mục tiêu (Goals)
- **Audit Logic:** Kiểm tra sự tương tác giữa các state, đảm bảo không "ngáng đường" nhau và hoạt động hòa hợp.
- **Tối ưu Market States:** Đánh giá độ phủ của các trạng thái thị trường và cải tiến logic lựa chọn state khi có nhiều state xảy ra đồng thời.
- **Nâng cấp chiến lược:** Tận dụng tốt logic Breakout và các biến thể, tích hợp Liquidity & Order Flow.
- **Tuyển chọn Indicators:** Đánh giá các indicator hiện có và lựa chọn các indicator tối ưu trong thư viện Kand/Yata (Rust) hoặc thiết kế logic tùy chỉnh.
- **Lập kế hoạch Porting:** Đóng gói kết quả nghiên cứu thành bộ tài liệu hướng dẫn chi tiết cho quá trình port sang Helios.

## 3. Yêu cầu chức năng (Functional Requirements)
- **Review đa chiều:** Phân tích logic từ góc nhìn chuyên gia (Trend Following, SMC, Momentum).
- **Thiết kế lại State Logic:** Đơn giản hóa nếu cần, nhưng phải đảm bảo tính thực tế và logic.
- **Indicator Gap Analysis:** So sánh hiệu quả giữa indicator cũ và các giải pháp trong Rust (Kand/Yata).
- **Hệ thống hóa tài liệu:** Viết các báo cáo nghiên cứu, sơ đồ logic và kế hoạch cải tiến dưới dạng Markdown.

## 4. Kết quả bàn giao (Deliverables)
- Thư mục tài liệu nghiên cứu tại: `strategies_backtests/docs/strategy_research/dymate/`.
- Các file Markdown mô tả: Logic trạng thái mới, Danh sách Indicator đề xuất, và Kế hoạch porting chi tiết.

## 5. Phạm vi (Scope)
- **Trong phạm vi:** Nghiên cứu, phân tích, thiết kế logic và lập kế hoạch.
- **Ngoài phạm vi:** Thực thi code Rust (việc này sẽ thuộc về một Track porting sau đó).
