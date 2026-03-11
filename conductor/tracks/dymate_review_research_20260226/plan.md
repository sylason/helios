# Implementation Plan: Dymate Strategy Logic Review & Research

## Phase 1: Initial Logic Audit & Documentation Setup
- [ ] **Task: Infrastructure Setup**
    - [ ] Khởi tạo thư mục `strategies_backtests/docs/strategy_research/dymate/`.
    - [ ] Tạo các template markdown cho báo cáo nghiên cứu.
- [ ] **Task: Current State Mapping**
    - [ ] Sơ đồ hóa logic hiện tại của Dymate (Python).
    - [ ] Liệt kê tất cả các Market States và các điều kiện kích hoạt.
    - [ ] Xác định các điểm chồng lấp hoặc mâu thuẫn giữa các state.
- [ ] **Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)**

## Phase 2: Deep Analysis & Optimization
- [ ] **Task: Market State & Selection Priority Review**
    - [ ] Kiểm tra độ phủ của các trạng thái thị trường đối với các điều kiện thực tế.
    - [ ] Tối ưu hóa logic ưu tiên (Priority) khi nhiều state xuất hiện cùng lúc.
- [ ] **Task: Strategy Enhancement (Breakout & Momentum)**
    - [ ] Nghiên cứu các biến thể Breakout (False Break, Re-test) để tích hợp vào Entry logic.
    - [ ] Đánh giá khả năng kết hợp Momentum Indicators để lọc tín hiệu nhiễu.
- [ ] **Task: Advanced Concepts (Liquidity & Smart Money)**
    - [ ] Phân tích cách áp dụng Order Flow/Liquidity vào việc xác định vùng Entry an toàn.
- [ ] **Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)**

## Phase 3: Indicator Selection & Rust Mapping
- [ ] **Task: Indicator Gap Analysis**
    - [ ] Đối chiếu các chỉ báo trong Python với các hàm có sẵn trong Kand/Yata.
    - [ ] Đề xuất thay thế hoặc tinh chỉnh các chỉ báo để tối ưu hiệu suất Rust.
- [ ] **Task: Custom Logic Definition**
    - [ ] Định nghĩa logic cho các chỉ báo không có sẵn trong thư viện.
- [ ] **Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)**

## Phase 4: Helios Porting Blueprint
- [ ] **Task: Simplified Logic Design**
    - [ ] Đề xuất mô hình State Machine đơn giản hóa nhưng hiệu quả hơn.
    - [ ] Hoàn thiện tài liệu hướng dẫn Porting (Blueprint) chi tiết cho Helio project.
- [ ] **Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)**
