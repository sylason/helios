use tracing::{info, error};
use rust_decimal::prelude::FromPrimitive;

use binance_sdk::derivatives_trading_usds_futures::rest_api::trade_api::{
    TradeApi, NewOrderParams, NewOrderSideEnum
};
use binance_sdk::derivatives_trading_usds_futures::rest_api::RestApi;
use binance_sdk::config::ConfigurationRestApi;

use crate::models::market_info::MarketFilter;
use crate::error::BotError;
use ferro_trade::broker::actions::Action;

pub struct OrderExecutor {
    rest_client: RestApi,
    market_filter: MarketFilter,
}

impl OrderExecutor {
    pub fn new(api_key: String, api_secret: String, use_testnet: bool, market_filter: MarketFilter) -> Self {
        let rest_conf = if use_testnet {
            ConfigurationRestApi::builder()
                .base_path("https://testnet.binancefuture.com")
                .api_key(api_key)
                .api_secret(api_secret)
                .build()
                .unwrap()
        } else {
            ConfigurationRestApi::builder()
                .api_key(api_key)
                .api_secret(api_secret)
                .build()
                .unwrap()
        };
        let rest_client = binance_sdk::derivatives_trading_usds_futures::DerivativesTradingUsdsFuturesRestApi::production(rest_conf);

        Self {
            rest_client,
            market_filter,
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
        
        let (side, is_close_all, risk_percentage) = match action {
            Action::Buy { size, .. } => (NewOrderSideEnum::Buy, false, size),
            Action::Sell { size, .. } => (NewOrderSideEnum::Sell, false, size),
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
            self.calculate_order_size(risk_usdt, current_price, leverage)?
        };

        info!("🚀 [{}] Chuẩn bị gửi lệnh: {:?} Qty: {:.4} @ Market", symbol, side, qty);

        let dec_qty = rust_decimal::Decimal::from_f64(qty).ok_or_else(|| BotError::Internal("Lỗi chuyển đổi kiểu dữ liệu khối lượng".to_string()))?;

        let params = NewOrderParams::builder(
            symbol.to_string(),
            side.clone(),
            "MARKET".to_string()
        )
        .quantity(dec_qty)
        .build().map_err(|e| BotError::Internal(e.to_string()))?;

        match self.rest_client.new_order(params).await {
            Ok(res) => {
                let data = res.data().await.map_err(|e| BotError::from(e))?;
                info!("✅ [{}] Khớp lệnh thành công! OrderID: {:?}", symbol, data.order_id);
                // TODO: Ghi log vào SQLite TradeHistory ở đây
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
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter);

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
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter);

        // Giả sử rủi ro 10 USDT, đòn bẩy x10 (thành 100 USDT), giá DOGE = 0.12345
        // Qty mong muốn: 100 / 0.12345 = 810.04455...
        // Làm tròn step_size 1.0 -> 810.0
        let qty = executor.calculate_order_size(10.0, 0.12345, 10.0).unwrap();
        assert_eq!(qty, 810.0);
    }

    #[test]
    fn test_calculate_order_size_min_notional_fail() {
        let filter = MarketFilter::default();
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter);

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
}
