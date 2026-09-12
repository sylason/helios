# 📋 TODO: FLAW-TSL.7 - UNIVERSAL BIDIRECTIONAL COOLDOWN & DIRECTIONAL EARLY RELEASE

> Quản lý tiến độ triển khai theo nguyên tắc Baby Steps. Cập nhật trạng thái liên tục.

- [x] **Giai đoạn 1: Nâng cấp DNA Config & CooldownTracker trong Helios (config.rs & mod.rs)**
  - [x] Thêm gene `cooldown_breakout_atr_mult` (`[0.5 -> 4.0, step = 0.25]`, default `1.5`) vào `RiskStopParams` trong `config.rs`.
  - [x] Nâng cấp Cooldown 2 chiều trong `mod.rs`: Khóa cả 2 chiều Long & Short khi lệnh trước bị cắt lỗ/hòa vốn danh nghĩa (`trade_pnl <= 0.001`).
  - [x] Triển khai Directional Early Release: Đánh thức sớm nếu Long bứt phá $\ge k \times ATR$ hoặc Short sập gãy $\ge k \times ATR$ ($k = \text{cooldown\_breakout\_atr\_mult}$).
  - [x] Cập nhật unit tests cooldown trong `mod.rs` và kiểm chứng `DNA_WIDTH = 113`.

- [x] **Giai đoạn 2: Tinh gọn AssetActor (trading_bot/bot_engine/src/controller/asset_actor.rs)**
  - [x] Gỡ bỏ struct `CooldownGuard` và trường `self.cooldown_guard` tự phát (15 bars).
  - [x] Giữ nguyên 100% chốt chặn an toàn hạ tầng: Cấm mở lệnh cùng cây nến vừa đóng vị thế (`closed_intra_candle`).

- [x] **Giai đoạn 3: Viết Unit Tests & Kiểm thử toàn diện**
  - [x] Viết test `test_bidirectional_cooldown_blocks_both_sides`: Khóa cả Long và Short sau khi thua trong vùng nhiễu.
  - [x] Viết test `test_directional_early_release_long_breakout`: Mở khóa Long khi giá tăng $\ge k \times ATR$.
  - [x] Viết test `test_directional_early_release_short_breakout`: Mở khóa Short khi giá giảm $\ge k \times ATR$.
  - [x] Chạy `cargo test --lib` cho cả `helios` và `bot_engine` đạt 100% Pass (31/31 helios, 34/34 bot_engine).
