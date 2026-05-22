# TODO List cho Dự án Helios

Tài liệu này ghi lại các bối cảnh, ràng buộc và các tác vụ cấp cao cần được thực hiện trong quá trình phát triển Helios.

---

### **Phase 0: Chuẩn bị Nền tảng (Pre-Development)**

-   [x] **Hoàn thiện `ferro_trade`:**
    -   **Bối cảnh:** `ferro_trade` là bản port Rust của `backtesting.py` nhưng chưa hoàn thiện.
    -   **Hành động:** Cần xác định và port các tính năng còn thiếu của `backtesting.py` mà Helios sẽ cần đến. Ưu tiên các tính năng liên quan đến quản lý lệnh, commission, và thống kê.

### **Phase 1: Thiết kế Kiến trúc Helios (Architecture Design)**

-   [x] **Thiết kế Kiến trúc Tổng thể:**
    -   **Bối cảnh:** Đây là một bản cải tiến của Dymate, cần được thiết kế lại từ đầu trước khi viết code.
    -   **Hành động:** Xây dựng một bản thiết kế chi tiết, tập trung vào Rust idioms, clean code, và cấu trúc module hóa. (Đang thực hiện).

### **Phase 2: Phát triển Helios (Development)**

-   [ ] **Review & Tái triển khai Logic Cốt lõi:**
    -   **Bối cảnh:** Logic của Dymate (chiến lược, market state) chưa được kiểm định đầy đủ và có thể chứa lỗi.
    -   **Hành động:** **Không port một cách mù quáng.** Trước khi implement từng phần, cần review lại logic tương ứng trong Dymate, đặt câu hỏi về tính hợp lý và sửa chữa nếu cần.

-   [ ] **Port các Module "Sạch" từ Dymate:**
    -   **Bối cảnh:** Một số phần của Dymate có logic tương đối độc lập và vững chắc.
    *   **Hành động:** Xác định các thành phần này (ví dụ: các hàm tính toán chỉ báo trong `indicator_functions.py`, logic trong `RiskManager`) và ưu tiên chuyển đổi chúng sang Rust để tăng tốc độ phát triển.

-   [ ] **Tập trung vào Rust Idioms:**
    *   **Bối cảnh:** Đây là một dự án Rust, cần tận dụng tối đa sức mạnh của ngôn ngữ.
    *   **Hành động:** Trong suốt quá trình code, luôn áp dụng các nguyên tắc của Rust: ownership và borrowing, xử lý lỗi với `Result` và `Option`, sử dụng `Traits` để tạo sự trừu tượng, và `Enums` mạnh mẽ để định nghĩa trạng thái.

- [x] **Phát triển Cỗ máy Tối ưu hóa "Prospector":**

    *   **Bối cảnh:** Thiết kế chi tiết cho cỗ máy tối ưu hóa đã được hoàn thành trong `helios_optimizer_design.md`. Thư viện `genetic_algorithm` đã được chọn làm nền tảng.

    *   **Hành động:** Bắt đầu implement cỗ máy này bên trong `ferro_trade` (cụ thể là `src/optimizer/genetic.rs`).

    *   **Kế hoạch Chi tiết:**

        -   [x] **Cài đặt:** Thêm `genetic-algorithm = "0.26.0"` vào `ferro_trade/Cargo.toml`.

        -   [x] **Định nghĩa Cấu trúc:** Trong `ferro_trade/src/optimizer/genetic.rs`, định nghĩa các `struct` cần thiết: `Job`, `HotspotQueue`, `BacklogQueue`, `Champion`, `Prospector`, `GAConfig`.

        -   [x] **Tạo Genotype:** Đã tạo hàm helper `create_genotype_from_job` để chuyển đổi `Job` thành `MultiRangeGenotype`.

        -   [x] **Implement `Fitness Trait`:** Đã tạo `struct BacktestFitness` và implement `trait Fitness` để chạy backtest và chấm điểm cho `chromosome`.

        -   [x] **Implement Vòng lặp Chính `Backtest::optimize()`:** Đã implement `Prospector::run` và `Prospector::run_job` để chạy một phiên GA hoàn chỉnh cho một `Job` duy nhất.

        -   [ ] **(ĐANG LÀM) Implement Logic Phân tích & Tạo Job:** Viết logic để phân tích kết quả GA, cập nhật `Champions List`, và tạo các `Job` con để đưa vào `Hotspot` và `Backlog Queue`.

        -   [ ] **Tối ưu hóa Hiệu năng (Global Cache & Pre-Resample):**
            -   **Vấn đề:** Hiện tại mỗi lần chạy backtest, strategy lại tự gọi `resample` gây lãng phí CPU. Cache indicator thì tốn RAM.
            -   **Giải pháp:** Trong `Prospector`, trước khi chạy GA, quét tất cả tham số Timeframe (vd: `["5m", "15m"]`). Thực hiện Resample OHLCV **một lần duy nhất** và lưu vào `GlobalCache`. Inject cache này vào `Backtest` để Strategy sử dụng lại.

        -   [ ] **Tích hợp vào `lib.rs`:** Thêm `pub mod optimizer;` vào `ferro_trade/src/lib.rs` và re-export các thành phần cần thiết.

        -   [ ] **Tạo Ví dụ:** Tạo một file trong `ferro_trade/examples` để kiểm thử cỗ máy tối ưu hóa.

        -   [ ] **Hệ thống Lưu trữ & Phục hồi (Save/Checkpoint System):**
            -   **Bối cảnh:** Dymate dùng `pickle` để lưu trạng thái quần thể (population) và logbook sau mỗi thế hệ để có thể chạy tiếp nếu bị ngắt quãng.
            -   **Hành động:** Implement Serialization/Deserialization cho `Prospector` state trong Rust để có thể resume process.

        -   [ ] **Hệ thống Thông báo (Notification System):**
            -   **Bối cảnh:** Dymate gửi thông báo Discord khi bắt đầu, kết thúc thế hệ, và khi tìm thấy ứng viên tốt.
            -   **Hành động:** Implement Discord Webhook client trong `ferro_trade` (hoặc module tiện ích) để gửi report trạng thái.
---

---

### **Phase 3: Lập Kế hoạch Porting Chi tiết (Implementation Planning)**
*Trạng thái: ✅ = Hoàn thành, 🟡 = Đang thực hiện, ⬜ = Chưa bắt đầu*

#### **Module 0: `Types` & `Config` (Nền tảng)**
- [✅] Port toàn bộ enums (`MarketRegime`, `StrategyName`...) và structs DTO (`MarketStateInfo`...) sang `src/types.rs`.
- [⬜] **Dynamic Config Parsing (QUAN TRỌNG):**
    - Hiện tại `HeliosStrategyConfig` đang hardcode giá trị.
    - Cần viết logic map từ `HashMap<String, f64>` (từ Optimizer) vào struct config.

#### **Module 1: `IndicatorCache` (Lớp Dữ liệu)**
- [⏸️] **TẠM DỪNG:** Chờ `ferro_trade` hoàn thiện cơ chế Cache & Resample tập trung. Helios sẽ chỉ đóng vai trò consumer.

#### **Module 2: `MarketAnalyzer` (Lớp Phân tích)**
- [✅] Cấu trúc cơ bản và phân tích các yếu tố cơ bản (Trend, Volatility).
- [⬜] **Logic Phân tích Pha (Phasing):**
    - Hoàn thiện `_analyze_strong_move_phase`.
    - Port logic xác định `Climax` (dựa trên Percentile & ATR multiplier).
    - Port logic xác định `Reversal` (Engulfing pattern).
- [⬜] **Logic Phân kỳ (Divergence):**
    - Cải thiện hàm `_find_peaks` trong Rust (thêm tham số `distance` để lọc nhiễu giống `scipy`).
    - Hoàn thiện `_analyze_divergence`.
- [⬜] **Percentile:** Kiểm tra độ chính xác của `quantile_as_series` so với `np.percentile`.
- [⬜] Port logic `_determine_macro_regime`.
- [⬜] Port logic `_synthesize_and_evaluate` (Confidence Score).

#### **Module 3: `StrategyEngine` (Lớp Quyết định)**
- [✅] Tạo `struct StrategyEngine` và `StrategyEngineConfig`.
- [✅] Port các `Playbook` (Trending, Ranging, Squeeze).
- [✅] Port **toàn bộ** các hàm `_check_*_trigger` cho các chiến lược vào lệnh.
- [⬜] **Review Logic Triggers:**
    - Rà soát lại toàn bộ phép so sánh số học (float comparison) trong các hàm `_check_*_trigger`.
    - Kiểm tra kỹ logic `TrendInitiation` (Zero-cross MACD: so sánh nến hiện tại vs nến trước).
    - Kiểm tra logic `The Fade` (dựa trên trạng thái `Reversal` từ MarketAnalyzer).
    - Port logic `Anomaly Handling` (phủ quyết tín hiệu nếu `is_anomaly_event` = true).

#### **Module 4: `RiskManager` (Lớp Rủi ro)**
- [✅] Tạo `struct RiskManager` và `RiskManagerConfig`.
- [✅] Port logic `calculate_execution_order` để tính toán lệnh mới.
- [✅] Port logic `manage_open_position` để quản lý Trailing Stop Loss.
- [⬜] **Anchor Price Logic:**
    - Review hàm `_get_anchor_price`. Đảm bảo slicing (`tail(lookback)`) lấy đúng cửa sổ nến như Python (bao gồm nến tín hiệu).
- [⬜] **Pyramiding & Order Modification:**
    - Implement logic tạo `UpdateOrderCommand` để dời SL về hòa vốn (Breakeven) cho các lệnh cũ khi nhồi lệnh mới.

#### **Module 5: `HeliosOrchestrator` (Lớp Điều phối)**
- [✅] Tạo `struct HeliosOrchestrator` và `HeliosConfig`.
- [✅] Implement hàm `step()` để điều phối các module theo đúng trình tự.
- [✅] Tích hợp logic gọi `manage_open_position` cho các vị thế đang mở.
- [⬜] **Position Synchronization:**
    - Cần cơ chế đồng bộ `open_positions` từ `ferro_trade` (thực tế khớp lệnh) vào `helios` (trạng thái chiến lược) đầu mỗi nến để tránh lệch trạng thái.

---

### **Phase 4: Hoàn thiện Cấu trúc & Tích hợp (Structuring & Integration)**

- [ ] **Xây dựng hệ thống Config an toàn và linh hoạt:**
    - **Bối cảnh:** `dymate` sử dụng nhiều `dictionary` Python để quản lý hàng trăm tham số, vốn không an toàn về kiểu dữ liệu.
    - **Hành động (Kiến trúc đã thống nhất):**
        1.  **Tạo các `struct` Config:** Định nghĩa các `struct` có kiểu dữ liệu rõ ràng cho từng module (`IndicatorConfig`, `AnalyzerConfig`, `RiskManagerConfig`, v.v.).
        2.  **Tạo `struct` tổng `HeliosConfig`:** Gộp tất cả các `struct` con vào một `struct` duy nhất để quản lý tập trung.
        3.  **Sử dụng `serde` để Deserialize:** Viết một file cấu hình duy nhất (ví dụ `helios_config.toml`) và dùng `serde` để đọc file này vào `struct HeliosConfig` khi chương trình khởi động.
        4.  **Luồng dữ liệu:** `main` -> `HeliosConfig` object -> `HeliosOrchestrator` -> các module con.
