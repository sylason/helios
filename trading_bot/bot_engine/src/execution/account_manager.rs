use binance_sdk::config::ConfigurationRestApi;
use binance_sdk::derivatives_trading_usds_futures::{
    rest_api::{
        RestApi, AccountApi, TradeApi,
        ChangeInitialLeverageParams, ChangeMarginTypeParams, AccountInformationV2Params,
        ChangeMarginTypeMarginTypeEnum
    },
    DerivativesTradingUsdsFuturesRestApi
};
use tracing::{info, error};
use tokio::sync::broadcast;
use crate::models::account_state::AccountEvent;
use crate::error::BotError;

use binance_sdk::common::models::{RateLimitType, Interval as BinanceInterval};
use crate::models::weight_monitor::WeightMonitor;
use std::sync::Arc;

pub struct AccountManager {
    rest_client: RestApi,
    account_tx: broadcast::Sender<AccountEvent>,
    monitor: Arc<WeightMonitor>,
}

impl AccountManager {
    pub fn new(api_key: String, api_secret: String, use_testnet: bool, account_tx: broadcast::Sender<AccountEvent>, monitor: Arc<WeightMonitor>) -> Self {
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
        let rest_client = if use_testnet {
            DerivativesTradingUsdsFuturesRestApi::testnet(rest_conf)
        } else {
            DerivativesTradingUsdsFuturesRestApi::production(rest_conf)
        };

        Self {
            rest_client,
            account_tx,
            monitor,
        }
    }

    /// Khởi tạo trạng thái ban đầu: Set đòn bẩy, Margin Type và đồng bộ số dư.
    pub async fn initialize_account(&self, symbol: &str, leverage: i32) -> Result<(), BotError> {
        info!("🔄 Đang khởi tạo tài khoản (Leverage: {}, Margin: ISOLATED) cho {}", leverage, symbol);

        // 1. Chỉnh Margin Type sang ISOLATED
        let margin_params = ChangeMarginTypeParams::builder(
            symbol.to_string(),
            ChangeMarginTypeMarginTypeEnum::Isolated,
        ).build().map_err(|e| BotError::Internal(e.to_string()))?;
        
        self.monitor.wait_if_needed(1).await;
        match self.rest_client.change_margin_type(margin_params).await {
            Ok(resp) => {
                if let Some(ref limits) = resp.rate_limits {
                    for limit in limits {
                        if limit.rate_limit_type == RateLimitType::RequestWeight && limit.interval == BinanceInterval::Minute {
                            self.monitor.update(limit.count);
                        }
                    }
                }
                self.monitor.report_status();
                let _ = resp.data().await; // Lấy kết quả, lỗi thì bỏ qua (có thể do đã là ISOLATED)
                info!("✅ Đã đổi Margin Type sang ISOLATED (hoặc đã ở trạng thái ISOLATED)");
            },
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("No need to change margin type") {
                    info!("✅ Margin Type đã là ISOLATED, bỏ qua");
                } else {
                    error!("❌ Lỗi đổi Margin Type: {}", err_msg);
                    return Err(BotError::from(e));
                }
            }
        }

        // 2. Chỉnh Leverage
        let leverage_params = ChangeInitialLeverageParams::builder(
            symbol.to_string(),
            leverage as i64,
        ).build().map_err(|e| BotError::Internal(e.to_string()))?;

        self.monitor.wait_if_needed(1).await;
        match self.rest_client.change_initial_leverage(leverage_params).await {
            Ok(resp) => {
                if let Some(ref limits) = resp.rate_limits {
                    for limit in limits {
                        if limit.rate_limit_type == RateLimitType::RequestWeight && limit.interval == BinanceInterval::Minute {
                            self.monitor.update(limit.count);
                        }
                    }
                }
                self.monitor.report_status();
                match resp.data().await {
                    Ok(res) => {
                        if let Some(lev) = res.leverage {
                            info!("✅ Đã thiết lập đòn bẩy thành công: {}x", lev);
                        }
                    },
                    Err(e) => {
                        error!("❌ Lỗi data thiết lập đòn bẩy: {}", e);
                        return Err(BotError::from(e));
                    }
                }
            },
            Err(e) => {
                error!("❌ Lỗi gọi API thiết lập đòn bẩy: {}", e);
                return Err(e.into());
            }
        }

        // 3. Lấy Account Information (V2) để đồng bộ Balance và Position ban đầu
        let acc_params = AccountInformationV2Params::builder().build().map_err(|e| BotError::Internal(e.to_string()))?;
        self.monitor.wait_if_needed(5).await; // Account Info V2 tốn 5 weight
        match self.rest_client.account_information_v2(acc_params).await {
            Ok(resp) => {
                if let Some(ref limits) = resp.rate_limits {
                    for limit in limits {
                        if limit.rate_limit_type == RateLimitType::RequestWeight && limit.interval == BinanceInterval::Minute {
                            self.monitor.update(limit.count);
                        }
                    }
                }
                self.monitor.report_status();
                let acc_info = resp.data().await.map_err(|e| BotError::from(e))?;
                // Cập nhật Wallet Balance và Available Balance cho USDT
                if let Some(assets) = acc_info.assets {
                    for asset in assets {
                        if asset.asset.as_deref() == Some("USDT") {
                            let wallet_balance = asset.wallet_balance.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            let available_balance = asset.available_balance.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

                            let _ = self.account_tx.send(AccountEvent::AccountUpdate {
                                wallet_balance,
                                available_balance,
                            });
                            info!("💰 Cập nhật số dư USDT từ REST API: Wallet = {}, Available = {}", wallet_balance, available_balance);
                        }
                    }
                }

                // Cập nhật Position
                if let Some(positions) = acc_info.positions {
                    for pos in positions {
                        if pos.symbol.as_deref() == Some(symbol) {
                            let position_amt = pos.position_amt.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            let entry_price = pos.entry_price.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            let unrealized_profit = pos.unrealized_profit.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            
                            let _ = self.account_tx.send(AccountEvent::PositionUpdate {
                                symbol: symbol.to_string(),
                                position_amt,
                                entry_price,
                                unrealized_profit,
                            });
                            info!("📊 Cập nhật vị thế ban đầu: Mở {} {}, Giá: {}, PNL: {}", position_amt, symbol, entry_price, unrealized_profit);
                        }
                    }
                }
            },
            Err(e) => {
                error!("❌ Lỗi khi lấy thông tin tài khoản: {}", e);
                return Err(e.into());
            }
        }

        Ok(())
    }
}
