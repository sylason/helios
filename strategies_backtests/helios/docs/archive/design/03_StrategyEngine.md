# Blueprint Thành phần: `StrategyEngine`

## 1. Triết lý & Vai trò
`StrategyEngine` là **"Chỉ huy Chiến thuật"** của hệ thống. Vai trò của nó là nhận bản báo cáo tình báo (`MarketStateInfo`) và dựa trên một "cuốn sách chiến thuật" (`Strategy` trait) đã được định sẵn, nó sẽ ra quyết định hành động: "Với tình hình này, chúng ta nên tấn công theo hướng nào, và khi nào thì bóp cò?".

Kiến trúc này tách biệt rõ ràng logic "làm thế nào để quyết định" khỏi việc điều phối chung của hệ thống, tuân thủ Nguyên tắc Đơn trách nhiệm.

## 2. Trách nhiệm Chính
*   **Đóng gói Logic Chiến lược:** Đóng gói toàn bộ logic ra quyết định, bao gồm cả việc chọn Playbook (logic của `StrategySelector` trong Dymate) và kiểm tra tín hiệu vào lệnh (logic của `StrategyExecutor` trong Dymate).
*   **Thực thi "Hợp đồng" `Strategy`:** Là "người thực thi" của `trait Strategy`. Nó không tự mình chứa logic, mà nó gọi các phương thức được định nghĩa trong `trait Strategy` mà nó đang giữ.
*   **Tạo Tín hiệu Hành động:** Đầu ra của nó là các tín hiệu hành động cấp cao như `TradeSignal` (để vào lệnh mới) hoặc `ManagementSignal` (để quản lý vị thế nâng cao). Nó không quyết định "mua bao nhiêu" hay "SL ở đâu".

## 3. Cấu trúc Dữ liệu (Fields)
```rust
// Pseudocode cho struct StrategyEngine
pub struct StrategyEngine {
    // Chứa các tham số cấu hình liên quan đến việc thực thi chiến lược
    // (ví dụ: các ngưỡng RSI để vào lệnh), được port 1:1 từ dymate_config.py.
    config: StrategyConfig,

    // "Khe cắm" chứa chiến lược cụ thể sẽ được chạy.
    // Đây là chìa khóa cho kiến trúc framework linh hoạt.
    strategy: Box<dyn Strategy>,
}
```

## 4. Các Hành vi Chính (Methods)
*   `pub fn new(config: StrategyConfig, strategy: Box<dyn Strategy>) -> Self`
    *   **Mục đích:** Hàm khởi tạo, nhận vào cấu hình và một đối tượng chiến lược cụ thể.
    *   **Input:**
        *   `config`: Struct chứa các tham số thực thi chiến lược.
        *   `strategy`: Một đối tượng đã được đóng hộp (`Box`) thực thi `trait Strategy`.
    *   **Output:** Một instance của `StrategyEngine`.

*   `pub fn find_signal( &self, state: &MarketStateInfo, indicator_cache: &IndicatorCache ) -> Option<TradeSignal>`
    *   **Mục đích:** Tìm kiếm tín hiệu vào lệnh MỚI cho cây nến hiện tại.
    *   **Luồng hoạt động nội bộ:**
        1.  Gọi `self.strategy.select_playbook(state)` để lấy Playbook phù hợp.
        2.  Lặp qua danh sách `entry_strategies` trong `playbook` theo thứ tự ưu tiên.
        3.  Với mỗi chiến lược con, gọi `self.strategy.check_entry(strategy_name, state, ...)` để kiểm tra.
        4.  Trả về `Some(TradeSignal)` đầu tiên tìm thấy, hoặc `None`.

*   `pub fn find_management_action( &self, position: &Position, state: &MarketStateInfo, indicator_cache: &IndicatorCache ) -> Option<ManagementSignal>`
    *   **Mục đích:** Tìm kiếm các hành động quản lý NÂNG CAO cho các vị thế đang mở (ví dụ: Pyramiding, Climax Exit).
    *   **Luồng hoạt động nội bộ:**
        1.  Gọi `self.strategy.check_management(position, state, ...)`
        2.  Trả về `Some(ManagementSignal)` nếu có một hành động đặc biệt cần thực hiện.
    *   **Lưu ý:** Logic Trailing Stop Loss thông thường sẽ do `RiskManager` xử lý.

## 5. Luận điểm Thiết kế
*   **Tính Module hóa:** Tách `StrategyEngine` ra khỏi `HeliosOrchestrator` giúp cho `Orchestrator` chỉ cần quan tâm đến việc "điều phối", trong khi `StrategyEngine` chỉ cần quan tâm đến "logic giao dịch". Điều này làm cho code sạch hơn và dễ bảo trì hơn.
*   **Tính Linh hoạt:** Việc sử dụng `Box<dyn Strategy>` cho phép chúng ta dễ dàng "cắm" bất kỳ chiến lược mới nào vào hệ thống mà không cần sửa đổi `StrategyEngine` hay `HeliosOrchestrator`. Đây là nền tảng của một framework thực thụ.
*   **Bản sao Trung thành:** Logic bên trong các hàm `check_entry` và `select_playbook` của `DymatePortStrategy` (bản implement đầu tiên của `trait Strategy`) sẽ được port 1:1 từ `strategy.py` của Dymate.
