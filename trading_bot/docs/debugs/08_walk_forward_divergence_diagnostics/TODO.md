# Master TODO: Bộ Chẩn Đoán Phân Kỳ Walk-Forward (Probes 1, 3, 4)

Quản lý tiến độ triển khai bộ chẩn đoán hợp nhất phân kỳ giữa Single Backtest và Paper Trading.

---

## 📋 Danh Sách Hạng Mục Triển Khai (Baby Steps)

- [x] **Bước 1: Triển khai Đầu Dò 4 (DNA Hot-Swap & Matrix Guard)**
  - [x] Thêm log telemetry trong `HeliosStrategyV3::update_from_dna` (in các tham số rủi ro then chốt của DNA mới).
  - [x] Đảm bảo `self.matrix` được cập nhật đồng bộ khi nhận DNA mới.

- [x] **Bước 2: Triển khai Đầu Dò 3 (Stagnant Decay & Cooldown Telemetry)**
  - [x] Thêm log telemetry trong `RiskManager::manage_trailing_stop` (in chi tiết `decay_cycles`, `effective_stagnant_bars`, `time_countdown_bars`, `current_pnl` trước khi `tick.close(None)`).
  - [x] Thêm log telemetry trong `HeliosStrategyV3::next` khi Cooldown được thiết lập (`locked_until_bar`), khi Cooldown chặn lệnh, và khi phá ngưỡng bứt phá.
  - [x] Khớp nối telemetry với `PaperTrader::close_position`.

- [x] **Bước 3: Triển khai Đầu Dò 1 (4h Checkpoint Comparison)**
  - [x] Đồng bộ `BACKTEST_INTERVAL_MINS = 240` khớp nhịp với `REOPT_INTERVAL_MINS = 240` trong `walk_forward_smart_seed.rs`.
  - [x] Cấu hình khoảng thời gian chạy kiểm thử tối ưu: `TOTAL_FETCH_DAYS = 34` (30 ngày Train + 4 ngày OOS Paper Trade test).

- [x] **Bước 4: Kiểm Tra Compile & Build**
  - [x] Chạy `cargo check` trên cả `helios` và `bot_engine` (Đã pass 100%).

- [ ] **Bước 5: Bàn Giao & Hướng Dẫn Chạy Thử Nghiệm**
  - [ ] Hướng dẫn người dùng chạy lệnh `cargo run --release --example walk_forward_smart_seed` và cung cấp lệnh git commit tương ứng.
