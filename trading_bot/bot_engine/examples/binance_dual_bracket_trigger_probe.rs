// trading_bot/bot_engine/examples/binance_dual_bracket_trigger_probe.rs
//
// 🔬 BÀI TEST THỰC NGHIỆM CHUYÊN SÂU: VÒNG ĐỜI LỆNH KÉP SONG SONG & HÀNH VI SAU KHI CẮN LỆNH
// 
// Mục tiêu khoa học & kỹ thuật:
// 1. Treo đồng thời SL và TP (closePosition = true) qua WebSocket API.
// 2. Ép cắn SL bằng Dynamic Tightening Loop bám sát giá theo thời gian thực.
// 3. Đợi cho tới khi vị thế đóng về 0.000 ETH (Polling loop, không dùng sleep mù).
// 4. Khi vị thế = 0, TRÍCH XUẤT FULL RAW JSON PAYLOAD từ Binance:
//    - Trạng thái Algo SL đã cắn và ID lệnh Market thực tế (actualOrderId).
//    - Chi tiết lệnh Market thực tế đóng vị thế (query_order: avgPrice, executedQty, status).
//    - Trạng thái của lệnh TP còn lại trên hệ thống Algo (query_algo_order: CANCELED hay vẫn NEW?).
//    - Danh sách Algo Orders còn mở (current_all_algo_open_orders).
// 5. Xuất toàn bộ ra file JSON độc lập theo timestamp và file latest.json.

use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use anyhow::{Context, Result};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use binance_sdk::config::{ConfigurationWebsocketApi, ConfigurationRestApi, ConfigurationWebsocketStreams};
use binance_sdk::derivatives_trading_usds_futures::{
    DerivativesTradingUsdsFuturesWsApi, DerivativesTradingUsdsFuturesRestApi, DerivativesTradingUsdsFuturesWsStreams,
    websocket_api::{
        WebsocketApi,
        NewAlgoOrderAlgoTypeEnum, NewAlgoOrderParams, NewAlgoOrderSideEnum, NewAlgoOrderTypeEnum,
        NewAlgoOrderClosePositionEnum, NewAlgoOrderWorkingTypeEnum,
        CancelAlgoOrderParams,
        NewOrderParams, NewOrderSideEnum, NewOrderTypeEnum,
        PositionInformationV2Params,
    },
    websocket_streams::UserDataStreamEventsResponse,
    rest_api::{
        CancelAllAlgoOpenOrdersParams,
        CurrentAllAlgoOpenOrdersParams,
        QueryAlgoOrderParams,
        QueryOrderParams,
        SymbolPriceTickerParams,
    },
};

const SYMBOL: &str = "ETHUSDT";
const ORDER_QTY: f64 = 0.01; // 0.01 ETH (~$25 USDT)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataStreamEventAudit {
    pub received_at_iso: String,
    pub event_type: String,
    pub payload: Value,
}

fn format_price(price: f64) -> Decimal {
    Decimal::from_f64((price * 100.0).round() / 100.0).unwrap().round_dp(2)
}

fn format_qty(qty: f64) -> Decimal {
    Decimal::from_f64((qty * 1000.0).round() / 1000.0).unwrap().round_dp(3)
}

async fn get_current_price(rest_client: &binance_sdk::derivatives_trading_usds_futures::rest_api::RestApi, conn: &WebsocketApi) -> Result<f64> {
    let p_ticker = SymbolPriceTickerParams::builder()
        .symbol(SYMBOL.to_string())
        .build()
        .unwrap();
    if let Ok(resp) = rest_client.symbol_price_ticker(p_ticker).await {
        if let Ok(data) = resp.data().await {
            let val = serde_json::to_value(&data).unwrap_or_default();
            if let Some(p_str) = val.get("price").and_then(|v| v.as_str()) {
                if let Ok(p) = p_str.parse::<f64>() {
                    if p > 10.0 {
                        return Ok(p);
                    }
                }
            }
        }
    }

    let (_, entry) = get_position_amt(conn).await;
    if let Ok(p) = entry.parse::<f64>() {
        if p > 10.0 {
            return Ok(p);
        }
    }

    anyhow::bail!("Không thể lấy giá thị trường thực tế từ Binance")
}

async fn get_position_amt(conn: &WebsocketApi) -> (f64, String) {
    let p_pos = PositionInformationV2Params::builder().symbol(SYMBOL.to_string()).build().unwrap();
    if let Ok(resp) = conn.position_information_v2(p_pos).await {
        if let Ok(data) = resp.data() {
            for pos in data {
                if pos.symbol.as_deref() == Some(SYMBOL) {
                    let amt = pos.position_amt.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let entry = pos.entry_price.unwrap_or_else(|| "0.0".to_string());
                    return (amt, entry);
                }
            }
        }
    }
    (0.0, "0.0".to_string())
}

async fn close_current_position(conn: &WebsocketApi, amt: f64) {
    if amt.abs() < 1e-4 {
        return;
    }
    let side = if amt > 0.0 { NewOrderSideEnum::Sell } else { NewOrderSideEnum::Buy };
    let p_close = NewOrderParams::builder(
        SYMBOL.to_string(),
        side,
        NewOrderTypeEnum::Market,
    )
    .quantity(format_qty(amt.abs()))
    .build().unwrap();

    info!("🧹 Đang Market Close vị thế tồn đọng ({:.4} ETH)...", amt);
    let _ = conn.new_order(p_close).await;
    tokio::time::sleep(Duration::from_millis(600)).await;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DualBracketAuditReport {
    pub test_metadata: Value,
    pub phases: Value,
    pub scientific_conclusions: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    fs::create_dir_all("examples/logs/dual_bracket_runs")?;
    let run_id = format!("run_{}", Utc::now().format("%Y%m%d_%H%M%S"));
    let log_file_name = format!("examples/logs/dual_bracket_runs/{}.log", run_id);
    let file_appender = tracing_appender::rolling::never("examples/logs/dual_bracket_runs", format!("{}.log", run_id));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("================================================================================");
    info!("🔬 BẮT ĐẦU THỰC NGHIỆM CHUYÊN SÂU: DUAL BRACKET TRIGGER & POST-FILL AUDIT");
    info!("   Run ID: {} | Symbol: {} | Mode: Binance Futures Testnet", run_id, SYMBOL);
    info!("================================================================================");

    let api_key = env::var("BINANCE_TESTNET_API_KEY")
        .or_else(|_| env::var("BINANCE_API_KEY"))
        .unwrap_or_default();
    let api_secret = env::var("BINANCE_TESTNET_API_SECRET")
        .or_else(|_| env::var("BINANCE_TESTNET_SECRET_KEY"))
        .or_else(|_| env::var("BINANCE_API_SECRET"))
        .unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        error!("❌ Không tìm thấy API Key / Secret Testnet trong file .env!");
        return Ok(());
    }

    let rest_conf = ConfigurationRestApi::builder()
        .api_key(api_key.clone())
        .api_secret(api_secret.clone())
        .build()?;
    let rest_client = DerivativesTradingUsdsFuturesRestApi::testnet(rest_conf);

    let ws_api_conf = ConfigurationWebsocketApi::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;
    let ws_api_client = DerivativesTradingUsdsFuturesWsApi::testnet(ws_api_conf);

    info!("📡 Đang kết nối tới Binance Futures WebSocket API (Testnet)...");
    let connection = ws_api_client
        .connect()
        .await
        .context("Không thể kết nối tới Binance WebSocket API")?;
    info!("✅ Kết nối WebSocket API thành công!");

    // ============================================================================
    // 📡 KHỞI TẠO USER DATA STREAM (EVENT-DRIVEN MONITOR)
    // ============================================================================
    info!("🔑 Đang xin listenKey từ Binance REST API để mở User Data Stream...");
    let resp_listen_key = rest_client.start_user_data_stream().await?;
    let listen_key_data = resp_listen_key.data().await?;
    let listen_key = listen_key_data.listen_key.context("Không nhận được listenKey từ Binance")?;
    let short_key = if listen_key.len() > 12 { &listen_key[..12] } else { &listen_key };
    info!("✅ Nhận listenKey thành công: {}...", short_key);

    let ws_streams_conf = ConfigurationWebsocketStreams::builder().build()?;
    let ws_streams_handle = DerivativesTradingUsdsFuturesWsStreams::testnet(ws_streams_conf);
    let ws_streams = ws_streams_handle.connect().await.context("Không thể kết nối WebSocket Streams")?;
    let user_data_stream = ws_streams.user_data(listen_key.clone(), None).await.context("Không thể mở User Data Stream")?;
    info!("✅ Đã kết nối User Data Stream thành công! Bắt đầu lắng nghe sự kiện tài khoản.");

    let stream_events: Arc<Mutex<Vec<UserDataStreamEventAudit>>> = Arc::new(Mutex::new(Vec::new()));
    let stream_events_clone = stream_events.clone();

    user_data_stream.on_message(move |event: UserDataStreamEventsResponse| {
        let events_vec = stream_events_clone.clone();
        tokio::spawn(async move {
            let received_at = Utc::now().to_rfc3339();
            let (evt_type, val) = match &event {
                UserDataStreamEventsResponse::OrderTradeUpdate(b) => {
                    let v = serde_json::to_value(b).unwrap_or_default();
                    let status = v.get("o").and_then(|o| o.get("X")).and_then(|x| x.as_str()).unwrap_or("?");
                    let side = v.get("o").and_then(|o| o.get("S")).and_then(|s| s.as_str()).unwrap_or("?");
                    let order_id = v.get("o").and_then(|o| o.get("i")).and_then(|x| x.as_i64()).unwrap_or(0);
                    info!("📡 [UDS ORDER_TRADE_UPDATE]: OrderID: {}, Side: {}, Status: {}", order_id, side, status);
                    ("ORDER_TRADE_UPDATE".to_string(), v)
                },
                UserDataStreamEventsResponse::AccountUpdate(b) => {
                    let v = serde_json::to_value(b).unwrap_or_default();
                    info!("📡 [UDS ACCOUNT_UPDATE]: Số dư hoặc vị thế đã thay đổi!");
                    ("ACCOUNT_UPDATE".to_string(), v)
                },
                UserDataStreamEventsResponse::AlgoUpdate(b) => {
                    let v = serde_json::to_value(b).unwrap_or_default();
                    let algo_status = v.get("o").and_then(|o| o.get("X")).and_then(|x| x.as_str()).unwrap_or("?");
                    let algo_id = v.get("o").and_then(|o| o.get("si")).and_then(|x| x.as_i64()).unwrap_or(0);
                    let order_type = v.get("o").and_then(|o| o.get("ot")).and_then(|x| x.as_str()).unwrap_or("?");
                    info!("📡 [UDS ALGO_UPDATE]: AlgoID: {}, Type: {}, Status: {}", algo_id, order_type, algo_status);
                    ("ALGO_UPDATE".to_string(), v)
                },
                UserDataStreamEventsResponse::ConditionalOrderTriggerReject(b) => {
                    let v = serde_json::to_value(b).unwrap_or_default();
                    warn!("⚠️ [UDS CONDITIONAL_ORDER_TRIGGER_REJECT]: {:?}", v);
                    ("CONDITIONAL_ORDER_TRIGGER_REJECT".to_string(), v)
                },
                UserDataStreamEventsResponse::MarginCall(b) => {
                    let v = serde_json::to_value(b).unwrap_or_default();
                    warn!("🚨 [UDS MARGIN_CALL]: {:?}", v);
                    ("MARGIN_CALL".to_string(), v)
                },
                UserDataStreamEventsResponse::AccountConfigUpdate(b) => {
                    ("ACCOUNT_CONFIG_UPDATE".to_string(), serde_json::to_value(b).unwrap_or_default())
                },
                UserDataStreamEventsResponse::ListenKeyExpired(b) => {
                    warn!("⚠️ [UDS LISTEN_KEY_EXPIRED]");
                    ("listenKeyExpired".to_string(), serde_json::to_value(b).unwrap_or_default())
                },
                UserDataStreamEventsResponse::TradeLite(b) => {
                    ("TRADE_LITE".to_string(), serde_json::to_value(b).unwrap_or_default())
                },
                UserDataStreamEventsResponse::StrategyUpdate(b) => {
                    ("STRATEGY_UPDATE".to_string(), serde_json::to_value(b).unwrap_or_default())
                },
                UserDataStreamEventsResponse::GridUpdate(b) => {
                    ("GRID_UPDATE".to_string(), serde_json::to_value(b).unwrap_or_default())
                },
                UserDataStreamEventsResponse::Other(v) => {
                    ("OTHER".to_string(), v.clone())
                }
            };
            events_vec.lock().await.push(UserDataStreamEventAudit {
                received_at_iso: received_at,
                event_type: evt_type,
                payload: val,
            });
        });
    });
    info!("--------------------------------------------------------------------------------");
    info!("🧹 [PHA 0] Pre-Clean triệt để tài khoản...");
    let p_cancel_all = CancelAllAlgoOpenOrdersParams::builder(SYMBOL.to_string()).build().unwrap();
    let pre_cancel_json = if let Ok(resp) = rest_client.cancel_all_algo_open_orders(p_cancel_all).await {
        if let Ok(data) = resp.data().await {
            serde_json::to_value(&data).unwrap_or_default()
        } else {
            json!({"status": "cleared"})
        }
    } else {
        json!({"status": "cleared_or_none"})
    };

    let (cur_amt_pre, entry_pre) = get_position_amt(&connection).await;
    if cur_amt_pre.abs() > 1e-4 {
        close_current_position(&connection, cur_amt_pre).await;
    }
    let (cur_amt_after_clean, _) = get_position_amt(&connection).await;
    info!("✅ [PHA 0] Tài khoản sạch sẽ 100% (Position: {:.4} ETH)", cur_amt_after_clean);

    let pre_clean_report = json!({
        "initial_position_amt": cur_amt_pre,
        "entry_price": entry_pre,
        "final_cleaned_position": cur_amt_after_clean,
        "cancel_algo_orders_response": pre_cancel_json
    });

    let start_price = get_current_price(&rest_client, &connection).await?;
    info!("💵 Giá tham chiếu ban đầu: {:.2} USDT", start_price);

    // ============================================================================
    // 📈 PHA 1: MỞ VỊ THẾ LONG 0.01 ETH
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("👉 [PHA 1] Gửi Market Buy {} ETH qua WebSocket API...", ORDER_QTY);
    let p_open = NewOrderParams::builder(
        SYMBOL.to_string(),
        NewOrderSideEnum::Buy,
        NewOrderTypeEnum::Market,
    )
    .quantity(format_qty(ORDER_QTY))
    .build()?;

    let resp_open = connection.new_order(p_open).await?;
    let data_open = resp_open.data()?;
    let entry_order_json = serde_json::to_value(&data_open).unwrap_or_default();
    let order_id = data_open.order_id.unwrap_or(0);
    info!("📥 [BINANCE RESPONSE - PHA 1]: OrderID: {}, Status: {:?}", order_id, data_open.status);

    tokio::time::sleep(Duration::from_millis(600)).await;
    let (pos_opened, entry_price_str) = get_position_amt(&connection).await;
    let entry_price_f64 = entry_price_str.parse::<f64>().unwrap_or(start_price);
    info!("📌 Vị thế đã mở: {:.4} ETH (Entry: {:.2} USDT)", pos_opened, entry_price_f64);

    // ============================================================================
    // 🛡️ PHA 2: TREO SONG SONG CẢ SL VÀ TP (closePosition = true)
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("👉 [PHA 2] Đặt song song cả SL (-2.0%) và TP (+2.0%) có cờ closePosition=true...");
    let sl_initial = entry_price_f64 * 0.98;
    let tp_initial = entry_price_f64 * 1.02;

    // 1. Đặt SL
    let p_sl = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(sl_initial))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;
    let resp_sl = connection.new_algo_order(p_sl).await?;
    let data_sl = resp_sl.data()?;
    let sl_algo_id = data_sl.algo_id.unwrap_or(0);
    let sl_json = serde_json::to_value(&data_sl).unwrap_or_default();
    info!("📥 [PHA 2 SL]: AlgoID: {}, TriggerPrice: {:.2}, Status: {:?}", sl_algo_id, sl_initial, data_sl.algo_status);

    // 2. Đặt TP
    let p_tp = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::TakeProfitMarket,
    )
    .trigger_price(format_price(tp_initial))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;
    let resp_tp = connection.new_algo_order(p_tp).await?;
    let data_tp = resp_tp.data()?;
    let tp_algo_id = data_tp.algo_id.unwrap_or(0);
    let tp_json = serde_json::to_value(&data_tp).unwrap_or_default();
    info!("📥 [PHA 2 TP]: AlgoID: {}, TriggerPrice: {:.2}, Status: {:?}", tp_algo_id, tp_initial, data_tp.algo_status);

    // ============================================================================
    // 🎯 PHA 3 & 4: DYNAMIC TIGHTENING LOOP & CHỜ VỊ THẾ ĐÓNG VỀ 0
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("👉 [PHA 3 & 4] BẮT ĐẦU DYNAMIC TIGHTENING LOOP: ÉP CẮN SL VÀ CHỜ VỊ THẾ VỀ 0.0...");

    // Hủy SL xa cũ để chuẩn bị áp sát
    let _ = connection.cancel_algo_order(CancelAlgoOrderParams::builder().algo_id(sl_algo_id).build()?).await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    let mut current_sl_id = 0i64;
    let mut current_sl_price = 0.0f64;
    let mut tightened_history = Vec::new();
    let mut position_closed = false;
    let mut poll_iterations = 0;
    let start_wait = std::time::Instant::now();
    let max_timeout = Duration::from_secs(45);

    // Lần đặt áp sát đầu tiên
    let live_price_tight = get_current_price(&rest_client, &connection).await?;
    let target_sl = live_price_tight - 0.05; // Cách 0.05 USDT
    let p_tight = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(target_sl))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    match connection.new_algo_order(p_tight).await {
        Ok(resp) => {
            let data = resp.data()?;
            current_sl_id = data.algo_id.unwrap_or(0);
            current_sl_price = target_sl;
            info!("🎯 Đặt SL áp sát lần 1 thành công! AlgoID: {}, TriggerPrice: {:.2} (MarkPrice: {:.2})", current_sl_id, target_sl, live_price_tight);
            tightened_history.push(serde_json::to_value(&data).unwrap_or_default());
        },
        Err(e) => {
            warn!("⚠️ Đặt SL áp sát lần 1 lỗi ({}). Sẽ thử lại trong vòng lặp.", e);
        }
    }

    let mut last_re_tighten = std::time::Instant::now();

    while start_wait.elapsed() < max_timeout {
        poll_iterations += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let (amt_now, _) = get_position_amt(&connection).await;
        if amt_now.abs() < 1e-4 {
            info!("🎉🎉 [TRIGGER THÀNH CÔNG] Vị thế đã đóng hoàn toàn về 0.000 ETH sau {:.2?} (Vòng lặp #{})!", start_wait.elapsed(), poll_iterations);
            position_closed = true;
            break;
        }

        // Nếu sau 4 giây chưa cắn (Testnet giá đứng im hoặc di chuyển ra xa): Tái áp sát!
        if last_re_tighten.elapsed() >= Duration::from_secs(4) {
            let p_now = get_current_price(&rest_client, &connection).await.unwrap_or(0.0);
            if p_now > 10.0 {
                info!("⏳ [TÁI ÁP SÁT SL] Giá hiện tại: {:.2} | SL hiện tại: {:.2}. Đang dời SL sát hơn...", p_now, current_sl_price);
                if current_sl_id > 0 {
                    let _ = connection.cancel_algo_order(CancelAlgoOrderParams::builder().algo_id(current_sl_id).build()?).await;
                    tokio::time::sleep(Duration::from_millis(150)).await;
                }
                let new_tight_sl = p_now - 0.05;
                let p_tight_new = NewAlgoOrderParams::builder(
                    NewAlgoOrderAlgoTypeEnum::Conditional,
                    SYMBOL.to_string(),
                    NewAlgoOrderSideEnum::Sell,
                    NewAlgoOrderTypeEnum::StopMarket,
                )
                .trigger_price(format_price(new_tight_sl))
                .close_position(Some(NewAlgoOrderClosePositionEnum::True))
                .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
                .build()?;

                if let Ok(resp) = connection.new_algo_order(p_tight_new).await {
                    if let Ok(data) = resp.data() {
                        current_sl_id = data.algo_id.unwrap_or(0);
                        current_sl_price = new_tight_sl;
                        info!("🎯 Đã tái áp sát SL thành công! New AlgoID: {}, TriggerPrice: {:.2}", current_sl_id, new_tight_sl);
                        tightened_history.push(serde_json::to_value(&data).unwrap_or_default());
                    }
                }
            }
            last_re_tighten = std::time::Instant::now();
        }
    }

    if !position_closed {
        warn!("⚠️ [TIMEOUT] Hết 45s nhưng lệnh SL chưa cắn tự nhiên trên Testnet. Chủ động Market Close để tiếp tục audit.");
        let (rem_amt, _) = get_position_amt(&connection).await;
        close_current_position(&connection, rem_amt).await;
    }

    // ============================================================================
    // 🔍 PHA 5: POST-FILL AUDIT (TRUY VẤN FULL RAW JSON TỪ BINANCE)
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🔍 [PHA 5] POST-FILL AUDIT: Đang truy vấn trực tiếp trạng thái các lệnh từ Binance...");
    tokio::time::sleep(Duration::from_millis(800)).await;

    // 1. Query trạng thái lệnh SL
    info!("   1️⃣ Query trạng thái lệnh SL (AlgoID: {})...", current_sl_id);
    let p_query_sl = QueryAlgoOrderParams::builder().algo_id(current_sl_id).build().unwrap();
    let sl_algo_audit_json = if let Ok(resp) = rest_client.query_algo_order(p_query_sl).await {
        if let Ok(data) = resp.data().await {
            serde_json::to_value(&data).unwrap_or_default()
        } else {
            json!({"error": "Failed to extract SL data"})
        }
    } else {
        json!({"error": "Failed to query SL"})
    };

    let actual_order_id_str = sl_algo_audit_json.get("actualOrderId").and_then(|v| v.as_str()).unwrap_or_default();
    let actual_order_id = actual_order_id_str.parse::<i64>().ok();

    // 2. Query lệnh Market thực tế (nếu SL đã kích hoạt sinh ra actualOrderId)
    let actual_order_execution_json = if let Some(a_id) = actual_order_id {
        info!("   2️⃣ Trích xuất Full Payload lệnh Market thực tế đã khớp (OrderID: {})...", a_id);
        let p_query_order = QueryOrderParams::builder(SYMBOL.to_string()).order_id(a_id).build().unwrap();
        if let Ok(resp_actual) = rest_client.query_order(p_query_order).await {
            if let Ok(data_actual) = resp_actual.data().await {
                serde_json::to_value(&data_actual).unwrap_or_default()
            } else {
                json!({"order_id": a_id, "note": "Failed to extract body"})
            }
        } else {
            json!({"order_id": a_id, "note": "Query order returned error"})
        }
    } else {
        info!("   2️⃣ Không tìm thấy actualOrderId từ Algo SL (vị thế có thể đóng bằng cơ chế khác)");
        json!({"actual_order_id": null})
    };

    // 3. Query trạng thái lệnh TP còn lại (CÂU HỎI SỐNG CÒN)
    info!("   3️⃣ Query trạng thái lệnh TP còn lại (AlgoID: {})...", tp_algo_id);
    let p_query_tp = QueryAlgoOrderParams::builder().algo_id(tp_algo_id).build().unwrap();
    let tp_algo_audit_json = if let Ok(resp) = rest_client.query_algo_order(p_query_tp).await {
        if let Ok(data) = resp.data().await {
            serde_json::to_value(&data).unwrap_or_default()
        } else {
            json!({"error": "Failed to extract TP data"})
        }
    } else {
        json!({"error": "Failed to query TP"})
    };

    // 4. Query tất cả Algo Open Orders còn mở của Symbol
    info!("   4️⃣ Query danh sách Algo Open Orders còn mở trên sàn...");
    let p_open_algo = CurrentAllAlgoOpenOrdersParams::builder().symbol(SYMBOL.to_string()).build().unwrap();
    let open_algo_list_json = if let Ok(resp) = rest_client.current_all_algo_open_orders(p_open_algo).await {
        if let Ok(data) = resp.data().await {
            serde_json::to_value(&data).unwrap_or_default()
        } else {
            json!([])
        }
    } else {
        json!([])
    };

    // 5. Query trạng thái Position sau khi đóng
    let (pos_final_check, _) = get_position_amt(&connection).await;

    // 6. Trích xuất toàn bộ sự kiện thu thập được từ User Data Stream
    let uds_events = stream_events.lock().await.clone();
    let order_updates_count = uds_events.iter().filter(|e| e.event_type == "ORDER_TRADE_UPDATE").count();
    let account_updates_count = uds_events.iter().filter(|e| e.event_type == "ACCOUNT_UPDATE").count();
    let algo_updates_count = uds_events.iter().filter(|e| e.event_type == "ALGO_UPDATE").count();
    info!("   5️⃣ Đã thu thập {} sự kiện từ User Data Stream thời gian thực (ORDER: {}, ACCOUNT: {}, ALGO: {})",
        uds_events.len(), order_updates_count, account_updates_count, algo_updates_count);

    // Phân tích kết luận
    let tp_status_str = tp_algo_audit_json.get("algoStatus").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
    let tp_auto_canceled = tp_status_str == "CANCELED" || tp_status_str == "EXPIRED";
    let is_tp_still_in_open_list = open_algo_list_json.as_array().map(|arr| {
        arr.iter().any(|item| item.get("algoId").and_then(|v| v.as_i64()) == Some(tp_algo_id))
    }).unwrap_or(false);

    // Kiểm tra xem User Data Stream có bắn event ALGO_UPDATE cho lệnh TP không
    let tp_uds_event = uds_events.iter().find(|e| {
        e.event_type == "ALGO_UPDATE" && e.payload.get("o").and_then(|o| o.get("si")).and_then(|si| si.as_i64()) == Some(tp_algo_id)
    });

    info!("================================================================================");
    info!("🎯🎯🎯 KẾT LUẬN THỰC CHỨNG TỪ SÀN BINANCE:");
    info!("   - Trạng thái Algo SL: {:?}", sl_algo_audit_json.get("algoStatus"));
    info!("   - Lệnh Market thực tế đóng vị thế: {:?}", actual_order_id);
    info!("   - Trạng thái Lệnh TP còn lại (REST): {}", tp_status_str);
    info!("   - Lệnh TP có còn nằm trong Open Algo Orders không? {}", is_tp_still_in_open_list);
    info!("   - User Data Stream có bắn event ALGO_UPDATE cho TP không? {}", tp_uds_event.is_some());
    if tp_auto_canceled {
        info!("   ✅ SÀN BINANCE TỰ ĐỘNG HỦY LỆNH TP SAU KHI VỊ THẾ VỀ 0!");
    } else {
        warn!("   🚨 CẢNH BÁO: SÀN BINANCE KHÔNG TỰ HỦY LỆNH TP! LỆNH TP VẪN TREO TRÊN SỔ LỆNH!");
        warn!("      Hệ thống bắt buộc phải chủ động cancel TP khi vị thế đóng!");
    }
    info!("================================================================================");

    // ============================================================================
    // 💾 PHA 6: XUẤT BÁO CÁO JSON ĐỘC LẬP
    // ============================================================================
    let audit_report = DualBracketAuditReport {
        test_metadata: json!({
            "run_id": run_id,
            "timestamp_iso": Utc::now().to_rfc3339(),
            "symbol": SYMBOL,
            "order_qty": ORDER_QTY,
            "log_file": log_file_name,
            "user_data_stream_active": true
        }),
        phases: json!({
            "phase_0_pre_clean": pre_clean_report,
            "phase_1_entry_order": entry_order_json,
            "phase_2_dual_bracket": {
                "initial_sl": sl_json,
                "initial_tp": tp_json
            },
            "phase_3_trigger_tightening_history": tightened_history,
            "phase_4_wait_result": {
                "position_closed": position_closed,
                "poll_iterations": poll_iterations,
                "elapsed_seconds": start_wait.elapsed().as_secs_f64(),
                "final_position_amt": pos_final_check
            },
            "phase_5_post_fill_audit": {
                "sl_algo_order_status": sl_algo_audit_json,
                "actual_market_order_execution": actual_order_execution_json,
                "tp_algo_order_status": tp_algo_audit_json,
                "open_algo_orders_list": open_algo_list_json,
                "position_amt_after_fill": pos_final_check,
                "user_data_stream_events": uds_events
            }
        }),
        scientific_conclusions: json!({
            "position_successfully_closed": position_closed,
            "sl_actual_order_id": actual_order_id,
            "tp_algo_id": tp_algo_id,
            "tp_status_after_sl_filled": tp_status_str,
            "tp_auto_canceled_by_binance": tp_auto_canceled,
            "tp_still_in_open_orders_list": is_tp_still_in_open_list,
            "user_data_stream_captured_tp_event": tp_uds_event.is_some(),
            "architectural_recommendation": if tp_auto_canceled {
                "Binance automatically purges remaining TP order. User Data Stream provides realtime notification. Safe to leave on exchange."
            } else {
                "CRITICAL: OrderExecutor MUST explicitly issue cancel_algo_order(tp_algo_id) upon position closure to prevent phantom fills on future trades!"
            }
        }),
    };

    let report_json_str = serde_json::to_string_pretty(&audit_report)?;
    let output_path = format!("examples/logs/dual_bracket_runs/{}.json", run_id);
    let latest_path = "examples/logs/dual_bracket_runs/latest.json";
    fs::write(&output_path, &report_json_str)?;
    fs::write(latest_path, &report_json_str)?;
    info!("💾 Đã lưu báo cáo JSON độc lập: {}", output_path);
    info!("💾 Đã cập nhật file latest: {}", latest_path);

    // ============================================================================
    // 🧹 PHA 7: POST-CLEAN AN TOÀN
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧹 [PHA 7] Post-Clean dọn dẹp sạch sẽ tài khoản...");
    let _ = rest_client.cancel_all_algo_open_orders(CancelAllAlgoOpenOrdersParams::builder(SYMBOL.to_string()).build().unwrap()).await;
    let (rem_pos, _) = get_position_amt(&connection).await;
    if rem_pos.abs() > 1e-4 {
        close_current_position(&connection, rem_pos).await;
    }
    let _ = connection.disconnect().await;
    info!("✅ [PHA 7] Hoàn tất kiểm thử và đóng kết nối an toàn.");

    Ok(())
}
