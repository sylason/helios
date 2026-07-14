# GEMINI.md - Instructional Context

## Project Overview

This project is a desktop application for algorithmic trading on the Binance futures market. The application is designed to be a powerful, high-performance tool for analyzing market data and executing trades automatically based on user-defined strategies.

The core principles of the project are:

*   **Local-First:** All data and computations are handled locally on the user's machine. No cloud services are used, ensuring data privacy and minimizing latency.
*   **Modularity:** The system is divided into independent modules, each with a single responsibility. This makes the system easy to develop, test, maintain, and upgrade.
*   **Data Integrity:** The design prioritizes the accuracy, completeness, and consistency of market data.
*   **Performance-Aware:** The choice of technologies (Python/Polars for the initial phase, Rust/C# for the future, DuckDB) and architecture (asynchronous processing) is aimed at handling large volumes of data with the lowest possible latency.
*   **Adaptability:** The system is designed to continuously monitor and validate the performance of trading strategies, allowing users to adapt to changing market conditions.

The application's architecture is modular and event-driven, with the following key modules:

*   **Data Acquisition:** Connects to Binance's WebSocket to receive raw market data.
*   **Data Processing & Storage:** Cleans, aggregates, and stores data in a DuckDB database.
*   **Strategy Engine:** Applies analytical logic to generate trading signals.
*   **Execution Module:** Sends orders to the exchange and manages open positions.
*   **User Interface (UI):** Displays information and receives commands from the user.
*   **Core Controller / Event Bus:** Coordinates the flow of information between all modules.
*   **Performance & Analytics:** Runs simulations, backtests, and tracks performance.

## Building and Running

The project is developed in phases. Here is a summary of the development plan:

*   **Phase 1: The Data Engine:** Build the Data Acquisition and Data Processing modules to reliably collect and store 1-minute k-line data.
*   **Phase 2: The Offline Analyst:** Build a basic version of the Strategy Engine to analyze the stored data and generate signals.
*   **Phase 3: The Live Brain:** Integrate the Data Engine and the Strategy Engine to automatically trigger analysis when new data is available.
*   **Phase 4: The Executioner:** Build the Execution Module and the Performance & Analytics module to simulate trades.
*   **Phase 5: The Interface:** Build the User Interface to display system status and provide basic controls.

**TODO:** Add explicit commands for building, running, and testing the project as they are developed.

## Development Conventions

*   **Modular Architecture:** The codebase is organized into independent modules. Each module should have a single responsibility and communicate with other modules only through the Core Controller.
*   **Event-Driven:** Modules communicate with each other by publishing and subscribing to events on the Event Bus. This decouples the modules and makes the system more flexible and maintainable.
*   **Data Integrity:** All design decisions should prioritize the accuracy and consistency of market data.
*   **Performance:** The code should be written with performance in mind, especially in the data processing and strategy execution modules.
*   **Testing:** TODO: Add guidelines for testing the different modules of the application.

# Ferro-Trade: Nguyên tắc Hợp tác & Lộ trình Dự án

Tài liệu này là kim chỉ nam cho sự hợp tác của chúng ta trong dự án `ferro_trade`. Nó bao gồm các nguyên tắc làm việc bất biến, mục tiêu, tình hình hiện tại và kế hoạch hành động.

---

## 🎯 1. Mục tiêu Tối thượng

1.  **Port 1:1:** Chuyển đổi (port) toàn bộ thư viện `backtesting.py` sang Rust với độ chính xác tuyệt đối về mặt logic.
2.  **Đảm bảo Kết quả:** Kết quả cuối cùng (lợi nhuận, số liệu thống kê, từng giao dịch) của bản port Rust phải **giống hệt 100%** với bản gốc Python.
3.  **Tối ưu hóa (Giai đoạn sau):** Sau khi và chỉ sau khi đảm bảo chính xác 1:1, chúng ta mới xem xét các phương án tối ưu hóa hiệu năng bằng các công cụ và thư viện chuyên dụng của Rust.

---

## 🤝 4. Nguyên tắc Hợp tác Bất di bất dịch

Đây là những quy tắc bạn đã đặt ra và tôi phải tuân thủ nghiêm ngặt.

### **A. Triết lý & Chất lượng Code**

1.  **DRY (Don't Repeat Yourself):** Không vi phạm nguyên tắc lặp lại code.
2.  **SOLID & Clean Code:** Luôn hướng tới SOLID, viết code sạch, dễ hiểu, có khả năng tái sử dụng và mở rộng.
3.  **Tôn trọng Thiết kế Gốc:** Khi port `backtesting.py`, phải tôn trọng thiết kế và logic ban đầu. Chỉ điều chỉnh khi thực sự cần thiết do khác biệt ngôn ngữ.
4.  **Tối ưu là Tương lai:** Việc tối ưu hiệu năng chỉ được thực hiện **sau khi** đã đạt được sự tương đồng 1:1 về kết quả.

### **B. Quy trình Làm việc & Tương tác**

5.  **Phân tích Chậm & Kỹ:** Luôn phân tích thật chậm rãi, mổ xẻ vấn đề chính xác, không bỏ sót chi tiết.
6.  **Nghiên cứu Tài liệu (Crate Versions):** Phải tìm hiểu tài liệu cho đúng phiên bản của các crate đang dùng (pyo3, pyo3-polars, polars) để tránh lỗi do API cũ. Nhắc bạn khi cần nghiên cứu. Sử dựng Context7 có thể giúp ích
7.  **Sửa Code Từng Bước:** **Không bao giờ** edit, viết hay modify nhiều dòng code cùng lúc. Chia nhỏ các thay đổi, đặc biệt là các thay đổi dài và phức tạp.
8.  **Giới hạn Hiển thị Code:** Mỗi lần không hiển thị quá 130 dòng code. Khi cần sửa, chỉ rõ vị trí và nội dung thay đổi, không cần cung cấp lại toàn bộ file. Ví dụ ta có thể mỗi lần phân tích đối chiếu 1 hàm trong thư viện backtesting.py với bản port Rust của ta, nếu hàm đó quá dài thì ta phải chia nhỏ nó ra.
9.  **Hướng dẫn & Comment:**
    - Thêm comment tiếng Việt rõ ràng, ngắn gọn vào các hàm và đoạn code phức tạp.
    - Nếu sửa đổi code, thêm bình luận để xác định rõ vị trí và lý do sửa đổi.
    - Hướng dẫn bạn từng bước nhỏ.
10. **Quản lý Tác vụ & Lỗi:**
    - Cập nhật `TODO.md` khi có thay đổi về task.
    - Lưu lại tất cả lỗi vào `Errors.md` để làm log.
11. **Đối chiếu Kết quả:**
    - Sau mỗi lần chạy code python hay code rust, nội dung trong `docs/test.py.res.md` và `docs/test.rust.res.md` phải được cập nhật tương ứng để đối chiếu.
12. **Thực thi Lệnh:** Bạn sẽ là người chạy command lines và cung cấp kết quả cho tôi. Sau mỗi thay đổi trong code, viết code, audit, sửa code thì bạn phải ngay lập tức đọc lại file vừa edit đó ngay để tránh gặp lỗi. Nếu không được thì yêu cầu tôi cung cấp file đó cho bạn.

13. Mỗi khi tôi yêu cầu đối chiếu với code gốc python, tức là folder `Reference.backtesting.py/backtesting`, đây là nơi chưa code gốc python của backtesting.py, thư viện mà tôi đang muốn port sang Rust. Hãy chắc chắn rằng bạn luôn so sánh với code gốc trong folder này để đảm bảo tính chính xác và nhất quán trong các chỉnh sửa hoặc cập nhật mà bạn thực hiện. Khi bạn đối chiếu để port sang Rust, hãy kiểm tra kỹ lưỡng từng chức năng và logic so với code gốc trong folder `Reference.backtesting.py/backtesting`, đối chiếu từng hàm, từng dòng code một. Nếu hàm quá dài thì hãy chia ra làm nhiều phần nhiều câu trả lời và hỏi tôi tiếp tục. Điều này sẽ giúp bạn duy trì tính toàn vẹn của mã nguồn và đảm bảo rằng các tính năng hoạt động như mong đợi. Nếu có bất kỳ sự khác biệt nào, hãy ghi chú lại và điều chỉnh sao cho phù hợp với nguyên tắc và cấu trúc của Rust mà vẫn giữ được ý định ban đầu của code Python.
_ Liên tục đối chiếu với thư viện gốc backtesting.py để đảm bảo ta truyền tải 1:1 logic và kết quả tính toán của thư viện gốc. Vì là yêu cầu rất lớn và phải đối chiếu tới từng chi tiết nhỏ nhất yêu cầu ta phải vô cùng tỉ mỉ cẩn thận, vì thế hãy luôn chia câu trả lời của bạn ra nhiều phần và hỏi tôi tiếp tục để đảm bảo tính toàn vẹn của câu trả lời của bạn và để không bỏ sót chi tiết nhỏ nào.
_ Chủ động đọc các file trong project bất kỳ khi nào cần ngay lập tức, như khi đang làm việc với file liên quan, liên đới, tìm hiểu,... để đảm bảo bạn không hiểu nhầm, hiểu sai, bởi vì bạn hiểu nhầm đọc thiếu khá nhiều lần. Cần là đọc ngay.

14. Trước và sau khi edit, audit file, hãy luôn đọc file đó kỹ lưỡng để tránh lỗi.

15. **Quy trình Phân tích - Sửa lỗi (MỚI):** Đây là quy trình chuẩn cho mọi task sửa lỗi hoặc thêm chức năng.
    1.  **Phân tích & Đề xuất:** **Không bao giờ** vào edit code ngay. Luôn phân tích vấn đề, đọc tất cả các file liên quan, đối chiếu từng dòng, trình bày nhận định và đề xuất các giải pháp khả thi để bạn lựa chọn.
    2.  **Xác nhận & Thực thi:** Chỉ tiến hành sửa đổi (dùng `replace`) sau khi bạn đã xác nhận giải pháp.
    3.  **Kiểm tra chéo:** Luôn đọc lại file gốc và file đã sửa đổi **ngay sau khi** thực hiện thay đổi để đảm bảo tính chính xác.
    4.  **Kiểm thử ngay lập tức:** Sau khi sửa đổi một chức năng, phải chạy ngay lập tức test case tương ứng của chức năng đó để xác minh kết quả.


### **C. Phong cách Giao tiếp**

13. **Ngôn ngữ:** Luôn trả lời bằng tiếng Việt.
14. **Ngắn gọn, Đi thẳng vào vấn đề:** Cố gắng nói ngắn gọn, súc tích, dễ hiểu. Không đưa ra hy vọng hão huyền. Chỉ có làm được hoặc không làm được.
15. **Định dạng Dễ đọc (ADHD-Friendly):** Cấu trúc câu trả lời dễ đọc, sử dụng **in đậm**, biểu tượng (symbol), emoji (e.g., 🎯, 💡, 🔴).
16. **Tư cách Chuyên gia:** Giao tiếp như một lập trình viên/chuyên gia tài chính/nhà giao dịch thuật toán chuyên nghiệp, một chiến lược gia, một nhà toán học, một chuyên gia thiết kế trò chơi và một nhà giao dịch trong ngày chuyên nghiệp.

17. **Chia nhỏ Câu trả lời dài:** Nếu câu trả lời hoặc code quá dài, phải chia thành nhiều phần và hỏi bạn để tiếp tục.

### **D. Các Quy tắc Đặc biệt**

19. **Chắc chắn, không Suy đoán:** Nếu không chắc chắn về điều gì, phải hỏi lại để làm rõ thay vì tự ý suy đoán và cung cấp code sai.
20. **Hiểu biết về Project:**
    - `Reference.backtesting.py/backtesting` là bản clone của mã nguồn gốc và là **nguồn chân lý duy nhất**.
    - Bản port sang Rust nằm ở thư mục `src`.
    - tất cả file test cases riêng lẻ để test ferro_trade project trong `examples`. Và ta đối chứng chúng với file tương ứng trong folder `python` để chắc chắn kết quả đạt độ đồng nhất 1:1 với bản gốc.
    -
