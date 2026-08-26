// strategies_backtests/helios/examples/v3_run_helios_optimize.rs

mod common;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;
use ferro_trade::backtest::{BacktestBuilder, EarlyTermination, TerminationReason};
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
        // [A4.2] Cấu hình Early Termination dựa trên Objective Function:
        // - Ngắt ngay nếu Max Drawdown vượt quá 40% (ngưỡng đỏ của Objective)
        // - Ngắt ngay nếu sau 10 ngày (14,400 nến 1m) mà vào ít hơn 5 lệnh (< 0.5 trades/day)
        .early_termination(EarlyTermination::Any(vec![
            EarlyTermination::MaxDrawdown(40.0),
            EarlyTermination::WarmupMinTrades { after_bars: 14400, min_trades: 5 },
        ]))
        // Tùy chọn: Hàm phạt khớp chính xác với điểm phạt của Objective
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
    
    // Hàm mục tiêu Quant Tối ưu (Pure Calmar Score): Lãi cao, Drawdown thấp, Gồng siêu sóng bền bỉ
    let objective = |stats: &ferro_trade::stats::Stats| {
        let ret = stats.return_pct;
        let dd = stats.max_drawdown_pct.abs();
        let trades = stats.total_trades;

        // Phòng vệ Edge Cases: Cần ít nhất 2 lệnh độc lập và lợi nhuận dương
        if ret.is_nan() || ret <= 0.0 || trades < 2 {
            return -100.0 - dd;
        }

        // Điểm số Calmar tối thượng với hằng số làm mượt +3.0 bảo vệ rung lắc
        ret / (dd + 3.0)
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

        let ret = c.stats.return_pct;
        let dd = c.stats.max_drawdown_pct;
        let trades = c.stats.total_trades;
        let win_rate = c.stats.win_rate_pct;
        let best = c.stats.best_trade_pct;
        let worst = c.stats.worst_trade_pct;
        let pf = c.stats.profit_factor;
        let sharpe = c.stats.sharpe_ratio;
        let sortino = c.stats.sortino_ratio;

        println!("{:<5} | {:<10.2} | {:<10.2} | {:<8} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.4} | {:<10.4}", 
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
                println!("{:<5} | {:<8} | {:<10} | {:<10} | {:<10} | {:<10} | {:<15}", 
                    "ID", "Type", "Entry Bar", "PnL ($)", "PnL (%)", "Duration", "Playbook Tag");
                println!("{}", "-".repeat(85));

                for t in res.trades {
                    let pnl_usd = t.pl(None); 
                    let pnl_pct = t.pl_pct(None) * 100.0;
                    let duration = t.exit_bar.map(|eb| eb - t.entry_bar).unwrap_or(0);

                    println!("{:<5} | {:<8} | {:<10} | {:<10.2} | {:<9.2}% | {:<10} | {:<15}", 
                        t.id, 
                        if t.size > 0.0 { "LONG" } else { "SHORT" },
                        t.entry_bar,
                        pnl_usd,
                        pnl_pct,
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
