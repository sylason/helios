use std::sync::Arc;
use polars::prelude::*;
use ferro_trade::broker::Broker;
use ferro_trade::backtest::manager::IndicatorManager;
use ferro_trade::backtest::context::{InitContext, TickContext};
use ferro_trade::strategy::Strategy;
use ferro_trade::broker::actions::Action;
use ferro_trade::stats::{Stats, StatsInput, compute_stats};
use ferro_trade::data::Timeframe;
use smallvec::SmallVec;
use crate::models::models::Candle;
use tracing::{debug, error};

/// Danh sách các khung thời gian tiêu chuẩn cho chiến lược Helios V3
pub const HELIOS_REQUIRED_TIMEFRAMES: [Timeframe; 4] = [
    Timeframe::T5m,
    Timeframe::T15m,
    Timeframe::T1h,
    Timeframe::T4h,
];

pub struct HeliosBridge;

impl HeliosBridge {
    pub fn analyze(
        _symbol: &str,
        candles: &[Candle],
        strategy: &mut dyn Strategy,
    ) -> (Vec<Action>, Option<Stats>) {
        if candles.is_empty() {
            return (Vec::new(), None);
        }

        // 1. Chuyển đổi dữ liệu sang DataFrame
        let df = match Self::candles_to_df(candles) {
            Ok(df) => df,
            Err(e) => {
                error!("❌ HeliosBridge: Lỗi chuyển đổi DataFrame: {}", e);
                return (Vec::new(), None);
            }
        };

        // 2. Chuẩn bị môi trường tính toán (Giống ferro_trade::backtest::engine)
        let cache = Arc::new(ferro_trade::data::global_cache::GlobalCache::new(ferro_trade::data::global_cache::CacheConfig::default()));
        let raw_cache = match ferro_trade::data::RawDataCache::new(&df) {
            Ok(c) => c,
            Err(e) => {
                error!("❌ HeliosBridge: Lỗi tạo RawDataCache: {}", e);
                return (Vec::new(), None);
            }
        };

        // 2.5 TIỀN TÍNH TOÁN MTF (Resampling) theo danh sách Timeframe chuẩn từ ferro_trade
        let tf_list: Vec<String> = HELIOS_REQUIRED_TIMEFRAMES
            .iter()
            .map(|tf| tf.as_polars_rule().to_string())
            .collect();
        let _ = ferro_trade::utils::resample::precompute_timeframes(&tf_list, &df, &cache, &raw_cache.datetime);

        let data_view = ferro_trade::data::DataView::new(raw_cache);
        drop(df);
        
        // 3. Khởi tạo Broker (Simulated)
        let mut broker = Broker::new(
            data_view.clone(),
            10000.0, // Initial Cash (không quan trọng cho việc lấy signal)
            1.0,     // Leverage
            1.0,     // Margin rate
            Box::new(|_, _| 0.0), // Zero commission cho simulation
            0.0,     // Spread
            false,   // trade_on_close
            false,   // hedging
            true,    // exclusive_orders
            None,    // max_position_size
        );

        // 4. Khởi tạo Strategy và Indicators
        let mut indicator_manager = IndicatorManager::new();
        let mut init_context = InitContext {
            raw_data: broker.data.raw_cache(),
            data_view: &broker.data,
            cache: &cache,
            indicator_manager: &mut indicator_manager,
        };

        strategy.init(&mut init_context);

        let warmup = indicator_manager.warmup_period;
        let full_len = broker.data.total_len();
        
        if full_len <= warmup {
            debug!("⚠️ HeliosBridge: Dữ liệu chưa đủ thời gian Warmup ({} < {}).", full_len, warmup);
            return (Vec::new(), None);
        }

        // 5. Chạy mô phỏng từng cây nến từ warmup đến nến mới nhất
        let mut last_actions = Vec::new();
        let mut actions_buffer: SmallVec<[Action; 4]> = SmallVec::new();

        for i in warmup..full_len {
            broker.data.set_len(i + 1);
            let _ = broker.next(i);

            let position_snapshot = broker.position();
            let current_equity = broker.equity();
            actions_buffer.clear();
            
            {
                let mut tick_context = TickContext {
                    i,
                    equity: current_equity,
                    data_view: &broker.data,
                    indicator_manager: &indicator_manager,
                    trades: &broker.trades,
                    actions: &mut actions_buffer,
                    position: position_snapshot,
                };
                strategy.next(&mut tick_context);
            }
            
            if !actions_buffer.is_empty() {
                // Nếu đây là nến cuối cùng, ta bắt lấy các action này!
                if i == full_len - 1 {
                    last_actions = actions_buffer.to_vec();
                }
                broker.execute_actions(std::mem::take(&mut actions_buffer));
            }
        }

        // 6. Tính toán thống kê trên toàn bộ dữ liệu (Diagnostic)
        broker.close_position(1.0);
        let _ = broker.next(full_len - 1);

        let stats_opt = if !broker.closed_trades.is_empty() {
            let input = StatsInput {
                initial_cash: broker.initial_cash,
                equity_curve: &broker.equity_curve,
                timestamps: broker.data.datetimes_slice(),
                close_prices: broker.data.closes_slice(),
                warmup_period: warmup,
                risk_free_rate: 0.0,
            };
            Some(compute_stats(&input, &broker.closed_trades, None))
        } else {
            None
        };

        (last_actions, stats_opt)
    }

    /// Đánh giá tín hiệu ở nến HIỆN TẠI (i = full_len - 1) với Vị thế thật từ Binance WebSocket
    pub fn evaluate_live_tick(
        symbol: &str,
        candles: &[Candle],
        slice_start: usize,
        live_position: Option<&crate::controller::asset_actor::LivePositionState>,
        strategy: &mut dyn Strategy,
        current_equity: f64,
    ) -> Vec<Action> {
        if candles.is_empty() {
            return Vec::new();
        }

        let df = match Self::candles_to_df(candles) {
            Ok(df) => df,
            Err(e) => {
                error!("❌ HeliosBridge: Lỗi chuyển đổi DataFrame: {}", e);
                return Vec::new();
            }
        };

        // Cache MTF Resampled Data & Lookup Tables theo mốc nến đóng của khung thời gian lớn
        static LIVE_MTF_STORE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, GlobalMtfCache>>> = std::sync::OnceLock::new();

        struct GlobalMtfCache {
            _symbol: String,
            last_candle_count: usize,
            _last_candle_ts: i64,
            cache: Arc<ferro_trade::data::global_cache::GlobalCache>,
            tf_buckets: std::collections::HashMap<String, i64>,
        }


        let last_candle_ts = candles.last().map(|c| c.timestamp).unwrap_or(0);
        let candles_len = candles.len();

        let mut store_map = LIVE_MTF_STORE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new())).lock().unwrap();
        let cache = if let Some(store) = store_map.get_mut(symbol) {
            if store.last_candle_count <= candles_len {
                store.cache.clone()
            } else {
                let new_cache = Arc::new(ferro_trade::data::global_cache::GlobalCache::new(ferro_trade::data::global_cache::CacheConfig::default()));
                *store = GlobalMtfCache {
                    _symbol: symbol.to_string(),
                    last_candle_count: candles_len,
                    _last_candle_ts: last_candle_ts,
                    cache: new_cache.clone(),
                    tf_buckets: std::collections::HashMap::new(),
                };
                new_cache
            }
        } else {
            let new_cache = Arc::new(ferro_trade::data::global_cache::GlobalCache::new(ferro_trade::data::global_cache::CacheConfig::default()));
            store_map.insert(symbol.to_string(), GlobalMtfCache {
                _symbol: symbol.to_string(),
                last_candle_count: candles_len,
                _last_candle_ts: last_candle_ts,
                cache: new_cache.clone(),
                tf_buckets: std::collections::HashMap::new(),
            });
            new_cache
        };


        let raw_cache = ferro_trade::data::RawDataCache::new(&df).expect("RawDataCache error");
        let base_times = &raw_cache.datetime;
        let base_df = &df;

        let store = store_map.get_mut(symbol).unwrap();

        for &tf in &HELIOS_REQUIRED_TIMEFRAMES {
            let current_bucket = tf.get_bucket_index(last_candle_ts);
            let prev_bucket = store.tf_buckets.get(&tf.to_string()).copied().unwrap_or(-1);

            let maybe_flat = if current_bucket != prev_bucket || cache.get_resampled_ohlcv(tf).is_none() {
                if let Ok(df_res) = ferro_trade::utils::resample::resample_ohlcv(tf.as_polars_rule(), base_df) {
                    if let Ok(flat_ohlcv) = ferro_trade::utils::resample::df_to_resampled_ohlcv(&df_res) {
                        let flat_arc = Arc::new(flat_ohlcv);
                        cache.put_resampled_ohlcv(tf, flat_arc.clone());
                        store.tf_buckets.insert(tf.to_string(), current_bucket);
                        Some(flat_arc)
                    } else { None }
                } else { None }
            } else {
                cache.get_resampled_ohlcv(tf)
            };

            if let Some(flat_arc) = maybe_flat {
                let table = Arc::new(ferro_trade::utils::resample::create_lookup_table(base_times, &flat_arc.datetime));
                cache.put_lookup(tf, table);
            }
        }
        drop(store_map);

        // [CRITICAL FIX] Xóa sạch cache indicators cũ để đảm bảo nến mới được tính toán chỉ báo tươi mới 100%
        cache.clear_indicators();

        let data_view = ferro_trade::data::DataView::new(raw_cache);
        drop(df);

        let full_len = data_view.total_len();
        if full_len < 200 {
            return Vec::new();
        }

        let mut indicator_manager = IndicatorManager::new();
        let mut init_context = InitContext {
            raw_data: data_view.raw_cache(),
            data_view: &data_view,
            cache: &cache,
            indicator_manager: &mut indicator_manager,
        };

        strategy.init(&mut init_context);

        let warmup = indicator_manager.warmup_period;
        if full_len <= warmup {
            return Vec::new();
        }

        let last_idx = full_len - 1;

        // Tiêm vị thế thật từ Binance vào TickContext
        let mut live_trades = Vec::new();
        let mut live_position_snapshot = ferro_trade::broker::position::Position::default();

        if let Some(pos) = live_position {
            if pos.position_amt != 0.0 {
                let mapped_entry_bar = pos.live_entry_bar.saturating_sub(slice_start);
                let mut trade = ferro_trade::broker::trade::Trade::new(
                    1,
                    pos.position_amt,
                    pos.entry_price,
                    mapped_entry_bar.min(last_idx),
                    None,
                    pos.opening_playbook_tag.clone(),
                );
                trade.sl_price = pos.current_sl_price;
                trade.tp_price = pos.current_tp_price;
                trade.sl_order_id = if pos.current_sl_price.is_some() { Some(1) } else { None };
                trade.tp_order_id = if pos.current_tp_price.is_some() { Some(2) } else { None };
                live_trades.push(trade);


                live_position_snapshot = ferro_trade::broker::position::Position {
                    size: pos.position_amt,
                    pnl: 0.0,
                    pnl_pct: 0.0,
                    is_long: pos.position_amt > 0.0,
                    is_short: pos.position_amt < 0.0,
                };
            }
        }

        let mut actions_buffer: SmallVec<[Action; 4]> = SmallVec::new();

        {
            let mut tick_context = TickContext {
                i: last_idx,
                equity: current_equity,
                data_view: &data_view,
                indicator_manager: &indicator_manager,
                trades: &live_trades,
                actions: &mut actions_buffer,
                position: live_position_snapshot,
            };
            strategy.next(&mut tick_context);
        }

        actions_buffer.to_vec()
    }

    /// Chuyển đổi danh sách Candle (shared model) sang DataFrame (ferro_trade format)
    fn candles_to_df(candles: &[Candle]) -> PolarsResult<DataFrame> {
        let n = candles.len();
        let mut dt_vec = Vec::with_capacity(n);
        let mut open_vec = Vec::with_capacity(n);
        let mut high_vec = Vec::with_capacity(n);
        let mut low_vec = Vec::with_capacity(n);
        let mut close_vec = Vec::with_capacity(n);
        let mut vol_vec = Vec::with_capacity(n);

        for c in candles {
            dt_vec.push(c.timestamp * 1_000_000);
            open_vec.push(c.open);
            high_vec.push(c.high);
            low_vec.push(c.low);
            close_vec.push(c.close);
            vol_vec.push(c.volume);
        }

        let s_dt = Series::new("Datetime".into(), dt_vec)
            .cast(&DataType::Datetime(TimeUnit::Nanoseconds, None))?;
        let s_open = Series::new("Open".into(), open_vec);
        let s_high = Series::new("High".into(), high_vec);
        let s_low = Series::new("Low".into(), low_vec);
        let s_close = Series::new("Close".into(), close_vec);
        let s_vol = Series::new("Volume".into(), vol_vec);

        DataFrame::new(vec![
            Column::from(s_dt),
            Column::from(s_open),
            Column::from(s_high),
            Column::from(s_low),
            Column::from(s_close),
            Column::from(s_vol),
        ])
    }

    /// [DIAGNOSTIC] Chạy lại toàn bộ pipeline Backtest giống hệt như optimizer/v2_run_helios
    /// Sử dụng bộ DNA đã tối ưu và `BacktestBuilder` của `ferro_trade` để đảm bảo kết quả 1:1.
    pub fn diagnostic_backtest(
        candles: &[Candle],
        dna: &[f64],
    ) -> Result<ferro_trade::backtest::engine::BacktestResult, String> {
        Self::diagnostic_backtest_with_config(candles, dna, &crate::models::config::EngineConfig::default())
    }

    /// [DIAGNOSTIC] Chạy lại toàn bộ pipeline Backtest với cấu hình EngineConfig động
    pub fn diagnostic_backtest_with_config(
        candles: &[Candle],
        dna: &[f64],
        config: &crate::models::config::EngineConfig,
    ) -> Result<ferro_trade::backtest::engine::BacktestResult, String> {
        if candles.is_empty() {
            return Err("Empty candles".into());
        }

        // Khóa Mutex đồng bộ hóa để tránh đụng độ IO lock file ferro_backlog.db khi nhiều luồng cùng gọi
        static DIAGNOSTIC_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = DIAGNOSTIC_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // 1. Chuyển đổi DataFrame
        let df = Self::candles_to_df(candles).map_err(|e| e.to_string())?;

        // 2. Khởi tạo Backtest giống hệt prospector_runner
        let available_tfs: Vec<String> = HELIOS_REQUIRED_TIMEFRAMES
            .iter()
            .map(|tf| tf.as_polars_rule().to_string())
            .collect();
        let backtest = ferro_trade::backtest::BacktestBuilder::new()
            .data(df)
            .timeframes(&available_tfs)
            .cash(config.initial_cash)
            .commission(config.commission_rate)
            .margin(config.margin_rate)
            .fractional_unit(config.fractional_unit)
            .trade_on_close(true)
            .exclusive_orders(true)
            .hedging(false)
            .finalize_trades(true)
            .calculate_metrics(&[
                ferro_trade::stats::StatItem::ReturnPct,
                ferro_trade::stats::StatItem::TotalTrades,
                ferro_trade::stats::StatItem::MaxDrawdownPct,
            ])
            .build()
            .map_err(|e| e.to_string())?;

        // 3. Tiền tính toán MTF bằng Prospector
        let mut prospector_config = ferro_trade::optimizer::prospector::ProspectorConfig::default();
        prospector_config.available_timeframes = available_tfs;
        let prospector = ferro_trade::optimizer::prospector::Prospector::new(prospector_config)
            .map_err(|e| e.to_string())?;
        
        prospector.precompute_mtf_data(&backtest);

        // 4. Run với DNA
        let result = backtest.run_with_dna::<helios::strategy::core_v3::HeliosStrategyV3>(dna)
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}
