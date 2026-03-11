# Specification: Design & Prototype - Trading Bot

## 1. Overview
Thiết kế lại logic chức năng của ứng dụng Trading Bot và xây dựng một bản Prototype (Demo) hoàn chỉnh về luồng hoạt động. Mục tiêu là định hình cách app vận hành trước khi đi vào hoàn thiện chi tiết.

## 2. Goals
- **Collaborative Design:** Thảo luận, đặt câu hỏi và nghiên cứu để hoàn thiện ý tưởng ứng dụng.
- **Text-based UML:** Sử dụng các sơ đồ UML dạng văn bản để mô tả kiến trúc và luồng dữ liệu.
- **Functional Prototype:** Xây dựng bản demo kết nối thành công Rust Backend (`ferro_trade`/`helios`) với một giao diện điều khiển cơ bản.
- **Baseline Integration:** Kế thừa và cải tiến từ mã nguồn `trading_bot` hiện tại.

## 3. Scope
- **Page Logic:** Định nghĩa chức năng cho Dashboard, Strategy Lab, Data Manager và Settings.
- **App Behavior:** Luồng từ lúc nhận dữ liệu Tick -> Xử lý Polars -> Strategy Engine -> Tín hiệu -> UI Monitoring.
- **UI/UX:** Giữ ở mức tối giản (Minimalist/Pro Terminal), ưu tiên tính năng hơn thẩm mỹ.

## 4. Technical Constraints
- **Language:** Rust (Backend logic), Python (UI Prototype hoặc Rust UI cơ bản).
- **Integration:** Đọc trực tiếp code từ `ferro_trade` và `helios` để đảm bảo tính tương thích.
