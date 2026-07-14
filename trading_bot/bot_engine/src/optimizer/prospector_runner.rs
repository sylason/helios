use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{info, warn, error};
use serde::Deserialize;

use ferro_trade::backtest::BacktestBuilder;
use ferro_trade::optimizer::prospector::{Prospector, ProspectorConfig};
use ferro_trade::optimizer::params::DnaMappable;
use ferro_trade::stats::StatItem;
use helios::strategy::HeliosStrategy;
use polars::prelude::*;

use crate::models::repository::{CandleRepository, StrategyRepository};
use crate::data_engine::sqlite_impl::SqliteStore;

// --- Định nghĩa Constants cứng từ common.rs của Helios ---
pub const INITIAL_CASH: f64 = 60.0;
pub const COMMISSION: f64 = 0.0004;
pub const MARGIN: f64 = 1.0 / 20.0;
pub const FRACTIONAL_UNIT: f64 = 0.0001;

#[derive(Debug, Deserialize)]
struct FullOptimizerConfig {
    prospector: ProspectorConfig,
}

pub struct ProspectorRunner;

impl ProspectorRunner {
    /// Chạy 1 chu kỳ tối ưu hóa dựa trên 30 ngày dữ liệu gần nhất
    pub async fn run_optimization_cycle(
        symbol: &str,
        db: Arc<SqliteStore>,
        config_path: &str, // VD: "../../strategies_backtests/helios/examples/helios_optimizer_config.json"
        prospector_db_path: &str,
        dna_tx: Option<tokio::sync::watch::Sender<Option<crate::models::models::OptimizedDna>>>,
    ) -> Result<(), String> {
        info!("🧬 [Optimizer] Bắt đầu chu kỳ tối ưu hóa cho {}...", symbol);

        // 1. Lấy dữ liệu 30 ngày từ DuckDB
        let now = chrono::Utc::now().timestamp_millis();
        let thirty_days_ms = 30 * 24 * 60 * 60 * 1000;
        let start_time = now - thirty_days_ms;

        let candles = db.get_candles_by_range(symbol, start_time, now).await.map_err(|e| e.to_string())?;
        if candles.is_empty() {
            warn!("⚠️ [Optimizer] Không có dữ liệu nến cho {} để tối ưu hóa.", symbol);
            return Ok(());
        }

        // Bổ sung: Bảo vệ thuật toán khỏi việc thiếu dữ liệu (cần ít nhất 5000 nến để các Indicator dài hạn không bị crash)
        if candles.len() < 5000 {
            warn!("⚠️ [Optimizer] Không đủ dữ liệu nến (hiện có {}, cần > 5000) để tối ưu hóa. Bỏ qua chu kỳ này.", candles.len());
            return Ok(());
        }

        info!("🧬 [Optimizer] Đã lấy {} nến từ DuckDB.", candles.len());

        // 2. Chuyển đổi dữ liệu sang DataFrame của Polars (Tái sử dụng hàm từ HeliosBridge)
        let df = Self::candles_to_df(&candles).map_err(|e| e.to_string())?;

        // 3. Khởi tạo BacktestBuilder (Thông số cứng từ common.rs)
        let backtest = BacktestBuilder::new()
            .data(df)
            .cash(INITIAL_CASH)
            .commission(COMMISSION)
            .margin(MARGIN)
            .fractional_unit(Some(FRACTIONAL_UNIT))
            .trade_on_close(true)
            .exclusive_orders(true)
            .hedging(false)
            .finalize_trades(true)
            .calculate_metrics(&[
                StatItem::ReturnPct,
                StatItem::TotalTrades,
                StatItem::MaxDrawdownPct,
            ])
            .build().map_err(|e| e.to_string())?;

        // 4. Load Cấu hình Prospector từ file JSON
        let file = File::open(config_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let full_config: FullOptimizerConfig = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
        
        let mut prospector_config = full_config.prospector;
        prospector_config.db_path = prospector_db_path.to_string();

        let db_exists = Path::new(prospector_db_path).exists();
        let mut prospector = Prospector::new(prospector_config).map_err(|e| e.to_string())?;

        // Thêm job đầu tiên nếu chưa có
        if !db_exists || prospector.backlog_storage.get_max_job_id() == 0 {
            let flat_search_space = helios::strategy::config::HeliosConfig::parameter_defs();
            prospector.add_initial_job(flat_search_space);
        }

        // 5. Khởi chạy thuật toán GA
        let objective = |stats: &ferro_trade::stats::Stats| {
            let total_days = 30.0;
            let trades_per_day = stats.total_trades as f64 / total_days;
            
            if trades_per_day < 1.0 {
                return -2000.0 * (1.0 - trades_per_day); 
            }
            
            let dd = stats.max_drawdown_pct.abs();
            if dd > 40.0 {
                return -1000.0 - dd; 
            }

            let ret = stats.return_pct;
            if ret.is_nan() || ret <= 0.0 {
                return -50.0;
            }

            ret
        };

        info!("🧬 [Optimizer] Đang chạy thuật toán Di truyền (Genetic Algorithm)...");
        prospector.run::<HeliosStrategy, _>(backtest, objective);

        // 6. Lấy kết quả Top 1 (Champion)
        let champion_opt = {
            let champs = prospector.champions.read();
            champs.first().cloned()
        };

        if let Some(champion) = champion_opt {
            let ret = champion.stats.get("Return %").unwrap_or(&0.0);
            let dd = champion.stats.get("Max DD %").unwrap_or(&0.0);
            let trades = champion.stats.get("Trades").unwrap_or(&0.0);

            info!("🏆 [Optimizer] TOP 1 FOUND! Return: {:.2}%, Max DD: {:.2}%, Trades: {}", ret, dd, trades);

            // Lưu DNA vào bảng optimized_params
            if let Some(dna) = champion.dna {
                use crate::models::models::OptimizedDna;
                use std::collections::HashMap;

                let opt_dna = OptimizedDna {
                    symbol: symbol.to_string(),
                    strategy_id: "helios_v1".to_string(), // Tạm fix ID
                    dna: dna,
                    mapping: HashMap::new(), // Không cần mapping với DNA v2
                    fitness: *ret,
                    stats: champion.stats,
                    updated_at: now,
                };

                db.save_best_dna(opt_dna.clone()).await.map_err(|e| e.to_string())?;
                info!("💾 [Optimizer] Đã lưu DNA mới nhất vào Database.");

                // Phát tín hiệu Event-Driven
                if let Some(tx) = &dna_tx {
                    let _ = tx.send(Some(opt_dna));
                    info!("📢 [Optimizer] Đã phát tín hiệu (Event-Driven) mang DNA mới tới các Actor.");
                }
            }
        } else {
            warn!("⚠️ [Optimizer] Không tìm thấy Champion nào có Return > 0.");
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
            Column::from(s_dt),
            Column::from(s_open),
            Column::from(s_high),
            Column::from(s_low),
            Column::from(s_close),
            Column::from(s_vol),
        ])
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
}
