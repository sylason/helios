// trading_bot/bot_engine/examples/binance_ws_sl_tp_lifecycle_test.rs
//
// 🛡️ BÀI TEST THỰC NGHIỆM WEBSOCKET API: VÒNG ĐỜI TRAILING SL & TAKE PROFIT TRÊN BINANCE TESTNET
// 
// Kịch bản kiểm thử:
// 1. Kết nối Binance Futures Testnet qua WebSocket API (DerivativesTradingUsdsFuturesWsApi).
// 2. Pre-Clean: Hủy toàn bộ lệnh cũ và đóng vị thế tồn đọng.
// 3. PHA A (Trailing SL Lifecycle):
//    - Mở Long 0.01 ETH -> Đặt Algo STOP_MARKET (-2%) -> Modify dời SL (-1%) -> Test cắn SL (Trigger) -> Kiểm tra phản hồi.
// 4. PHA B (Take Profit Lifecycle):
//    - Mở Long 0.01 ETH -> Đặt Algo TAKE_PROFIT_MARKET (+2%) -> Modify dời TP (+3%) -> Test cắn TP (Trigger) -> Kiểm tra phản hồi.
// 5. PHA C (Dual SL + TP Probe):
//    - Mở Long 0.01 ETH -> Đặt song song cả SL & TP (closePosition=true) -> Xem sàn chấp nhận hay từ chối.
// 6. Post-Clean: Dọn dẹp sạch sẽ tài khoản Testnet.

use std::env;
use std::time::Duration;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use binance_sdk::config::{ConfigurationWebsocketApi, ConfigurationRestApi};
use binance_sdk::derivatives_trading_usds_futures::{
    DerivativesTradingUsdsFuturesWsApi, DerivativesTradingUsdsFuturesRestApi,
    websocket_api::{
        WebsocketApi,
        NewAlgoOrderAlgoTypeEnum, NewAlgoOrderParams, NewAlgoOrderSideEnum, NewAlgoOrderTypeEnum,
        NewAlgoOrderClosePositionEnum, NewAlgoOrderWorkingTypeEnum,
        CancelAlgoOrderParams,
        NewOrderParams, NewOrderSideEnum, NewOrderTypeEnum,
        PositionInformationV2Params,
        SymbolPriceTickerParams,
    },
    rest_api::{
        CancelAllAlgoOpenOrdersParams,
    },
};

const SYMBOL: &str = "ETHUSDT";
const ORDER_QTY: f64 = 0.01; // 0.01 ETH (khoảng $25 - $35 USDT, vượt trần Min Notional $5)

fn format_price(price: f64) -> Decimal {
    Decimal::from_f64((price * 100.0).round() / 100.0).unwrap().round_dp(2)
}

fn format_qty(qty: f64) -> Decimal {
    Decimal::from_f64((qty * 1000.0).round() / 1000.0).unwrap().round_dp(3)
}

async fn get_current_price(rest_client: &binance_sdk::derivatives_trading_usds_futures::rest_api::RestApi, conn: &WebsocketApi) -> Result<f64> {
    let p_ticker = binance_sdk::derivatives_trading_usds_futures::rest_api::SymbolPriceTickerParams::builder()
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

    // Fallback: Lấy entry price nếu đang mở vị thế
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // Khởi tạo Logging ra Console + File log
    std::fs::create_dir_all("examples/logs")?;
    let file_appender = tracing_appender::rolling::never("examples/logs", "binance_ws_sl_tp_test.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("================================================================================");
    info!("🚀 BẮT ĐẦU BÀI TEST: VÒNG ĐỜI TRAILING SL & TAKE PROFIT QUA WEBSOCKET API");
    info!("   Target Symbol: {} | Mode: Binance Futures Testnet", SYMBOL);
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

    // 1. Khởi tạo REST client hỗ trợ dọn dẹp ban đầu
    let rest_conf = ConfigurationRestApi::builder()
        .api_key(api_key.clone())
        .api_secret(api_secret.clone())
        .build()?;
    let rest_client = DerivativesTradingUsdsFuturesRestApi::testnet(rest_conf);

    // 2. Khởi tạo WebSocket API client
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
    // 🧹 PRE-CLEAN: DỌN DẸP SẠCH SẼ TRƯỚC KHI TEST
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧹 [PRE-CLEAN] Dọn dẹp tài khoản Testnet...");
    let p_cancel_all = CancelAllAlgoOpenOrdersParams::builder(SYMBOL.to_string()).build().unwrap();
    let _ = rest_client.cancel_all_algo_open_orders(p_cancel_all).await;
    let (cur_amt, _) = get_position_amt(&connection).await;
    close_current_position(&connection, cur_amt).await;
    info!("✅ [PRE-CLEAN] Tài khoản sạch sẽ.");

    let start_price = get_current_price(&rest_client, &connection).await?;
    info!("💵 Giá tham chiếu hiện tại [{}]: {:.2} USDT", SYMBOL, start_price);

    // ============================================================================
    // 🧪 PHA A: VÒNG ĐỜI TRAILING STOP LOSS (ĐẶT -> MODIFY -> ÉP CẮN SL)
    // ============================================================================
    info!("================================================================================");
    info!("🧪 [PHA A] BẮT ĐẦU KIỂM THỬ VÒNG ĐỜI TRAILING STOP LOSS (SL RIÊNG BIỆT)");
    info!("================================================================================");

    // A1. Mở vị thế Long 0.01 ETH
    info!("👉 [A1] Gửi Market Buy {} ETH qua WebSocket API...", ORDER_QTY);
    let p_open_a = NewOrderParams::builder(
        SYMBOL.to_string(),
        NewOrderSideEnum::Buy,
        NewOrderTypeEnum::Market,
    )
    .quantity(format_qty(ORDER_QTY))
    .build()?;
    
    let resp_open_a = connection.new_order(p_open_a).await?;
    let data_open_a = resp_open_a.data()?;
    info!("📥 [BINANCE RESPONSE - OPEN A]: OrderID: {:?}, Status: {:?}", data_open_a.order_id, data_open_a.status);

    tokio::time::sleep(Duration::from_millis(600)).await;
    let (pos_a, entry_a) = get_position_amt(&connection).await;
    let entry_a_f64 = entry_a.parse::<f64>().unwrap_or(start_price);
    info!("📌 Vị thế hiện tại: {:.4} ETH (Entry: {:.2})", pos_a, entry_a_f64);

    // A2. Đặt Trailing SL ban đầu (-2% dưới giá vào)
    let live_price_a = get_current_price(&rest_client, &connection).await?;
    let sl_initial = live_price_a * 0.98;
    info!("👉 [A2] Đặt Trailing SL ban đầu tại {:.2} USDT (-2% so với giá {:.2})...", sl_initial, live_price_a);
    let p_sl_a = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(sl_initial))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    let resp_sl_a = connection.new_algo_order(p_sl_a).await?;
    let data_sl_a = resp_sl_a.data()?;
    let sl_algo_id = data_sl_a.algo_id.unwrap_or(0);
    info!("📥 [BINANCE RESPONSE - INITIAL SL]: AlgoID: {}, Status: {:?}", sl_algo_id, data_sl_a.algo_status);

    // A3. Thực hiện "Modify" dời Trailing SL lên (-1%)
    let sl_modified = live_price_a * 0.99;
    info!("👉 [A3] Thực hiện 'Modify' dời Trailing SL lên {:.2} USDT qua Cancel -> New...", sl_modified);
    
    let p_cancel_sl = CancelAlgoOrderParams::builder().algo_id(sl_algo_id).build()?;
    let resp_cancel_sl = connection.cancel_algo_order(p_cancel_sl).await?;
    let data_cancel_sl = resp_cancel_sl.data()?;
    info!("📥 [BINANCE RESPONSE - CANCEL OLD SL]: AlgoID: {:?}, Code: {:?}, Msg: {:?}", data_cancel_sl.algo_id, data_cancel_sl.code, data_cancel_sl.msg);

    tokio::time::sleep(Duration::from_millis(150)).await;

    let p_sl_mod = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(sl_modified))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    let resp_sl_mod = connection.new_algo_order(p_sl_mod).await?;
    let data_sl_mod = resp_sl_mod.data()?;
    let sl_algo_id_mod = data_sl_mod.algo_id.unwrap_or(0);
    info!("📥 [BINANCE RESPONSE - NEW MODIFIED SL]: New AlgoID: {}, TriggerPrice: {:.2}", sl_algo_id_mod, sl_modified);

    // A4. TEST KÍCH HOẠT KHỚP LỆNH TRAILING SL (Trigger Test)
    info!("👉 [A4] TEST KÍCH HOẠT KHỚP LỆNH TRAILING SL:");
    let trigger_now_price = get_current_price(&rest_client, &connection).await?;
    
    // Thử nghiệm 1: Gửi lệnh đã vi phạm vùng trigger để kiểm chứng phản hồi lỗi sàn
    info!("   🔍 Thử nghiệm 1: Cố tình đặt SL cao hơn giá thị trường ({:.2} > {:.2}) để kiểm chứng phản hồi sàn...", trigger_now_price + 5.0, trigger_now_price);
    let p_sl_invalid = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(trigger_now_price + 5.0))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;
    match connection.new_algo_order(p_sl_invalid).await {
        Ok(r) => warn!("⚠️ Bất ngờ: Sàn chấp nhận lệnh: {:?}", r.data()?),
        Err(e) => info!("   🎯 [XÁC NHẬN SÀN CHẶN LỆNH LỖI] Binance từ chối chính xác: {} (Bảo vệ không cho gửi lệnh đã bị cắn)", e),
    }

    // Thử nghiệm 2: Đặt SL siêu sát Mark Price (-0.3 USDT) để chờ sàn cắn tự nhiên
    info!("   🔍 Thử nghiệm 2: Hủy SL cũ và đặt SL siêu sát Mark Price ({:.2} USDT, -0.3 USDT)...", trigger_now_price - 0.3);
    let _ = connection.cancel_algo_order(CancelAlgoOrderParams::builder().algo_id(sl_algo_id_mod).build()?).await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    let p_sl_tight = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(trigger_now_price - 0.3))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;
    let resp_tight = connection.new_algo_order(p_sl_tight).await?;
    let tight_id = resp_tight.data()?.algo_id.unwrap_or(0);
    info!("📥 Đặt SL siêu sát thành công! AlgoID: {}", tight_id);
    info!("⏳ Đang chờ 5 giây xem biến động Testnet có kích hoạt cắn SL không...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    let (pos_after_sl, _) = get_position_amt(&connection).await;
    if pos_after_sl.abs() < 1e-4 {
        info!("🎉 [TEST PASSED] Giá thị trường đã chạm mức SL sát nút! Vị thế đã tự động đóng về 0.0!");
    } else {
        info!("ℹ️ Vị thế vẫn còn ({:.4} ETH do Testnet biến động chậm). Đóng vị thế chủ động để sang Pha B.", pos_after_sl);
        let _ = connection.cancel_algo_order(CancelAlgoOrderParams::builder().algo_id(tight_id).build()?).await;
        close_current_position(&connection, pos_after_sl).await;
    }

    // ============================================================================
    // 🧪 PHA B: VÒNG ĐỜI TAKE PROFIT (ĐẶT -> MODIFY -> ÉP CẮN TP)
    // ============================================================================
    info!("================================================================================");
    info!("🧪 [PHA B] BẮT ĐẦU KIỂM THỬ VÒNG ĐỜI TAKE PROFIT (TP RIÊNG BIỆT)");
    info!("================================================================================");

    // B1. Mở vị thế Long 0.01 ETH
    info!("👉 [B1] Gửi Market Buy {} ETH qua WebSocket API...", ORDER_QTY);
    let p_open_b = NewOrderParams::builder(
        SYMBOL.to_string(),
        NewOrderSideEnum::Buy,
        NewOrderTypeEnum::Market,
    )
    .quantity(format_qty(ORDER_QTY))
    .build()?;
    
    let resp_open_b = connection.new_order(p_open_b).await?;
    let data_open_b = resp_open_b.data()?;
    info!("📥 [BINANCE RESPONSE - OPEN B]: OrderID: {:?}, Status: {:?}", data_open_b.order_id, data_open_b.status);

    tokio::time::sleep(Duration::from_millis(600)).await;

    // B2. Đặt Take Profit ban đầu (+2% trên giá hiện tại)
    let live_price_b = get_current_price(&rest_client, &connection).await?;
    let tp_initial = live_price_b * 1.02;
    info!("👉 [B2] Đặt Take Profit ban đầu tại {:.2} USDT (+2% so với giá {:.2})...", tp_initial, live_price_b);
    let p_tp_b = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::TakeProfitMarket,
    )
    .trigger_price(format_price(tp_initial))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    let resp_tp_b = connection.new_algo_order(p_tp_b).await?;
    let data_tp_b = resp_tp_b.data()?;
    let tp_algo_id = data_tp_b.algo_id.unwrap_or(0);
    info!("📥 [BINANCE RESPONSE - INITIAL TP]: AlgoID: {}, Status: {:?}", tp_algo_id, data_tp_b.algo_status);

    // B3. Thực hiện "Modify" dời Take Profit lên (+3%)
    let tp_modified = live_price_b * 1.03;
    info!("👉 [B3] Thực hiện 'Modify' dời Take Profit lên {:.2} USDT qua Cancel -> New...", tp_modified);
    
    let p_cancel_tp = CancelAlgoOrderParams::builder().algo_id(tp_algo_id).build()?;
    let resp_cancel_tp = connection.cancel_algo_order(p_cancel_tp).await?;
    let data_cancel_tp = resp_cancel_tp.data()?;
    info!("📥 [BINANCE RESPONSE - CANCEL OLD TP]: AlgoID: {:?}, Code: {:?}, Msg: {:?}", data_cancel_tp.algo_id, data_cancel_tp.code, data_cancel_tp.msg);

    tokio::time::sleep(Duration::from_millis(150)).await;

    let p_tp_mod = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::TakeProfitMarket,
    )
    .trigger_price(format_price(tp_modified))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    let resp_tp_mod = connection.new_algo_order(p_tp_mod).await?;
    let data_tp_mod = resp_tp_mod.data()?;
    let tp_algo_id_mod = data_tp_mod.algo_id.unwrap_or(0);
    info!("📥 [BINANCE RESPONSE - NEW MODIFIED TP]: New AlgoID: {}, Trigger: {:.2}", tp_algo_id_mod, tp_modified);

    // B4. TEST KÍCH HOẠT KHỚP LỆNH TAKE PROFIT (Trigger Test)
    info!("👉 [B4] TEST KÍCH HOẠT KHỚP LỆNH TAKE PROFIT:");
    let trigger_price_tp = get_current_price(&rest_client, &connection).await?;

    // Thử nghiệm 1: Gửi lệnh đã vi phạm vùng trigger TP
    info!("   🔍 Thử nghiệm 1: Cố tình đặt TP thấp hơn giá thị trường ({:.2} < {:.2}) để kiểm chứng sàn...", trigger_price_tp - 5.0, trigger_price_tp);
    let p_tp_invalid = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::TakeProfitMarket,
    )
    .trigger_price(format_price(trigger_price_tp - 5.0))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;
    match connection.new_algo_order(p_tp_invalid).await {
        Ok(r) => warn!("⚠️ Bất ngờ: Sàn chấp nhận lệnh: {:?}", r.data()?),
        Err(e) => info!("   🎯 [XÁC NHẬN SÀN CHẶN LỆNH LỖI] Binance từ chối chính xác: {} (Order would immediately trigger)", e),
    }

    // Thử nghiệm 2: Đặt TP siêu sát Mark Price (+0.3 USDT)
    info!("   🔍 Thử nghiệm 2: Hủy TP cũ và đặt TP siêu sát Mark Price ({:.2} USDT, +0.3 USDT)...", trigger_price_tp + 0.3);
    let _ = connection.cancel_algo_order(CancelAlgoOrderParams::builder().algo_id(tp_algo_id_mod).build()?).await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    let p_tp_tight = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::TakeProfitMarket,
    )
    .trigger_price(format_price(trigger_price_tp + 0.3))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;
    let resp_tp_tight = connection.new_algo_order(p_tp_tight).await?;
    let tight_tp_id = resp_tp_tight.data()?.algo_id.unwrap_or(0);
    info!("📥 Đặt TP siêu sát thành công! AlgoID: {}", tight_tp_id);
    info!("⏳ Đang chờ 5 giây xem biến động Testnet có kích hoạt cắn TP không...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    let (pos_after_tp, _) = get_position_amt(&connection).await;
    if pos_after_tp.abs() < 1e-4 {
        info!("🎉 [TEST PASSED] Giá thị trường đã chạm mức TP sát nút! Vị thế đã tự động chốt lời về 0.0!");
    } else {
        info!("ℹ️ Vị thế vẫn còn ({:.4} ETH do Testnet biến động chậm). Đóng vị thế chủ động để sang Pha C.", pos_after_tp);
        let _ = connection.cancel_algo_order(CancelAlgoOrderParams::builder().algo_id(tight_tp_id).build()?).await;
        close_current_position(&connection, pos_after_tp).await;
    }

    // ============================================================================
    // 🧪 PHA C: ĐẶT SONG SONG CẢ SL & TP (DUAL-PROBE TEST)
    // ============================================================================
    info!("================================================================================");
    info!("🧪 [PHA C] KIỂM THỬ ĐẶT ĐỒNG THỜI CẢ SL & TP (closePosition = true)");
    info!("================================================================================");

    // C1. Mở vị thế Long 0.01 ETH
    info!("👉 [C1] Mở vị thế Long {} ETH...", ORDER_QTY);
    let p_open_c = NewOrderParams::builder(
        SYMBOL.to_string(),
        NewOrderSideEnum::Buy,
        NewOrderTypeEnum::Market,
    )
    .quantity(format_qty(ORDER_QTY))
    .build()?;
    let _ = connection.new_order(p_open_c).await?;
    tokio::time::sleep(Duration::from_millis(600)).await;

    let live_price_c = get_current_price(&rest_client, &connection).await?;
    let sl_c = live_price_c * 0.95;
    let tp_c = live_price_c * 1.05;

    info!("👉 [C2] Gửi lệnh 1: Algo STOP_MARKET tại {:.2} USDT (closePosition=true)...", sl_c);
    let p_sl_c = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::StopMarket,
    )
    .trigger_price(format_price(sl_c))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    let res_sl_c = connection.new_algo_order(p_sl_c).await;
    match res_sl_c {
        Ok(r) => info!("✅ Đặt SL song song thành công! AlgoID: {:?}", r.data()?.algo_id),
        Err(e) => error!("❌ Lỗi đặt SL song song: {:?}", e),
    }

    info!("👉 [C3] Gửi lệnh 2: Algo TAKE_PROFIT_MARKET tại {:.2} USDT (closePosition=true)...", tp_c);
    let p_tp_c = NewAlgoOrderParams::builder(
        NewAlgoOrderAlgoTypeEnum::Conditional,
        SYMBOL.to_string(),
        NewAlgoOrderSideEnum::Sell,
        NewAlgoOrderTypeEnum::TakeProfitMarket,
    )
    .trigger_price(format_price(tp_c))
    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
    .working_type(Some(NewAlgoOrderWorkingTypeEnum::MarkPrice))
    .build()?;

    let res_tp_c = connection.new_algo_order(p_tp_c).await;
    match res_tp_c {
        Ok(r) => {
            info!("🎯 [KẾT QUẢ QUAN TRỌNG] Binance CHẤP NHẬN 2 lệnh closePosition=true song song!");
            info!("   TP AlgoID: {:?}", r.data()?.algo_id);
        },
        Err(e) => {
            warn!("⚠️ [KẾT QUẢ QUAN TRỌNG] Binance TỪ CHỐI 2 lệnh closePosition=true cùng lúc: {:?}", e);
        }
    }

    // ============================================================================
    // 🧹 BƯỚC POST-CLEAN: DỌN DẸP SẠCH SẼ SAU KHI TEST
    // ============================================================================
    info!("--------------------------------------------------------------------------------");
    info!("🧹 [POST-CLEAN] Dọn dẹp tài khoản sau bài test...");
    let _ = rest_client.cancel_all_algo_open_orders(CancelAllAlgoOpenOrdersParams::builder(SYMBOL.to_string()).build().unwrap()).await;
    let (pos_final, _) = get_position_amt(&connection).await;
    close_current_position(&connection, pos_final).await;

    connection.disconnect().await.context("Ngắt kết nối WebSocket thất bại")?;
    info!("✅ [POST-CLEAN] Đã ngắt kết nối WebSocket và dọn dẹp sạch sẽ.");

    info!("================================================================================");
    info!("🎉 HOÀN THÀNH TẤT CẢ CÁC BÀI TEST THỰC NGHIỆM WEBSOCKET API SL/TP!");
    info!("================================================================================");

    Ok(())
}
