# BÁO CÁO KIỂM TOÁN KỸ THUẬT: DECOUPLING VÀ CẤU HÌNH TRONG BOT_ENGINE

**Dự án:** `trading_bot/bot_engine`  
**Ngày kiểm toán:** 24/08/2026  
**Phạm vi kiểm toán:**  
- `src/optimizer/helios_bridge.rs`  
- `src/optimizer/prospector_runner.rs`  
- `src/models/config.rs`  
- `examples/paper_forward_test.rs`  
**File lưu trữ báo cáo đề xuất:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\debugs\01_decoupling_and_config\audit_report.md`

---

## 1. TỔNG QUAN VÀ TÓM TẮT HÀNH ĐỘNG (EXECUTIVE SUMMARY)

Qua quá trình rà soát và kiểm toán chuyên sâu mã nguồn `bot_engine`, phát hiện hệ thống đang gặp các vấn đề nghiêm trọng về **kiến trúc cấu hình (configuration architecture)** và **độ thắt chặt (tight coupling)**:

1. **Hằng số bị Hardcode phân tán:** Các thông số giao dịch cốt lõi (`INITIAL_CASH = 60.0`, `COMMISSION = 0.0004`, `MARGIN = 1/20`, `FRACTIONAL_UNIT = 0.0001`, `equity = 10000.0`, `cash = 10000.0`, v.v.) bị gán trực tiếp tại nhiều vị trí khác nhau trong các file `prospector_runner.rs`, `helios_bridge.rs` và `paper_forward_test.rs`.
2. **Đường dẫn File bị Hardcode:** Rất nhiều đường dẫn file dữ liệu (`.db`), file log (`.log`), và file cấu hình JSON (`helios_optimizer_config.json`) bị viết cứng, trong đó có cả đường dẫn tuyệt đối dành riêng cho hệ điều hành Windows (`E:\Projects\Automations\...`).
3. **Phụ thuộc chặt vào Chiến thuật `helios`:** `bot_engine` bị phụ thuộc cứng (tightly coupled) vào struct chiến thuật cụ thể `HeliosStrategyV3` và `HeliosConfigV3` thay vì sử dụng cơ chế trừu tượng (trait abstraction/factory) để chạy bất kỳ chiến thuật nào.
4. **Thiếu struct `EngineConfig` hợp nhất:** File `src/models/config.rs` hiện chỉ chứa `AppConfig` cơ bản (đọc API Key Binance từ `.env`), thiếu hoàn toàn cấu trúc quản lý tập trung cho Broker, Optimizer, Storage và Strategy.

---

## 2. KIỂM TOÁN CHI TIẾT HẰNG SỐ HARDCODE (HARDCODED CONSTANTS AUDIT)

### 2.1. Bảng tổng hợp Hằng số Hardcode

| File | Dòng | Giá trị Hardcode | Mục đích sử dụng | Rủi ro & Tác động |
| :--- | :--- | :--- | :--- | :--- |
| `prospector_runner.rs` | L20-L23 | `INITIAL_CASH = 60.0`<br>`COMMISSION = 0.0004`<br>`MARGIN = 1/20`<br>`FRACTIONAL_UNIT = 0.0001` | Khai báo `pub const` chung cho Optimizer | Không thể thay đổi mức vốn/phí giao dịch theo từng cặp tiền nếu không sửa code và biên dịch lại. |
| `prospector_runner.rs` | L44, L59 | `30 * 24 * 60 * 60 * 1000` | Cửa sổ huấn luyện 30 ngày hardcode trong hàm | Giới hạn linh hoạt khi muốn test cửa sổ 7, 14, 60 ngày. |
| `prospector_runner.rs` | L128 | `candles.len() < 5000` | Số lượng nến tối thiểu để chạy GA | Giá trị ma thuật (magic number), không tùy chỉnh được theo khung thời gian. |
| `prospector_runner.rs` | L152-L155 | `INITIAL_CASH`, `COMMISSION`, `MARGIN`, `FRACTIONAL_UNIT` | Cấu hình `BacktestBuilder` trong GA | Lặp lại khai báo hằng số thay vì truyền qua Config Struct. |
| `prospector_runner.rs` | L173-L188 | `MaxDrawdown(40.0)`, `after_bars: 14400`, `min_trades: 5`, `-5000.0`, `-2000.0` | Ngưỡng cắt tỉa sớm (Early Termination) & Phạt fitness GA | Các tham số thuật toán di truyền bị cố định cứng, không thể tinh chỉnh từ file JSON. |
| `prospector_runner.rs` | L215-L230 | `total_days = 30.0`, `trades_per_day < 0.5`, `dd > 40.0`, `-5000.0`, `-50.0` | Hàm mục tiêu GA (Objective Function) | Trói buộc logic đánh giá GA vào quy tắc hardcode. |
| `helios_bridge.rs` | L45-L49 | `["3m", "5m", ..., "1d"]` | Danh sách Timeframe chuẩn bị MTF | Trùng lặp danh sách Timeframe với ProspectorConfig. |
| `helios_bridge.rs` | L58 | `10000.0` | Initial Cash trong Broker giả lập lấy signal | Khác biệt bất hợp lý với `INITIAL_CASH = 60.0` ở ProspectorRunner. |
| `helios_bridge.rs` | L61 | `Box::new(\|_ , _\| 0.0)` | Phí commission 0.0 cho simulation | Không phản ánh chi phí thực tế khi đo PnL/signal. |
| `helios_bridge.rs` | L253 | `full_len < 200` | Số nến tối thiểu cho `evaluate_live_tick` | Magic number hardcode. |
| `helios_bridge.rs` | L292-L293 | `sl_order_id = Some(1)`, `tp_order_id = Some(2)` | Order ID giả lập cho SL/TP | Gắn ID cố định gây rủi ro xung đột order. |
| `helios_bridge.rs` | L311 | `equity: 10000.0` | Vốn giả định truyền vào `TickContext` | Bỏ qua số dư vốn thực tế của tài khoản/PaperTrader! |
| `helios_bridge.rs` | L388-L391 | `cash(60.0)`, `commission(0.0004)`, `margin(1/20)`, `fractional_unit(0.0001)` | Cấu hình Backtest trong `diagnostic_backtest` | Giá trị ma thuật trùng lặp với `prospector_runner.rs` (Vi phạm DRY). |
| `paper_forward_test.rs` | L34-L35 | `SYMBOL = "ETHUSDT"`, `STRATEGY_ID = "paper_forward_v3"` | Cấu hình cặp tiền & ID | Viết cứng hằng số đầu file example. |
| `paper_forward_test.rs` | L38-L48 | `TOTAL_FETCH_DAYS = 77`, `TRAIN_WINDOW_DAYS = 62`, `STEP_INTERVAL_MINS = 240`, `INITIAL_CAPITAL = 60.0`, `LEVERAGE = 20.0`, `MAX_LIVE_MEM_CANDLES = 86400` | Tham số mô phỏng Paper Forward | Khai báo hằng số riêng biệt thay vì nạp từ cấu hình tập trung. |
| `paper_forward_test.rs` | L76 | `WeightMonitor::new(2400)` | Giới hạn trọng số Binance REST API | Magic number `2400` hardcode. |

---

## 3. KIỂM TOÁN CHI TIẾT ĐƯỜNG DẪN FILE HARDCODE (HARDCODED FILE PATHS AUDIT)

### 3.1. Bảng tổng hợp Đường dẫn Hardcode

| File | Dòng | Đường dẫn Hardcode | Phân loại | Rủi ro & Tác động |
| :--- | :--- | :--- | :--- | :--- |
| `paper_forward_test.rs` | L53-L55 | `"examples/logs"`, `{}.log` | Đường dẫn tương đối log | Lỗi Runtime nếu thư mục chạy lệnh không phải là root của workspace. |
| `paper_forward_test.rs` | L73-L74 | `"examples/storage"`, `examples/storage/{}.db` | Đường dẫn Database chính | Gắn chặt vị trí lưu DB vào thư mục `examples`. |
| `paper_forward_test.rs` | L115 | `examples/storage/{}_prospector.db` | Prospector DB ban đầu | Cố định mẫu tên file DB. |
| `paper_forward_test.rs` | L120-L125 | `examples/helios_optimizer_config.json`<br>`r#"E:\Projects\Automations\trading_backtest\strategies_backtests\helios\examples\helios_optimizer_config.json"#` | File cấu hình Optimizer JSON | **RỰC RỠ VỀ RỦI RO PORTABILITY:** Đường dẫn tuyệt đối Windows `E:\...` khiến code lập tức gãy khi chạy trên Linux server, Docker container hoặc máy phát triển khác! |
| `paper_forward_test.rs` | L247 | `examples/storage/{}_prospector_step_{}.db` | Prospector DB theo từng bước | Tạo ra hàng loạt file DB phân tán không được quản lý tự động cleanup. |
| `helios_bridge.rs` | L371 | `ferro_backlog.db` (trong comment/log) | IO lock file backlog | Nguy cơ đụng độ IO lock file giữa các luồng. |

---

## 4. PHÂN TÍCH THẮT CHẶT KIẾN TRÚC & PHỤ THUỘC MÔ-ĐUN (ARCHITECTURAL DECOUPLING ANALYSIS)

### 4.1. Phụ thuộc chặt chẽ giữa `bot_engine` và Chiến thuật `helios`
* **Vấn đề:** Trong `prospector_runner.rs` và `helios_bridge.rs`, `bot_engine` trực tiếp import và gọi:
  ```rust
  use helios::strategy::core_v3::HeliosStrategyV3;
  // ...
  let flat_search_space = helios::strategy::core_v3::config::HeliosConfigV3::parameter_defs();
  prospector.run::<HeliosStrategyV3, _>(backtest, objective);
  ```
* **Tác động:** `bot_engine` biến thành một engine **chỉ chạy được cho Helios Strategy V3**. Khi muốn thử nghiệm chiến thuật mới (ví dụ `AegisStrategy` hay `AlphaTrendStrategy`), ta bắt buộc phải sửa mã nguồn Rust của `prospector_runner.rs` và `helios_bridge.rs`. Điều này vi phạm nghiêm trọng **Open/Closed Principle (OCP)** trong SOLID.

### 4.2. Phân tán Cấu hình và Thiếu struct `EngineConfig` Tập trung
* **Hiện trạng của `src/models/config.rs`:**
  ```rust
  pub struct AppConfig {
      pub api_key: String,
      pub api_secret: String,
      pub use_testnet: bool,
      pub opt_num_threads: usize,
  }
  ```
* **Đánh giá:** `AppConfig` quá đơn sơ, chỉ đóng vai trò nạp môi trường kết nối Binance API. Tất cả thông số quản trị hệ thống khác bị xé lẻ ra 4 nơi:
  1. `.env` (API Keys, Testnet)
  2. `helios_optimizer_config.json` (Cấu hình GA Prospector)
  3. `pub const` trong `prospector_runner.rs` (Vốn, phí, margin)
  4. `const` trong `paper_forward_test.rs` (Cửa sổ time, leverage, sliding window)

---

## 5. GIẢI PHÁP REFACTORING TOÀN DIỆN & THIẾT KẾ CẤU TRÚC MỚI (REFACTORING BLUEPRINT)

Để giải quyết triệt để 4 nhóm vấn đề trên, đề xuất giải pháp Refactoring theo mô hình **Baby Steps**, đảm bảo không phá vỡ logic giao dịch hiện tại:

### 5.1. Bổ sung `EngineConfig` Hợp nhất trong `src/models/config.rs`

Tạo cấu trúc cấu hình phân tầng đầy đủ (Hierarchical Configuration Schema):

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    pub initial_cash: f64,
    pub commission: f64,
    pub margin: f64,
    pub fractional_unit: Option<f64>,
    pub leverage: f64,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            initial_cash: 60.0,
            commission: 0.0004,
            margin: 1.0 / 20.0,
            fractional_unit: Some(0.0001),
            leverage: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_dir: PathBuf,
    pub db_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("."),
            db_dir: PathBuf::from("storage"),
            logs_dir: PathBuf::from("logs"),
            config_dir: PathBuf::from("config"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub app: AppConfig,
    pub broker: BrokerConfig,
    pub storage: StorageConfig,
    pub execution_mode: ExecutionMode,
}
```

### 5.2. Giải quyết Đường dẫn File Hardcode qua `PathResolver`

Xây dựng module utility giải quyết đường dẫn động, loại bỏ hoàn toàn các đường dẫn tuyệt đối Windows (`E:\...`):

```rust
pub struct PathResolver;

impl PathResolver {
    pub fn resolve_config_path(filename: &str) -> PathBuf {
        let candidates = vec![
            PathBuf::from(filename),
            PathBuf::from("config").join(filename),
            PathBuf::from("examples").join(filename),
            PathBuf::from("../config").join(filename),
        ];

        for path in candidates {
            if path.exists() {
                return path;
            }
        }
        PathBuf::from(filename)
    }
}
```

### 5.3. Trừu tượng hóa Chiến thuật (Strategy Decoupling / Strategy Factory)

Chuyển `ProspectorRunner` và `HeliosBridge` sang dạng Generic hoặc Trait-based execution:

1. Đổi tên `HeliosBridge` thành `StrategyBridge` tổng quát.
2. `ProspectorRunner::run_optimization_cycle` nhận tham số `BrokerConfig` và `SearchSpace` thay vì gọi hằng số cứng và `HeliosConfigV3`.

---

## 6. KẾT LUẬN & ĐỀ XUẤT BƯỚC TIẾP THEO

Báo cáo chi tiết kiểm toán này đã chỉ ra chính xác từng dòng code vi phạm quy chuẩn thiết kế. Người dùng có thể tiến hành phê duyệt kế hoạch Refactoring theo thứ tự:
1. **Bước 1:** Tạo file `audit_report.md` tại `trading_bot/docs/debugs/01_decoupling_and_config/audit_report.md`.
2. **Bước 2:** Cập nhật `src/models/config.rs` để định nghĩa `EngineConfig` và `BrokerConfig`.
3. **Bước 3:** Refactor `prospector_runner.rs` và `helios_bridge.rs` để tiếp nhận `BrokerConfig`.
4. **Bước 4:** Refactor `examples/paper_forward_test.rs` để sử dụng `EngineConfig` thống nhất.
