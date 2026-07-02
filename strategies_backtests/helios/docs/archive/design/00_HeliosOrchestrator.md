# Blueprint Thành phần: `HeliosOrchestrator`

## 1. Triết lý & Vai trò

`HeliosOrchestrator` là **"Hệ thần kinh Trung ương"** hay **"Nhạc trưởng"** của ứng dụng. Nó không trực tiếp thực hiện các phép tính toán học phức tạp hay phân tích sâu. Thay vào đó, vai trò của nó là **sở hữu** và **điều phối** hoạt động của tất cả các thành phần chuyên biệt khác, đảm bảo dữ liệu chảy đúng luồng và đúng thứ tự ở mỗi nhịp của backtest.

## 2. Trách nhiệm Chính

1.  **Sở hữu (Ownership):** Là chủ sở hữu duy nhất của tất cả các module cấp cao khác (`IndicatorCache`, `MarketAnalyzer`, `StrategyEngine`...). Điều này tạo ra một cây cấu trúc rõ ràng và đảm bảo các module được khởi tạo và hủy một cách an toàn.
2.  **Cấu hình (Configuration):** Chịu trách nhiệm đọc file cấu hình tổng thể và truyền các phần cấu hình liên quan đến cho từng module con trong quá trình khởi tạo.
3.  **Điều khiển Vòng lặp Chính (Main Loop Control):** Chứa phương thức `run()` để bắt đầu quá trình backtest. Bên trong, nó sẽ lặp qua từng cây nến và gọi phương thức `tick()` để điều phối chuỗi hành động.

## 3. Cấu trúc Dữ liệu (Fields)

```rust
// Pseudocode cho struct HeliosOrchestrator
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

// Các types này sẽ được định nghĩa trong types.rs và các blueprints khác
use crate::types::{MarketStateInfo, AccountState, Position, HeliosConfig, HeliosError};
use crate::indicator_cache::IndicatorCache;
use crate::market_analyzer::MarketAnalyzer;
use crate::strategy_engine::StrategyEngine;
use crate::risk_manager::RiskManager;
use crate::execution_gateway::ExecutionGateway; // trait ExecutionGateway

// DataSource là một trait mà bất kỳ nguồn dữ liệu nào cũng phải implement (ví dụ: đọc từ CSV)
pub trait DataSource {
    fn next_candle(&mut self) -> Option<Ohlcv>; // Ohlcv là một struct cơ bản cho dữ liệu nến
    fn total_candles(&self) -> usize;
    fn get_datetime_at_index(&self, index: usize) -> Option<DateTime<Utc>>;
}

// Cấu trúc cơ bản cho một cây nến (Open, High, Low, Close, Volume)
pub struct Ohlcv { /* ... */ }

pub struct HeliosOrchestrator {
    // Các module con mà nó sở hữu
    indicator_cache: IndicatorCache,
    market_analyzer: MarketAnalyzer,
    strategy_engine: StrategyEngine,
    risk_manager: RiskManager,
    // Sử dụng Trait Object để trừu tượng hóa gateway
    execution_gateway: Box<dyn ExecutionGateway>, 

    // Trạng thái nội bộ của hệ thống
    state_history: VecDeque<MarketStateInfo>, // Lịch sử N trạng thái gần nhất
    account_state: AccountState, // Thông tin tài khoản (vốn, equity...)
    open_positions: Vec<Position>, // Danh sách các vị thế đang mở
    current_candle_index: usize, // Chỉ số cây nến hiện tại
}
```

## 4. Các Hành vi Chính (Methods)

*   `pub fn new(config: HeliosConfig, strategy_impl: Box<dyn Strategy>, execution_gateway_impl: Box<dyn ExecutionGateway>) -> Result<Self, HeliosError>`
    *   **Mục đích:** Hàm khởi tạo.
    *   **Input:**
        *   `config`: Đối tượng cấu hình tổng thể của Helios.
        *   `strategy_impl`: Một đối tượng `Box<dyn Strategy>` chứa logic chiến lược cụ thể.
        *   `execution_gateway_impl`: Một đối tượng `Box<dyn ExecutionGateway>` để giao tiếp với môi trường thực thi.
    *   **Output:** Một instance của `HeliosOrchestrator` hoặc `HeliosError`.
    *   **Luồng hoạt động:** Đọc `config`, khởi tạo tất cả các module con (truyền các phần config liên quan), và "cắm" `strategy_impl` vào `StrategyEngine`, `execution_gateway_impl` vào `execution_gateway`.

*   `pub fn run<T: DataSource>(&mut self, mut data_source: T) -> Result<BacktestResult, HeliosError>`
    *   **Mục đích:** Phương thức chính để bắt đầu một phiên backtest.
    *   **Input:** `data_source`: Một đối tượng implement `trait DataSource`.
    *   **Output:** `Result<BacktestResult, HeliosError>`.
    *   **Luồng hoạt động:** Lặp qua từng cây nến từ `data_source` bằng cách gọi `data_source.next_candle()`, và gọi phương thức `tick()` cho mỗi cây nến.

*   `fn tick(&mut self, candle: &Ohlcv, current_timestamp: DateTime<Utc>) -> Result<(), HeliosError>`
    *   **Mục đích:** "Nhịp tim" của hệ thống, thực hiện chuỗi hành động tuần tự cho một cây nến.
    *   **Luồng hoạt động:**
        1.  Tăng `current_candle_index`.
        2.  `let current_indicators_map = self.indicator_cache.get_current_values(self.current_candle_index)?;`
        3.  `let market_state = self.market_analyzer.analyze(&current_indicators_map, &self.indicator_cache, &self.state_history, &current_timestamp);`
        4.  `self.state_history.push_back(market_state.clone());`
        5.  **Quản lý Vị thế Đang mở:**
            *   Cập nhật `self.open_positions` và `self.account_state` bằng cách gọi `self.execution_gateway.get_open_positions()` và `get_account_state()`.
            *   `let management_signal = self.strategy_engine.find_management_action(&self.open_positions, &market_state, &self.indicator_cache);`
            *   `if let Some(signal) = management_signal {`
            *   `let action_order = self.risk_manager.handle_advanced_signal(&signal, &self.open_positions, ...);`
            *   `if let Some(order) = action_order { self.execution_gateway.send_order(order)?; }`
            *   `}`
            *   `self.risk_manager.manage_open_position(&mut self.open_positions, &market_state, &current_indicators_map);` (Cho TSL mặc định)
        6.  **Tìm kiếm & Thực thi Lệnh Mới:**
            *   `if self.open_positions.is_empty() {`
            *   `let trade_signal = self.strategy_engine.find_signal(&market_state, &self.indicator_cache);`
            *   `if let Some(signal) = trade_signal {`
            *   `let execution_order = self.risk_manager.calculate_execution_order(&signal, &market_state, &self.account_state, current_candle_price);`
            *   `if let Some(order) = execution_order { self.execution_gateway.send_order(order)?; }`
            *   `}`
            *   `}`
        7.  `self.execution_gateway.execute_pending_actions();` (Gửi tất cả các lệnh đã được tích lũy đến `ferro_trade`).

## 5. Luận điểm Thiết kế
*   **Điều phối Trung tâm:** `HeliosOrchestrator` là điểm vào duy nhất cho toàn bộ hệ thống, đơn giản hóa việc khởi tạo và chạy backtest.
*   **Rõ ràng về Sở hữu:** Việc nó sở hữu tất cả các module con đảm bảo một cấu trúc bộ nhớ rõ ràng và an toàn theo nguyên tắc của Rust.
*   **Linh hoạt:** Việc nhận `Box<dyn Strategy>` và `Box<dyn ExecutionGateway>` trong constructor là chìa khóa để cho phép Helios chạy bất kỳ chiến lược nào hoặc kết nối với bất kỳ môi trường thực thi nào.
*   **Kiểm soát Luồng:** Phương thức `tick()` phác thảo rõ ràng toàn bộ luồng xử lý dữ liệu và quyết định trong một nhịp backtest.