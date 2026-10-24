// strategies_backtests/helios/examples/measure_momentum_bounds.rs

mod common;

use std::collections::HashMap;
use ferro_trade::backtest::BacktestBuilder;
use ferro_trade::data::load_data;
use ferro_trade::strategy::{Strategy, TickContext, InitContext};
use ferro_trade::indicators::factory::BulkIndicatorFactory;
use ferro_trade::optimizer::params::ParamValue;
use helios::strategy::core_v3::config::HeliosConfigV3;
use helios::strategy::sensors::HeliosSensors;

pub struct EmpiricalMomentumMeasurer {
    config: HeliosConfigV3,
    sensors: HeliosSensors,
    // Thống kê
    all_bb_half_width_atr: Vec<f64>,
    breakout_dist_from_mid_atr: Vec<f64>,
    breakout_dist_from_upper_atr: Vec<f64>,
    breakout_price_z: Vec<f64>,
    // Lọc theo điều kiện Momentum Ignition (close > bb_u && vol_z >= 1.5 && rsi >= 60)
    mom_cand_dist_mid: Vec<f64>,
    mom_cand_dist_upper: Vec<f64>,
    mom_cand_price_z: Vec<f64>,
}

impl Strategy for EmpiricalMomentumMeasurer {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfigV3::default(),
            sensors: HeliosSensors::default(),
            all_bb_half_width_atr: Vec::new(),
            breakout_dist_from_mid_atr: Vec::new(),
            breakout_dist_from_upper_atr: Vec::new(),
            breakout_price_z: Vec::new(),
            mom_cand_dist_mid: Vec::new(),
            mom_cand_dist_upper: Vec::new(),
            mom_cand_price_z: Vec::new(),
        }
    }

    fn init(&mut self, ctx: &mut InitContext) {
        let ind = &self.config.indicators;
        let mut s = HeliosSensors::default();

        let (bb_u, bb_m, bb_l, _, _, _, _) = ctx.bbands(ind.bbands.timeframe, "BB", ind.bbands.period, ind.bbands.nbdev);
        s.bb_upper = bb_u;
        s.bb_middle = bb_m;
        s.bb_lower = bb_l;

        s.z_score = ctx.z_score(ind.price_z_score.timeframe, "PriceZScore", ind.price_z_score.lookback);
        s.vol_z_score = ctx.z_score_volume(ind.vol_z_score.timeframe, "VolZScore", ind.vol_z_score.lookback);
        s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);
        let (rsi_m, _, _) = ctx.rsi(ind.rsi_momentum.timeframe, "RSI_Mom", ind.rsi_momentum.period);
        s.rsi_momentum = rsi_m;

        self.sensors = s;
    }

    fn next(&mut self, tick: &mut TickContext<'_>) {
        let i = tick.i;
        let bb_u = self.sensors.bb_upper.get(i).copied().unwrap_or(f64::NAN);
        let bb_m = self.sensors.bb_middle.get(i).copied().unwrap_or(f64::NAN);
        let atr = self.sensors.atr_sl.get(i).copied().unwrap_or(f64::NAN);
        let z_p = self.sensors.z_score.get(i).copied().unwrap_or(f64::NAN);
        let z_v = self.sensors.vol_z_score.get(i).copied().unwrap_or(f64::NAN);
        let rsi = self.sensors.rsi_momentum.get(i).copied().unwrap_or(f64::NAN);
        let close = tick.closes()[i];

        if bb_u.is_nan() || bb_m.is_nan() || atr.is_nan() || atr <= 0.0 || z_p.is_nan() || z_v.is_nan() || rsi.is_nan() {
            return;
        }

        let half_width_atr = (bb_u - bb_m) / atr;
        self.all_bb_half_width_atr.push(half_width_atr);

        if close > bb_u {
            let dist_mid = (close - bb_m) / atr;
            let dist_u = (close - bb_u) / atr;
            self.breakout_dist_from_mid_atr.push(dist_mid);
            self.breakout_dist_from_upper_atr.push(dist_u);
            self.breakout_price_z.push(z_p);

            // Ứng viên momentum: volume bùng nổ và rsi bứt phá
            if z_v >= 1.5 && rsi >= 60.0 {
                self.mom_cand_dist_mid.push(dist_mid);
                self.mom_cand_dist_upper.push(dist_u);
                self.mom_cand_price_z.push(z_p);
            }
        }
    }
}

fn percentile(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() { return 0.0; }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((v.len() - 1) as f64 * p).round() as usize;
    v[idx]
}

fn print_stats(name: &str, mut v: Vec<f64>) {
    if v.is_empty() {
        println!("{}: RỖNG (0 mẫu)", name);
        return;
    }
    let count = v.len();
    let min = percentile(&mut v, 0.0);
    let p25 = percentile(&mut v, 0.25);
    let median = percentile(&mut v, 0.50);
    let p75 = percentile(&mut v, 0.75);
    let p90 = percentile(&mut v, 0.90);
    let p95 = percentile(&mut v, 0.95);
    let p99 = percentile(&mut v, 0.99);
    let max = percentile(&mut v, 1.0);
    let mean = v.iter().sum::<f64>() / count as f64;

    println!("--------------------------------------------------");
    println!("📊 THỐNG KÊ: {} (N = {})", name, count);
    println!("  Min   : {:.3}", min);
    println!("  P25   : {:.3}", p25);
    println!("  Median: {:.3}", median);
    println!("  Mean  : {:.3}", mean);
    println!("  P75   : {:.3}", p75);
    println!("  P90   : {:.3}", p90);
    println!("  P95   : {:.3}", p95);
    println!("  P99   : {:.3}", p99);
    println!("  Max   : {:.3}", max);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CHẠY THỰC NGHIỆM ĐO ĐẠC PHÂN PHỐI MOMENTUM IGNITION ===");
    let df = load_data(common::DATA_PATH)?;
    println!("Dữ liệu nạp: {} nến 1m từ {}", df.height(), common::DATA_PATH);

    let mut measurer = EmpiricalMomentumMeasurer::new(&HashMap::new());
    
    let backtest = BacktestBuilder::new()
        .data(df)
        .timeframes(&["3m", "5m", "15m", "1h", "4h"])
        .cash(1000.0)
        .commission(0.0004)
        .margin(1.0)
        .trade_on_close(true)
        .exclusive_orders(true)
        .hedging(false)
        .finalize_trades(true)
        .fractional_unit(Some(common::FRACTIONAL_UNIT))
        .build()?;

    backtest.run(&mut measurer)?;

    println!("\n================== KẾT QUẢ ĐO ĐẠC TOÁN HỌC THỰC TẾ =================");
    print_stats("1. Độ rộng nửa dải BB 15m tính theo ATR 5m [(BB_Upper - BB_Mid) / ATR]", measurer.all_bb_half_width_atr.clone());
    
    print_stats("2. [Close > BB_Upper] Khoảng cách từ Close tới BB_Middle theo ATR", measurer.breakout_dist_from_mid_atr.clone());
    print_stats("3. [Close > BB_Upper] Khoảng cách từ Close tới BB_Upper theo ATR (Penetration)", measurer.breakout_dist_from_upper_atr.clone());
    print_stats("4. [Close > BB_Upper] Price Z-Score (5m)", measurer.breakout_price_z.clone());

    println!("\n=== BỘ LỌC ỨNG VIÊN MOMENTUM (Close > BB_Upper && Vol_Z >= 1.5 && RSI >= 60) ===");
    let total_cands = measurer.mom_cand_dist_mid.len();
    println!("Tổng số nến thỏa mãn điều kiện bứt phá xung lực: {}", total_cands);

    let count_mid_le_1_5 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 1.5).count();
    let count_mid_le_2_0 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 2.0).count();
    let count_mid_le_2_5 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 2.5).count();
    let count_mid_le_3_0 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 3.0).count();
    let count_mid_le_3_5 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 3.5).count();
    let count_mid_le_4_0 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 4.0).count();
    let count_mid_le_5_0 = measurer.mom_cand_dist_mid.iter().filter(|&&d| d <= 5.0).count();

    println!("\n[Đối soát Spec Gốc: Cự ly tính từ đường trung bình (BB_Middle)]:");
    println!("  Cự ly <= 1.5 ATR từ Middle: {} / {} nến ({:.2}%)  <-- !!! SPEC GỐC YÊU CẦU NGƯỠNG NÀY", count_mid_le_1_5, total_cands, count_mid_le_1_5 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Cự ly <= 2.0 ATR từ Middle: {} / {} nến ({:.2}%)", count_mid_le_2_0, total_cands, count_mid_le_2_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Cự ly <= 2.5 ATR từ Middle: {} / {} nến ({:.2}%)", count_mid_le_2_5, total_cands, count_mid_le_2_5 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Cự ly <= 3.0 ATR từ Middle: {} / {} nến ({:.2}%)", count_mid_le_3_0, total_cands, count_mid_le_3_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Cự ly <= 3.5 ATR từ Middle: {} / {} nến ({:.2}%)", count_mid_le_3_5, total_cands, count_mid_le_3_5 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Cự ly <= 4.0 ATR từ Middle: {} / {} nến ({:.2}%)", count_mid_le_4_0, total_cands, count_mid_le_4_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Cự ly <= 5.0 ATR từ Middle: {} / {} nến ({:.2}%)", count_mid_le_5_0, total_cands, count_mid_le_5_0 as f64 / total_cands.max(1) as f64 * 100.0);

    print_stats("5. [Ứng viên Momentum] Khoảng cách từ Close tới BB_Middle theo ATR", measurer.mom_cand_dist_mid.clone());
    print_stats("6. [Ứng viên Momentum] Độ xuyên thấu ngoài BB_Upper theo ATR", measurer.mom_cand_dist_upper.clone());
    print_stats("7. [Ứng viên Momentum] Price Z-Score (5m)", measurer.mom_cand_price_z.clone());

    let count_z_le_2_0 = measurer.mom_cand_price_z.iter().filter(|&&z| z.abs() <= 2.0).count();
    let count_z_le_2_5 = measurer.mom_cand_price_z.iter().filter(|&&z| z.abs() <= 2.5).count();
    let count_z_le_3_0 = measurer.mom_cand_price_z.iter().filter(|&&z| z.abs() <= 3.0).count();
    let count_z_gt_3_0 = measurer.mom_cand_price_z.iter().filter(|&&z| z.abs() > 3.0).count();

    println!("\n[Đối soát Spec Gốc: Trần Z-score |Price_Z| <= 3.0]:");
    println!("  |Price_Z| <= 2.0 sigma: {} / {} ({:.2}%)", count_z_le_2_0, total_cands, count_z_le_2_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  |Price_Z| <= 2.5 sigma: {} / {} ({:.2}%)", count_z_le_2_5, total_cands, count_z_le_2_5 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  |Price_Z| <= 3.0 sigma: {} / {} ({:.2}%)  <-- SPEC GỐC YÊU CẦU NGƯỠNG NÀY", count_z_le_3_0, total_cands, count_z_le_3_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  |Price_Z| >  3.0 sigma (Bão quá độ): {} / {} ({:.2}%)", count_z_gt_3_0, total_cands, count_z_gt_3_0 as f64 / total_cands.max(1) as f64 * 100.0);

    let count_upper_le_0_5 = measurer.mom_cand_dist_upper.iter().filter(|&&d| d <= 0.5).count();
    let count_upper_le_1_0 = measurer.mom_cand_dist_upper.iter().filter(|&&d| d <= 1.0).count();
    let count_upper_le_1_5 = measurer.mom_cand_dist_upper.iter().filter(|&&d| d <= 1.5).count();
    let count_upper_le_2_0 = measurer.mom_cand_dist_upper.iter().filter(|&&d| d <= 2.0).count();
    let count_upper_gt_1_5 = measurer.mom_cand_dist_upper.iter().filter(|&&d| d > 1.5).count();

    println!("\n[Đối soát Phương Án: Cự ly xuyên thấu (Close - BB_Upper) / ATR]:");
    println!("  Xuyên thấu <= 0.5 ATR: {} / {} ({:.2}%)", count_upper_le_0_5, total_cands, count_upper_le_0_5 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Xuyên thấu <= 1.0 ATR: {} / {} ({:.2}%)", count_upper_le_1_0, total_cands, count_upper_le_1_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Xuyên thấu <= 1.5 ATR: {} / {} ({:.2}%)", count_upper_le_1_5, total_cands, count_upper_le_1_5 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Xuyên thấu <= 2.0 ATR: {} / {} ({:.2}%)", count_upper_le_2_0, total_cands, count_upper_le_2_0 as f64 / total_cands.max(1) as f64 * 100.0);
    println!("  Xuyên thấu >  1.5 ATR (Vọt xà túi khí): {} / {} ({:.2}%)", count_upper_gt_1_5, total_cands, count_upper_gt_1_5 as f64 / total_cands.max(1) as f64 * 100.0);

    Ok(())
}
