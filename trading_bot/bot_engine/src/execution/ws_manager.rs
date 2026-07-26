use std::sync::Arc;
use tokio::sync::{mpsc, watch, broadcast};
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, debug};
use binance_sdk::config::{ConfigurationWebsocketStreams, ConfigurationWebsocketApi, ConfigurationRestApi};
use binance_sdk::derivatives_trading_usds_futures::{
    DerivativesTradingUsdsFuturesWsStreams, 
    DerivativesTradingUsdsFuturesWsApi,
    DerivativesTradingUsdsFuturesRestApi,
    websocket_streams::KlineCandlestickStreamsParams,
    rest_api::RestApi,
    websocket_api::{NewOrderParams, NewOrderSideEnum},
};
use crate::models::weight_monitor::WeightMonitor;
use crate::data_engine::sqlite_impl::SqliteStore;
use crate::data_engine::binance_rest::InitialSync;
use crate::models::models::Candle;

use crate::models::account_state::AccountEvent;

pub struct WsManager {
    api_key: String,
    api_secret: String,
    use_testnet: bool,
    weight_monitor: Arc<WeightMonitor>,
    db_store: Arc<SqliteStore>,
    data_tx: broadcast::Sender<(Candle, bool)>,
    account_tx: broadcast::Sender<AccountEvent>,
}

impl WsManager {
    pub fn new(
        api_key: String, 
        api_secret: String, 
        use_testnet: bool, 
        weight_monitor: Arc<WeightMonitor>,
        db_store: Arc<SqliteStore>,
        data_tx: broadcast::Sender<(Candle, bool)>,
        account_tx: broadcast::Sender<AccountEvent>,
    ) -> Self {
        Self {
            api_key,
            api_secret,
            use_testnet,
            weight_monitor,
            db_store,
            data_tx,
            account_tx,
        }
    }

    /// Khởi động luồng Market Data (K-lines)
    pub fn start_market_data_stream(&self, symbol: String, timeframe: String) {
        let symbol_lower = symbol.to_lowercase();
        let db_store = self.db_store.clone();
        let data_tx = self.data_tx.clone();
        let monitor_clone = self.weight_monitor.clone();
        let use_testnet = self.use_testnet;

        tokio::spawn(async move {
            let conf = if use_testnet {
                ConfigurationWebsocketStreams::builder().ws_url("wss://stream.binancefuture.com").build().unwrap()
            } else {
                ConfigurationWebsocketStreams::builder().build().unwrap()
            };

            let client = if use_testnet {
                DerivativesTradingUsdsFuturesWsStreams::testnet(conf)
            } else {
                DerivativesTradingUsdsFuturesWsStreams::production(conf)
            };

            loop {
                info!("Đang kết nối WebSocket Market Data...");
                match client.connect().await {
                    Ok(connection) => {
                        // [GAP FILLER]
                        let syncer = InitialSync::new(db_store.clone(), monitor_clone.clone());
                        match syncer.fill_gap(&symbol, 30).await {
                            Ok(gap_candles) => {
                                for candle in gap_candles {
                                    let _ = data_tx.send((candle, true)); // is_historical = true
                                }
                            }
                            Err(e) => warn!("⚠️ Lỗi đồng bộ dữ liệu Gap Filler: {}", e),
                        }

                        let params = KlineCandlestickStreamsParams::builder(symbol_lower.clone(), timeframe.clone()).build().unwrap();
                        match connection.kline_candlestick_streams(params).await {
                            Ok(stream) => {
                                info!("✅ Đã kết nối Market Data ({}-{})", symbol, timeframe);
                                
                                let tx = data_tx.clone();
                                stream.on_message(move |message| {
                                    if let (Some(s), Some(k)) = (message.s.as_ref(), message.k.as_ref()) {
                                        if k.x == Some(true) {
                                            let parse_f64 = |opt_s: &Option<String>| {
                                                opt_s.as_ref().and_then(|str_val| str_val.parse::<f64>().ok()).unwrap_or(0.0)
                                            };

                                            let candle = Candle {
                                                symbol: s.clone(),
                                                timestamp: k.t.unwrap_or(0),
                                                open: parse_f64(&k.o),
                                                high: parse_f64(&k.h),
                                                low: parse_f64(&k.l),
                                                close: parse_f64(&k.c),
                                                volume: parse_f64(&k.v),
                                            };

                                            debug!("📊 Nến mới đóng [WS]: {:?}", candle);
                                            let _ = tx.send((candle, false)); // is_historical = false
                                        }
                                    }
                                });

                                loop {
                                    sleep(Duration::from_secs(60)).await;
                                }
                            }
                            Err(e) => error!("❌ Lỗi subscribe Market Data: {:?}", e),
                        }
                    }
                    Err(e) => error!("❌ Lỗi kết nối Market Data: {:?}", e),
                }

                warn!("♻️ Đang thử kết nối lại Market Data sau 5 giây...");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    /// Khởi động luồng User Data và tự động Keep-alive listenKey
    pub fn start_user_data_stream(&self) {
        let api_key = self.api_key.clone();
        let api_secret = self.api_secret.clone();
        let use_testnet = self.use_testnet;
        let weight_monitor = self.weight_monitor.clone();
        let account_tx = self.account_tx.clone();

        tokio::spawn(async move {
            let rest_conf = if use_testnet {
                ConfigurationRestApi::builder().base_path("https://testnet.binancefuture.com").api_key(api_key.clone()).api_secret(api_secret.clone()).build().unwrap()
            } else {
                ConfigurationRestApi::builder().api_key(api_key.clone()).api_secret(api_secret.clone()).build().unwrap()
            };
            let rest_client = if use_testnet {
                DerivativesTradingUsdsFuturesRestApi::testnet(rest_conf)
            } else {
                DerivativesTradingUsdsFuturesRestApi::production(rest_conf)
            };

            loop {
                // 1. Lấy listenKey
                let listen_key = match Self::get_listen_key(&rest_client).await {
                    Ok(key) => key,
                    Err(_) => {
                        sleep(Duration::from_secs(10)).await;
                        continue;
                    }
                };

                // 2. Kết nối WS User Data
                let ws_conf = if use_testnet {
                    ConfigurationWebsocketStreams::builder().ws_url("wss://stream.binancefuture.com").build().unwrap()
                } else {
                    ConfigurationWebsocketStreams::builder().build().unwrap()
                };
                let ws_client = if use_testnet {
                    DerivativesTradingUsdsFuturesWsStreams::testnet(ws_conf)
                } else {
                    DerivativesTradingUsdsFuturesWsStreams::production(ws_conf)
                };

                if let Ok(connection) = ws_client.connect().await {
                    if let Ok(stream) = connection.user_data(listen_key.clone(), None).await {
                        info!("✅ Đã subscribe User Data Stream");
                        let acc_tx = account_tx.clone();
                        stream.on_message(move |data| {
                            let events = Self::parse_user_data_event(&data);
                            for event in events {
                                let _ = acc_tx.send(event);
                            }
                        });
                        // 3. Vòng lặp Keep-alive mỗi 30 phút
                        let mut interval = tokio::time::interval(Duration::from_secs(30 * 60));
                        loop {
                            interval.tick().await;
                            if let Err(e) = rest_client.keepalive_user_data_stream().await {
                                error!("❌ Lỗi Keep-alive listenKey: {:?}", e);
                                break; 
                            } else {
                                info!("🔄 Đã keep-alive listenKey thành công");
                            }
                        }
                    } else {
                        error!("❌ Lỗi subscribe User Data stream");
                    }
                } else {
                    error!("❌ Lỗi kết nối User Data WS");
                }

                warn!("♻️ Đang thử lấy lại listenKey sau 5 giây...");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
    
    async fn get_listen_key(rest_client: &RestApi) -> Result<String, ()> {
        match rest_client.start_user_data_stream().await {
            Ok(res) => {
                let data = match res.data().await {
                    Ok(d) => d,
                    Err(_) => return Err(()),
                };
                info!("🔑 Lấy listenKey thành công");
                Ok(data.listen_key.clone().unwrap_or_default())
            }
            Err(e) => {
                error!("❌ Không lấy được listenKey: {:?}", e);
                Err(())
            }
        }
    }

    /// Khởi động luồng Trade API (WebSocket API siêu tốc)
    pub async fn start_trade_api_stream(&self) {
        let api_key = self.api_key.clone();
        let api_secret = self.api_secret.clone();
        let use_testnet = self.use_testnet;

        tokio::spawn(async move {
            loop {
                let conf = if use_testnet {
                    ConfigurationWebsocketApi::builder().ws_url("wss://testnet.binancefuture.com/ws-api/v1").api_key(api_key.clone()).api_secret(api_secret.clone()).build().unwrap()
                } else {
                    ConfigurationWebsocketApi::builder().api_key(api_key.clone()).api_secret(api_secret.clone()).build().unwrap()
                };

                let client = DerivativesTradingUsdsFuturesWsApi::production(conf);

                info!("Đang kết nối WebSocket Trade API...");
                match client.connect().await {
                    Ok(connection) => {
                        info!("✅ Đã kết nối Trade API");
                        // Luồng này sẽ được dùng để bắn lệnh. 
                        // Do tính chất Request-Response của WS API, chúng ta cần lưu trữ connection này trong 1 channel
                        // hoặc Actor để OrderExecutor có thể gửi lệnh qua.
                        
                        // Ở phiên bản hiện tại, chỉ loop để giữ kết nối không bị ngắt.
                        loop {
                            sleep(Duration::from_secs(60)).await;
                        }
                    }
                    Err(e) => error!("❌ Lỗi kết nối Trade API: {:?}", e),
                }

                warn!("♻️ Đang thử kết nối lại Trade API sau 5 giây...");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    /// Tách logic phân tích JSON của User Data Stream thành một hàm riêng để dễ dàng Unit Test
    pub fn parse_user_data_event(data: &binance_sdk::derivatives_trading_usds_futures::websocket_streams::UserDataStreamEventsResponse) -> Vec<crate::models::account_state::AccountEvent> {
        use binance_sdk::derivatives_trading_usds_futures::websocket_streams::UserDataStreamEventsResponse;
        let mut events = Vec::new();

        match data {
            UserDataStreamEventsResponse::AccountUpdate(update) => {
                if let Some(ref a) = update.a {
                    // 1. Cập nhật Balances
                    if let Some(ref balances) = a.b_uppercase {
                        for b in balances {
                            if b.a.as_deref() == Some("USDT") {
                                let wallet_balance = b.wb.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                                let available_balance = b.cw.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                                
                                events.push(crate::models::account_state::AccountEvent::AccountUpdate {
                                    wallet_balance,
                                    available_balance,
                                });
                            }
                        }
                    }
                    // 2. Cập nhật Positions
                    if let Some(ref positions) = a.p_uppercase {
                        for p in positions {
                            let symbol = p.s.clone().unwrap_or_default();
                            let position_amt = p.pa.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            let entry_price = p.ep.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            let unrealized_profit = p.up.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            
                            events.push(crate::models::account_state::AccountEvent::PositionUpdate {
                                symbol,
                                position_amt,
                                entry_price,
                                unrealized_profit,
                            });
                        }
                    }
                }
            }
            UserDataStreamEventsResponse::OrderTradeUpdate(_update) => {
                // TODO: Xử lý trạng thái lệnh
            }
            _ => {}
        }
        
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binance_sdk::derivatives_trading_usds_futures::websocket_streams::UserDataStreamEventsResponse;
    use crate::models::account_state::AccountEvent;

    #[test]
    fn test_parse_account_update_json() {
        // Mô phỏng chuỗi JSON trả về từ Binance
        let json_str = r#"{
            "e": "ACCOUNT_UPDATE",
            "T": 1564745798939,
            "E": 1564745798940,
            "a": {
              "m": "ORDER",
              "B": [
                {
                  "a": "USDT",
                  "wb": "122624.12345678",
                  "cw": "100.12345678",
                  "bc": "50.0"
                },
                {
                  "a": "BUSD",
                  "wb": "1.0",
                  "cw": "0.0",
                  "bc": "0.0"
                }
              ],
              "P": [
                {
                  "s": "BTCUSDT",
                  "pa": "1.5",
                  "ep": "50000.0",
                  "cr": "200.0",
                  "up": "-100.0",
                  "mt": "isolated",
                  "iw": "0.0",
                  "ps": "BOTH",
                  "ma": "USDT"
                }
              ]
            }
        }"#;

        // Parse JSON thành Struct của SDK
        let parsed: UserDataStreamEventsResponse = serde_json::from_str(json_str)
            .expect("Không thể parse JSON từ Binance");
            
        // Chạy logic trích xuất Event
        let events = WsManager::parse_user_data_event(&parsed);
        
        // Phải có 2 events (1 cho Balance, 1 cho Position)
        assert_eq!(events.len(), 2);
        
        let mut has_acc_update = false;
        let mut has_pos_update = false;

        for event in events {
            match event {
                AccountEvent::AccountUpdate { wallet_balance, available_balance } => {
                    assert_eq!(wallet_balance, 122624.12345678);
                    assert_eq!(available_balance, 100.12345678);
                    has_acc_update = true;
                }
                AccountEvent::PositionUpdate { symbol, position_amt, entry_price, unrealized_profit } => {
                    assert_eq!(symbol, "BTCUSDT");
                    assert_eq!(position_amt, 1.5);
                    assert_eq!(entry_price, 50000.0);
                    assert_eq!(unrealized_profit, -100.0);
                    has_pos_update = true;
                }
            }
        }

        assert!(has_acc_update, "Thiếu AccountUpdate event");
        assert!(has_pos_update, "Thiếu PositionUpdate event");
    }
}
