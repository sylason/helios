use binance_sdk::config::ConfigurationWebsocketStreams;
use binance_sdk::derivatives_trading_usds_futures::{
    DerivativesTradingUsdsFuturesWsStreams,
    websocket_streams::{KlineCandlestickStreamsParams, KlineCandlestickStreamsIntervalEnum},
};
use tokio::sync::broadcast;
use tracing::{error, info, debug};
use crate::models::models::Candle;

/// Khởi chạy luồng dữ liệu nến 1m cho ETHUSDT
pub async fn start_kline_stream(symbol: String, data_tx: broadcast::Sender<(Candle, bool)>) {
    let symbol_lower = symbol.to_lowercase();
    
    // 1. Cấu hình & Client
    let ws_conf = ConfigurationWebsocketStreams::builder().build().expect("Cấu hình WS lỗi");
    let client = DerivativesTradingUsdsFuturesWsStreams::production(ws_conf);

    // 2. Kết nối
    let connection = match client.connect().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Lỗi kết nối WS Binance: {:?}", e);
            return;
        }
    };

    info!("👀 Đã kết nối WebSocket. Đang đăng ký Kline 1m cho {}", symbol.to_uppercase());

    // 3. Đăng ký stream Kline 1m
    let params = KlineCandlestickStreamsParams::builder(symbol_lower, KlineCandlestickStreamsIntervalEnum::Interval1m).build().unwrap();
    let stream = connection.kline_candlestick_streams(params).await.expect("Không thể mở Kline Stream");

    // 4. Xử lý tin nhắn (Bóc tách Option đa tầng)
    stream.on_message(move |message| {
        // Bóc lớp 1: Symbol và Kline Data chính
        if let (Some(symbol_str), Some(k)) = (message.s.as_ref(), message.k.as_ref()) {
            
            // Bóc lớp 2: Chỉ xử lý nếu trường x (is_final) tồn tại và là true
            if k.x == Some(true) {
                
                // Hàm tiện ích để parse Option<String> sang f64
                let parse_f64 = |opt_s: &Option<String>| {
                    opt_s.as_ref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0)
                };

                let c_val = parse_f64(&k.c);
                let v_val = parse_f64(&k.v);
                let qv_raw = parse_f64(&k.q);
                let qv = if qv_raw > 0.0 { qv_raw } else { c_val * v_val };

                let candle = Candle {
                    symbol: symbol_str.clone(),
                    timestamp: k.t.unwrap_or(0),
                    open: parse_f64(&k.o),
                    high: parse_f64(&k.h),
                    low: parse_f64(&k.l),
                    close: c_val,
                    volume: v_val,
                    quote_volume: qv,
                };

                debug!("📊 Nến mới đóng: {:?}", candle);
                
                // Gửi dữ liệu nến vào Hệ thần kinh (EventBus)
                if let Err(e) = data_tx.send((candle, false)) {
                    error!("❌ Lỗi gửi nến vào Bus: {}", e);
                }
            }
        }
    });

    // 5. Duy trì task (Lắng nghe ngầm)
    info!("🚀 Market Data Stream đang chạy...");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
