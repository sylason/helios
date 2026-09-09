PS E:\Projects\Automations\trading_backtest\trading_bot\bot_engine> cargo run --release --example walk_forward_smart_seed
   Compiling helios v0.1.0 (E:\Projects\Automations\trading_backtest\strategies_backtests\helios)
   Compiling bot_engine v0.1.0 (E:\Projects\Automations\trading_backtest\trading_bot\bot_engine)
    Finished `release` profile [optimized] target(s) in 1m 48s
     Running `target\release\examples\walk_forward_smart_seed.exe`
2026-09-09T15:43:10.623342Z  INFO walk_forward_smart_seed: ================================================================================
2026-09-09T15:43:10.623656Z  INFO walk_forward_smart_seed: 🧪 BẮT ĐẦU ĐÁNH GIÁ CHIẾN LƯỢC TỊNH TIẾN CÓ SMART SEEDING 0.5%
2026-09-09T15:43:10.625916Z  INFO walk_forward_smart_seed:    Symbol: ETHUSDT, Strategy: walk_forward_smart_seed
2026-09-09T15:43:10.626083Z  INFO walk_forward_smart_seed:    Total Fetch: 21 days | Train Window: 14 days
2026-09-09T15:43:10.626208Z  INFO walk_forward_smart_seed:    📌 [N MINS] Tần suất Re-Optimize GA:       240 phút
2026-09-09T15:43:10.626318Z  INFO walk_forward_smart_seed:    📌 [M MINS] Tần suất Single Backtest Eval:  240 phút
2026-09-09T15:43:10.626434Z  INFO walk_forward_smart_seed:    💰 [ACCOUNT] Initial Cash: $60.00 | Leverage: 20x | Comm: 0.0004
2026-09-09T15:43:10.626592Z  INFO walk_forward_smart_seed:    🧠 [MEMORY] Max Live Candles: 43100 (Đảm bảo 50+ nến MTF 4h)
2026-09-09T15:43:10.626705Z  INFO walk_forward_smart_seed:    🌱 [SMART SEEDING]: Chiếm 0.5% quần thể (chỉ 1 Champion Rank 1), 99.5% cá thể mới
2026-09-09T15:43:10.626809Z  INFO walk_forward_smart_seed:    🎯 [PAPER TRADING]: BẬT (Active)
2026-09-09T15:43:10.626894Z  INFO walk_forward_smart_seed: ================================================================================
2026-09-09T15:43:10.788701Z  INFO walk_forward_smart_seed: 📥 [Data Ingestion] Đang đồng bộ dữ liệu 21 ngày từ Binance...
2026-09-09T15:43:17.252857Z  INFO walk_forward_smart_seed: 📊 [Data Summary] Tổng số nến: 30211 | Từ 2026-08-19 15:44 UTC -> Đến 2026-09-09 15:43 UTC
2026-09-09T15:43:17.253140Z  INFO walk_forward_smart_seed: --------------------------------------------------------------------------------
2026-09-09T15:43:17.253282Z  INFO walk_forward_smart_seed: 🧬 [INITIAL OPTIMIZE] Chạy GA Optimizer huấn luyện ban đầu trên 30 ngày...
2026-09-09T15:43:17.254116Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho ETHUSDT (Khoảng MS: 1787154240000 -> 1788363840000)...
2026-09-09T15:43:17.278393Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đã lấy 20142 nến từ Database. Bắt đầu tối ưu hóa...
2026-09-09T15:43:17.280672Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [WFE Split] In-Sample Train: 17262 nến | Out-Of-Sample Validation: 2880 nến (48h)
2026-09-09T15:43:17.321927Z  INFO bot_engine::optimizer::prospector_runner: 🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...
--- Prospector Helios v2.0 (Dynamic Dispatcher) Starting ---
[Prospector] 🔍 Validating precomputed MTF data for Timeframes: ["5m", "15m", "1h", "4h"]
[Prospector] ✅ MTF Cache validation completed (100% Zero-Polars Runtime).
Workers: 6

thread '<unnamed>' (12208) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\backtest\context.rs:136:18:
[FATAL] MTF OHLCV for 15s not found in cache. Did you forget to precompute?
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread '<unnamed>' (9044) panicked at E:\Projects\Automations\trading_backtest\ferro_trade\src\backtest\context.rs:136:18:
[FATAL] MTF OHLCV for 15s not found in cache. Did you forget to precompute?
error: process didn't exit successfully: `target\release\examples\walk_forward_smart_seed.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)

---

## 🔍 PHÂN TÍCH NGUYÊN NHÂN & GIẢI PHÁP (2026-09-09)

### 1. Nguyên Nhân Gốc Rễ:
- **Proc-Macro `DnaMappableMacro`:** Trong `ferro_trade/ferro_trade_derive/src/lib.rs:72, 91`, macro tự động xem bất kỳ trường nào có type chứa `"Timeframe"` là `is_enum = true` và `is_mappable = true` ngay cả khi không có thuộc tính `#[dna]`.
- **Fallback duyệt toàn bộ Timeframe:** Khi trường `timeframe` trong `MacdParams` và `VolMaParams` bị xóa `#[dna(subset = "5m, 15m, 1h, 4h")]`, macro tự động sinh code lấy toàn bộ variants của `Timeframe` (từ `1s`, `5s`, `15s`, `1m`...). Khi GA sinh số rơi vào index 2 (`T15s`), trường `timeframe` bị gán thành 15 giây.
- **`IndicatorsConfig` vẫn còn `#[dna(nest)]`:** Hai trường `macd` và `vol_ma` vẫn bị kéo vào vector DNA.
- **Tính toán thừa:** Trong `core_v3/mod.rs:80, 150`, `volume_sma` và `macd` được tính toán với timeframe 15s. Vì `GlobalCache` chỉ precompute `["5m", "15m", "1h", "4h"]`, `context.rs:136` ném panic fatal.
- Cả `vol_sma` và `macd_*` hoàn toàn không có bất kỳ Scanner hay Matrix nào trong V3 đọc đến.

### 2. Giải Pháp Triển Khai:
- **Cắt bỏ `#[dna(nest)]`:** Trong `IndicatorsConfig`, xóa `#[dna(nest)]` khỏi `macd: MacdParams` và `vol_ma: VolMaParams`.
- **Loại bỏ `DnaMappableMacro`:** Xóa derive `DnaMappableMacro` khỏi `MacdParams` và `VolMaParams`.
- **Triệt tiêu tính toán thừa:** Trong `core_v3/mod.rs`, loại bỏ các lệnh gọi `ctx.volume_sma` và `ctx.macd`, gán vector rỗng cho `s.vol_sma` và `s.macd_*`.
- **Chuẩn hóa DNA:** Số chiều DNA của `HeliosConfigV3` giảm từ 108 xuống chuẩn **106 chiều**.