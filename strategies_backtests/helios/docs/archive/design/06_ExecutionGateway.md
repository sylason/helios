# Blueprint Thành phần: `trait ExecutionGateway` & `struct HeliosAdapter`

## 1. Triết lý & Vai trò của `trait ExecutionGateway`
`trait ExecutionGateway` là **"Người đưa tin"**. Vai trò của nó là trừu tượng hóa mọi chi tiết về việc giao tiếp với nền tảng thực thi lệnh (có thể là một backtester như `ferro_trade`, hoặc một sàn giao dịch live như Binance). `HeliosOrchestrator` sẽ chỉ làm việc với `trait` này, mà không cần quan tâm đến cách lệnh được gửi đi hay trạng thái được cập nhật. Điều này giúp hệ thống Helios hoàn toàn độc lập với môi trường thực thi.

## 2. Định nghĩa Giao diện (API Definition) của `trait ExecutionGateway`
```rust
// Pseudocode cho trait ExecutionGateway
use rust_decimal::Decimal; // Sử dụng Decimal cho các giá trị tiền tệ

// Các types này sẽ được định nghĩa trong types.rs
use crate::types::{ExecutionOrder, AccountState, Position, OrderConfirmation, GatewayError, HeliosError}; // Import GatewayError

pub trait ExecutionGateway: Send + Sync { // Send + Sync để an toàn khi chạy đa luồng
    /// Gửi một lệnh giao dịch mới đến nền tảng thực thi.
    fn send_order(&mut self, order: ExecutionOrder) -> Result<OrderConfirmation, HeliosError>;

    /// Lấy trạng thái tài khoản hiện tại từ nền tảng.
    fn get_account_state(&self) -> Result<AccountState, HeliosError>;

    /// Lấy danh sách các vị thế đang mở từ nền tảng.
    fn get_open_positions(&self) -> Result<Vec<Position>, HeliosError>;

    /// Cập nhật giá Stop-Loss (SL) và/hoặc Take-Profit (TP) cho một trade cụ thể.
    fn update_trade_sl_tp(&mut self, trade_id: usize, new_sl: Option<Decimal>, new_tp: Option<Decimal>) -> Result<(), HeliosError>;

    /// Đóng một phần hoặc toàn bộ vị thế.
    fn close_position(&mut self, trade_id: usize, portion: Decimal) -> Result<(), HeliosError>;

    /// Gửi tất cả các lệnh đang chờ xử lý đến nền tảng thực thi (quan trọng cho ferro_trade).
    fn execute_pending_actions(&mut self) -> Result<(), HeliosError>;
}
```

## 3. `struct HeliosAdapter` - "Người phiên dịch" cho `ferro_trade`
`HeliosAdapter` là một `struct` cụ thể implement `trait ExecutionGateway`. Nó chịu trách nhiệm chuyển đổi các yêu cầu từ Helios sang cú pháp và kiểu dữ liệu mà `ferro_trade` hiểu được.

### 3.1. Triết lý & Vai trò
`HeliosAdapter` là **"Người phiên dịch"**. Nó là cây cầu nối giữa hệ thống logic của Helios và `ferro_trade::Broker`. Khi `HeliosOrchestrator` yêu cầu `send_order`, `HeliosAdapter` sẽ dịch `ExecutionOrder` của Helios thành `ferro_trade::broker::Action` và chuyển tiếp đến `ferro_trade::Broker`.

### 3.2. Cấu trúc Dữ liệu (Fields)
```rust
// Pseudocode cho struct HeliosAdapter
use std::sync::{Arc, Mutex};
// Import các types từ ferro_trade
use ferro_trade::broker::Broker as FerroBroker; // Đổi tên để tránh xung đột
use ferro_trade::data::DataView as FerroDataView;
use ferro_trade::broker::actions::Action as FerroAction;

pub struct HeliosAdapter {
    // Handle đến Broker của ferro_trade. Dùng Arc<Mutex> vì Broker là mutable và shared.
    ferro_broker: Arc<Mutex<FerroBroker>>,
    // Handle đến DataView của ferro_trade.
    ferro_data_view: Arc<FerroDataView>,
    // Lưu trữ các actions đã được Helios tạo ra để gửi cho ferro_trade::Broker
    pending_actions: Vec<FerroAction>,
}
```

### 3.3. Các Hành vi Chính (Methods) của `HeliosAdapter`

*   `pub fn new(ferro_broker: Arc<Mutex<FerroBroker>>, ferro_data_view: Arc<FerroDataView>) -> Self`
    *   Hàm khởi tạo. Nhận các handle đến các thành phần của `ferro_trade`.

*   `impl ExecutionGateway for HeliosAdapter { ... }` (Các hàm này sẽ implement `trait ExecutionGateway`)
    *   **`send_order(&mut self, order: ExecutionOrder) -> ...`**: Chuyển đổi `ExecutionOrder` thành `FerroAction::Buy` hoặc `FerroAction::Sell` và thêm vào `self.pending_actions`.
    *   **`get_account_state(&self) -> ...`**: Khóa `ferro_broker`, lấy `equity` và `cash`, chuyển đổi sang `AccountState` của Helios.
    *   **`get_open_positions(&self) -> ...`**: Khóa `ferro_broker`, lấy `trades` từ đó, và chuyển đổi mỗi `ferro_trade::Trade` thành `helios::Position` của Helios.
    *   **`update_trade_sl_tp(...)` & `close_position(...)`**: Chuyển đổi yêu cầu thành `FerroAction` tương ứng và thêm vào `self.pending_actions`.
    *   **`execute_pending_actions(&mut self) -> ...`**: Khóa `ferro_broker`, gọi `ferro_broker.execute_actions(self.pending_actions.drain(..).collect())` để gửi tất cả các `FerroAction` đã được tích lũy.

## 4. Luận điểm Thiết kế
*   **Tính Tương thích:** `HeliosAdapter` là cầu nối đảm bảo hệ thống Helios, vốn được thiết kế ở mức logic cao, có thể hoạt động được trên một backtest engine cụ thể là `ferro_trade`.
*   **Tách biệt (Decoupling):** Logic cốt lõi của Helios không hề biết đến sự tồn tại của `ferro_trade`. Nó chỉ làm việc với `trait ExecutionGateway`.
*   **Quản lý luồng:** Việc sử dụng `pending_actions` cho phép Helios đưa ra nhiều quyết định trong một `tick` và gửi tất cả chúng đến `ferro_trade::Broker` một lần duy nhất vào cuối `tick`.