// strategies_backtests/helios/examples/run_helios.rs

mod common;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;

use ferro_trade::backtest::BacktestBuilder;
use ferro_trade::data::load_data;
use ferro_trade::optimizer::prospector::{Prospector, ProspectorConfig};
use ferro_trade::optimizer::config_parser::flatten_nested_params;
use ferro_trade::stats::StatItem;
use ferro_trade::strategy::Strategy; // [FIX]: Import trait để dùng được hàm .new() và .update_from_dna()
use helios::strategy::HeliosStrategy;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct FullOptimizerConfig {
    prospector: ProspectorConfig,
    search_space: serde_json::Value,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HELIOS STRATEGY: RANK 1 DIAGNOSTIC RUN (BIAS ANALYSIS) ===");

    // 1. DNA của RANK 1
    let rank_1_dna = vec![
        2.882672231882439, 0.23623290130732047, 75.49141693224317, 0.7860688778725414, 3.8221155865991063, 99.75429343460955, 
        25.69114392539398, 0.2910854621818793, 11.108881740643065, 3.774288433182724, 2.45260548705128, 24.147319438133955, 
        2.880904874490677, 30.951207265224, 19.600885035181882, 0.21509468467503817, 14.905654831615212, 2.6512628243311607, 
        4.354219449070611, 3.014525613308213, 11.433558128188054, 2.504191510689791, 27.325143172913652, 4.094927125747008, 
        40.79380591493594, 0.7634848296875456, 14.59883955559564, 0.010686151641873922, 13.810807665320382, 1.0095341531836264, 
        3.927332512416409, 1.5624685191845624, 1.7499066729751194, 20.431963367276143, 11.570864220324015, 9.911571098219598, 
        19.32412159686901, 1.6934798203040704, 0.21002276654475122, 13.308374640607225, 0.5433600221495568, 24.597819260042773, 
        0.5030061887681762, 0.00882929775818097, 1.147449425456129, 1.571159083763063, 1.5459623597849752, 0.4489488407865958, 
        0.1885614923342968, 0.42610909666527796, 1.1642411885130257, 67.59645052396726, 0.8738747519396207, 0.4355986709337101, 
        0.9893080516869808, 2.667824032261625, 1.611071213323435, 0.9226118495887646, 0.5424220292986937, 1.0552615365777585, 
        0.7427043242443293, 0.9676287962703263, 1.4723212273346569, 0.8720125629652363, 0.2288990816180395, 0.4308095369127567, 
        27.686092921499892, 72.76960550865266, 0.16162974325228002, 2.4777023833355756, 0.04426990356211478, 0.0027352180886297136, 
        0.10625619386979326, 0.843813364403416, 4.2692844698139645, 4.767441160806781, 1.2763234033171127, 1.5351812735376718, 
        0.5755817695333004, 1.002246879477523, 1.2259363838830373, 4.173922622317256, 2.293175482909392, 1.4433569988989206, 
        0.25602615200571677, 0.2858500851143314, 0.7103868603924065, 1.7203234783340484, 0.0043815769875611595, 0.3633035897331722, 
        0.07286700365538033, 86.23230469726238, 22.69796744086282, 0.9157642827951206, 0.8979880569639394, 0.21343467767479746
    ];

    // 2. Load Cấu hình & Mapping
    let config_path = "examples/helios_optimizer_config.json";
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let full_config: FullOptimizerConfig = serde_json::from_reader(reader)?;
    
    let mut flat_params = HashMap::new();
    flatten_nested_params(&full_config.search_space, "".to_string(), &mut flat_params)?;
    let mut param_names: Vec<String> = flat_params.keys().cloned().collect();
    param_names.sort();
    
    let mut mapping = HashMap::new();
    for (idx, name) in param_names.iter().enumerate() {
        mapping.insert(name.clone(), idx);
    }

    // 3. Khởi tạo Backtest
    let df = load_data(common::DATA_PATH).expect("Failed to load data");
    let mut backtest = BacktestBuilder::new()
        .data(df)
        .cash(common::INITIAL_CASH)
        .commission(common::COMMISSION)
        .margin(common::MARGIN)
        .fractional_unit(Some(common::FRACTIONAL_UNIT))
        .trade_on_close(true)
        .exclusive_orders(true) 
        .hedging(false)
        .finalize_trades(true)
        .calculate_metrics(&[
            StatItem::ReturnPct, StatItem::TotalTrades, StatItem::WinRatePct, StatItem::MaxDrawdownPct, StatItem::SharpeRatio, StatItem::ProfitFactor
        ])
        .build()?;

    // 4. TIỀN TÍNH TOÁN MTF
    let prospector = Prospector::new(full_config.prospector)?;
    prospector.precompute_mtf_data(&backtest);

    // 5. Giải mã Config Rank 1 để xem các ngưỡng quan trọng
    let mut diag_strategy = HeliosStrategy::new(&HashMap::new());
    diag_strategy.update_from_dna(&rank_1_dna, &mapping);
    
    println!("\n--- RANK 1 PARAMETER DECODING ---");
    println!("Min Score Threshold: {:.4}", diag_strategy.config.market_regime.priority.min_score_threshold);
    println!("Weights -> Trend: {:.2}, Squeeze: {:.2}, Anomaly: {:.2}, Ranging: {:.2}", 
        diag_strategy.config.market_regime.priority.trend,
        diag_strategy.config.market_regime.priority.squeeze,
        diag_strategy.config.market_regime.priority.anomaly,
        diag_strategy.config.market_regime.priority.ranging
    );
    println!("Anomaly Z-Threshold: {:.2}", diag_strategy.config.market_regime.anomaly.z_score_threshold);
    println!("Trend Angle Healthy: {:.2}", diag_strategy.config.market_regime.trend.angle_healthy);

    // 6. Chạy Backtest với DNA
    println!("\nRunning backtest...");
    let start = Instant::now();
    let result = backtest.run_with_dna::<HeliosStrategy>(&rank_1_dna, &mapping)?;
    let duration = start.elapsed();

    // 7. Kết quả chẩn đoán lệnh
    println!("\n=== TRADE LISTING ===");
    println!("{:<5} | {:<10} | {:<12} | {:<10.2} | {:<10} | {:<20}", 
        "ID", "Type", "Entry Bar", "PnL", "Duration", "Playbook Tag");
    println!("{}", "-".repeat(85));

    for t in result.trades {
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

    Ok(())
}
