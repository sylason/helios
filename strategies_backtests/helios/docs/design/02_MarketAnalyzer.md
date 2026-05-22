# Blueprint Thành phần: `MarketAnalyzer`

## 1. Triết lý & Vai trò
`MarketAnalyzer` là **"Bộ máy Nhận thức" (Perception Engine)** của Helios. Vai trò duy nhất của nó là nhận các giá trị chỉ báo kỹ thuật từ `IndicatorCache` và diễn giải chúng thành một bức tranh thị trường cấp cao, có cấu trúc, được đóng gói trong đối tượng `MarketStateInfo`. Nó trả lời câu hỏi: **"Chuyện gì đang thực sự diễn ra trên thị trường?"**.

## 2. Trách nhiệm Chính
*   **Phân tích Phi trạng thái (Stateless Analysis):** `MarketAnalyzer` sẽ không lưu trữ bất kỳ trạng thái nào giữa các lần gọi. Mọi dữ liệu nó cần (giá trị hiện tại, lịch sử để tính percentile, lịch sử trạng thái để nhận diện mẫu hình) đều được truyền vào qua tham số của hàm `analyze()`.
*   **Thực thi Logic Phân loại:** Chịu trách nhiệm thực thi quy trình phân tích 4 bước trung thành với logic của Dymate (Yếu tố Cơ bản -> Chế độ Vĩ mô -> Trạng thái Vi mô -> Tổng hợp).
*   **Tính toán Ngưỡng động:** Là người dùng chính của phương thức `get_historical_series()` từ `IndicatorCache` để tính toán các ngưỡng phân vị (percentile) một cách linh hoạt ở mỗi cây nến.
*   **Tạo `MarketStateInfo`:** Đầu ra duy nhất của nó là một đối tượng `MarketStateInfo` được điền đầy đủ thông tin.

## 3. Cấu trúc Dữ liệu (Fields)
```rust
// Pseudocode cho struct MarketAnalyzer
use std::collections::VecDeque;

pub struct MarketAnalyzer {
    // Chỉ chứa cấu hình. Hoàn toàn không có trạng thái thay đổi.
    config: AnalyzerConfig,
}

// AnalyzerConfig sẽ chứa tất cả các ngưỡng, chu kỳ, hệ số...
// cần cho việc phân tích, được port 1:1 từ dymate_config.py
pub struct AnalyzerConfig {
    percentile_lookback: usize,
    adx_strong_pct: u8,
    anomaly_tr_ratio: f64,
    // ... và nhiều tham số khác
}
```

## 4. Các Hành vi Chính (Methods)
*   `pub fn new(config: AnalyzerConfig) -> Self`
    *   **Mục đích:** Hàm khởi tạo, nhận vào và lưu trữ cấu hình của nó.
    *   **Input:** `config`: Struct chứa các tham số phân tích.
    *   **Output:** Một instance của `MarketAnalyzer`.

*   `pub fn analyze( &self, current_indicators: &HashMap<String, f64>, indicator_cache: &IndicatorCache, state_history: &VecDeque<MarketStateInfo>, timestamp: &DateTime<Utc> ) -> MarketStateInfo`
    *   **Mục đích:** Phương thức công khai duy nhất, thực hiện toàn bộ quá trình phân tích cho một cây nến.
    *   **Input:**
        *   `&self`: Tham chiếu bất biến đến chính nó (để đọc `config`).
        *   `&current_indicators`: Tham chiếu đến bản snapshot các giá trị chỉ báo hiện tại.
        *   `&indicator_cache`: Tham chiếu đến cache để có thể lấy dữ liệu lịch sử.
        *   `&state_history`: Tham chiếu đến lịch sử N trạng thái gần nhất để nhận diện các mẫu hình.
        *   `&timestamp`: Dấu thời gian của nến hiện tại.
    *   **Output:** Một đối tượng `MarketStateInfo`.
    *   **Luồng hoạt động nội bộ:** Phương thức này sẽ điều phối một chuỗi các lệnh gọi đến các hàm `private` bên trong nó, mỗi hàm thực hiện một bước trong quy trình phân tích của Dymate.

## 5. Luận điểm Thiết kế
*   **Tính nhất quán:** Bằng cách thiết kế `MarketAnalyzer` như một thành phần phi trạng thái, chúng ta đảm bảo rằng với cùng một bộ dữ liệu đầu vào, nó sẽ luôn luôn trả về cùng một kết quả đầu ra. Điều này làm cho việc kiểm thử và gỡ lỗi trở nên cực kỳ dễ dàng.
*   **Bản sao Trung thành:** Logic bên trong hàm `analyze()` và các hàm con của nó sẽ được port 1:1 từ file `market_analyzer.py` của Dymate để đảm bảo tính tương thích và tuân thủ nguyên tắc "port trước, cải tiến sau".
*   **An toàn Dữ liệu:** Việc chỉ nhận các tham chiếu (`&`) đảm bảo `MarketAnalyzer` không thể vô tình thay đổi dữ liệu của `IndicatorCache` hay các thành phần khác, tuân thủ nguyên tắc an toàn của Rust.