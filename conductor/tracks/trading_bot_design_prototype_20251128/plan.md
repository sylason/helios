# Implementation Plan - Trading Bot Design & Prototype

## Phase 1: Research & Discovery (Nghiên cứu cơ sở)
- [ ] Task: Đọc và phân tích mã nguồn `trading_bot` hiện tại (docs/code).
- [ ] Task: Phân tích các điểm tích hợp (Integration Points) với `ferro_trade` và `helios`.
- [ ] Task: Conductor - User Manual Verification 'Research Complete' (Protocol in workflow.md).

## Phase 2: Functional Design & UML (Thiết kế chi tiết)
- [ ] Task: Thảo luận và định nghĩa chức năng chi tiết cho từng trang (Page-by-Page Features).
- [ ] Task: Xây dựng sơ đồ Sequence UML mô tả luồng xử lý Tick-to-Signal.
- [ ] Task: Xây dựng sơ đồ Component UML mô tả sự liên kết giữa Bot, Engine và Strategy.
- [ ] Task: Conductor - User Manual Verification 'Design Approved' (Protocol in workflow.md).

## Phase 3: Prototype Scaffolding (Xây dựng khung Demo)
- [ ] Task: Thiết lập cấu trúc dự án Prototype (Sử dụng Rust UI tối giản hoặc Dear PyGui cho demo nhanh).
- [ ] Task: Triển khai logic điều phối (Orchestrator) cơ bản để kết nối các module.
- [ ] Task: Conductor - User Manual Verification 'Prototype Ready' (Protocol in workflow.md).

## Phase 4: Integration & Demo (Tích hợp & Chạy thử)
- [ ] Task: Kết nối dữ liệu thật (Binance WS) và engine `ferro_trade` vào Prototype.
- [ ] Task: Chạy thử demo giám sát chiến lược Helios trên UI.
- [ ] Task: Conductor - User Manual Verification 'Demo Successful' (Protocol in workflow.md).
