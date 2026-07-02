// strategies_backtests/helios/examples/run_helios.rs

mod common;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use ferro_trade::backtest::BacktestBuilder;
use ferro_trade::data::load_data;
use ferro_trade::optimizer::prospector::{Prospector, ProspectorConfig};
use ferro_trade::stats::StatItem;
use ferro_trade::strategy::Strategy;
use helios::strategy::HeliosStrategy;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct FullOptimizerConfig {
    prospector: ProspectorConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HELIOS STRATEGY: RANK 1 DIAGNOSTIC RUN (BIAS ANALYSIS) ===");

    // 1. DNA của RANK 1 (Trích xuất từ kết quả run_helios_optimize mới nhất)
    let rank_1_dna = vec![
        8.305916257532342,
        41.705940486941515,
        0.46688867329357564,
        34.400664367444854,
        3.715058661495383,
        29.275713537206833,
        1.4002719559900252,
        8.297874603655263,
        3.4613611318367714,
        0.6475573739222568,
        64.43423571021839,
        2.7064494121093037,
        1.104617599397405,
        4.454388041623047,
        32.37569978277165,
        113.56332324447484,
        19.413766183755474,
        0.3445629126410373,
        6.1706399311169555,
        12.01587199619645,
        21.87639796824379,
        10.334692474336755,
        1.0965450192751371,
        0.3777759518200057,
        3.8558767170498207,
        43.11289627907866,
        3.465751103449504,
        48.940103012690884,
        3.137437896542548,
        33.57617710753023,
        0.9410211597532732,
        129.72614245134346,
        0.6744108613809462,
        131.651052633349,
        0.0014692741724795515,
        11.91846846001085,
        94.24870996510843,
        211.26182305788055,
        403.0381790268037,
        418.23327188981784,
        3.58236643580906,
        0.022466145875388033,
        0.05098798612786235,
        0.23714815588241336,
        8.74972556539951,
        136.78459121432883,
        379.13228545885204,
        0.8144978261026883,
        0.4081486473931863,
        0.22620731870625602,
        46.078101579721704,
        68.44903900124001,
        0.852108766856046,
        0.8595460651495473,
        0.9175842249887851,
        5.500450781800463,
        0.8469054219431889,
        0.02725172812759448,
        0.7748061275223186,
        0.6344996891995149,
        2.5955078568706114,
        63.71544226849669,
        0.8392615613686896,
        0.24380136162320912,
        4.2475833034795905,
        0.5840625141540342,
        0.5153361019101047,
        0.9387799472675811,
        1.0241449087943197,
        0.756089709959877,
        0.3067801687724495,
        0.01302620711743372,
        0.42899167640725866,
        5.177489787594807,
        0.6218088911671903,
        0.8744589654978819,
        0.8250360303856674,
        13.685309785764403,
        1.2364455584776821,
        0.2577898944873166,
        0.7337872093761231,
        0.03024252698086487,
        0.5168381353441274,
        4.6885616458983375,
        0.9373584917097273,
        0.5155139119489109,
        13.596603270721957,
        0.6607154004110337,
        0.8882149105064512,
        0.5858766690763475,
        6.43623020818651,
        0.775843917460591,
        0.8803579732977123,
        14.946903911843286,
        0.13525400203713459,
        0.003596096045994636,
        0.1169550546090083,
        0.6168900558382759,
        0.10017705199076435,
        6.397421311380079,
        7.063657495047471,
        1.7360594098056326,
        2.513363649704332,
        0.8389642212331001,
        2.201398399085262,
        1.1823822552162346,
        2.768418436811973,
        0.6214510675109154,
        2.0650073675864675,
        6.75378937503114,
    ];

    // 2. Load Cấu hình (Chỉ lấy phần cấu hình Prospector để precompute MTF)
    let config_path = "examples/helios_optimizer_config.json";
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let full_config: FullOptimizerConfig = serde_json::from_reader(reader)?;

    // Mapping trống vì DNA v2.0 đã cố định thứ tự DNA trong Struct
    // Đã xóa do không còn cần thiết với Macro v2.0
    // 3. Khởi tạo Backtest
    let df = load_data(common::DATA_PATH).expect("Failed to load data");
    println!("Loaded {} bars.", df.height());

    let backtest = BacktestBuilder::new()
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
            StatItem::ReturnPct, 
            StatItem::TotalTrades, 
            // StatItem::WinRatePct, 
            // StatItem::BestTradePct, 
            // StatItem::WorstTradePct, 
            StatItem::MaxDrawdownPct,
            // StatItem::SharpeRatio,
            // StatItem::SortinoRatio,
            // StatItem::ProfitFactor
        ])
        .build()?;

    // 4. TIỀN TÍNH TOÁN MTF
    let prospector = Prospector::new(full_config.prospector)?;
    prospector.precompute_mtf_data(&backtest);

    // 5. Giải mã Config Rank 1 để kiểm tra mapping
    let mut diag_strategy = HeliosStrategy::new(&HashMap::new());
    diag_strategy.update_from_dna(&rank_1_dna);

    println!("\n--- RANK 1 DECODED PARAMETERS (Verification) ---");
    println!(
        "Lookbacks: Trend={}, Sqz={}, Mom={}, Vol={}, Z={}",
        diag_strategy.config.matrix.trend_lookback,
        diag_strategy.config.matrix.squeeze_lookback,
        diag_strategy.config.matrix.momentum_lookback,
        diag_strategy.config.matrix.volume_lookback,
        diag_strategy.config.indicators.z_score_lookback
    );
    println!(
        "Thresholds: Min Score={:.4}, Env ER={:.4}, Anomaly Z={:.4}",
        diag_strategy
            .config
            .market_regime
            .priority
            .min_score_threshold,
        diag_strategy.config.environment_fuse.er_threshold,
        diag_strategy.config.market_regime.anomaly.z_score_threshold
    );
    println!(
        "Playbook Pullback RSI Buy Thresh: {:.4}",
        diag_strategy.config.tactical_sniper.pullback.rsi_buy_thresh
    );

    // 6. Chạy Backtest với DNA
    println!("\n>>> STARTING BACKTEST WITH CHAMPION DNA <<<");
    let start_time = Instant::now();
    let result = backtest.run_with_dna::<HeliosStrategy>(&rank_1_dna)?;
    let elapsed = start_time.elapsed();

    // 7. Kết quả chẩn đoán lệnh
    println!("\n=== BACKTEST RESULTS ===");
    println!("Time: {:?}", elapsed);
    println!("Return: {:.2}%", result.stats.return_pct);
    println!("Trades: {}", result.stats.total_trades);
    println!("Max DD: {:.2}%", result.stats.max_drawdown_pct);

    println!("\n=== TRADE LISTING ===");
    println!(
        "{:<5} | {:<10} | {:<12} | {:<10.2} | {:<10} | {:<20}",
        "ID", "Type", "Entry Bar", "PnL", "Duration", "Playbook Tag"
    );
    println!("{}", "-".repeat(85));

    for t in result.trades {
        let pnl = t.pl(None);
        let duration = t.exit_bar.map(|eb| eb - t.entry_bar).unwrap_or(0);
        println!(
            "{:<5} | {:<10} | {:<12} | {:<10.2} | {:<10} | {:<20}",
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
