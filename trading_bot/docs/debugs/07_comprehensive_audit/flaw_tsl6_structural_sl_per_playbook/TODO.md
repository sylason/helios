# 📋 TODO: FLAW-TSL.6 - STRUCTURAL STOP LOSS PER-PLAYBOOK & DYNAMIC DNA EXPANSION

> Quản lý tiến độ triển khai theo nguyên tắc Baby Steps. Cập nhật trạng thái liên tục.

- [x] **Giai đoạn 1: Nâng cấp DNA Config trong Helios (`config.rs`)**
  - [x] Thêm gene `struct_sl_buffer_atr` vào `TacticalPullbackParams` (`[0.10 -> 4.00, step = 0.15]`, default `0.5`).
  - [x] Thêm gene `struct_sl_buffer_atr` vào `TacticalMomentumParams` (`[0.10 -> 3.00, step = 0.15]`, default `0.3`).
  - [x] Thêm gene `struct_sl_buffer_atr` vào `TacticalRangeParams` (`[0.10 -> 3.00, step = 0.15]`, default `0.3`).
  - [x] Thêm gene `struct_sl_buffer_atr` vào `TacticalAbsorptionParams` (`[0.25 -> 10.00, step = 0.25]`, default `2.0`).
  - [x] Định nghĩa các hàm `default_*` tương ứng trong `config.rs`.

- [x] **Giai đoạn 2: Triển khai Structural SL & Sanity Guard cho 4 Scanners**
  - [x] Cập nhật `trend_pb.rs`: Tính SL neo theo `min(Low, KAMA)` (Long) / `max(High, KAMA)` (Short) + Sanity Guard.
  - [x] Cập nhật `momentum_ignition.rs`: Tính SL neo theo `Low` (Long) / `High` (Short) + Sanity Guard.
  - [x] Cập nhật `range_osc.rs`: Tính SL neo theo `min(Low, BB_Lower)` (Long) / `max(High, BB_Upper)` (Short) + Sanity Guard.
  - [x] Cập nhật `absorption.rs`: Tính SL neo theo `min(Low, BB_Lower)` (Long) / `max(High, BB_Upper)` (Short) + Sanity Guard.

- [x] **Giai đoạn 3: Viết Unit Tests & Kiểm thử toàn diện**
  - [x] Cập nhật test `test_dna_width_and_parameter_defs_integrity`: Kiểm chứng `DNA_WIDTH = 117` và defs có đủ 4 gene mới.
  - [x] Viết bộ 4 unit tests kiểm chứng Structural SL của cả 4 Scanners không bao giờ trả về `suggested_sl: None` và luôn nằm đúng phía của giá.
  - [x] Chạy `cargo test --lib` cho cả `helios` và `bot_engine` đạt 100% Pass (32/32 helios + 34/34 bot_engine = 66/66 unit tests pass).
