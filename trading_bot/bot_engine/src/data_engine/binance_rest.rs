use binance_sdk::config::ConfigurationRestApi;
use binance_sdk::derivatives_trading_usds_futures::{
    rest_api::{
        KlineCandlestickDataIntervalEnum, 
        KlineCandlestickDataParams, 
        RestApi, 
        KlineCandlestickDataResponseItemInner
    },
};
use crate::models::models::Candle;
use crate::models::repository::CandleRepository;
use crate::models::weight_monitor::WeightMonitor;
use binance_sdk::common::models::{RateLimitType, Interval as BinanceInterval};
use tracing::{info, error, debug};
use std::sync::Arc;
use crate::data_engine::sqlite_impl::SqliteStore;

pub struct InitialSync {
    rest_client: RestApi,
    db: Arc<SqliteStore>,
    monitor: Arc<WeightMonitor>,
}

impl InitialSync {
    pub fn new(db: Arc<SqliteStore>, monitor: Arc<WeightMonitor>) -> Self {
        let rest_conf = ConfigurationRestApi::builder().build().expect("Cấu hình REST lỗi");
        let rest_client = binance_sdk::derivatives_trading_usds_futures::DerivativesTradingUsdsFuturesRestApi::production(rest_conf);
        
        Self { rest_client, db, monitor }
    }

    /// Tải dữ liệu lịch sử nến 1m để lấp đầy khoảng trống (Gap Filler)
    pub async fn fill_gap(&self, symbol: &str, default_limit_days: i64) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        let symbol_upper = symbol.to_uppercase();
        let now = chrono::Utc::now().timestamp_millis();
        
        // Tìm nến cuối cùng trong DB
        let latest_candle = self.db.get_latest_candle(&symbol_upper).await?;
        
        let start_target = calculate_sync_start(latest_candle.as_ref().map(|c| c.timestamp), default_limit_days, now);
        if latest_candle.is_some() {
            info!("🔄 Tìm thấy nến cuối cùng. Đang tải dữ liệu lấp khoảng trống (Gap Filler) từ {}...", start_target);
        } else {
            info!("📥 Không có dữ liệu cũ. Đang đồng bộ {} ngày dữ liệu lịch sử cho {}...", default_limit_days, symbol_upper);
        }
        
        let mut current_start = start_target;
        let mut total_fetched = 0;
        let mut all_fetched_candles = Vec::new();

        loop {
            // 1. Kiểm tra Rate Limit trước khi gọi (Ước tính cost 5 cho klines)
            self.monitor.wait_if_needed(5).await;

            let params = KlineCandlestickDataParams::builder(
                symbol_upper.clone(),
                KlineCandlestickDataIntervalEnum::Interval1m,
            )
            .start_time(current_start)
            .limit(1000)
            .build()?;

            let response = self.rest_client.kline_candlestick_data(params).await?;
            
            // 2. Cập nhật Weight Monitor từ Response
            if let Some(ref limits) = response.rate_limits {
                for limit in limits {
                    if limit.rate_limit_type == RateLimitType::RequestWeight && limit.interval == BinanceInterval::Minute {
                        self.monitor.update(limit.count);
                    }
                }
            }

            let data = response.data().await?;
            if data.is_empty() { break; }

            let mut candles = Vec::new();
            let mut last_timestamp = 0;

            let to_i64 = |item: &KlineCandlestickDataResponseItemInner| -> Option<i64> {
                match item {
                    KlineCandlestickDataResponseItemInner::Integer(v) => Some(*v),
                    KlineCandlestickDataResponseItemInner::String(s) => s.parse().ok(),
                    _ => None,
                }
            };

            let to_f64 = |item: &KlineCandlestickDataResponseItemInner| -> Option<f64> {
                match item {
                    KlineCandlestickDataResponseItemInner::String(s) => s.parse().ok(),
                    KlineCandlestickDataResponseItemInner::Integer(v) => Some(*v as f64),
                    _ => None,
                }
            };

            for item_array in data {
                if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                    item_array.get(0).and_then(to_i64),
                    item_array.get(1).and_then(to_f64),
                    item_array.get(2).and_then(to_f64),
                    item_array.get(3).and_then(to_f64),
                    item_array.get(4).and_then(to_f64),
                    item_array.get(5).and_then(to_f64),
                ) {
                    candles.push(Candle {
                        symbol: symbol_upper.clone(),
                        timestamp: t,
                        open: o,
                        high: h, low: l, close: c, volume: v,
                    });
                    last_timestamp = t;
                }
            }

            if candles.is_empty() { break; }

            let count = candles.len();
            
            // Push clone to all_fetched_candles
            all_fetched_candles.extend(candles.clone());
            
            if let Err(e) = self.db.save_candles(candles).await {
                error!("❌ Lỗi lưu batch nến vào DB: {}", e);
            }

            total_fetched += count;
            debug!("  + Đã tải {} nến. Tổng cộng: {}", count, total_fetched);
            self.monitor.report_status();

            current_start = last_timestamp + 60000;
            if current_start >= now { break; }
            
            // Không cần sleep cứng nữa, Guardian đã lo
        }

        info!("✅ Hoàn tất đồng bộ lịch sử về hiện tại. Tổng số nến: {}", total_fetched);
        Ok(all_fetched_candles)
    }

    /// Đảm bảo trong CSDL có đủ số ngày dữ liệu quá khứ. Nếu chưa đủ, fetch lùi về quá khứ.
    pub async fn ensure_history(&self, symbol: &str, target_days: i64) -> Result<(), Box<dyn std::error::Error>> {
        let symbol_upper = symbol.to_uppercase();
        let target_candles = (target_days * 24 * 60) as usize; // 1m = 1440 candles / day
        
        // 1. Đếm số nến hiện tại
        let current_count = self.db.count_candles(&symbol_upper).await?;
        if current_count >= target_candles {
            info!("✅ [API] Dữ liệu lịch sử đã đủ ({} nến >= {} yêu cầu). Bỏ qua Backward Sync.", current_count, target_candles);
            return Ok(());
        }

        info!("📥 [API] Dữ liệu lịch sử chưa đủ ({} / {}). Bắt đầu đồng bộ lùi về quá khứ...", current_count, target_candles);

        // 2. Tìm nến cũ nhất để fetch ngược
        let oldest_candle = self.db.get_oldest_candle(&symbol_upper).await?;
        let now = chrono::Utc::now().timestamp_millis();
        
        let mut current_end_time = if let Some(oldest) = oldest_candle {
            oldest.timestamp - 60000 // Nến trước nến cũ nhất
        } else {
            now
        };

        // Target start time based on 'now'
        let absolute_start_target = now - (target_days * 24 * 60 * 60 * 1000);
        let mut total_fetched = 0;

        loop {
            if current_end_time <= absolute_start_target {
                break;
            }

            // Gọi tối đa 1000 nến lùi từ current_end_time
            let start_time = current_end_time - (1000 * 60 * 1000); // Lùi 1000 phút
            let fetch_start = std::cmp::max(start_time, absolute_start_target);

            self.monitor.wait_if_needed(5).await;

            let params = KlineCandlestickDataParams::builder(
                symbol_upper.clone(),
                KlineCandlestickDataIntervalEnum::Interval1m,
            )
            .start_time(fetch_start)
            .end_time(current_end_time)
            .limit(1000)
            .build()?;

            let response = self.rest_client.kline_candlestick_data(params).await?;
            
            if let Some(ref limits) = response.rate_limits {
                for limit in limits {
                    if limit.rate_limit_type == RateLimitType::RequestWeight && limit.interval == BinanceInterval::Minute {
                        self.monitor.update(limit.count);
                    }
                }
            }

            let data = response.data().await?;
            if data.is_empty() { break; }

            let mut candles = Vec::new();
            let mut first_timestamp = current_end_time;

            let to_i64 = |item: &KlineCandlestickDataResponseItemInner| -> Option<i64> {
                match item {
                    KlineCandlestickDataResponseItemInner::Integer(v) => Some(*v),
                    KlineCandlestickDataResponseItemInner::String(s) => s.parse().ok(),
                    _ => None,
                }
            };
            let to_f64 = |item: &KlineCandlestickDataResponseItemInner| -> Option<f64> {
                match item {
                    KlineCandlestickDataResponseItemInner::String(s) => s.parse().ok(),
                    KlineCandlestickDataResponseItemInner::Integer(v) => Some(*v as f64),
                    _ => None,
                }
            };

            for item_array in data {
                if let (Some(t), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                    item_array.get(0).and_then(to_i64),
                    item_array.get(1).and_then(to_f64),
                    item_array.get(2).and_then(to_f64),
                    item_array.get(3).and_then(to_f64),
                    item_array.get(4).and_then(to_f64),
                    item_array.get(5).and_then(to_f64),
                ) {
                    candles.push(Candle {
                        symbol: symbol_upper.clone(),
                        timestamp: t,
                        open: o, high: h, low: l, close: c, volume: v,
                    });
                    if t < first_timestamp {
                        first_timestamp = t;
                    }
                }
            }

            if candles.is_empty() { break; }

            let count = candles.len();
            if let Err(e) = self.db.save_candles(candles).await {
                error!("❌ Lỗi lưu batch nến Backward vào DB: {}", e);
            }

            total_fetched += count;
            debug!("  + Đã tải lùi {} nến. Tổng cộng: {}", count, total_fetched);
            self.monitor.report_status();

            current_end_time = first_timestamp - 60000;
        }

        info!("✅ [API] Hoàn tất Backward Sync. Đã kéo thêm: {} nến.", total_fetched);
        Ok(())
    }

    pub async fn fetch_market_filter(&self, symbol: &str) -> Result<crate::models::market_info::MarketFilter, Box<dyn std::error::Error>> {
        self.monitor.wait_if_needed(1).await;
        
        let info = self.rest_client.exchange_information().await?;
        if let Some(weight_str) = info.headers.get("X-MBX-USED-WEIGHT-1M").or_else(|| info.headers.get("x-mbx-used-weight-1m")) {
            if let Ok(weight) = weight_str.parse::<u32>() {
                self.monitor.update(weight);
            }
        }
        
        let response = info.data().await?;
        let symbols = response.symbols.unwrap_or_default();
        let target_symbol = symbol.to_uppercase();
        
        for s in symbols {
            if let Some(s_name) = &s.symbol {
                if s_name == &target_symbol {
                    let mut filter_info = crate::models::market_info::MarketFilter::default();
                    if let Some(filters) = s.filters {
                        for f in filters {
                            if let Some(t) = f.filter_type {
                                match t.as_str() {
                                    "PRICE_FILTER" => {
                                        if let Some(tick) = f.tick_size {
                                            filter_info.tick_size = tick.parse().unwrap_or(0.1);
                                        }
                                    }
                                    "LOT_SIZE" => {
                                        if let Some(step) = f.step_size {
                                            filter_info.step_size = step.parse().unwrap_or(0.001);
                                        }
                                        if let Some(min) = f.min_qty {
                                            filter_info.min_qty = min.parse().unwrap_or(0.001);
                                        }
                                        if let Some(max) = f.max_qty {
                                            filter_info.max_qty = max.parse().unwrap_or(1000000.0);
                                        }
                                    }
                                    "MIN_NOTIONAL" => {
                                        if let Some(notional) = f.notional {
                                            filter_info.min_notional = notional.parse().unwrap_or(5.0);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    return Ok(filter_info);
                }
            }
        }
        
        Err(format!("Symbol {} not found in exchangeInfo", symbol).into())
    }
}
/// Tính toán thời điểm bắt đầu đồng bộ dựa vào nến cuối cùng trong DB
pub fn calculate_sync_start(latest_candle_time: Option<i64>, default_limit_days: i64, now_ms: i64) -> i64 {
    if let Some(timestamp) = latest_candle_time {
        timestamp + 60000 // Nến tiếp theo
    } else {
        now_ms - (default_limit_days * 24 * 60 * 60 * 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sync_start_with_gap() {
        let now = 1000000000; // Giả lập Now
        let latest = 999900000;
        
        // Nếu có nến cũ, nó phải cộng thêm 60,000ms
        let start = calculate_sync_start(Some(latest), 30, now);
        assert_eq!(start, 999960000);
    }

    #[test]
    fn test_calculate_sync_start_empty_db() {
        let now = 1000000000;
        let days = 1;
        
        // Nếu DB rỗng, nó lấy 1 ngày trước đó
        let start = calculate_sync_start(None, days, now);
        assert_eq!(start, now - (1 * 24 * 60 * 60 * 1000));
    }
}
