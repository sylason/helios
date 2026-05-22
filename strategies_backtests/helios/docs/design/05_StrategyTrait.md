# Blueprint Thành phần: `trait Strategy` (Hợp đồng Chiến lược)

## 1. Triết lý & Vai trò

`trait Strategy` là một **"Bản hợp đồng" (Contract)** trong kiến trúc của Helios. Nó không phải là một chiến lược cụ thể, mà là một bản mô tả công việc (interface), định nghĩa rõ ràng: "Để được coi là một 'Chiến lược' có thể chạy được bởi cỗ máy Helios, một `struct` phải có khả năng làm những việc gì?".

Việc định nghĩa một `trait` rõ ràng như thế này là chìa khóa để biến Helios thành một **framework thực thụ**, cho phép "cắm" các ý tưởng chiến lược mới vào mà không cần thay đổi cỗ máy vận hành.

## 2. Định nghĩa Giao diện (API Definition)

Đây là những hành vi mà mọi `struct` chiến lược phải cung cấp.

```rust
// Pseudocode cho trait Strategy
use rust_decimal::Decimal;
use chrono::NaiveDateTime;
use std::collections::HashMap; // IndicatorCache uses HashMap

// Các types này sẽ được định nghĩa trong types.rs
use crate::types::{MarketStateInfo, SubStrategyProfile, TradeSignal, ManagementSignal, SubStrategy, TradeDirection};
// Các struct này sẽ được định nghĩa ở các component khác
use crate::indicator_cache::IndicatorCache;
use crate::broker_components::{Position, AccountState}; // Position, AccountState cần được định nghĩa hoặc import

pub trait Strategy {
    /// Trả về tên của chiến lược, dùng để định danh và ghi log.
    fn name(&self) -> &'static str;

    /// PHƯƠNG THỨC QUYẾT ĐỊNH VĨ MÔ
    /// Dựa trên trạng thái thị trường, chọn ra một Playbook chiến thuật phù hợp.
    /// Đây là nơi chứa logic của `StrategySelector` cũ.
    fn select_playbook(&self, state: &MarketStateInfo) -> SubStrategyProfile;

    /// PHƯƠNG THỨC KIỂM TRA TÍN HIỆU VI MÔ
    /// `StrategyEngine` sẽ gọi hàm này và truyền vào tên của một chiến lược con
    /// (ví dụ: `SubStrategy::TrendPullback`) từ playbook đã được chọn.
    /// Hàm này sẽ kiểm tra các điều kiện trigger chi tiết.
    fn check_entry(
        &self,
        strategy_name: SubStrategy,
        state: &MarketStateInfo,
        indicator_cache: &IndicatorCache
    ) -> Option<TradeSignal>;

    /// PHƯƠNG THỨC QUẢN LÝ NÂNG CAO
    /// Kiểm tra xem với vị thế hiện tại và trạng thái thị trường, có cần
    /// thực hiện một hành động quản lý nâng cao (Pyramiding, Climax Exit) hay không.
    fn check_management(
        &self,
        position: &Position,
        state: &MarketStateInfo,
        indicator_cache: &IndicatorCache
    ) -> Option<ManagementSignal>;
}
```

## 3. Luận điểm Thiết kế

*   **Tính Trừu tượng (Abstraction):** `trait` này trừu tượng hóa hoàn toàn "logic nghiệp vụ" của một chiến lược. Các thành phần cấp cao như `HeliosOrchestrator` và `StrategyEngine` chỉ cần làm việc với `trait` này mà không cần biết chi tiết về `DymatePortStrategy` hay `WyckoffStrategy`.
*   **Tính Mở rộng (Extensibility):** Bất kỳ ai (kể cả bạn trong tương lai) cũng có thể tạo ra một chiến lược hoàn toàn mới bằng cách tạo một `struct` mới và `impl Strategy` cho nó. Hệ thống Helios có thể chạy chiến lược đó mà không cần sửa đổi.
*   **Tính Tuân thủ (Compliance):** Trình biên dịch của Rust sẽ đảm bảo rằng bất kỳ `struct` nào tự nhận là `Strategy` đều phải cung cấp đầy đủ các hành vi đã được định nghĩa, đảm bảo tính nhất quán và giảm thiểu lỗi.