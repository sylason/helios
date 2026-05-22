# Blueprint Thành phần: `IndicatorCache`

## 1. Triết lý & Vai trò
`IndicatorCache` là **"Thư viện Chỉ báo Kỹ thuật"** của hệ thống. Nó tuân thủ nguyên tắc "Tính toán Một lần, Sử dụng Nhiều lần" để đảm bảo hiệu năng tối đa. Vai trò duy nhất của nó là nhận dữ liệu OHLCV thô và cấu hình, tính toán **toàn bộ** các biến thể của các chỉ báo cần thiết **một lần duy nhất** ngay khi khởi tạo, và sau đó cung cấp khả năng truy cập các giá trị này một cách cực kỳ nhanh chóng trong suốt quá trình backtest.

## 2. Trách nhiệm Chính
*   **Tính toán Trước (Pre-computation):** Trong hàm khởi tạo `new()`, nó sẽ nhận toàn bộ `DataFrame` lịch sử và tính toán tất cả các chỉ báo trên tất cả các khung thời gian được yêu cầu trong file cấu hình.
*   **Lưu trữ Hiệu năng cao (High-Performance Storage):** Lưu trữ kết quả tính toán vào `HashMap<String, polars::Series>`, nơi `key` là định danh duy nhất (ví dụ: `"rsi_14_5min"`) và `value` là một chuỗi dữ liệu Polars.
    *   **Polars:** Là thư viện DataFrame được viết bằng Rust, cực kỳ nhanh và hiệu quả cho việc xử lý dữ liệu tài chính.
*   **Cung cấp Giao diện Truy cập (Provide Access Interface):** Cung cấp các phương thức đơn giản để các module khác có thể lấy giá trị chỉ báo tức thời hoặc chuỗi lịch sử.

## 3. Cấu trúc Dữ liệu (Fields)
```rust
use polars::prelude::*;
use std::collections::HashMap;

pub struct IndicatorCache {
    // Không giữ dữ liệu OHLCV gốc, chỉ giữ kết quả tính toán của các chỉ báo.
    // Key: Tên chỉ báo (ví dụ: "rsi_14_5min", "kama_short_50_15min")
    // Value: Series chứa toàn bộ chuỗi dữ liệu của chỉ báo đó
    indicators: HashMap<String, Series>,
    // Có thể bao gồm cấu hình ban đầu để tham chiếu khi cần
    config: IndicatorConfig, 
}
```

## 4. Các Hành vi Chính (Methods)
*   `pub fn new(config: IndicatorConfig, ohlcv_data: &DataFrame) -> Result<Self, CacheError>`
    *   **Mục đích:** Hàm khởi tạo, thực hiện tất cả các phép tính chỉ báo ban đầu.
    *   **Input:**
        *   `config`: Cấu hình cho tất cả các chỉ báo cần tính toán (thời gian, chu kỳ, tham số, khung thời gian).
        *   `ohlcv_data`: `DataFrame` chứa toàn bộ dữ liệu OHLCV lịch sử.
    *   **Output:** `Result<Self, CacheError>` (một `IndicatorCache` đã được điền đầy đủ hoặc một lỗi).

*   `pub fn get_current_values(&self, index: usize) -> Result<HashMap<String, f64>, CacheError>`
    *   **Mục đích:** Lấy một bản snapshot của tất cả các giá trị chỉ báo tại một cây nến cụ thể.
    *   **Input:** `index`: Chỉ số của cây nến hiện tại trong chuỗi dữ liệu.
    *   **Output:** `Result<HashMap<String, f64>, CacheError>` (một map chứa tên chỉ báo và giá trị `f64` tương ứng tại `index`, hoặc một lỗi).

*   `pub fn get_historical_series(&self, indicator_name: &str) -> Option<&Series>`
    *   **Mục đích:** Trả về một tham chiếu (read-only) đến toàn bộ chuỗi dữ liệu lịch sử của một chỉ báo cụ thể.
    *   **Input:** `indicator_name`: Tên định danh của chỉ báo (ví dụ: "adx_14_5min").
    *   **Output:** `Option<&Series>` (tham chiếu đến `Series` nếu tồn tại, hoặc `None`).

*   `pub fn get_full_ohlcv_series(&self, column_name: &str) -> Option<&Series>`
    *   **Mục đích:** Cung cấp quyền truy cập vào các Series OHLCV gốc (ví dụ: `Close`, `High`, `Low`) nếu chúng được lưu trữ trong `IndicatorCache` hoặc được truyền vào từ bên ngoài.
    *   **Input:** `column_name`: "Open", "High", "Low", "Close", "Volume".
    *   **Output:** `Option<&Series>`.

## 5. Luận điểm Thiết kế
*   **Hiệu năng:** Việc tính toán trước tất cả chỉ báo đảm bảo `MarketAnalyzer` và các module khác có thể truy cập dữ liệu cực nhanh ở mỗi tick, loại bỏ độ trễ.
*   **Tính bất biến (Immutability):** Dữ liệu trong `IndicatorCache` là bất biến sau khi khởi tạo, giúp an toàn và dễ kiểm thử.
*   **Polars:** Lựa chọn `Polars` mang lại hiệu năng xử lý dữ liệu vượt trội, tận dụng tối đa sức mạnh của Rust.
