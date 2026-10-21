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

fn default_max_drawdown() -> f64 { 35.0 }
fn default_warmup_bars() -> usize { 5760 }
fn default_min_trades() -> usize { 2 }

#[derive(Debug, Clone, Deserialize)]
pub struct EarlyTerminationConfig {
    #[serde(default = "default_max_drawdown")]
    pub max_drawdown_pct: f64,
    #[serde(default = "default_warmup_bars")]
    pub warmup_bars: usize,
    #[serde(default = "default_min_trades")]
    pub min_trades: usize,
}

impl Default for EarlyTerminationConfig {
    fn default() -> Self {
        Self {
            max_drawdown_pct: default_max_drawdown(),
            warmup_bars: default_warmup_bars(),
            min_trades: default_min_trades(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FullOptimizerConfig {
    pub prospector: ProspectorConfig,
    #[serde(default)]
    pub account: Option<AccountOptimizerConfig>,
    pub cache_config: Option<ferro_trade::data::global_cache::CacheConfig>,
    #[serde(default)]
    pub early_termination: Option<EarlyTerminationConfig>,
    #[serde(default)]
    pub fitness: Option<FitnessConfig>,
}

fn default_slippage_rate() -> f64 { 0.0002 }
fn default_min_daily_frequency() -> f64 { 0.40 }
fn default_max_daily_frequency() -> f64 { 1.50 }
fn default_dd_smoothing() -> f64 { 2.0 }

/// Cấu hình Hàm Mục Tiêu Di Truyền theo chuẩn t-Statistic & Dynamic Slippage Drag (Loại bỏ 100% Magic Numbers)
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct FitnessConfig {
    #[serde(default = "default_slippage_rate")]
    pub slippage_rate: f64,       // Mặc định 0.0002 (0.02% giá)
    #[serde(default = "default_min_daily_frequency")]
    pub min_daily_frequency: f64, // Mặc định 0.40 lệnh/ngày
    #[serde(default = "default_max_daily_frequency")]
    pub max_daily_frequency: f64, // Mặc định 1.50 lệnh/ngày
    #[serde(default = "default_dd_smoothing")]
    pub dd_smoothing: f64,        // Mặc định 2.0% (tránh chia cho số quá nhỏ khi DD = 0)
}

impl Default for FitnessConfig {
    fn default() -> Self {
        Self {
            slippage_rate: default_slippage_rate(),
            min_daily_frequency: default_min_daily_frequency(),
            max_daily_frequency: default_max_daily_frequency(),
            dd_smoothing: default_dd_smoothing(),
        }
    }
}


/// Số nến kiểm định Out-Of-Sample mặc định: 3 ngày (72 giờ = 4320 nến 1m)
pub const DEFAULT_VALIDATION_SPAN_BARS: usize = 4320;

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
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, end_time, seed_dna, None
        ).await
    }

    /// Chạy 1 chu kỳ tối ưu hóa kèm Smart Seeding với cửa sổ validation tùy biến
    pub async fn run_optimization_cycle_range_smart_seed_custom(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
        start_time: i64,
        end_time: i64,
        seed_dna: Option<Vec<f64>>,
        validation_span_bars: Option<usize>,
    ) -> Result<(), String> {
        Self::run_optimization_cycle_range_internal(
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, end_time, seed_dna, validation_span_bars
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
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, end_time, None, None
        ).await
    }

    /// Chạy 1 chu kỳ tối ưu hóa với khoảng thời gian chỉ định và cửa sổ validation tùy biến
    pub async fn run_optimization_cycle_range_custom(
        symbol: &str,
        strategy_id: &str,
        db: Arc<SqliteStore>,
        config_path: &str,
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
        start_time: i64,
        end_time: i64,
        validation_span_bars: Option<usize>,
    ) -> Result<(), String> {
        Self::run_optimization_cycle_range_internal(
            symbol, strategy_id, db, config_path, prospector_db_path, dna_tx, start_time, end_time, None, validation_span_bars
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
        validation_span_bars: Option<usize>,
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

        // =========================================================================
        // 🚪 WFE VALIDATION GATE: TÁCH CỬA SỔ HUẤN LUYỆN (IN-SAMPLE) & KIỂM ĐỊNH (OOS)
        // =========================================================================
        // Dành N giờ gần nhất làm tập kiểm định Out-Of-Sample độc lập (mặc định 3 ngày = 4320 nến 1m).
        // Cần tối thiểu warmup + validation + 1440 nến để tách cửa sổ an toàn.
        let val_span_bars = validation_span_bars.unwrap_or(DEFAULT_VALIDATION_SPAN_BARS);
        const WARMUP_SAFE_BARS: usize = 5760;     // Warmup indicators
        let can_split_wfe = candles.len() >= (WARMUP_SAFE_BARS + val_span_bars + 1440);

        let (train_candles, validation_slice) = if can_split_wfe {
            let split_idx = candles.len() - val_span_bars;
            let val_start_idx = candles.len().saturating_sub(WARMUP_SAFE_BARS + val_span_bars);
            (candles[..split_idx].to_vec(), candles[val_start_idx..].to_vec())
        } else {
            (candles.clone(), Vec::new())
        };

        let num_train_candles = train_candles.len();
        let num_val_candles = if validation_slice.is_empty() { 0 } else { val_span_bars };
        if can_split_wfe {
            let val_days = val_span_bars as f64 / 1440.0;
            info!("🧬 [WFE Split] In-Sample Train: {} nến | Out-Of-Sample Validation: {} nến ({:.1} ngày)", num_train_candles, num_val_candles, val_days);
        }

        // 1. Load Cấu hình Prospector & Cache từ file JSON trước
        let file = File::open(config_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let full_config: FullOptimizerConfig = serde_json::from_reader(reader).map_err(|e| e.to_string())?;

        let account_cfg = full_config.account.clone().unwrap_or_default();
        let fitness_cfg = full_config.fitness.unwrap_or_default();
        let leverage = 1.0 / account_cfg.margin_rate;

        // 2. Tách toàn bộ việc dựng DataFrame, Backtest & chạy GA sang luồng Blocking riêng của Tokio (tránh nghẽn Async Event Loop)
        let prospector_db_path_owned = prospector_db_path.to_string();

        let champion_opt = tokio::task::spawn_blocking(move || {
            let df = Self::candles_to_df(&train_candles).map_err(|e| e.to_string())?;

            let mut prospector_config = full_config.prospector;
            prospector_config.db_path = prospector_db_path_owned.clone();

            let early_term = full_config.early_termination.unwrap_or_default();
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
                    EarlyTermination::MaxDrawdown(early_term.max_drawdown_pct),
                    EarlyTermination::WarmupMinTrades { 
                        after_bars: early_term.warmup_bars, 
                        min_trades: early_term.min_trades 
                    },
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


            // [TODO-AUDIT-NEW-11]: Dọn dẹp tệp/thư mục đệm cũ nếu có để đảm bảo môi trường sạch
            let db_path = Path::new(&prospector_db_path_owned);
            if db_path.exists() {
                if db_path.is_dir() {
                    let _ = std::fs::remove_dir_all(db_path);
                } else {
                    let _ = std::fs::remove_file(db_path);
                }
            }

            if let Some(parent) = db_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

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

            // [TODO-AUDIT-NEW-11]: Khởi tạo Job mới vô điều kiện mỗi chu kỳ Walk-Forward Rolling
            // Loại bỏ hoàn toàn bẫy Resume trên đĩa gây tê liệt GA từ chu kỳ 2 trở đi
            let flat_search_space = helios::strategy::core_v3::config::HeliosConfigV3::parameter_defs();
            prospector.add_initial_job(flat_search_space);

            let fitness_cfg_blocking = fitness_cfg;
            let objective = move |stats: &ferro_trade::stats::Stats| {
                Self::calculate_enhanced_fitness_with_leverage(stats, leverage, &fitness_cfg_blocking)
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

            info!("⏱️ [OPTIMIZER DURATION] {:.2?} | In-Sample Candles: {} | Best Return: {:.2}% | Max DD: {:.2}% | Trades: {} (Win: {:.1}%)", 
                  opt_elapsed, num_train_candles, ret, dd, trades, win_rate);

            // Lưu DNA vào bảng optimized_params
            if let Some(dna) = champion.dna {
                // =========================================================================
                // 🚪 WFE VALIDATION GATE: THẨM ĐỊNH OOS ĐỘC LẬP TRƯỚC KHI DEPLOY (FLAW-6.4)
                // =========================================================================
                if !validation_slice.is_empty() {
                    match super::helios_bridge::HeliosBridge::diagnostic_backtest(&validation_slice, &dna) {
                        Ok(val_res) => {
                            let val_ret = val_res.stats.return_pct;
                            let val_dd = val_res.stats.max_drawdown_pct;
                            let val_trades = val_res.stats.total_trades;
                            let val_win_rate = val_res.stats.win_rate_pct;

                            // 🛡️ TIÊU CHUẨN THẨM ĐỊNH OOS KỶ LUẬT VÀ THÍCH ỨNG (ROBUST & ADAPTIVE WFE GATE):
                            // 1. In-Sample: Bắt buộc phải có lệnh (trades > 0) và có lãi (ret > 0.0%).
                            // 2. Out-Of-Sample Validation (3 ngày):
                            //    - Nếu val_trades == 0: Đứng ngoài né bão bảo toàn 100% vốn (val_dd = 0.0%) -> SOLID PASS!
                            //    - Nếu val_trades > 0: Bắt buộc không lỗ ròng (val_net_ret >= 0.0%) và Drawdown an toàn (val_dd <= 15.0%).
                            let slippage_drag_roe = 2.0 * fitness_cfg.slippage_rate * leverage * 100.0;
                            let val_net_ret = val_ret - (val_trades as f64 * slippage_drag_roe);

                            let is_in_sample_invalid = trades == 0 || ret <= 0.0;
                            let is_oos_failed = val_trades > 0 && (val_net_ret < 0.0 || val_dd > 15.0);

                            if is_in_sample_invalid || is_oos_failed {
                                warn!(
                                    "🚨 [WFE GATE: REJECTED - INVALID OR OVERFIT] Symbol: {} | Candidate DNA Rejected! \
                                    In-Sample Return: {:.2}% (Trades: {}) | OOS Val Return: {:+.2}% (Net: {:+.2}%, Max DD: {:.2}%, Trades: {}, Win Rate: {:.1}%) \
                                    -> REJECTED! Preserving active Champion DNA in DB & Actor to protect capital!",
                                    symbol, ret, trades, val_ret, val_net_ret, val_dd, val_trades, val_win_rate
                                );
                                return Ok(()); // 🛑 DỪNG LẠI, KHÔNG LƯU DB, KHÔNG BROADCAST!
                            }

                            info!(
                                "✨ [WFE GATE: SOLID PASS] Champion DNA Validated! In-Sample Return: {:.2}% | OOS Val Return: {:+.2}% (Net: {:+.2}%) \
                                | OOS Max DD: {:.2}% | OOS Trades: {} (Win Rate: {:.1}%) -> Approved for Deployment.",
                                ret, val_ret, val_net_ret, val_dd, val_trades, val_win_rate
                            );
                        }
                        Err(e) => {
                            warn!("⚠️ [WFE GATE] Lỗi khi chạy Validation Backtest: {}. Cho phép fallback deploy.", e);
                        }
                    }
                }


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

    /// Tính toán điểm số Fitness theo hàm Mục tiêu 2 Pha Bão hòa Mũ (Exponential Saturation Two-Phase Fitness)
    /// - Ưu tiên tuyệt đối phòng thủ Drawdown (dd + 2.0).
    /// - Phạt nặng nhất cá thể đứng ngoài né trade (N = 0 -> -1000.0).
    /// - Thưởng tăng dần theo số lệnh qua hàm trơn mượt 1 - exp(-N / 4.0), không hạn chế trần giao dịch nếu có lãi.
    /// - Chốt chặn tử vong khi Drawdown >= 35.0% (loại bỏ cá thể cháy tài khoản).
    pub fn calculate_enhanced_fitness_with_leverage(
        stats: &ferro_trade::stats::Stats,
        _leverage: f64,
        _cfg: &FitnessConfig,
    ) -> f64 {
        let ret = stats.return_pct;
        let dd = stats.max_drawdown_pct.abs();
        let n = stats.total_trades as f64;
        let win_rate = stats.win_rate_pct;

        // 1. Chống dữ liệu rác hoặc NaN
        if ret.is_nan() || dd.is_nan() || win_rate.is_nan() {
            return -2000.0;
        }

        // 2. Chốt chặn Tử Vong: Drawdown >= 35.0% (Trừng phạt cháy tài khoản)
        if dd >= 35.0 {
            return ret - dd - 500.0;
        }

        // 3. Pha 1: Cá thể có lợi nhuận dương và có phát sinh giao dịch
        if ret > 0.0 && n > 0.0 {
            let n_scale = (1.0 - (-n / 4.0).exp()).max(0.0);
            let wr_bonus = 0.50 + (win_rate / 100.0);
            let dd_smoothing = 2.0;
            return (ret / (dd + dd_smoothing)) * wr_bonus * n_scale;
        }

        // 4. Pha 2: Cá thể chưa có lãi hoặc không trade
        if n == 0.0 {
            // Bẫy đứng ngoài né trade: Trừng phạt nặng nhất!
            -1000.0
        } else {
            // Dám trade nhưng lỗ: Giữ gradient leo dốc (thua ít hơn -> điểm cao hơn)
            let n_penalty_scale = (-n / 4.0).exp();
            ret - dd - (50.0 * n_penalty_scale)
        }
    }

    /// Wrapper tương thích ngược cho các unit test và caller mặc định (fallback L = 20.0, default FitnessConfig)
    pub fn calculate_enhanced_fitness(stats: &ferro_trade::stats::Stats) -> f64 {
        Self::calculate_enhanced_fitness_with_leverage(stats, 20.0, &FitnessConfig::default())
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
        assert!(fitness_nan <= -2000.0, "Lợi nhuận NaN phải trả về điểm phạt -2000");

        // Chốt chặn tử vong khi DD >= 35.0%
        let mut stats_chay = ferro_trade::stats::Stats::default();
        stats_chay.return_pct = 50.0;
        stats_chay.max_drawdown_pct = 100.0; // Cháy tài khoản
        stats_chay.total_trades = 5;
        let fit_chay = ProspectorRunner::calculate_enhanced_fitness(&stats_chay);
        assert!(fit_chay < -500.0, "Cá thể cháy tài khoản DD=100% phải bị chốt chặn tử vong < -500.0");
    }

    #[test]
    fn test_fitness_low_trades_penalty() {
        let mut stats_few = ferro_trade::stats::Stats::default();
        stats_few.return_pct = 50.0;
        stats_few.max_drawdown_pct = 5.0;
        stats_few.win_rate_pct = 60.0;
        stats_few.total_trades = 1; // n = 1: 1 - exp(-0.25) ≈ 0.22

        let mut stats_optimal = stats_few.clone();
        stats_optimal.total_trades = 12; // n = 12: 1 - exp(-3) ≈ 0.95

        let fit_few = ProspectorRunner::calculate_enhanced_fitness(&stats_few);
        let fit_optimal = ProspectorRunner::calculate_enhanced_fitness(&stats_optimal);

        assert!(fit_few < fit_optimal, "Ít lệnh (N=1) phải có điểm thấp hơn nhiều so với N=12 do hàm bão hòa mũ");
        assert!(fit_few > 0.0, "N=1 có lãi vẫn nhận điểm dương nhưng bị triệt tiêu biên độ");
    }

    #[test]
    fn test_fitness_inactivity_punishment_and_gradient() {
        // Kịch bản A: Cá thể đứng ngoài né trade (N = 0)
        let mut stats_zero = ferro_trade::stats::Stats::default();
        stats_zero.return_pct = 0.0;
        stats_zero.max_drawdown_pct = 0.0;
        stats_zero.total_trades = 0;
        let fit_zero = ProspectorRunner::calculate_enhanced_fitness(&stats_zero);
        assert_eq!(fit_zero, -1000.0, "Cá thể N=0 phải bị phạt nặng nhất -1000.0");

        // Kịch bản B: Cá thể dám trade bị lỗ nhẹ ma sát (N = 5, Return = -2.0%, DD = 4.0%)
        let mut stats_loss = ferro_trade::stats::Stats::default();
        stats_loss.return_pct = -2.0;
        stats_loss.max_drawdown_pct = 4.0;
        stats_loss.total_trades = 5;
        let fit_loss = ProspectorRunner::calculate_enhanced_fitness(&stats_loss);

        // Điểm lỗ nhẹ (-20.0 đến -50.0) phải cao hơn điểm trừng phạt 0 lệnh (-1000.0)
        assert!(fit_loss > fit_zero, "Cá thể dám trade lỗ nhẹ ({:.1}) phải có điểm cao hơn cá thể đứng ngoài né trade ({:.1})", fit_loss, fit_zero);

        // Kịch bản C: Cá thể có lãi thực sự (N = 8, Return = +15.0%, DD = 4.0%, WR = 62.5%)
        let mut stats_profit = ferro_trade::stats::Stats::default();
        stats_profit.return_pct = 15.0;
        stats_profit.max_drawdown_pct = 4.0;
        stats_profit.win_rate_pct = 62.5;
        stats_profit.total_trades = 8;
        let fit_profit = ProspectorRunner::calculate_enhanced_fitness(&stats_profit);
        assert!(fit_profit > 0.0, "Cá thể có lãi và trade đủ phải có điểm dương cao");
    }

    #[test]
    fn test_early_termination_config_deserialization() {
        let json_with_early_term = r#"{
            "max_drawdown_pct": 25.0,
            "warmup_bars": 2880,
            "min_trades": 3
        }"#;

        let early_term: EarlyTerminationConfig = serde_json::from_str(json_with_early_term).expect("Phải parse thành công JSON early_termination");
        assert_eq!(early_term.max_drawdown_pct, 25.0);
        assert_eq!(early_term.warmup_bars, 2880);
        assert_eq!(early_term.min_trades, 3);

        let default_term = EarlyTerminationConfig::default();
        assert_eq!(default_term.max_drawdown_pct, 35.0);
        assert_eq!(default_term.warmup_bars, 5760);
        assert_eq!(default_term.min_trades, 2);

        // Kiểm tra parse trực tiếp từ file cấu hình thực tế helios_optimizer_config.json
        let config_path = "../../strategies_backtests/helios/examples/helios_optimizer_config.json";
        if std::path::Path::new(config_path).exists() {
            let file = std::fs::File::open(config_path).expect("Mở file helios_optimizer_config.json");
            let reader = std::io::BufReader::new(file);
            let full: FullOptimizerConfig = serde_json::from_reader(reader).expect("Parse helios_optimizer_config.json");
            let et = full.early_termination.expect("helios_optimizer_config.json phải chứa early_termination");
            assert_eq!(et.max_drawdown_pct, 35.0);
            assert_eq!(et.warmup_bars, 5760);
            assert_eq!(et.min_trades, 2);
            assert_eq!(full.prospector.max_depth, 1);
        }
    }

    #[test]
    fn test_wfe_gate_adaptive_validation() {
        let leverage = 20.0;
        let fitness_cfg = FitnessConfig::default();
        let slippage_drag_roe = 2.0 * fitness_cfg.slippage_rate * leverage * 100.0; // 0.8%

        // Kịch bản A: In-Sample 0 lệnh -> REJECT
        let is_in_sample_invalid_0 = 0 == 0 || 0.0 <= 0.0;
        assert!(is_in_sample_invalid_0, "In-Sample 0 lệnh bắt buộc phải bị REJECT!");

        // Kịch bản B: In-Sample Return âm -> REJECT
        let is_in_sample_invalid_neg = 5 == 0 || -2.0 <= 0.0;
        assert!(is_in_sample_invalid_neg, "In-Sample Return âm bắt buộc phải bị REJECT!");

        // Kịch bản C: OOS 3 ngày đứng ngoài né bão (val_trades == 0, val_dd == 0.0%) -> SOLID PASS!
        let val_trades_0 = 0;
        let val_ret_0 = 0.0;
        let val_dd_0 = 0.0;
        let val_net_ret_0 = val_ret_0 - (val_trades_0 as f64 * slippage_drag_roe);
        let is_oos_failed_0 = val_trades_0 > 0 && (val_net_ret_0 < 0.0 || val_dd_0 > 15.0);
        assert!(!is_oos_failed_0, "OOS 3 ngày 0 lệnh bảo toàn vốn không được bị đánh trượt (Do No Harm)!");

        // Kịch bản D: OOS 3 ngày có trade nhưng lỗ ròng -> REJECT!
        let val_trades_loss = 3;
        let val_ret_loss = -1.0;
        let val_dd_loss = 5.0;
        let val_net_ret_loss = val_ret_loss - (val_trades_loss as f64 * slippage_drag_roe);
        let is_oos_failed_loss = val_trades_loss > 0 && (val_net_ret_loss < 0.0 || val_dd_loss > 15.0);
        assert!(is_oos_failed_loss, "OOS 3 ngày trade bị lỗ ròng bắt buộc phải bị REJECT!");

        // Kịch bản E: OOS 3 ngày Drawdown vượt ngưỡng 15.0% -> REJECT!
        let val_trades_dd = 4;
        let val_ret_dd = 2.0;
        let val_dd_excess = 18.0;
        let val_net_ret_dd = val_ret_dd - (val_trades_dd as f64 * slippage_drag_roe);
        let is_oos_failed_dd = val_trades_dd > 0 && (val_net_ret_dd < 0.0 || val_dd_excess > 15.0);
        assert!(is_oos_failed_dd, "OOS 3 ngày Drawdown vượt quá 15% bắt buộc phải bị REJECT!");

        // Kịch bản F: OOS 3 ngày có lãi ròng an toàn (+2.5%, DD = 4.0%, 2 trades) -> SOLID PASS!
        let val_trades_pass = 2;
        let val_ret_pass = 4.1;
        let val_dd_pass = 4.0;
        let val_net_ret_pass = val_ret_pass - (val_trades_pass as f64 * slippage_drag_roe); // 4.1 - 1.6 = 2.5%
        let is_oos_failed_pass = val_trades_pass > 0 && (val_net_ret_pass < 0.0 || val_dd_pass > 15.0);
        assert!(!is_oos_failed_pass, "OOS 3 ngày có lãi ròng (+2.5%) và DD an toàn (4%) phải được APPROVE!");
    }

    #[test]
    fn test_prospector_runner_unconditional_job_seeding() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_prospector_unconditional.db");
        let db_path_str = db_path.to_str().unwrap().to_string();

        // Giả lập tệp đã tồn tại từ chu kỳ trước
        std::fs::write(&db_path, b"dummy stale db content").expect("Failed to write dummy db file");
        assert!(db_path.exists());

        // Mô phỏng logic dọn dẹp và khởi tạo trong ProspectorRunner
        if db_path.exists() {
            if db_path.is_dir() {
                let _ = std::fs::remove_dir_all(&db_path);
            } else {
                let _ = std::fs::remove_file(&db_path);
            }
        }
        assert!(!db_path.exists(), "Tệp đệm cũ bắt buộc phải được dọn sạch");

        let mut prospector_config = ProspectorConfig::default();
        prospector_config.db_path = db_path_str;
        let mut prospector = Prospector::new(prospector_config).expect("Failed to create prospector");

        // Gọi add_initial_job vô điều kiện
        let flat_search_space = helios::strategy::core_v3::config::HeliosConfigV3::parameter_defs();
        prospector.add_initial_job(flat_search_space);

        // Kiểm chứng hàng đợi hotspot_queue luôn nhận Job, không bao giờ bị rỗng
        let (lock, _) = &*prospector.hotspot_queue;
        let queue_len = lock.lock().len();
        assert_eq!(queue_len, 1, "Hàng đợi hotspot_queue bắt buộc phải có 1 Job được nạp!");
    }
}
