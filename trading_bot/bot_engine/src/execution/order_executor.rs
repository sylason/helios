use tracing::{info, warn, error};
use rust_decimal::prelude::FromPrimitive;

use binance_sdk::derivatives_trading_usds_futures::rest_api::trade_api::{
    NewOrderParams, NewOrderSideEnum, NewOrderTypeEnum,
    NewAlgoOrderParams, NewAlgoOrderWorkingTypeEnum, NewAlgoOrderAlgoTypeEnum,
    NewAlgoOrderTypeEnum, NewAlgoOrderClosePositionEnum,
    CancelAllAlgoOpenOrdersParams, NewAlgoOrderSideEnum
};

use binance_sdk::derivatives_trading_usds_futures::rest_api::RestApi;
use binance_sdk::config::ConfigurationRestApi;
use chrono::Utc;

use crate::models::market_info::MarketFilter;
use crate::error::BotError;
use ferro_trade::broker::actions::Action;
use crate::data_engine::sqlite_impl::SqliteStore;
use binance_sdk::common::models::{RateLimitType, Interval as BinanceInterval};
use crate::models::weight_monitor::WeightMonitor;
use std::sync::Arc;

pub struct OrderExecutor {
    rest_client: RestApi,
    market_filter: MarketFilter,
    db_store: Arc<SqliteStore>,
    monitor: Arc<WeightMonitor>,
}

impl OrderExecutor {
    pub fn new(api_key: String, api_secret: String, use_testnet: bool, market_filter: MarketFilter, db_store: Arc<SqliteStore>, monitor: Arc<WeightMonitor>) -> Self {
        let rest_conf = ConfigurationRestApi::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .build()
            .unwrap();
            
        let rest_client = if use_testnet {
            binance_sdk::derivatives_trading_usds_futures::DerivativesTradingUsdsFuturesRestApi::testnet(rest_conf)
        } else {
            binance_sdk::derivatives_trading_usds_futures::DerivativesTradingUsdsFuturesRestApi::production(rest_conf)
        };

        Self {
            rest_client,
            market_filter,
            db_store,
            monitor,
        }
    }

    /// Tính toán khối lượng giao dịch hợp lệ
    pub fn calculate_order_size(&self, risk_usdt: f64, current_price: f64, leverage: f64) -> Result<f64, BotError> {
        if current_price <= 0.0 {
            return Err(BotError::Internal("Giá thị trường không hợp lệ".to_string()));
        }

        // Tính số lượng Coin (Qty)
        let raw_qty = (risk_usdt * leverage) / current_price;
        
        // Làm tròn theo Market Filter step_size
        let qty = self.market_filter.round_qty(raw_qty);

        // Kiểm tra Min Notional (Thường là 5 USDT trên Binance Futures)
        let notional = qty * current_price;
        if notional < 5.0 {
            return Err(BotError::MarketFilterError(format!("Giá trị lệnh ({:.2} USDT) nhỏ hơn Min Notional (5 USDT)", notional)));
        }

        Ok(qty)
    }

    /// Làm tròn giá thành Decimal chuẩn theo tick_size của sàn
    fn format_price_decimal(&self, price: f64) -> rust_decimal::Decimal {
        let rounded = self.market_filter.round_price(price);
        let decimals = MarketFilter::count_decimals(self.market_filter.tick_size);
        rust_decimal::Decimal::from_f64(rounded).unwrap().round_dp(decimals as u32)
    }

    /// Làm tròn khối lượng thành Decimal chuẩn theo step_size của sàn
    fn format_qty_decimal(&self, qty: f64) -> rust_decimal::Decimal {
        let rounded = self.market_filter.round_qty(qty);
        let decimals = MarketFilter::count_decimals(self.market_filter.step_size);
        rust_decimal::Decimal::from_f64(rounded).unwrap().round_dp(decimals as u32)
    }

    /// Thực thi tín hiệu giao dịch
    pub async fn execute_action(
        &self, 
        symbol: &str, 
        action: Action, 
        current_price: f64, 
        available_balance: f64,
        position_amt: f64,
        leverage: f64
    ) -> Result<(), BotError> {
        
        let mut sl_price_to_set = None;

        // Xử lý riêng Action::UpdateTradeSlTp
        if let Action::UpdateTradeSlTp { new_sl_price, .. } = &action {
            let p_cancel = CancelAllAlgoOpenOrdersParams::builder(symbol.to_string()).build().unwrap();
            self.monitor.wait_if_needed(1).await;
            if let Err(e) = self.rest_client.cancel_all_algo_open_orders(p_cancel).await {
                error!("❌ [{}] Lỗi hủy Algo Orders cũ: {}", symbol, e);
            } else {
                info!("🧹 [{}] Đã hủy Algo SL cũ để cập nhật mới.", symbol);
            }

            // Chờ 150ms để Binance Futures Server cập nhật sổ lệnh hủy thành công
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

            if let Some(sl) = new_sl_price {
                let sl_side = if position_amt > 0.0 { NewAlgoOrderSideEnum::Sell } else { NewAlgoOrderSideEnum::Buy };
                let rounded_sl = self.format_price_decimal(*sl);
                let p_sl = NewAlgoOrderParams::builder(
                    NewAlgoOrderAlgoTypeEnum::Conditional,
                    symbol.to_string(),
                    sl_side,
                    NewAlgoOrderTypeEnum::StopMarket
                )
                .trigger_price(rounded_sl)
                .close_position(Some(NewAlgoOrderClosePositionEnum::True))
                .working_type(NewAlgoOrderWorkingTypeEnum::MarkPrice)
                .build().unwrap();
                
                // Thử lại tối đa 3 lần nếu dính lỗi độ trễ hủy lệnh trên Binance
                let mut success = false;
                for attempt in 1..=3 {
                    self.monitor.wait_if_needed(1).await;
                    match self.rest_client.new_algo_order(p_sl.clone()).await {
                        Ok(_) => {
                            info!("🛡️ [{}] Đã cập nhật Trailing SL mới tại: {}", symbol, rounded_sl);
                            success = true;
                            break;
                        },
                        Err(e) => {
                            warn!("⚠️ [{}] Thử lần {}/3 đặt Trailing SL thất bại ({}), thử lại sau 150ms...", symbol, attempt, e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                        }
                    }
                }

                if !success {
                    error!("❌ [{}] Đã thử 3 lần nhưng không thể đặt Trailing SL mới tại: {}", symbol, rounded_sl);
                }
            }
            return Ok(());
        }

        let (side, is_close_all, risk_percentage) = match action {
            Action::Buy { size, stop, .. } => {
                sl_price_to_set = stop;
                (NewOrderSideEnum::Buy, false, size)
            },
            Action::Sell { size, stop, .. } => {
                sl_price_to_set = stop;
                (NewOrderSideEnum::Sell, false, size)
            },
            Action::ClosePosition { .. } | Action::CloseLongs { .. } | Action::CloseShorts { .. } => {
                // Đảo ngược vị thế hiện tại để đóng lệnh
                if position_amt > 0.0 {
                    (NewOrderSideEnum::Sell, true, 0.0)
                } else if position_amt < 0.0 {
                    (NewOrderSideEnum::Buy, true, 0.0)
                } else {
                    info!("ℹ️ [{}] Nhận tín hiệu Close nhưng đang không có vị thế mở.", symbol);
                    return Ok(());
                }
            },
            _ => {
                info!("ℹ️ [{}] Bỏ qua Action không hỗ trợ: {:?}", symbol, action);
                return Ok(());
            }
        };

        let qty = if is_close_all {
            position_amt.abs()
        } else {
            let risk_usdt = available_balance * risk_percentage;
            let target_qty = self.calculate_order_size(risk_usdt, current_price, leverage)?;
            
            // Binance One-way Mode logic: to reverse a position, we must add the current position size
            match side {
                NewOrderSideEnum::Buy => {
                    if position_amt < 0.0 {
                        // Đang Short, muốn Buy -> Phải đóng Short trước rồi mới mở Long
                        target_qty + position_amt.abs()
                    } else {
                        target_qty
                    }
                }
                NewOrderSideEnum::Sell => {
                    if position_amt > 0.0 {
                        // Đang Long, muốn Sell -> Phải đóng Long trước rồi mới mở Short
                        target_qty + position_amt
                    } else {
                        target_qty
                    }
                }
            }
        };

        info!("🚀 [{}] Chuẩn bị gửi lệnh: {:?} Qty: {:.4} @ Market", symbol, side, qty);

        let dec_qty = self.format_qty_decimal(qty);

        let params = NewOrderParams::builder(
            symbol.to_string(),
            side.clone(),
            NewOrderTypeEnum::Market
        )
        .quantity(dec_qty)
        .build().map_err(|e| BotError::Internal(e.to_string()))?;

        self.monitor.wait_if_needed(1).await;
        match self.rest_client.new_order(params).await {
            Ok(res) => {
                if let Some(ref limits) = res.rate_limits {
                    for limit in limits {
                        if limit.rate_limit_type == RateLimitType::RequestWeight && limit.interval == BinanceInterval::Minute {
                            self.monitor.update(limit.count);
                        }
                    }
                }
                self.monitor.report_status();
                let data = res.data().await.map_err(|e| BotError::from(e))?;
                info!("✅ [{}] Khớp lệnh thành công! OrderID: {:?}", symbol, data.order_id);
                
                // Ghi log vào SQLite TradeHistory
                let order_id_str = data.order_id.map_or("UNKNOWN".to_string(), |id| id.to_string());
                let side_str = match side {
                    NewOrderSideEnum::Buy => "BUY",
                    NewOrderSideEnum::Sell => "SELL",
                };
                let current_time = Utc::now().timestamp_millis();
                
                if let Err(e) = self.db_store.save_trade_history(
                    &order_id_str,
                    symbol,
                    side_str,
                    current_price,
                    qty,
                    current_time
                ) {
                    error!("❌ [{}] Lỗi ghi log giao dịch vào SQLite: {:?}", symbol, e);
                }

                // Nếu Strategy có báo StopLoss ngay lúc vào lệnh, ta đặt luôn Algo Order
                if let Some(sl) = sl_price_to_set {
                    let sl_side = match side {
                        NewOrderSideEnum::Buy => NewAlgoOrderSideEnum::Sell,
                        NewOrderSideEnum::Sell => NewAlgoOrderSideEnum::Buy,
                    };
                    let rounded_sl = self.format_price_decimal(sl);
                    let p_sl = NewAlgoOrderParams::builder(
                        NewAlgoOrderAlgoTypeEnum::Conditional,
                        symbol.to_string(),
                        sl_side,
                        NewAlgoOrderTypeEnum::StopMarket
                    )
                    .trigger_price(rounded_sl)
                    .close_position(Some(NewAlgoOrderClosePositionEnum::True))
                    .working_type(NewAlgoOrderWorkingTypeEnum::MarkPrice)
                    .build().unwrap();
                    
                    // Thử lại tối đa 3 lần cho Hard SL
                    let mut success = false;
                    for attempt in 1..=3 {
                        self.monitor.wait_if_needed(1).await;
                        match self.rest_client.new_algo_order(p_sl.clone()).await {
                            Ok(_) => {
                                info!("🛡️ [{}] Đã đặt Hard SL bảo vệ tại: {}", symbol, rounded_sl);
                                success = true;
                                break;
                            },
                            Err(e) => {
                                warn!("⚠️ [{}] Thử lần {}/3 đặt Hard SL thất bại ({}), thử lại sau 150ms...", symbol, attempt, e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                            }
                        }
                    }

                    if !success {
                        error!("❌ [{}] Đã thử 3 lần nhưng không thể đặt Hard SL bảo vệ tại: {}", symbol, rounded_sl);
                    }
                }

                Ok(())
            },
            Err(e) => {
                error!("❌ [{}] Lỗi gửi lệnh qua API: {:?}", symbol, e);
                Err(BotError::from(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_order_size_btc() {
        let filter = MarketFilter {
            tick_size: 0.1,
            step_size: 0.001,
            min_qty: 0.001,
            max_qty: 1000.0,
            min_notional: 5.0,
        };
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Giả sử có 100 USDT, Risk 1.0 (100%), đòn bẩy x1, giá BTC = 60000.5
        // Qty mong muốn: 100 / 60000.5 = 0.001666...
        // Làm tròn step_size 0.001 -> 0.002
        let qty = executor.calculate_order_size(100.0, 60000.5, 1.0).unwrap();
        assert_eq!(qty, 0.002);
    }

    #[test]
    fn test_calculate_order_size_doge() {
        let filter = MarketFilter {
            tick_size: 0.00001,
            step_size: 1.0,
            min_qty: 1.0,
            max_qty: 10000000.0,
            min_notional: 5.0,
        };
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Giả sử rủi ro 10 USDT, đòn bẩy x10 (thành 100 USDT), giá DOGE = 0.12345
        // Qty mong muốn: 100 / 0.12345 = 810.04455...
        // Làm tròn step_size 1.0 -> 810.0
        let qty = executor.calculate_order_size(10.0, 0.12345, 10.0).unwrap();
        assert_eq!(qty, 810.0);
    }

    #[test]
    fn test_calculate_order_size_min_notional_fail() {
        let filter = MarketFilter::default();
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Rủi ro chỉ 1 USDT, đòn bẩy x1, giá 60000
        // Qty = 0.000016 -> làm tròn thành 0.0 (không đủ min qty) hoặc 0.000
        // Notional = 0 < 5.0 -> lỗi MinNotional
        let res = executor.calculate_order_size(1.0, 60000.0, 1.0);
        assert!(res.is_err());
        match res {
            Err(BotError::MarketFilterError(_)) => (),
            _ => panic!("Phải trả về MarketFilterError"),
        }
    }

    #[tokio::test]
    #[ignore] // Chỉ chạy thủ công khi cần test với Testnet thật
    async fn test_integration_execute_order_testnet() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("BINANCE_TESTNET_API_KEY").unwrap_or_default();
        let api_secret = std::env::var("BINANCE_TESTNET_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || api_secret.is_empty() {
            println!("Skipping integration test due to missing BINANCE_TESTNET_API_KEY");
            return;
        }

        let filter = MarketFilter {
            tick_size: 0.00001,
            step_size: 1.0,
            min_qty: 1.0,
            max_qty: 10000000.0,
            min_notional: 5.0,
        };
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new(api_key, api_secret, true, filter, db_store, monitor);

        let action = Action::Buy {
            size: 1.0, // Sử dụng 100% của mức rủi ro truyền vào
            limit: None,
            stop: None,
            policy: None,
            tag: None,
        };
        
        // Tăng vốn lên 100 USDT để đảm bảo Notional (Risk) > 5 USDT bất chấp giá Market thay đổi
        let res = executor.execute_action("DOGEUSDT", action, 0.12, 100.0, 0.0, 1.0).await;
        
        assert!(res.is_ok(), "Lỗi gửi lệnh E2E Testnet: {:?}", res.err());
    }
}
