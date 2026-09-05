# Master TODO: Bộ Chẩn Đoán Phân Kỳ & Tối Ưu Tốc Độ Walk-Forward

Quản lý tiến độ triển khai bộ chẩn đoán phân kỳ và cấu hình siêu tốc cho Walk-Forward Smart Seeding.

---

## 📋 Danh Sách Hạng Mục Triển Khai (Baby Steps)

### Giai Đoạn 1: Bộ Chẩn Đoán Phân Kỳ (Đã Hoàn Thành)
- [x] **Bước 1: Triển khai Đầu Dò 4 (DNA Hot-Swap & Matrix Guard)**
  - [x] Thêm log telemetry trong `HeliosStrategyV3::update_from_dna` (in các tham số rủi ro then chốt của DNA mới).
  - [x] Đảm bảo `self.matrix` được cập nhật đồng bộ khi nhận DNA mới.
- [x] **Bước 2: Triển khai Đầu Dò 3 (Stagnant Decay & Cooldown Telemetry)**
  - [x] Thêm log telemetry trong `RiskManager::manage_trailing_stop`.
  - [x] Thêm log telemetry trong `HeliosStrategyV3::next` khi Cooldown được thiết lập và khi chặn lệnh.
  - [x] Khớp nối telemetry với `PaperTrader::close_position`.
- [x] **Bước 3: Triển khai Đầu Dò 1 (4h Checkpoint Comparison)**
  - [x] Đồng bộ `BACKTEST_INTERVAL_MINS = 240` khớp nhịp với `REOPT_INTERVAL_MINS = 240`.
  - [x] Dọn sạch triệt để lỗi "vỡ trận log GA" trong vòng lặp đánh giá.

---

### Giai Đoạn 2: Cấu Hình Siêu Tốc & Tham Số Hóa Early Termination (Đã Hoàn Thành)
- [x] **Bước 4: Tham số hóa `EarlyTermination` sang JSON Config**
  - [x] Định nghĩa struct `EarlyTerminationConfig` trong `prospector_runner.rs` (default `max_drawdown_pct = 35.0`, `warmup_bars = 5760`, `min_trades = 2`).
  - [x] Thêm trường `pub early_termination: Option<EarlyTerminationConfig>` vào `FullOptimizerConfig`.
  - [x] Thay thế đoạn hardcode trong `run_optimization_cycle_range_internal` bằng cấu hình được nạp từ JSON.
  - [x] Khai báo khối `"early_termination"` trong `helios_optimizer_config.json`.

- [x] **Bước 5: Tinh Chỉnh Cấu Hình Tăng Tốc GA & Rút Ngắn Cửa Sổ Train**
  - [x] Chỉnh `max_depth: 1` trong `helios_optimizer_config.json`.
  - [x] Chỉnh `population_size: 40`, `n_generations: 8` trong `helios_optimizer_config.json`.
  - [x] Chỉnh `TRAIN_WINDOW_DAYS = 14` và `TOTAL_FETCH_DAYS = 18` trong `walk_forward_smart_seed.rs`.

- [x] **Bước 6: Chuẩn Hóa Telemetry Held Bars & Trailing SL trong `PaperTrader`**
  - [x] Sửa `bars_held` trong `Action::ClosePosition` dùng độ lệch timestamp thay vì phép trừ bar index của rolling buffer.
  - [x] Bổ sung log `info!` cho sự kiện nâng mức `[TRAILING SL UPDATE]` khi có thay đổi đáng kể.

- [x] **Bước 7: Biên Dịch, Chạy Kiểm Thử & Bàn Giao**
  - [x] Chạy `cargo check` trên cả `bot_engine` và `helios` (Đã pass 100%, không cảnh báo).
  - [x] Chạy `cargo test` cho deserialization và paper trader (Đã pass 100%).
  - [x] Cung cấp lệnh chạy test siêu tốc mới và câu lệnh commit thủ công cho người dùng.
