// strategies_backtests/helios/examples/v3_run_helios.rs

mod common;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use serde::Deserialize;

use ferro_trade::backtest::BacktestBuilder;
use ferro_trade::data::load_data;
use ferro_trade::optimizer::prospector::{Prospector, ProspectorConfig};
use ferro_trade::stats::StatItem;
use helios::strategy::core_v3::HeliosStrategyV3;

#[derive(Debug, Deserialize)]
struct FullOptimizerConfig {
    prospector: ProspectorConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Kích hoạt DEBUG MODE cho V3
    env::set_var("HELIOS_DEBUG", "1");
    
    println!("=== HELIOS STRATEGY V3: DIAGNOSTIC RUN ===");

    let rank_1_dna = vec![
        16.94981430500036, 89.99185660556986, 1.6474193378132775, 22.4136564984559, 1.979772843296138, 21.851126983081716, 1.0072089922569456, 40.27267798225382, 4.556300552836553, 1.4255024962373073, 18.653609114169797, 1.6926764200255664, 3.263443292758067, 4.350059176742498, 27.929938258727386, 57.176459051500714, 9.762082927143151, 2.259529557173139, 46.49764694364555, 9.137396536075496, 15.248650810893569, 11.245502920272878, 8.545712331306813, 0.4405899692094199, 2.7774418711944535, 25.3959070970525, 2.1209151137756943, 34.96178602021011, 2.4762165476243303, 33.18666272418943, 3.2295631506731466, 108.9749104010719, 2.9993064178463693, 38.01796250712025, 3.5477547742730637, 95.84582696064469, 444.82688588363345, 194.62409039297287, 208.13244436181702, 116.55074712350252, 2.9951198440438707, 0.10458831460061023, 0.059612760617711626, 0.2773795586995403, 40.80944386712355, 49.54198811450006, 253.30732295223876, 0.16194340282535413, 0.3300761394257865, 0.3047169657589851, 26.256821563736167, 63.08207530572044, 0.6612262066411435, 0.10719375196381833, 0.8631587329045474, 8.49278637462941, 0.6313309881176387, 0.00845470187083286, 0.8490611471159928, 0.7056679209608798, 1.285608805681743, 66.98446231325136, 1.261813212987032, 0.35012827651680223, 3.982629902880836, 2.034202630696551, 0.23546606614313692, 2.081978983308039, 1.1109100877384077, 0.8609302974401756, 0.38604798874315316, 0.017747556670386997, 0.22433728089805755, 12.931034913411454, 0.673152388208074, 0.9529349727684482, 0.8503218815822065, 2.4713181709152536, 1.9437453730870462, 0.10820422256888475, 0.8693500634248618, 0.04307000111085305, 0.20970231089932895, 1.8965023482313932, 0.9091006172946998, 0.15199776702407208, 8.793038449431222, 0.8245992406573691, 0.8772090195450687, 0.12270400109753574, 14.180831893328376, 0.8272381705945112, 0.9893645681919943, 9.982738071059606, 0.10586240950713378, 0.008016710224480732, 0.13449062461962405, 2.7271336926079695, 0.5565766241418625, 1.9581888828635465, 11.444362629560356, 1.3593660758151418, 2.171886001646615, 0.8572034014150929, 2.989007664083326, 1.4986795177417993, 1.6103779147552029, 1.0413232179387046, 1.1288190630322874, 3.6106382749474952
    ];

    let df = load_data(common::DATA_PATH).expect("Failed to load data");
    println!("Loaded {} bars.", df.height());

    let backtest = BacktestBuilder::new()
        .data(df)
        .cash(1000.0)
        .commission(0.0004)
        .margin(1.0)
        .trade_on_close(true)
        .exclusive_orders(true)
        .hedging(false)
        .finalize_trades(true)
        .fractional_unit(Some(common::FRACTIONAL_UNIT))
        .calculate_metrics(&[
            StatItem::ReturnPct, 
            StatItem::TotalTrades, 
            StatItem::MaxDrawdownPct,
        ])
        .build()?;

    // Load Cấu hình (Chỉ lấy phần cấu hình Prospector để precompute MTF)
    let config_path = "examples/helios_optimizer_config.json";
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let full_config: FullOptimizerConfig = serde_json::from_reader(reader)?;

    // TIỀN TÍNH TOÁN MTF
    let prospector = Prospector::new(full_config.prospector)?;
    prospector.precompute_mtf_data(&backtest);

    let start_time = Instant::now();
    let result = backtest.run_with_dna::<HeliosStrategyV3>(&rank_1_dna)?;
    let elapsed = start_time.elapsed();

    println!("\n=== BACKTEST RESULTS V3 ===");
    println!("Time: {:?}", elapsed);
    println!("Return: {:.2}%", result.stats.return_pct);
    println!("Trades: {}", result.stats.total_trades);
    println!("Max DD: {:.2}%", result.stats.max_drawdown_pct);

    Ok(())
}
