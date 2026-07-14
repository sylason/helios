use std::sync::Arc;
use std::cell::RefCell;
use polars::prelude::*;
use ferro_trade::broker::Broker;
use ferro_trade::backtest::manager::IndicatorManager;
use ferro_trade::backtest::context::{InitContext, TickContext};
use ferro_trade::strategy::Strategy;
use ferro_trade::broker::actions::Action;
use ferro_trade::stats::{Stats, StatsInput, compute_stats};
use smallvec::SmallVec;
use crate::models::models::Candle;
use tracing::{debug, error};

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

        // 1. Chuyá»ƒn Ä‘á»•i dá»¯ liá»‡u sang DataFrame
        let df = match Self::candles_to_df(candles) {
            Ok(df) => df,
            Err(e) => {
                error!("âŒ HeliosBridge: Lá»—i chuyá»ƒn Ä‘á»•i DataFrame: {}", e);
                return (Vec::new(), None);
            }
        };

        // 2. Chuáº©n bá»‹ mÃ´i trÆ°á»ng tÃ­nh toÃ¡n (Giá»‘ng ferro_trade::backtest::engine)
        let cache = Arc::new(ferro_trade::data::global_cache::GlobalCache::new(ferro_trade::data::global_cache::CacheConfig::default()));
        let raw_cache = match ferro_trade::data::RawDataCache::new(&df) {
            Ok(c) => c,
            Err(e) => {
                error!("âŒ HeliosBridge: Lá»—i táº¡o RawDataCache: {}", e);
                return (Vec::new(), None);
            }
        };
        let data_view = ferro_trade::data::DataView::new(df, raw_cache);
        
        // 2.5 TIá»€N TÃNH TOÃN MTF (Resampling) cho cÃ¡c Timeframe chuáº©n
        let base_times = &data_view.raw_cache().datetime;
        let base_df = data_view.get_full_df();
        for tf_str in &["3m", "5m", "15m", "1h", "4h"] {
            let tf = std::str::FromStr::from_str(tf_str).expect("Invalid timeframe");
            let df_res = ferro_trade::utils::resample::resample_ohlcv(tf_str, base_df).expect("Precompute resample fail");
            let flat_ohlcv = Arc::new(ferro_trade::utils::resample::df_to_resampled_ohlcv(&df_res).expect("Precompute convert fail"));
            cache.put_resampled_ohlcv(tf, flat_ohlcv.clone());

            let table = Arc::new(ferro_trade::utils::resample::create_lookup_table(base_times, &flat_ohlcv.datetime));
            cache.put_lookup(tf, table);
        }

        
        // 3. Khá»Ÿi táº¡o Broker (Simulated)
        let mut broker = Broker::new(
            data_view.clone(),
            10000.0, // Initial Cash (khÃ´ng quan trá»ng cho viá»‡c láº¥y signal)
            1.0,     // Leverage
            1.0,     // Margin rate
            Box::new(|_, _| 0.0), // Zero commission cho simulation
            0.0,     // Spread
            false,   // trade_on_close
            false,   // hedging
            true,    // exclusive_orders
            None,    // max_position_size
        );

        // 4. Khá»Ÿi táº¡o Strategy vÃ  Indicators
        let mut indicator_manager = IndicatorManager::new();
        let mut init_context = InitContext {
            raw_data: broker.data.raw_cache(),
            data_view: &broker.data,
            cache: &cache,
            indicator_manager: &mut indicator_manager,
        };

        strategy.init(&mut init_context);

        let warmup = indicator_manager.warmup_period;
        let full_len = broker.data.get_full_df().height();
        
        if full_len <= warmup {
            debug!("âš ï¸ HeliosBridge: Dá»¯ liá»‡u chÆ°a Ä‘á»§ thá»i gian Warmup ({} < {}).", full_len, warmup);
            return (Vec::new(), None);
        }

        let actions_buffer = RefCell::new(SmallVec::<[Action; 4]>::new());
        let mut last_actions = Vec::new();

        // 5. VÃ²ng láº·p mÃ´ phá»ng (Simulated Loop)
        // ChÃºng ta cháº¡y tá»« warmup + 1 Ä‘áº¿n náº¿n cuá»‘i cÃ¹ng.
        for i in (warmup + 1)..full_len {
            broker.data.set_len(i + 1);
            if broker.next(i).is_err() { break; }

            let position_snapshot = broker.position();
            let current_equity = broker.equity();
            
            {
                let tick_context = TickContext {
                    i,
                    equity: current_equity,
                    data_view: &broker.data,
                    indicator_manager: &indicator_manager,
                    trades: &broker.trades,
                    actions: &actions_buffer,
                    position: position_snapshot,
                };
                strategy.next(&tick_context);
            }
            
            let mut actions = actions_buffer.borrow_mut();
            if !actions.is_empty() {
                // Náº¿u Ä‘Ã¢y lÃ  náº¿n cuá»‘i cÃ¹ng, ta báº¯t láº¥y cÃ¡c action nÃ y!
                if i == full_len - 1 {
                    last_actions = actions.to_vec();
                }
                broker.execute_actions(std::mem::take(&mut *actions));
            }
        }

        // 6. TÃ­nh toÃ¡n thá»‘ng kÃª trÃªn toÃ n bá»™ dá»¯ liá»‡u (Diagnostic)
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

    /// Chuyá»ƒn Ä‘á»•i danh sÃ¡ch Candle (shared model) sang DataFrame (ferro_trade format)
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

    /// [DIAGNOSTIC] Cháº¡y láº¡i toÃ n bá»™ pipeline Backtest giá»‘ng há»‡t nhÆ° optimizer/v2_run_helios
    /// Sá»­ dá»¥ng bá»™ DNA Ä‘Ã£ tá»‘i Æ°u vÃ  `BacktestBuilder` cá»§a `ferro_trade` Ä‘á»ƒ Ä‘áº£m báº£o káº¿t quáº£ 1:1.
    pub fn diagnostic_backtest(
        candles: &[Candle],
        dna: &[f64],
    ) -> Result<ferro_trade::backtest::engine::BacktestResult, String> {
        if candles.is_empty() {
            return Err("Empty candles".into());
        }

        // 1. Chuyển đổi DataFrame
        let df = Self::candles_to_df(candles).map_err(|e| e.to_string())?;

        // 2. Khởi tạo Backtest giống hệt prospector_runner
        let backtest = ferro_trade::backtest::BacktestBuilder::new()
            .data(df)
            .cash(60.0) // INITIAL_CASH
            .commission(0.0004) // COMMISSION
            .margin(1.0 / 20.0) // MARGIN
            .fractional_unit(Some(0.0001)) // FRACTIONAL_UNIT
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
        prospector_config.available_timeframes = vec!["3m".into(), "5m".into(), "15m".into(), "1h".into(), "4h".into()];
        let prospector = ferro_trade::optimizer::prospector::Prospector::new(prospector_config)
            .map_err(|e| e.to_string())?;
        
        prospector.precompute_mtf_data(&backtest);

        // 4. Run với DNA
        let result = backtest.run_with_dna::<helios::strategy::HeliosStrategy>(dna)
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}
