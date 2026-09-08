# Trading Bot Project - Master Index (Blueprint)

Chào mừng đến với hệ thống tài liệu (Spec-driven) của dự án Trading Bot. Tài liệu này đóng vai trò là điểm gốc (Root) để bạn điều hướng qua các khu vực thiết kế của toàn bộ dự án.

## 1. Yêu Cầu Sản Phẩm (Product Requirements)
Xác định rõ những gì chúng ta cần xây dựng, phân biệt rõ ranh giới giữa bản Demo nghiên cứu và bản Production.
- [Demo PRD (Minimum Viable Product)](../01_Product_Requirements_Document_PRD/demo_prd.md)
- [Production PRD (Tầm nhìn dài hạn)](../01_Product_Requirements_Document_PRD/production_prd.md)

## 2. Kiến Trúc Hệ Thống (System Architecture)
Cái nhìn toàn cảnh về cách các hệ thống giao tiếp và lưu trữ.
- [Tổng quan Kiến trúc Hệ thống](../02_System_Architecture/system_overview.md)
- [Luồng Dữ Liệu (Data Flow)](../02_System_Architecture/data_flow.md)
- [Thiết Kế CSDL (DuckDB Schema)](../02_System_Architecture/database_schema.md)

## 3. Đặc Tả Tính Năng (Feature Specifications)
Đi sâu vào từng module cốt lõi của hệ thống Bot Engine.
- [Data Engine Spec](../03_Feature_Specifications/data_engine/spec.md)
- [Strategy Engine Spec](../03_Feature_Specifications/strategy_engine/spec.md)
- [Execution & Risk Management Spec](../03_Feature_Specifications/execution_and_risk/spec.md)
- [Client UI (Tauri/React) Spec](../03_Feature_Specifications/client_ui/spec.md)

## 4. Quản Lý Dự Án (Project Management)
Theo dõi tiến độ, phân chia các Task/Epic cần làm.
- [TODO List (Agile/Scrum style)](../04_Project_Management/TODO.md)

## 5. Thảo Luận & Quyết Định (Discussions & Decisions)
Nơi giải quyết các edge cases (trường hợp ngoại lệ) và lấp đầy các lỗ hổng thiết kế.
- [Hỏi đáp / Grill Me Questions](../05_Discussions_and_Decisions/grill_me_questions.md)

---
*Lưu ý: Mọi thay đổi về kiến trúc hoặc logic lớn cần được cập nhật vào các file Spec tương ứng trước khi tiến hành sửa code.*
