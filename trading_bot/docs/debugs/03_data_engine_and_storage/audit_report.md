# 📑 BÁO CÁO KIỂM THỬ VÀ KIỂM TRUYÊN SÂU (AUDIT REPORT)
## MODULE DỮ LIỆU VÀ LƯU TRỮ (DATA ENGINE & STORAGE LAYER)

**Dự án:** `trading_bot` (`bot_engine`)  
**Thư mục kiểm tra:** `src/data_engine/` & `src/models/`  
**Ngày thực hiện:** 24/08/2026  
**Người thực hiện:** Research Subagent  

---

### 🎯 1. TỔNG QUAN VÀ MỤC TIÊU AUDIT
Báo cáo kiểm thử rà soát toàn bộ kiến trúc và mã nguồn của 5 file cốt lõi trong hệ thống dữ liệu:
1. `src/data_engine/binance_rest.rs`
2. `src/data_engine/binance_stream.rs`
3. `src/data_engine/sqlite_impl.rs`
4. `src/data_engine/duckdb_impl.rs`
5. `src/models/market_info.rs`
*(Cùng các file đối chiếu liên quan: `src/models/models.rs` và `src/controller/asset_actor.rs`)*

---

### 🔍 2. CHI TIẾT CÁC PHÁT HIỆN LỖI VÀ HẠN CHẾ (AUDIT FINDINGS)

---

#### 🔴 PHÂN ĐOẠN 1: HẰNG SỐ CỨNG, KẾT NỐI API VÀ CẤU TRÚC LƯU TRỮ (HARDCODED ENDPOINTS, BATCHES & SQL)

##### 1.1. `binance_rest.rs`
* 💥 **Lỗi 1.1.1: Ép cứng Môi trường Production REST Client (Hardcoded Production REST)**
  - **Vị trí:** Dòng 27: `DerivativesTradingUsdsFuturesRestApi::production(rest_conf)`
  - **Tác hại:** REST client luôn kết nối đến Binance Mainnet (`fapi.binance.com`). Khi cấu hình `AppConfig.use_testnet = true`, REST API vẫn gọi lên Mainnet, gây sai lệch dữ liệu thử nghiệm và tiềm ẩn rủi ro đặt lệnh/đọc dữ liệu nhầm môi trường.
* 💥 **Lỗi 1.1.2: Hardcoded Batch Size & Magic Numbers trong Fetch Nến**
  - **Vị trí:** Dòng 60, 185: `.limit(1000)`
  - **Vị trí:** Dòng 174: `let start_time = current_end_time - (1000 * 60 * 1000);` (Lùi 1000 phút tính bằng ms).
  - **Vị trí:** Dòng 143, 165: `target_days * 24 * 60` và `target_days * 24 * 60 * 60 * 1000` (Ép cứng 1440 nến/ngày và `86_400_000` ms/ngày).
  - **Vị trí:** Dòng 130, 159, 250: Magic number `60000` ms (bước nhảy 1 phút).
  - **Vị trí:** Dòng 53, 177, 258: Magic numbers weight cost `wait_if_needed(5)` và `wait_if_needed(1)`.
* 💥 **Lỗi 1.1.3: Thiếu Timeout & Retry Exponential Backoff**
  - Các hàm gọi API như `kline_candlestick_data` và `exchange_information` thực thi trực tiếp qua `.await` mà không có wrapper timeout (`tokio::time::timeout`) hay Retry khi gặp sự cố mạng (HTTP 5xx / 429).
* 💥 **Lỗi 1.1.4: Vi phạm nguyên tắc Dependency Inversion (Tight Coupling)**
  - **Vị trí:** Dòng 20: `db: Arc<SqliteStore>` phụ thuộc trực tiếp vào struct cụ thể `SqliteStore` thay vì abstraction Trait `Arc<dyn CandleRepository>`. Điều này khiến `InitialSync` không thể hoạt động linh hoạt với `DuckDbStore` hoặc Mock DB trong Unit Tests.

##### 1.2. `binance_stream.rs`
* 💥 **Lỗi 1.2.1: Ép cứng Môi trường Production WS Client (Hardcoded Production Stream)**
  - **Vị trí:** Dòng 16: `DerivativesTradingUsdsFuturesWsStreams::production(ws_conf)` ép cứng Mainnet WebSocket (`fstream.binance.com`).
* 💥 **Lỗi 1.2.2: Phụ thuộc cứng khung nến 1m & Thiếu Auto-Reconnect**
  - **Vị trí:** Dòng 30: `KlineCandlestickStreamsIntervalEnum::Interval1m` ép cứng nến 1m.
  - **Vị trí:** Dòng 70-72:
    ```rust
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
    ```
  - **Tác hại:** Callback `stream.on_message` không có cơ chế phát hiện socket chết (Heartbeat/Ping-Pong timeout). Khi mất kết nối mạng, task rơi vào vòng lặp sleep 3600s vô hạn mà không hề tự động tái kết nối (Auto-reconnect).
* 💥 **Lỗi 1.2.3: Panic & Implict Data Corruption**
  - **Vị trí:** Dòng 31: `.expect("Không thể mở Kline Stream")` gây panic sập app nếu đứt luồng đăng ký.
  - **Vị trí:** Dòng 45: `unwrap_or(0.0)` âm thầm chuyển dữ liệu chuỗi parse lỗi thành `0.0`, tạo ra các nến rác `close = 0.0`.

##### 1.3. `sqlite_impl.rs` & `duckdb_impl.rs`
* 💥 **Lỗi 1.3.1: Ép cứng Tên Bảng & SQL Query Literals**
  - Tên bảng `candles_1m` được viết cứng trong tất cả các câu lệnh SQL. Nếu chuyển đổi khung thời gian (5m, 1h), bảng vẫn giữ tên `candles_1m`.
  - Các câu SQL `INSERT OR REPLACE INTO candles_1m ...` lặp lại nhiều lần, vi phạm DRY.
* 💥 **Lỗi 1.3.2: Nghẽn Cổ Chai Hiệu Năng do Sync `Mutex<Connection>`**
  - *SQLite & DuckDB:* Sử dụng `Arc<Mutex<Connection>>`. Mỗi khi ghi batch nến hoặc đọc nến, async task sẽ khoá toàn bộ kết nối DB. Điều này ngăn cản tính song song (Concurrency) giữa Engine trade và Background sync/analytics.
  - *SQLite:* Chưa bật chế độ WAL (`PRAGMA journal_mode=WAL;`), khiến việc ghi lock toàn bộ file DB.
* 💥 **Lỗi 1.3.3: Bất Đồng Bộ Giữa Chi Tiết Schema SQLite và DuckDB**
  - `sqlite_impl.rs` có lưu trữ `trade_history` (`save_trade_history`), nhưng `duckdb_impl.rs` hoàn toàn KHÔNG triển khai bảng này.
* 💥 **Lỗi 1.3.4: Nuốt Lỗi Deserialization JSON (Data Corruption)**
  - *SQLite (dòng 274-278) & DuckDB (dòng 248-252):* Sử dụng `serde_json::from_str(...).unwrap_or_default()`. Nếu JSON bị lỗi trong DB, hệ thống im lặng trả về Struct rỗng thay vì báo lỗi `BotError::DatabaseError`.

---

#### 🟡 PHÂN ĐOẠN 2: ĐỘ CHÍNH XÁC VÀ METADATA SÀN (TICK SIZE, STEP SIZE & MARKET FILTERS)

##### 2.1. `market_info.rs`
* 💥 **Lỗi 2.1.1: Hằng Số Giá Trị Mặc Định Giả Định Sai Lầm (Magic Default Fallbacks)**
  - **Vị trí:** Dòng 14-20:
    ```rust
    tick_size: 0.1,
    step_size: 0.001,
    min_qty: 0.001,
    max_qty: 1000000.0,
    min_notional: 5.0,
    ```
  - **Tác hại:** Các tham số mặc định được thiết kế dành riêng cho BTCUSDT/ETHUSDT. Khi giao dịch các cặp altcoin/memecoin (ví dụ DOGEUSDT tick 0.00001, SHIBUSDT step 1.0, PEPEUSDT), nếu dùng `MarketFilter::default()`, lệnh sẽ bị làm tròn sai hoàn toàn hoặc bị sàn từ chối.
* 💥 **Lỗi 2.1.2: Hạn Chế Về Độ Chính Xác Chuỗi Thập Phân (`count_decimals`)**
  - **Vị trí:** Dòng 86: `let s = format!("{:.8}", val);`
  - **Tác hại:** Định dạng cứng 8 chữ số thập phân (`.8`). Nếu sàn có `tick_size = 0.000000001` (9 chữ số thập phân), hàm sẽ cắt cụt và trả về `0`, làm tròn sai số. Đồng thời việc format String liên tục gây tốn RAM và CPU cấp phát heap (Garbage/Allocation).
* 💥 **Lỗi 2.1.3: Nguy Cơ Sai Số Chấm Động IEEE-754 trong `round_price` & `round_qty`**
  - Việc dùng `(price / tick_size).round() * tick_size` trên kiểu `f64` vẫn có thể sinh ra sai số đại số chấm động (ví dụ `10.100000000000001`).

##### 2.2. Parsing Exchange Metadata trong `binance_rest.rs`
* 💥 **Lỗi 2.2.1: Nuốt Lỗi Parse metadata sàn (`fetch_market_filter`)**
  - **Vị trí:** Dòng 281, 286, 289, 292, 297: Sử dụng `.unwrap_or(...)` với các hằng số cứng `0.1`, `0.001`, `1000000.0`, `5.0`. Nếu Binance thay đổi format string hoặc lọc không khớp, hệ thống im lặng dùng giá trị mặc định thay vì báo lỗi.

---

#### 🔵 PHÂN ĐOẠN 3: BỘ NHỚ ĐỆM NẾN (BUFFER LIMITS) VÀ LOGIC GAP-FILLER

##### 3.1. Mất Đồng Nhất Hằng Số Giới Hạn Bộ Nhớ (Buffer Limit Inconsistency)
* **Xung đột nghiêm trọng giữa các struct:**
  - `src/models/models.rs` (dòng 76, 86, 100): `pub ohlcv: VecDeque<Candle>` có dung lượng cứng **`43200`** nến (~ 30 ngày 1m).
  - `src/controller/asset_actor.rs` (dòng 14, 57): `pub const MAX_LIVE_MEM_CANDLES: usize = 15000;`.
  - **Tác hại:** Hai hằng số độc lập cùng đại diện cho giới hạn bộ nhớ đệm gây ra sự không đồng nhất về quản lý tài nguyên bộ nhớ RAM và độ dài dữ liệu cung cấp cho chỉ báo chiến lược.

##### 3.2. Giả Định Cứng Khung Thời Gian 1 Phút (Tight Timeframe Coupling)
* Các con số `43200` (30 * 24 * 60) và `15000` (~ 10.4 ngày) đều được tính dựa trên khung 1m.
* Nếu bot cấu hình giao dịch trên khung nến khác (ví dụ: 5m, 15m, 1h):
  - Khung 5m với 43200 nến = 150 ngày dữ liệu trong RAM.
  - Khung 1s với 43200 nến = 12 giờ dữ liệu trong RAM.
* Cấu trúc `Asset` và `AssetActor` không nhận biết `Interval` của chiến lược, dẫn đến việc phình to RAM hoặc thiếu dữ liệu quá khứ.

##### 3.3. Logic Gap-Filler & Nguy Cơ Lọc Nến Trùng Lập (Deduplication Vulnerability)
* Trong `binance_rest.rs` (`fill_gap` & `ensure_history`):
  - Logic tính toán thời gian `current_start = last_timestamp + 60000` ép cứng bước nhảy 60000ms.
  - Nếu REST API trả về nến không đủ hoặc nến bị lệch mốc thời gian, `last_timestamp` có thể bị đứng yên, gây ra vòng lặp vô hạn.
* Trong `models.rs` (`push_candle`):
  - **Vị trí:** Dòng 95-97: `if candle.timestamp <= last.timestamp { return; }`
  - Logic này chỉ hoạt động đúng khi nến được push theo thứ tự thời gian tăng dần tuyệt đối (Strictly Ascending). Nếu trong quá trình lấp khoảng trống (Gap-Filler), dữ liệu nến cũ hơn được đẩy vào sau khi đã có nến realtime, các nến lấp bù sẽ bị DROP hoàn toàn!

---

### 🛠️ 4. ĐỀ XUẤT GIẢI PHÁP REFACTORING (CLEAN CODE & ARCHITECTURE)

#### 📐 Giải pháp 1: Cấu hình Tập trung (Centralized Config & Exchange Context)
Tạo module `src/config/data_config.rs` để lưu trữ toàn bộ tham số cấu hình:
- REST / WS Base URLs dynamically chọn theo `AppConfig.use_testnet`.
- Limit Batch Sizes (mặc định 1000, configurable).
- Timeouts & Retry settings.
- Memory Buffer Rules (tự động tính dựa trên `Days * CandlesPerDay(Timeframe)`).

#### 📐 Giải pháp 2: Dependency Inversion cho `InitialSync` & Data Engine
Chuyển `InitialSync` trong `binance_rest.rs` sang sử dụng Trait `Arc<dyn CandleRepository>` thay vì `Arc<SqliteStore>`.

#### 📐 Giải pháp 3: Nâng cấp SQLite & DuckDB Storage Layer
- Bật `WAL` mode cho SQLite: `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;`.
- Sử dụng SQL Query Templates hoặc Prepared Statement Caching.
- Bổ sung bảng `trade_history` cho DuckDB để đảm bảo tính đồng nhất 1:1 với SQLite.
- Xử lý triệt để lỗi Deserialization JSON (trả về `BotError` chi tiết thay vì `unwrap_or_default`).

#### 📐 Giải pháp 4: Cải thiện `MarketFilter` & Thuật Toán Làm Tròn (Precision Engine)
- Sử dụng toán học số học hoặc kiểu dữ liệu `rust_decimal::Decimal` trong `MarketFilter` để đảm bảo độ chính xác tuyệt đối không bị sai số chấm động IEEE-754.
- Không dùng giá trị `Default` ảo cho `MarketFilter`, bắt buộc fetch metadata từ Binance `exchangeInfo` trước khi khởi chạy lệnh.

#### 📐 Giải pháp 5: Chuẩn hóa Bộ Nhớ Đệm Nến & Logic Gap-Filler
- Định nghĩa `BufferConfig` nhận `Timeframe` (1m, 5m, 1h) và `HistoryDays` (ví dụ 30 ngày) để tự động tính toán `max_candles_capacity = days * 86400 / timeframe_seconds`.
- Loại bỏ hằng số trùng lặp `MAX_LIVE_MEM_CANDLES` và `43200`.
- Nâng cấp `Asset::push_candle` hoặc bổ sung hàm `merge_candles(Vec<Candle>)` có tính năng tự động sắp xếp (Sort by timestamp) và loại bỏ trùng lặp (Deduplicate) an toàn.
