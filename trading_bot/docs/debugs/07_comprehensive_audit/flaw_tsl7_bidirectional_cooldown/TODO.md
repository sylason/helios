# 📋 TODO: FLAW-TSL.7 - UNIVERSAL BIDIRECTIONAL COOLDOWN & DIRECTIONAL EARLY RELEASE

> Quản lý tiến độ triển khai theo nguyên tắc Baby Steps. Cập nhật trạng thái liên tục.

- [x] **Giai đoạn 1: Nâng cấp DNA Config & CooldownTracker trong Helios (config.rs & mod.rs)**
  - [x] Thêm gene `cooldown_breakout_atr_mult` (`[0.5 -> 4.0, step = 0.25]`, default `1.5`) vào `RiskStopParams` trong `config.rs`.
  - [x] Nâng cấp Cooldown 2 chiều trong `mod.rs`: Khóa cả 2 chiều Long & Short khi lệnh trước bị cắt lỗ/hòa vốn danh nghĩa (`trade_pnl <= 0.001`).
  - [x] Triển khai Directional Early Release: Đánh thức sớm nếu Long bứt phá $\ge k \times ATR$ hoặc Short sập gãy $\ge k \times ATR$ ($k = \text{cooldown\_breakout\_atr\_mult}$).
  - [x] Cập nhật unit tests cooldown trong `mod.rs` và kiểm chứng `DNA_WIDTH = 113`.

- [x] **Giai đoạn 2: Triển khai EngineCooldownGuard trong AssetActor (trading_bot/bot_engine/src/controller/asset_actor.rs)**
  - [x] Khởi tạo `EngineCooldownGuard` và tích hợp `HeliosConfigV3` vào `AssetActor`.
  - [x] Đồng bộ tham số Cooldown (`enable_re_entry_cooldown`, `re_entry_cooldown_bars`, `cooldown_breakout_atr_mult`) từ DNA sang `EngineCooldownGuard` khi khởi động hoặc Hot-swap DNA.
  - [x] Tự động kích hoạt khóa Cooldown 2 chiều khi phát hiện lệnh đóng lỗ hoặc hòa vốn danh nghĩa (`trade_pnl <= 0.001`).
  - [x] Veto toàn bộ Action `Buy`/`Sell` vi phạm thời gian cách ly hoặc chưa đạt ngưỡng Directional Early Release ($\ge k \times ATR$).

- [x] **Giai đoạn 3: Viết Unit Tests & Kiểm thử toàn diện**
  - [x] Unit tests trong `helios`: `test_bidirectional_cooldown_blocks_both_sides`, `test_directional_early_release_long_breakout`, `test_directional_early_release_short_breakout`.
  - [x] Viết unit tests trong `bot_engine` kiểm chứng `EngineCooldownGuard`:
    - Test khóa cả 2 chiều sau khi đóng lệnh lỗ (`test_engine_cooldown_guard_blocks_both_sides_in_chop`).
    - Test Veto action Long/Short khi giá biến động nhỏ trong vùng cách ly.
    - Test Early Release khi giá bứt phá vượt ngưỡng $\ge k \times ATR$ (`test_engine_cooldown_guard_directional_early_release`).
    - Test tính ATR nến trượt (`test_calculate_rolling_atr_basic`).
    - Test hết hạn và tắt cooldown (`test_engine_cooldown_guard_expired_and_disabled`).
  - [x] Chạy `cargo test --lib` cho cả `helios` và `bot_engine` đạt 100% Pass (32/32 helios + 38/38 bot_engine = 70/70 unit tests pass).
