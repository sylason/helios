# Blueprint Thành phần: `RiskManager`

## 1. Triết lý & Vai trò
`RiskManager` là **"Giám đốc Tài chính & Rủi ro" (CFO/CRO)** của hệ thống. Vai trò của nó không phải là đặt câu hỏi "Tại sao vào lệnh?", mà là trả lời câu hỏi "**Vào lệnh như thế nào?**" để đảm bảo an toàn và tối ưu hóa tỷ lệ Rủi ro/Lợi nhuận. Mọi quyết định của nó đều dựa trên các quy tắc toán học, không cảm tính.

## 2. Trách nhiệm Chính
1.  **Tính toán Lệnh Mới:** Là trách nhiệm chính. Nó nhận một `TradeSignal` (một ý tưởng) và chuyển đổi nó thành một `ExecutionOrder` hoàn chỉnh (một kế hoạch hành động cụ thể: "mua 0.1 ETH tại giá X, dừng lỗ tại Y, chốt lời tại Z").
2.  **Quản lý Vị thế Mặc định:** Chịu trách nhiệm cho việc quản lý cơ bản và liên tục đối với các vị thế đang mở, chủ yếu là logic **Trailing Stop Loss Động (Dynamic TSL)**.
3.  **Xử lý Hành động Nâng cao:** Nhận các `ManagementSignal` từ `StrategyEngine` (ví dụ: `ConsiderPyramiding`) và thực hiện các tính toán, kiểm tra an toàn cần thiết để quyết định xem có nên thực hiện hành động đó hay không.

## 3. Cấu trúc Dữ liệu (Fields)
```rust
use rust_decimal::Decimal;

pub struct RiskManager {
    // Chứa tất cả các tham số liên quan đến rủi ro:
    // % rủi ro cơ sở, tỷ lệ R:R, hệ số ATR cho SL, đòn bẩy tối đa...
    config: RiskManagerConfig,
}
```
*   **Lưu ý:** Để tránh các lỗi làm tròn số, tất cả các giá trị liên quan đến tiền (`cash`, `price`, `pnl`...) nên được xử lý bằng `rust_decimal::Decimal`.

## 4. Các Hành vi Chính (Methods)
*   `pub fn new(config: RiskManagerConfig) -> Self`
    *   Hàm khởi tạo đơn giản, nhận và lưu cấu hình rủi ro.

*   `pub fn calculate_execution_order( &self, signal: &TradeSignal, state: &MarketStateInfo, ..., account: &AccountState ) -> Option<ExecutionOrder>`
    *   **Mục đích:** Chuyển đổi một tín hiệu thô thành một lệnh hoàn chỉnh.
    *   **Luồng hoạt động nội bộ:** Thực thi một **"Chuỗi Phê duyệt Rủi ro"** 5 bước, được port 1:1 từ logic của Dymate:
        1.  **Ngân sách Rủi ro Thích ứng:** Tính `% rủi ro` cuối cùng dựa trên `confidence_score`.
        2.  **Định vị Điểm Dừng lỗ:** Tính `stop_loss_price` ban đầu dựa trên logic của tín hiệu và `ATR`.
        3.  **Định cỡ Vị thế:** Tính `size_in_asset` sao cho nếu SL bị chạm, khoản lỗ sẽ bằng đúng ngân sách.
        4.  **Xác thực Đòn bẩy:** Kiểm tra `effective_leverage` và giảm `size` nếu cần.
        5.  **Thiết lập Chốt lời:** Tính `take_profit_price` dựa trên `stop_loss_price` và tỷ lệ `R:R`.
    *   **Output:** Trả về `Some(ExecutionOrder)` nếu tất cả các bước thành công, ngược lại `None`.

*   `pub fn manage_open_position( &self, position: &mut Position, state: &MarketStateInfo, ... ) -> Option<ManagementAction>`
    *   **Mục đích:** Xử lý logic Trailing Stop Loss mặc định.
    *   **Luồng hoạt động nội bộ:**
        1.  Tính toán một hệ số nhân ATR thích ứng (`adaptive_atr_multiplier`) dựa trên `MarketStateInfo`.
        2.  Tính một mức `new_sl_candidate` dựa trên giá hiện tại và ATR.
        3.  Nếu `new_sl_candidate` tốt hơn `position.current_stop_loss`, trả về `Some(ManagementAction::UpdateStopLoss(new_sl_candidate))`.

*   `pub fn handle_advanced_signal( &self, signal: &ManagementSignal, position: &Position, ... ) -> Option<Action>`
    *   **Mục đích:** Xử lý các tín hiệu quản lý nâng cao từ `StrategyEngine`.
    *   **Luồng hoạt động nội bộ:**
        *   `WHEN signal == ConsiderPyramiding`: Chạy logic tính toán lệnh nhồi, kiểm tra các quy tắc an toàn. Nếu hợp lệ, trả về `Some(Action::OpenNew(ExecutionOrder))`.
        *   `WHEN signal == ConsiderClimaxExit`: Trả về `Some(Action::CloseNow(position.id))`.

## 5. Luận điểm Thiết kế
*   **Tách biệt Trách nhiệm:** `RiskManager` chỉ tập trung vào "làm thế nào về mặt tài chính", hoàn toàn tách biệt khỏi logic "tại sao vào lệnh" của `StrategyEngine`.
*   **An toàn là trên hết:** "Chuỗi Phê duyệt Rủi ro" đảm bảo không có một lệnh nào được tạo ra mà không tuân thủ các quy tắc quản lý vốn cốt lõe.
*   **Chính xác Tài chính:** Việc ưu tiên sử dụng `rust_decimal::Decimal` cho các tính toán tiền tệ là một yêu cầu bắt buộc để đảm bảo tính chính xác và tránh các lỗi làm tròn số của `f64`.