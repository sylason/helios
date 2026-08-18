# Báo cáo Nghiên cứu Kỹ thuật: Binance Rust SDK v68.1.1 (Futures USD-M)

**File tài liệu mục tiêu:** `E:\Projects\Automations\trading_backtest\trading_bot\docs\tests\paper_forward_test\06_binance_sdk_v68_examples_research.md`  
**Dự án:** `trading_bot` / `ferro_trade`  
**Phiên bản Crate:** `binance-sdk v68.1.1` (`derivatives_trading_usds_futures`)

---

## 📌 Tổng quan mục tiêu
Nghiên cứu các mẫu mã nguồn (`examples/derivatives_trading_usds_futures`) trong SDK `binance-sdk v68.1.1` để chuyển đổi hệ thống `trading_bot` từ cách viết chuỗi thô (`String`) sang **Strongly-Typed Enums** và cấu trúc dữ liệu chuẩn mực hóa của SDK Rust Binance chính thức.

---

## 1. Khởi tạo & Parse dữ liệu K-Line REST API (`KlineCandlestickDataItemInner`)

### 1.1. Cấu trúc yêu cầu (Params Builder)
Trong REST API `kline_candlestick_data`, tham số đầu vào sử dụng `KlineCandlestickDataParams`:
```rust
use binance_sdk::derivatives_trading_usds_futures::rest_api::{
    KlineCandlestickDataIntervalEnum, KlineCandlestickDataParams,
};

let params = KlineCandlestickDataParams::builder(
    "BTCUSDT".to_string(),
    KlineCandlestickDataIntervalEnum::Interval1m,
)
.build()?;

let response = rest_client.kline_candlestick_data(params).await?;
let data = response.data().await?; // Kiểu dữ liệu: Vec<Vec<KlineCandlestickDataItemInner>>
```

### 1.2. Bản chất dữ liệu `KlineCandlestickDataItemInner`
Phản hồi từ REST API Binance trả về một mảng 2 chiều không đồng nhất (Heterogeneous 2D JSON Array):
`[[open_time, open, high, low, close, volume, close_time, quote_volume, trades_count, taker_buy_base, taker_buy_quote, ignore], ...]`

SDK tự động tạo kiểu untagged enum để đại diện cho từng phần tử:
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KlineCandlestickDataItemInner {
    Integer(i64),
    String(String),
    Other(serde_json::Value),
}
```

### 1.3. Thứ tự các chỉ số Mảng (Index Mapping) & Helper Parse
| Index | Tên thuộc tính | Kiểu dữ liệu trong Enum | Mục đích |
|---|---|---|---|
| `0` | `open_time` | `Integer(i64)` | Timestamp bắt đầu nến (ms) |
| `1` | `open` | `String(String)` | Giá mở cửa |
| `2` | `high` | `String(String)` | Giá cao nhất |
| `3` | `low` | `String(String)` | Giá thấp nhất |
| `4` | `close` | `String(String)` | Giá đóng cửa |
| `5` | `volume` | `String(String)` | Khối lượng base asset |
| `6` | `close_time` | `Integer(i64)` | Timestamp kết thúc nến (ms) |
| `7` | `quote_asset_volume` | `String(String)` | Khối lượng quote asset |
| `8` | `number_of_trades` | `Integer(i64)` | Số lượng giao dịch |
| `9` | `taker_buy_base_asset_volume` | `String(String)` | Taker buy base volume |
| `10` | `taker_buy_quote_asset_volume` | `String(String)` | Taker buy quote volume |
| `11` | `ignore` | `String(String)` | Trường dự phòng của Binance |

### 1.4. Mẫu mã nguồn Parse sang Struct Nến nội bộ
```rust
use binance_sdk::derivatives_trading_usds_futures::rest_api::models::KlineCandlestickDataItemInner;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct TradeCandle {
    pub open_time: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub close_time: i64,
    pub trades_count: i64,
}

pub fn parse_rest_kline_row(row: &[KlineCandlestickDataItemInner]) -> anyhow::Result<TradeCandle> {
    let extract_i64 = |idx: usize| -> anyhow::Result<i64> {
        match row.get(idx) {
            Some(KlineCandlestickDataItemInner::Integer(v)) => Ok(*v),
            Some(KlineCandlestickDataItemInner::String(s)) => Ok(s.parse::<i64>()?),
            _ => anyhow::bail!("Invalid i64 at index {}", idx),
        }
    };

    let extract_decimal = |idx: usize| -> anyhow::Result<Decimal> {
        match row.get(idx) {
            Some(KlineCandlestickDataItemInner::String(s)) => Ok(s.parse::<Decimal>()?),
            Some(KlineCandlestickDataItemInner::Integer(v)) => Ok(Decimal::from(*v)),
            _ => anyhow::bail!("Invalid Decimal at index {}", idx),
        }
    };

    Ok(TradeCandle {
        open_time: extract_i64(0)?,
        open: extract_decimal(1)?,
        high: extract_decimal(2)?,
        low: extract_decimal(3)?,
        close: extract_decimal(4)?,
        volume: extract_decimal(5)?,
        close_time: extract_i64(6)?,
        trades_count: extract_i64(8)?,
    })
}
```

---

## 2. Khởi tạo Lệnh thường với `NewOrderParams::builder`

### 2.1. Cấu trúc Builder & Tham số Bắt buộc
SDK quy định tham số bắt buộc ngay trong constructor của Builder (`symbol`, `side`, `r#type`):
```rust
use binance_sdk::derivatives_trading_usds_futures::rest_api::{
    NewOrderParams, NewOrderSideEnum, NewOrderTypeEnum,
};

let params = NewOrderParams::builder(
    "BTCUSDT".to_string(),
    NewOrderSideEnum::Buy,
    NewOrderTypeEnum::Limit,
)
.quantity(rust_decimal::dec!(0.01))
.price(rust_decimal::dec!(65000.0))
.time_in_force(binance_sdk::derivatives_trading_usds_futures::rest_api::NewOrderTimeInForceEnum::Gtc)
.build()?;

let response = rest_client.new_order(params).await?;
```

### 2.2. Chi tiết Danh sách Enums tương ứng
* **`NewOrderSideEnum`**: `Buy` (`"BUY"`), `Sell` (`"SELL"`)
* **`NewOrderTypeEnum`**:
  * `Limit` (`"LIMIT"`)
  * `Market` (`"MARKET"`)
  * `Stop` (`"STOP"`)
  * `StopMarket` (`"STOP_MARKET"`)
  * `TakeProfit` (`"TAKE_PROFIT"`)
  * `TakeProfitMarket` (`"TAKE_PROFIT_MARKET"`)
  * `TrailingStopMarket` (`"TRAILING_STOP_MARKET"`)
* **`NewOrderTimeInForceEnum`**: `Gtc`, `Ioc`, `Fok`, `Gtx`, `Gtd`, `Rpi`
* **`NewOrderReduceOnlyEnum`**: `True` (`"true"`), `False` (`"false"`)
* **`NewOrderNewOrderRespTypeEnum`**: `Ack`, `Result`, `Full`

> 💡 **Lưu ý:** Tất cả các tham số giá/số lượng (`price`, `quantity`) đều sử dụng `rust_decimal::Decimal` giúp chống sai số số thực trong giao dịch tài chính.

---

## 3. Khởi tạo Lệnh Điều kiện / Thuật toán với `NewAlgoOrderParams::builder`

### 3.1. Cấu trúc Builder & Tham số Bắt buộc
Lệnh Algo yêu cầu 4 tham số bắt buộc trong `NewAlgoOrderParams::builder`: `algo_type`, `symbol`, `side`, `r#type`.

```rust
use binance_sdk::derivatives_trading_usds_futures::rest_api::{
    NewAlgoOrderAlgoTypeEnum, NewAlgoOrderClosePositionEnum, NewAlgoOrderParams,
    NewAlgoOrderSideEnum, NewAlgoOrderTypeEnum, NewAlgoOrderWorkingTypeEnum,
};

let params = NewAlgoOrderParams::builder(
    NewAlgoOrderAlgoTypeEnum::Conditional,
    "BTCUSDT".to_string(),
    NewAlgoOrderSideEnum::Sell,
    NewAlgoOrderTypeEnum::StopMarket,
)
.trigger_price(rust_decimal::dec!(60000.0))
.close_position(NewAlgoOrderClosePositionEnum::True) // Đóng toàn bộ vị thế khi kích hoạt
.working_type(NewAlgoOrderWorkingTypeEnum::MarkPrice)
.build()?;

let response = rest_client.new_algo_order(params).await?;
```

### 3.2. Điểm đặc thù của `NewAlgoOrderClosePositionEnum`
Thay vì dùng boolean `true`/`false`, SDK định nghĩa enum chuẩn:
```rust
pub enum NewAlgoOrderClosePositionEnum {
    #[serde(rename = "true")]
    True,
    #[serde(rename = "false")]
    False,
}
```
Giúp loại bỏ hoàn toàn các lỗi sai kiểu dữ liệu truyền vào Binance API.

---

## 4. Đăng ký Stream Nến thời gian thực qua WebSocket (`KlineCandlestickStreamsParams`)

### 4.1. Cấu trúc Builder & Tham số Bắt buộc
Trong `websocket_streams`, `symbol` cần viết chữ thường (e.g. `"btcusdt"`) theo chuẩn Binance Stream:

```rust
use binance_sdk::config::ConfigurationWebsocketStreams;
use binance_sdk::derivatives_trading_usds_futures::{
    DerivativesTradingUsdsFuturesWsStreams,
    websocket_streams::{KlineCandlestickStreamsIntervalEnum, KlineCandlestickStreamsParams},
};

let ws_streams_conf = ConfigurationWebsocketStreams::builder().build()?;
let ws_client = DerivativesTradingUsdsFuturesWsStreams::production(ws_streams_conf);
let connection = ws_client.connect().await?;

let params = KlineCandlestickStreamsParams::builder(
    "btcusdt".to_string(),
    KlineCandlestickStreamsIntervalEnum::Interval1m,
)
.build()?;

let stream = connection.kline_candlestick_streams(params).await?;
stream.on_message(|data| {
    // data chứa struct `KlineCandlestickStreamsResponse`
    if let Some(kline) = data.k {
        let is_closed = kline.x.unwrap_or(false);
        let close_price = kline.c.as_deref().unwrap_or("0");
        tracing::info!(is_closed, close_price, "Nhận được nến WS");
    }
});
```

### 4.2. Danh mục Interval Enum (`KlineCandlestickStreamsIntervalEnum`)
* Nút khung nhỏ: `Interval1m` (`"1m"`), `Interval3m`, `Interval5m`, `Interval15m`, `Interval30m`
* Nút khung giờ: `Interval1h` (`"1h"`), `Interval2h`, `Interval4h`, `Interval6h`, `Interval8h`, `Interval12h`
* Nút khung ngày/tuần: `Interval1d` (`"1d"`), `Interval3d`, `Interval1w`, `Interval1M`

---

## 🎯 Kết luận & Khuyến nghị cho `trading_bot`

1. **Tuân thủ Type Safety 100%:** Loại bỏ việc tự format String thủ công (`"BUY"`, `"LIMIT"`, `"1m"`), chuyển sang dùng các Strongly-typed Enums của SDK để tránh runtime panic hoặc lỗi từ chối lệnh từ Binance server.
2. **Xử lý Nến REST API:** Sử dụng Helper function chuyển từ `Vec<KlineCandlestickDataItemInner>` sang `TradeCandle` với `rust_decimal::Decimal`.
3. **Đồng nhất API REST & WebSocket:** Cả REST API và WS API trong SDK v68.1.1 đều có chung tư duy thiết kế `Params::builder(...)`, rất thuận tiện cho việc mô-đun hóa trong `trading_bot`.
