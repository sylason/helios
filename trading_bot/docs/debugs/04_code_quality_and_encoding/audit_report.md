# BÁO CÁO KIỂM THỬ MÃ NGUỒN VÀ CHẤT LƯỢNG MÃ NGUỒN (AUDIT REPORT)
**Dự án:** `trading_bot/bot_engine`  
**Thư mục kiểm thử:** `trading_bot/bot_engine/src` & `trading_bot/bot_engine/examples`  
**Ngày thực hiện:** 24/08/2026  
**Đơn vị thực hiện:** Research Subagent  
**Đường dẫn lưu file đề xuất:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\debugs\04_code_quality_and_encoding\audit_report.md`

---

## I. TỔNG QUAN VÀ MỤC TIÊU KIỂM THỬ (AUDIT OVERVIEW)

Tiến hành kiểm thử toàn bộ mã nguồn Rust thuộc động cơ bot (`trading_bot/bot_engine`) bao gồm 31 file trong `src/` và 7 file trong `examples/` (tổng cộng 38 file `.rs`).

### Phạm vi kiểm thử chi tiết:
1. **Lỗi mã hóa ký tự (Mojibake / Broken ANSI/UTF-8 Encodings):** Phát hiện các comment/chuỗi ký tự tiếng Việt bị lỗi encode double-UTF8 hoặc Windows-1252/ISO-8859-1 (ví dụ `Cháº¡y láº¡i...`, `á»ž`, `Ä‘Ã£`).
2. **Cảnh báo trình biên dịch & Code Smells:** Tìm kiếm tất cả unused imports, unused variables (`_connection`, `_weight_monitor`), unused struct fields (`monitor`, `market_filter`), dead code (module/hàm không được sử dụng).
3. **Chất lượng Documentation & Comments:** Đánh giá việc thiếu docstrings (`///`, `//!`), comment sơ sài, thiếu giải thích tham số hàm.
4. **Lỗi Kiến trúc & Vi phạm DRY (Don't Repeat Yourself):** Phát hiện định nghĩa trùng lặp (`BotError` tại 2 file khác nhau), trùng lặp hàm tiện ích (`candles_to_df`, `to_i64`/`to_f64`).

---

## II. KẾT QUẢ KIỂM THỬ CHI TIẾT THEO HẠNG MỤC

### 1. Lỗi Mã hóa Ký tự (Mojibake / Broken UTF-8)

Phát hiện lỗi mã hóa Mojibake UTF-8 (chuỗi tiếng Việt UTF-8 bị lưu đè bằng ANSI/ISO-8859-1 rồi lưu lại UTF-8) tại **`trading_bot/bot_engine/src/optimizer/helios_bridge.rs`**:

*   **Dòng 324:**
    *   *Hiện tại:* `/// Chuyá»ƒn Ä‘á»•i danh sÃ¡ch Candle (shared model) sang DataFrame (ferro_trade format)`
    *   *Khắc phục:* `/// Chuyển đổi danh sách Candle (shared model) sang DataFrame (ferro_trade format)`
*   **Dòng 361:**
    *   *Hiện tại:* `/// [DIAGNOSTIC] Cháº¡y láº¡i toÃ n bá»™ pipeline Backtest giá»‘ng há»‡t nhÆ° optimizer/v2_run_helios`
    *   *Khắc phục:* `/// [DIAGNOSTIC] Chạy lại toàn bộ pipeline Backtest giống hệt như optimizer/v2_run_helios`
*   **Dòng 362:**
    *   *Hiện tại:* `/// Sá»­ dá»¥ng bá»™ DNA Ä‘Ã£ tá»‘i Æ°u vÃ  `BacktestBuilder` cá»§a `ferro_trade` Ä‘á»ƒ Ä‘áº£m báº£o káº¿t quáº£ 1:1.`
    *   *Khắc phục:* `/// Sử dụng bộ DNA đã tối ưu và `BacktestBuilder` của `ferro_trade` để đảm bảo kết quả 1:1.`

*(Các file khác trong `src/` và `examples/` đều đạt chuẩn UTF-8 Tiếng Việt).*

---

### 2. Cảnh báo Trình biên dịch & Code Smells

#### A. Unused Fields (Trường Struct không được sử dụng)
*   **`src/controller/asset_actor.rs`**:
    *   Trường `monitor: Arc<WeightMonitor>` (Dòng 36): Khai báo và khởi tạo trong `AssetActor::new()`, nhưng **không bao giờ được đọc hay truy cập** ở bất kỳ hàm nào trong file.
    *   Trường `market_filter: crate::models::market_info::MarketFilter` (Dòng 46): Khai báo và khởi tạo, nhưng **không bao giờ được đọc** trong logic của `AssetActor`.

#### B. Unused Variables & Unused Imports
*   **`src/main.rs`**:
    *   `use std::collections::HashMap;` (Dòng 18): Import không sử dụng (Dòng 187 dùng `std::collections::HashMap::new()` trực tiếp).
    *   `use ferro_trade::strategy::Strategy;` (Dòng 20): Import thừa không dùng.
    *   `let account_tx = event_bus.account_tx.clone();` (Dòng 143): Biến bị shadowing / khai báo trùng lặp lại ở Dòng 195.
*   **`src/execution/ws_manager.rs`**:
    *   `use ... websocket_api::{NewOrderParams, NewOrderSideEnum};` (Dòng 12): Import thừa không sử dụng.
    *   `let weight_monitor = self.weight_monitor.clone();` (Dòng 160): Khai báo biến `weight_monitor` trong `start_user_data_stream()` nhưng **không được sử dụng** bên trong closure async.
*   **`src/optimizer/helios_bridge.rs`**:
    *   Tham số `_symbol: &str` (Dòng 17) trong hàm `analyze()` không được dùng.

#### C. Dead Code & Unused Modules (Mã chết / Module dư thừa)
*   **`src/execution/rate_limit_monitor.rs`**:
    *   Module `pub mod rate_limit_monitor;` khai báo tại `src/execution/mod.rs`, nhưng `WeightMonitor` thực sự được bot sử dụng lại nằm ở `src/models/weight_monitor.rs`. File này hoàn toàn là dead code / mã thừa bị trùng lặp chức năng.
*   **`src/data_engine/duckdb_impl.rs`**:
    *   File tồn tại trong đĩa nhưng **không được khai báo** trong `src/data_engine/mod.rs` (`pub mod duckdb_impl;` bị thiếu). Do đó toàn bộ file bị trình biên dịch bỏ qua (Dead code).
*   **`src/interface/mod.rs` & `src/strategy_runner/mod.rs`**:
    *   Hai file rỗng (0 bytes), không chứa mã nguồn hoặc submodule.
*   **`src/controller/asset_actor.rs`**:
    *   Hàm `check_and_update_dna(&mut self)` (Dòng 325) không được gọi nội bộ trong `run()`.

---

### 3. Lỗi Kiến trúc & Vi phạm DRY (Don't Repeat Yourself)

#### A. Trùng lặp Enum `BotError` (Duplicate Type Def Flaw)
*   **Tồn tại 2 enum `BotError` độc lập và khác nhau:**
    1.  `src/error/mod.rs`: Khai báo `BotError` gồm `ApiError`, `NetworkError`, `MarginError`, `MarketFilterError`, `ParseError`, `StateError`, `Internal`. (Dùng bởi `account_manager.rs`, `order_executor.rs`, `asset_actor.rs`).
    2.  `src/models/error.rs`: Khai báo `BotError` gồm `DatabaseError`, `NetworkError`, `StrategyError`, `Unauthorized`, `Validation`. (Dùng bởi `sqlite_impl.rs`, `duckdb_impl.rs`, `repository.rs`, `market_info.rs`).
*   **Hậu quả:** Gây nhập nhằng namespace error, bắt buộc phải ép kiểu lỗi thủ công hoặc dùng string error khi giao tiếp giữa Data Engine và Execution Engine.

#### B. Trùng lặp Hàm Tiện ích (Utility Duplication)
*   Hàm `candles_to_df(candles: &[Candle]) -> PolarsResult<DataFrame>` bị viết lặp lại 100% tại cả `src/optimizer/helios_bridge.rs` (dòng 325) và `src/optimizer/prospector_runner.rs` (dòng 288).
*   Hàm closure bóc tách JSON `to_i64` và `to_f64` trong `src/data_engine/binance_rest.rs` bị viết lặp lại 2 lần giữa `fill_gap()` và `ensure_history()`.

---

### 4. Đánh giá Chất lượng Documentation & Comments

*   **Thiếu Module-level Docstrings (`//!`):** Tất cả các file `mod.rs` (`src/lib.rs`, `src/controller/mod.rs`, `src/data_engine/mod.rs`, `src/execution/mod.rs`, `src/models/mod.rs`, `src/optimizer/mod.rs`) thiếu mô tả tổng quan về kiến trúc và nhiệm vụ module.
*   **Thiếu Docstrings Tham số (`///`):** Hầu hết các struct công khai (`AssetActor`, `PaperTrader`, `PaperTradeRecord`, `LivePositionState`, `WsManager`, `AccountManager`) và các hàm chính (`new`, `execute_action`, `initialize_account`) chỉ có 1 dòng ghi chú tóm tắt, thiếu mô tả chi tiết các tham số truyền vào (`@param` / `# Arguments`).

---

## III. BẢNG TỔNG HỢP KIỂM THỬ THEO TỪNG FILE (FILE-BY-FILE AUDIT TABLE)

| STT | Tên File | Lỗi Encoding | Compiler Warnings / Dead Code | Quality / Documentation |
|---|---|---|---|---|
| **1** | `src/lib.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu `//!` Crate-level docs |
| **2** | `src/main.rs` | 🟢 Không | 🔴 Unused imports (`HashMap`, `Strategy`), Duplicate `let account_tx` | 🟡 Thiếu `///` docs cho `main()` |
| **3** | `src/controller/mod.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu `//!` docs |
| **4** | `src/controller/asset_actor.rs` | 🟢 Không | 🔴 Unused fields (`self.monitor`, `self.market_filter`), Dead method `check_and_update_dna` | 🟡 Thiếu docs tham số struct & `new` |
| **5** | `src/data_engine/mod.rs` | 🟢 Không | 🔴 Thiếu `pub mod duckdb_impl;` | 🟡 Thiếu `//!` docs |
| **6** | `src/data_engine/binance_rest.rs` | 🟢 Không | 🟡 Duplicate closures (`to_i64`/`to_f64`) | 🟡 Thiếu docs struct `InitialSync` |
| **7** | `src/data_engine/binance_stream.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu docs tham số |
| **8** | `src/data_engine/duckdb_impl.rs` | 🟢 Không | 🔴 Dead Code (Chưa được mod trong `data_engine/mod.rs`) | 🟡 Thiếu docs tham số |
| **9** | `src/data_engine/sqlite_impl.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu docs chi tiết `save_trade_history` |
| **10** | `src/error/mod.rs` | 🟢 Không | 🔴 Trùng lặp enum `BotError` với `models/error.rs` | 🟡 Thiếu `//!` docs |
| **11** | `src/execution/mod.rs` | 🟢 Không | 🔴 Mod `rate_limit_monitor` bị dư thừa | 🟡 Thiếu `//!` docs |
| **12** | `src/execution/account_manager.rs` | 🟢 Không | 🟡 Redundant closure `.map_err(\|e\| BotError::from(e))` | 🟡 Thiếu docs tham số `initialize_account` |
| **13** | `src/execution/order_executor.rs` | 🟢 Không | 🟡 Redundant closure `.map_err(\|e\| BotError::from(e))` | 🟡 Thiếu docs `format_price_decimal`, `format_qty_decimal` |
| **14** | `src/execution/paper_trader.rs` | 🟢 Không | 🟢 Không | 🟡 Struct `PaperTrader` & `PaperTradeRecord` thiếu field docs |
| **15** | `src/execution/rate_limit_monitor.rs` | 🟢 Không | 🔴 Dead Code (Module thừa, đã bị thay thế bởi `models/weight_monitor.rs`) | 🟡 Thiếu `//!` docs |
| **16** | `src/execution/ws_manager.rs` | 🟢 Không | 🔴 Unused imports (`NewOrderParams`, `NewOrderSideEnum`), Unused `weight_monitor` in line 160 | 🟡 Thiếu docs `parse_interval` |
| **17** | `src/interface/mod.rs` | 🟢 Không | 🔴 File rỗng (0 bytes) | 🔴 Thiếu nội dung & docs |
| **18** | `src/models/mod.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu `//!` docs |
| **19** | `src/models/account_state.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu field docs |
| **20** | `src/models/bus.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu field docs cho `EventBus` |
| **21** | `src/models/config.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu docs cho `ExecutionMode` variants |
| **22** | `src/models/error.rs` | 🟢 Không | 🔴 Trùng lặp enum `BotError` với `error/mod.rs` | 🟡 Thiếu docs variants |
| **23** | `src/models/market_info.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu docs tham số `validate_order` |
| **24** | `src/models/models.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu field docs cho `Candle`, `Asset` |
| **25** | `src/models/repository.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt (Có trait docs) |
| **26** | `src/models/weight_monitor.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt (Có struct & method docs) |
| **27** | `src/optimizer/mod.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu `//!` docs |
| **28** | `src/optimizer/context.rs` | 🟢 Không | 🟢 Không | 🟡 Thiếu method docs |
| **29** | `src/optimizer/helios_bridge.rs` | 🔴 Mojibake L324, L361, L362 | 🔴 Unused param `_symbol` in `analyze()` | 🟡 Docs bị lỗi UTF-8 |
| **30** | `src/optimizer/prospector_runner.rs` | 🟢 Không | 🟡 Duplicate `candles_to_df` với `helios_bridge.rs` | 🟡 Thiếu docs tham số |
| **31** | `src/strategy_runner/mod.rs` | 🟢 Không | 🔴 File rỗng (0 bytes) | 🔴 Thiếu nội dung & docs |
| **32** | `examples/binance_execution_test.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt |
| **33** | `examples/live_demo_runner.rs` | 🟢 Không | 🔴 Unused imports & duplicate `account_tx` (giống `main.rs`) | 🟡 Thiếu docs |
| **34** | `examples/live_demo_runner_non_opt.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt |
| **35** | `examples/paper_forward_test.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt |
| **36** | `examples/resilience_recovery_test.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt |
| **37** | `examples/walk_forward_evaluator.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt |
| **38** | `examples/walk_forward_smart_seed.rs` | 🟢 Không | 🟢 Không | 🟢 Tốt |

---

## IV. ĐỀ XUẤT GIẢI PHÁP VÀ KẾ HOẠCH KHẮC PHÚC (REMEDIATION PLAN)

### Step 1: Sửa triệt để lỗi Mojibake (Encoding Fix)
*   Sửa 3 dòng comment bị lỗi UTF-8 trong `src/optimizer/helios_bridge.rs` (Dòng 324, 361, 362) về chuẩn tiếng Việt UTF-8.

### Step 2: Hợp nhất Enum `BotError` (Refactoring Single Source of Truth)
*   Xóa file `src/models/error.rs`.
*   Bổ sung biến thể `DatabaseError`, `StrategyError`, `Unauthorized`, `Validation` vào `src/error/mod.rs`.
*   Cập nhật tất cả `use crate::models::error::BotError;` thành `use crate::error::BotError;`.

### Step 3: Dọn dẹp Unused Imports, Unused Fields & Dead Code
*   **Xóa file Dead Code:** `src/execution/rate_limit_monitor.rs`.
*   **Xóa các file rỗng không dùng:** `src/interface/mod.rs`, `src/strategy_runner/mod.rs` (hoặc bổ sung mã nguồn nếu thuộc lộ trình tương lai).
*   **Khai báo `duckdb_impl`:** Thêm `pub mod duckdb_impl;` vào `src/data_engine/mod.rs` nếu dùng, hoặc xóa file nếu chỉ dùng SQLite.
*   **Xóa unused fields:**
    *   Trong `AssetActor`: Loại bỏ `monitor` và `market_filter` nếu không dùng đến, hoặc đọc chúng khi tính toán / báo cáo.
    *   Trong `main.rs`: Xóa `use HashMap;`, `use Strategy;`, xóa dòng `let account_tx` trùng lặp ở dòng 143.
    *   Trong `ws_manager.rs`: Xóa `let weight_monitor = ...` ở dòng 160.

### Step 4: Refactor DRY (Tái sử dụng mã nguồn)
*   Chuyển hàm `candles_to_df` thành một public helper function nằm trong `src/models/models.rs` (ví dụ `impl Candle { pub fn vec_to_df(candles: &[Candle]) -> PolarsResult<DataFrame> }`), giúp `helios_bridge.rs` và `prospector_runner.rs` dùng chung.

---
*Báo cáo đã được tổng hợp đầy đủ và sẵn sàng để lưu vào `E:\Projects\Automations\trading_backtest\trading_bot\docs\debugs\04_code_quality_and_encoding\audit_report.md`.*
