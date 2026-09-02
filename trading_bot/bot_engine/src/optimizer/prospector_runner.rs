use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{info, warn};

use serde::Deserialize;

use ferro_trade::backtest::{BacktestBuilder, EarlyTermination, TerminationReason};
use ferro_trade::optimizer::prospector::{Prospector, ProspectorConfig};
use ferro_trade::optimizer::params::DnaMappable;
use ferro_trade::stats::StatItem;
use helios::strategy::core_v3::HeliosStrategyV3;
use polars::prelude::*;


use crate::models::repository::{CandleRepository, StrategyRepository};
use crate::data_engine::sqlite_impl::SqliteStore;

/// Cấu hình tài khoản và môi trường khớp lệnh cho Optimizer (Single Source of Truth)
#[derive(Debug, Clone, Deserialize)]
pub struct AccountOptimizerConfig {
    #[serde(default = "default_initial_cash")]
    pub initial_cash: f64,
    #[serde(default = "default_commission")]
    pub commission_rate: f64,
    #[serde(default = "default_margin")]
    pub margin_rate: f64,
    #[serde(default = "default_fractional_unit")]
    pub fractional_unit: Option<f64>,
}

fn default_initial_cash() -> f64 { 60.0 }
fn default_commission() -> f64 { 0.0004 }
fn default_margin() -> f64 { 1.0 / 20.0 }
fn default_fractional_unit() -> Option<f64> { Some(0.0001) }

impl Default for AccountOptimizerConfig {
    fn default() -> Self {
        Self {
            initial_cash: default_initial_cash(),
            commission_rate: default_commission(),
            margin_rate: default_margin(),
            fractional_unit: default_fractional_unit(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FullOptimizerConfig {
    pub prospector: ProspectorConfig,
    #[serde(default)]
    pub account: Option<AccountOptimizerConfig>,
    pub cache_config: Option<ferro_trade::data::global_cache::CacheConfig>,
}

pub struct ProspectorRunner;

impl ProspectorRunner {
    /// Chạy 1 chu kỳ tối ưu hóa dựa trên 30 ngày dữ liệu gần nhất
    pub async fn run_optimization_cycle(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp_millis();
        let thirty_days_ms = 30 * 24 * 60 * 60 * 1000;
        let start_time = now - thirty_days_ms;
        Self::run_optimization_cycle_range(symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, now).await
    }

    /// Chạy 1 chu kỳ tối ưu hóa 30 ngày gần nhất kèm Smart Seeding (tự động lấy DNA tốt nhất từ DB làm hạt giống mồi)
    pub async fn run_optimization_cycle_smart_seed(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp_millis();
        let thirty_days_ms = 30 * 24 * 60 * 60 * 1000;
        let start_time = now - thirty_days_ms;

        let seed_dna = match db.get_best_dna(symbol, strategy_id).await {
            Ok(Some(dna_data)) => {
                info!("🌱 [Smart Seeding] Đã chọn Rank 1 Champion DNA từ DB làm hạt giống mồi.");
                Some(dna_data.dna)
            }
            _ => None,
        };

        Self::run_optimization_cycle_range_smart_seed(
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, now, seed_dna
        ).await
    }

    /// Chạy 1 chu kỳ tối ưu hóa kèm Smart Seeding (Tiêm 1 cá thể Rank 1 Champion vào Generation 0)
    pub async fn run_optimization_cycle_range_smart_seed(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
        start_time: i64,
        end_time: i64,
        seed_dna: Option<Vec<f64>>,
    ) -> Result<(), String> {
        Self::run_optimization_cycle_range_internal(
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, end_time, seed_dna
        ).await
    }

    /// Chạy 1 chu kỳ tối ưu hóa dựa trên khoảng thời gian chỉ định [start_time -> end_time]
    pub async fn run_optimization_cycle_range(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
        start_time: i64,
        end_time: i64,
    ) -> Result<(), String> {
        Self::run_optimization_cycle_range_internal(
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, end_time, None
        ).await
    }

    async fn run_optimization_cycle_range_internal(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
        start_time: i64,
        end_time: i64,
        seed_dna: Option<Vec<f64>>,
    ) -> Result<(), String> {
        info!("🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho {} (Khoảng MS: {} -> {})...", symbol, start_time, end_time);

        let opt_start_timer = std::time::Instant::now();
        let candles = db.get_candles_by_range(symbol, start_time, end_time).await.map_err(|e| e.to_string())?;
        if candles.is_empty() {
            warn!("⚠️ [Optimizer] Không có dữ liệu nến cho {} để tối ưu hóa.", symbol);
            return Ok(());
        }

        // Bổ sung: Bảo vệ thuật toán khỏi việc thiếu dữ liệu (cần ít nhất 5000 nến để các Indicator dài hạn không bị crash)
        if candles.len() < 5000 {
            warn!("⚠️ [Optimizer] Không đủ dữ liệu nến (hiện có {}, cần > 5000) để tối ưu hóa. Bỏ qua chu kỳ này.", candles.len());
            return Ok(());
        }

        let num_candles = candles.len();
        info!("🧬 [Optimizer] Đã lấy {} nến từ Database. Bắt đầu tối ưu hóa...", num_candles);

        // 1. Load Cấu hình Prospector & Cache từ file JSON trước
        let file = File::open(config_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let full_config: FullOptimizerConfig = serde_json::from_reader(reader).map_err(|e| e.to_string())?;

        // 2. Tách toàn bộ việc dựng DataFrame, Backtest & chạy GA sang luồng Blocking riêng của Tokio (tránh nghẽn Async Event Loop)
        let prospector_db_path_owned = prospector_db_path.to_string();

        let champion_opt = tokio::task::spawn_blocking(move || {
            let df = Self::candles_to_df(&candles).map_err(|e| e.to_string())?;

            let mut prospector_config = full_config.prospector;
            prospector_config.db_path = prospector_db_path_owned.clone();

            let account = full_config.account.unwrap_or_default();
            let backtest = BacktestBuilder::new()
                .data(df)
                .timeframes(&prospector_config.available_timeframes)
                .cash(account.initial_cash)
                .commission(account.commission_rate)
                .margin(account.margin_rate)
                .fractional_unit(account.fractional_unit)
                .trade_on_close(true)
                .exclusive_orders(true)
                .hedging(false)
                .finalize_trades(true)
                .cache_config(full_config.cache_config.clone().unwrap_or_default())
                .calculate_metrics(&[
                    StatItem::ReturnPct,
                    StatItem::TotalTrades,
                    StatItem::WinRatePct,
                    StatItem::BestTradePct,
                    StatItem::WorstTradePct,
                    StatItem::MaxDrawdownPct,
                    StatItem::SharpeRatio,
                    StatItem::SortinoRatio,
                    StatItem::ProfitFactor,
                    StatItem::Sqn,
                    StatItem::Expectancy,
                ])
                .early_termination(EarlyTermination::Any(vec![
                    EarlyTermination::MaxDrawdown(40.0),
                    EarlyTermination::WarmupMinTrades { after_bars: 14400, min_trades: 5 },
                ]))
                .early_termination_penalty(|term_info| {
                    match &term_info.reason {
                        TerminationReason::MaxDrawdownExceeded { current_pct, .. } => {
                            -5000.0 - current_pct
                        }
                        TerminationReason::WarmupStagnant { bars_checked, trades_count, .. } => {
                            let days = *bars_checked as f64 / 1440.0;
                            let trades_per_day = *trades_count as f64 / days;
                            -2000.0 * (1.0 - trades_per_day)
                        }
                        _ => -5000.0 - (term_info.total_bars - term_info.bar_index) as f64 * 0.1,
                    }
                })
                .build().map_err(|e| e.to_string())?;


            if let Some(parent) = Path::new(&prospector_db_path_owned).parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let db_exists = Path::new(&prospector_db_path_owned).exists();
            let mut prospector = Prospector::new(prospector_config).map_err(|e| e.to_string())?;

            if let Some(dna) = seed_dna {
                let mut champs = prospector.champions.write();
                champs.push(ferro_trade::optimizer::ga_worker::Candidate {
                    params: std::collections::HashMap::new(),
                    fitness: 0.0,
                    stats: ferro_trade::stats::PackedStats::default(),
                    dna: Some(dna),
                });
                info!("🌱 [Smart Seeding] Tiêm 1 hạt giống Rank 1 Champion từ chu kỳ trước vào Generation 0.");
            }

            if !db_exists || prospector.backlog_storage.get_max_job_id() == 0 {
                let flat_search_space = helios::strategy::core_v3::config::HeliosConfigV3::parameter_defs();
                prospector.add_initial_job(flat_search_space);
            }

            let objective = |stats: &ferro_trade::stats::Stats| {
                Self::calculate_enhanced_fitness(stats)
            };

            info!("🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...");
            prospector.run::<HeliosStrategyV3, _>(backtest, objective);

            let champs = prospector.champions.read();
            let top_champ = champs.iter().find(|c| !c.params.is_empty()).cloned().or_else(|| champs.first().cloned());
            Ok::<_, String>(top_champ)
        }).await.map_err(|e| e.to_string())??;

        let opt_elapsed = opt_start_timer.elapsed();

        if let Some(champion) = champion_opt {
            let ret = champion.stats.return_pct;
            let dd = champion.stats.max_drawdown_pct;
            let trades = champion.stats.total_trades;
            let win_rate = champion.stats.win_rate_pct;

            info!("⏱️ [OPTIMIZER DURATION] {:.2?} | Candles: {} | Best Return: {:.2}% | Max DD: {:.2}% | Trades: {} (Win: {:.1}%)", 
                  opt_elapsed, num_candles, ret, dd, trades, win_rate);

            // Lưu DNA vào bảng optimized_params
            if let Some(dna) = champion.dna {
                use crate::models::models::OptimizedDna;
                use std::collections::HashMap;

                let now = chrono::Utc::now().timestamp_millis();
                let mut stats_map = HashMap::new();
                stats_map.insert("Return %".to_string(), champion.stats.return_pct);
                stats_map.insert("Max DD %".to_string(), champion.stats.max_drawdown_pct);
                stats_map.insert("Trades".to_string(), champion.stats.total_trades as f64);
                stats_map.insert("Profit Factor".to_string(), champion.stats.profit_factor);
                stats_map.insert("Sharpe".to_string(), champion.stats.sharpe_ratio);

                let opt_dna = OptimizedDna {
                    symbol: symbol.to_string(),
                    strategy_id: strategy_id.to_string(),
                    dna: dna,
                    mapping: HashMap::new(), // Không cần mapping với DNA v2
                    fitness: ret,
                    stats: stats_map,
                    updated_at: now,
                };

                db.save_best_dna(opt_dna.clone()).await.map_err(|e| e.to_string())?;

                // Phát tín hiệu Event-Driven
                if let Some(tx) = &dna_tx {
                    let _ = tx.send(Some(opt_dna));
                }
                info!("✨ [OPTIMIZER] New DNA deployed to Actor (Fitness: {:.2}%).", ret);
            }
        } else {
            warn!("⚠️ [Optimizer] Không tìm thấy Champion nào có Return > 0 (Thời gian chạy: {:.2?}).", opt_elapsed);
        }

        Ok(())
    }

    /// Clone hàm chuyển đổi DataFrame
    fn candles_to_df(candles: &[crate::models::models::Candle]) -> PolarsResult<DataFrame> {
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
            s_dt.into(),
            s_open.into(),
            s_high.into(),
            s_low.into(),
            s_close.into(),
            s_vol.into(),
        ])
    }

    /// Tính toán điểm số Fitness đa tiêu chí (Enhanced Calmar, Profit Factor, SQN, Trade Frequency)
    pub fn calculate_enhanced_fitness(stats: &ferro_trade::stats::Stats) -> f64 {
        let ret = stats.return_pct;
        let dd = stats.max_drawdown_pct.abs();
        let trades = stats.total_trades;

        // 1. Phòng vệ Edge Cases: Cần ít nhất 2 lệnh độc lập và lợi nhuận dương
        if ret.is_nan() || ret <= 0.0 || trades < 2 {
            return -100.0 - dd;
        }

        // 2. Điểm số nền tảng (Smoothed Calmar)
        let base_score = ret / (dd + 3.0);

        // 3. Hệ số Profit Factor (PF Multiplier): Thưởng cho hệ thống có biên độ lãi dày
        let pf = stats.profit_factor;
        let pf_mult = if pf.is_nan() || pf <= 0.0 {
            0.20
        } else if pf < 1.15 {
            (pf / 1.5).max(0.20)
        } else if pf < 1.50 {
            0.80 + (pf - 1.15) * 0.5714
        } else {
            (1.0 + (pf - 1.50) * 0.40).min(2.0)
        };

        // 4. Hệ số Độ Ổn Định Toán Học SQN (System Quality Number Multiplier)
        let sqn = stats.sqn;
        let sqn_mult = if sqn.is_nan() || sqn <= 0.0 {
            1.0
        } else {
            1.0 + 0.10 * sqn.clamp(0.0, 5.0)
        };

        // 5. Trọng Số Tần Suất Giao Dịch (Trade Frequency Weight)
        let freq_weight = if trades < 8 {
            let ratio = trades as f64 / 8.0;
            ratio * ratio
        } else if trades <= 55 {
            1.0
        } else {
            (1.0 - (trades - 55) as f64 * 0.008).max(0.70)
        };

        base_score * pf_mult * sqn_mult * freq_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candles_to_df() {
        let candles = vec![
            crate::models::models::Candle {
                symbol: "BTCUSDT".to_string(),
                timestamp: 1600000000,
                open: 1.0,
                high: 2.0,
                low: 0.5,
                close: 1.5,
                volume: 100.0,
            }
        ];
        
        let df = ProspectorRunner::candles_to_df(&candles).unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 6);
    }

    #[test]
    fn test_fitness_negative_or_nan() {
        let mut stats = ferro_trade::stats::Stats::default();
        stats.return_pct = -10.0;
        stats.max_drawdown_pct = 15.0;
        stats.total_trades = 20;

        let fitness = ProspectorRunner::calculate_enhanced_fitness(&stats);
        assert!(fitness < 0.0, "Lợi nhuận âm phải trả về điểm phạt âm");

        stats.return_pct = f64::NAN;
        let fitness_nan = ProspectorRunner::calculate_enhanced_fitness(&stats);
        assert!(fitness_nan < 0.0, "Lợi nhuận NaN phải trả về điểm phạt âm");
    }

    #[test]
    fn test_fitness_low_trades_penalty() {
        let mut stats_few = ferro_trade::stats::Stats::default();
        stats_few.return_pct = 100.0;
        stats_few.max_drawdown_pct = 10.0;
        stats_few.total_trades = 4; // Quá ít lệnh (< 8)
        stats_few.profit_factor = 2.0;
        stats_few.sqn = 3.0;

        let mut stats_optimal = stats_few.clone();
        stats_optimal.total_trades = 25; // Số lệnh tối ưu

        let fit_few = ProspectorRunner::calculate_enhanced_fitness(&stats_few);
        let fit_optimal = ProspectorRunner::calculate_enhanced_fitness(&stats_optimal);

        assert!(fit_few < fit_optimal, "Ít lệnh phải bị phạt trọng số tần suất so với số lệnh tối ưu");
    }

    #[test]
    fn test_fitness_high_pf_and_sqn_boost() {
        let mut stats_solid = ferro_trade::stats::Stats::default();
        stats_solid.return_pct = 200.0;
        stats_solid.max_drawdown_pct = 10.0;
        stats_solid.total_trades = 30;
        stats_solid.profit_factor = 2.5; // PF rất dày
        stats_solid.sqn = 3.5;          // SQN rất ổn định

        let mut stats_fragile = stats_solid.clone();
        stats_fragile.profit_factor = 1.05; // PF mỏng manh
        stats_fragile.sqn = 0.5;

        let fit_solid = ProspectorRunner::calculate_enhanced_fitness(&stats_solid);
        let fit_fragile = ProspectorRunner::calculate_enhanced_fitness(&stats_fragile);

        assert!(fit_solid > fit_fragile * 2.0, "Hệ thống có PF dày và SQN cao phải có điểm số vượt trội");
    }
}
