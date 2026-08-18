// strategies_backtests/helios/examples/v3_run_helios_optimize.rs

mod common;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;
use ferro_trade::backtest::BacktestBuilder;
use ferro_trade::data::load_data;
use ferro_trade::optimizer::prospector::{Prospector, ProspectorConfig};
use ferro_trade::stats::StatItem;
use helios::strategy::core_v3::HeliosStrategyV3;

// --- CONFIG WRAPPER ---
#[derive(Debug, Deserialize)]
struct FullOptimizerConfig {
    prospector: ProspectorConfig,
    cache_config: Option<ferro_trade::data::global_cache::CacheConfig>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HELIOS STRATEGY V3: ADVANCED OPTIMIZATION (PROSPECTOR) ===");

    // 1. Khởi tạo đường dẫn DB (Auto-Resume Logic)
    let db_path = r"E:/Projects/Automations/trading_backtest/strategies_backtests/helios/examples/helios_prospector_v3.db";
    let db_exists = Path::new(db_path).exists();

    if db_exists {
        println!("✅ Found existing DB at '{}'. Resuming...", db_path);
    } else {
        println!("🆕 Starting fresh optimization run for Helios V3.");
    }

    // 2. Load Dữ liệu (Sử dụng đường dẫn từ common)
    println!("Loading data: {}", common::DATA_PATH);
    let df = load_data(common::DATA_PATH).expect("Failed to load data");
    println!("Loaded {} rows.", df.height());

    // 3. Load Cấu hình từ JSON (Nguồn chân lý duy nhất)
    let config_path = "examples/helios_optimizer_config.json";
    let file = File::open(config_path).expect("Could not open helios_optimizer_config.json");
    let reader = BufReader::new(file);
    let full_config: FullOptimizerConfig = serde_json::from_reader(reader).expect("JSON format error");

    let mut prospector_config = full_config.prospector;
    prospector_config.db_path = db_path.to_string();

    println!("Config Loaded:");
    println!("  Workers: Pop={}, Gen={}", prospector_config.worker_config.population_size, prospector_config.worker_config.n_generations);
    println!("  Scout Method: {}", prospector_config.scout_method);
    println!("  Exploration Strategy: {}", prospector_config.exploration_strategy);
    println!("  Max Threads: {:?}", prospector_config.max_threads);

    // 4. Backtest Setup (Template cho mọi Worker)
    let backtest = BacktestBuilder::new()
        .data(df)
        .timeframes(&prospector_config.available_timeframes)
        .cash(common::INITIAL_CASH)
        .commission(common::COMMISSION)
        .margin(common::MARGIN)
        .fractional_unit(Some(common::FRACTIONAL_UNIT))
        .trade_on_close(true)
        .exclusive_orders(true)
        .hedging(false)
        .finalize_trades(true)
        .cache_config(full_config.cache_config.unwrap_or_default())
        .calculate_metrics(&[
            StatItem::ReturnPct, 
            StatItem::TotalTrades, 
            StatItem::WinRatePct, 
            StatItem::BestTradePct, 
            StatItem::WorstTradePct, 
            StatItem::MaxDrawdownPct,
            StatItem::SharpeRatio,
            StatItem::SortinoRatio,
            StatItem::ProfitFactor
        ])
        .build()?;

    // 5. Khởi tạo Prospector
    let mut prospector = Prospector::new(prospector_config)?;

    // 6. Định nghĩa Job Đầu tiên (Tự động trích xuất từ Struct qua Macro DNA v2.0)
    let needs_initial_job = !db_exists || prospector.backlog_storage.get_max_job_id() == 0;
    if needs_initial_job {
        use ferro_trade::optimizer::params::DnaMappable;
        let flat_search_space = helios::strategy::core_v3::config::HeliosConfigV3::parameter_defs();
        
        println!("Adding Initial Job... ({} parameters detected automatically from Struct)", flat_search_space.len());
        prospector.add_initial_job(flat_search_space);
    }

    // 7. KÍCH HOẠT HỆ THỐNG
    println!(">>> STARTING HELIOS V3 EVOLUTION <<<");
    
    // Hàm mục tiêu: Tối ưu hóa cho Lợi nhuận và bảo vệ Drawdown
    let objective = |stats: &ferro_trade::stats::Stats| {
        let total_days = 30.0; // Khoảng thời gian ước tính 1 tháng (43000 bars 1m)
        let trades_per_day = stats.total_trades as f64 / total_days;
        
        // 1. Phải có ít nhất khoảng 1 lệnh mỗi 2 ngày (0.5 trades/day) -> Tránh overfit với quá ít lệnh
        if trades_per_day < 0.5 {
            return -2000.0 * (1.0 - trades_per_day); 
        }
        
        let ret = stats.return_pct;
        let dd = stats.max_drawdown_pct.abs();
        
        // 2. Giới hạn đỏ Drawdown: Phạt cực nặng nếu lỗ hơn 40%
        if dd > 40.0 {
            return -5000.0 - dd; 
        }

        if ret.is_nan() || ret <= 0.0 {
            return -50.0;
        }

        // 3. Tối đa lợi nhuận nếu nằm trong ngưỡng an toàn
        // (Khuyến khích trade nhiều nếu return bằng nhau)
        ret + (trades_per_day * 0.1)
    };

    // Chạy Prospector với HeliosStrategyV3 (Sử dụng clone để giữ lại một bản cho chẩn đoán cuối game)
    prospector.run::<HeliosStrategyV3, _>(backtest.clone(), objective);

    // 8. In kết quả Top 10
    println!("\n=== TOP 10 HELIOS V3 CANDIDATES ===");
    println!("{:<5} | {:<10} | {:<10} | {:<8} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10}", 
        "Rank", "Return %", "Max DD %", "Trades", "Win Rate", "Best %", "Worst %", "ProfitF", "Sharpe", "Sortino");
    println!("{}", "-".repeat(120));

    let champs = prospector.champions.read();
    for (i, c) in champs.iter().enumerate() {
        if i >= 10 { break; }

        let ret = c.stats.get("Return %").unwrap_or(&0.0);
        let dd = c.stats.get("Max DD %").unwrap_or(&0.0);
        let trades = c.stats.get("Trades").unwrap_or(&0.0);
        let win_rate = c.stats.get("Win Rate %").unwrap_or(&0.0);
        let best = c.stats.get("Best Trade %").unwrap_or(&0.0);
        let worst = c.stats.get("Worst Trade %").unwrap_or(&0.0);
        let pf = c.stats.get("Profit Factor").unwrap_or(&0.0);
        let sharpe = c.stats.get("Sharpe").unwrap_or(&0.0);
        let sortino = c.stats.get("Sortino").unwrap_or(&0.0);

        println!("{:<5} | {:<10.2} | {:<10.2} | {:<8.0} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.4} | {:<10.4}", 
            i+1, ret, dd, trades, win_rate, best, worst, pf, sharpe, sortino);
        
        // if let Some(dna) = &c.dna {
        //     println!("  DNA: {:?}\n", dna);
        // }
        println!("{}", "-".repeat(120));
    }

    // 9. CHẨN ĐOÁN CHI TIẾT (DIAGNOSTIC RE-RUN) - NÂNG CẤP TOP 10
    println!("\n=== TOP 10 CHAMPIONS: DETAILED TRADE DIAGNOSTICS ===");
    
    // Lấy danh sách Top 10 từ Champions
    let top_champs = {
        let champs_lock = prospector.champions.read();
        champs_lock.iter().take(1).cloned().collect::<Vec<_>>()
    };

    for (rank_idx, c) in top_champs.into_iter().enumerate() {
        let rank = rank_idx + 1;
        if let Some(dna) = &c.dna {
            println!("\n>>> [RANK {}] Re-running for full trade extraction...", rank);
            
            // // [NEW] Giải mã và in Parameters chi tiết để đối chiếu với DNA
            // let mut sorted_keys: Vec<_> = c.params.keys().collect();
            // sorted_keys.sort();
            // let params_str = sorted_keys.iter()
            //     .map(|k| {
            //         let v = c.params.get(*k).unwrap();
            //         if let Some(val) = v.as_f64() {
            //             format!("{}: {:.4}", k, val)
            //         } else {
            //             format!("{}: {:?}", k, v)
            //         }
            //     })
            //     .collect::<Vec<String>>()
            //     .join(", ");
            
            // println!("   Params: {}", params_str);

            if let Ok(res) = backtest.run_with_dna::<HeliosStrategyV3>(dna) {
                println!("{:<5} | {:<10} | {:<12} | {:<10.2} | {:<10} | {:<20}", 
                    "ID", "Type", "Entry Bar", "PnL", "Duration", "Playbook Tag");
                println!("{}", "-".repeat(85));

                for t in res.trades {
                    let pnl = t.pl(None); 
                    let duration = t.exit_bar.map(|eb| eb - t.entry_bar).unwrap_or(0);

                    println!("{:<5} | {:<10} | {:<12} | {:<10.2} | {:<10} | {:<20}", 
                        t.id, 
                        if t.size > 0.0 { "LONG" } else { "SHORT" },
                        t.entry_bar,
                        pnl,
                        duration,
                        t.tag.as_deref().unwrap_or("None")
                    );
                }
                println!("--- End of Rank {} ---", rank);
            }
        }
    }

    println!("\n>>> Diagnostic Complete. Review the 'Playbook Tag' column across all ranks to compare squad performance. <<<");
    Ok(())
}
