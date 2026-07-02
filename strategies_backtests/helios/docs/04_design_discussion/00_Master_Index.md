# Helios Design Discussion: Master Index

Chào mừng đến với Trung tâm Thảo luận Thiết kế của dự án Helios. Đây là nơi lưu trữ toàn bộ các phân tích chuyên sâu, định hướng chiến lược và các bài học rút ra từ việc thiết kế hệ thống giao dịch thuật toán đòn bẩy cao (High Leverage Algorithmic Trading).

Dự án Helios không chỉ là một bot giao dịch thông thường, nó là một cỗ máy chiến tranh được thiết kế để sống sót và khai thác tối đa lợi nhuận trong môi trường phái sinh khắc nghiệt nhất.

## 📂 Danh mục Phân tích & Thảo luận

### 1. Tư duy & Triết lý Cốt lõi (Strategic Doctrine)
Những nguyên lý cơ bản nhất tạo nên linh hồn của Helios.
- [1a. Sự thích nghi của Mahoraga (The Mahoraga Adaptation)](./01_strategic_doctrine/1a_The_Mahoraga_Adaptation.md)
- [1b. Nghệ thuật Do thám StarCraft (The StarCraft Reconnaissance)](./01_strategic_doctrine/1b_The_StarCraft_Reconnaissance.md)
- [1c. Giao thức Hạt nhân (The Nuke Protocol)](./01_strategic_doctrine/1c_The_Nuke_Protocol.md)

### 2. Điểm yếu & Lỗ hổng Trí mạng (Flaws & Vulnerabilities)
Phân tích những sai lầm trong thiết kế mã nguồn hiện tại của Helios (đặc biệt trong file `tactical.rs`) có thể dẫn đến cháy tài khoản khi đánh đòn bẩy 100x.
- [2a. Quản lý vốn cho Đòn bẩy cao (Position Sizing For High Leverage)](./02_flaws_and_vulnerabilities/2a_Position_Sizing_For_High_Leverage.md) 🚨 **QUAN TRỌNG**
- [2b. Lỗ hổng Dữ liệu Vi mô (Microstructure Missing Links)](./02_flaws_and_vulnerabilities/2b_Microstructure_Missing_Links.md)
- [2c. Lỗ hổng Đa khung thời gian (MTF Confluence Gap)](./02_flaws_and_vulnerabilities/2c_MTF_Confluence_Gap.md)

### 3. Đề xuất Nâng cấp Logic (Business Logic Upgrades)
Các tính năng cần bổ sung vào mã nguồn để biến Helios thành một hệ thống Institutional Grade.
- [3a. Thực thi lệnh: Maker vs Taker](./03_business_logic_upgrades/3a_Maker_vs_Taker_Execution.md)
- [3b. Động cơ Chống Thanh lý (Liquidation Protector)](./03_business_logic_upgrades/3b_Liquidation_Engine_Protector.md)
- [3c. Kỹ thuật Nhồi lệnh (Pyramiding The Winners)](./03_business_logic_upgrades/3c_Pyramiding_The_Winners.md)

### 4. Đánh giá Tương tác (Interactive Review)
Khu vực dành riêng cho bạn (User) để trả lời các câu hỏi trắc nghiệm, giúp AI hiểu rõ hơn về khẩu vị rủi ro và mục tiêu của bạn.
- [4a. Bài Test Trắc nghiệm (Grill-Me Questionnaire)](./04_interactive_review/4a_Grill_Me_Questionnaire.md) 📝 **CẦN BẠN TRẢ LỜI**
