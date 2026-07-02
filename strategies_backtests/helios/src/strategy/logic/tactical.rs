// strategies_backtests/helios/src/strategy/logic/tactical.rs

use crate::types::*;
use crate::strategy::config::HeliosConfig;
use crate::strategy::memory::TacticalMemory;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::matrix::MarketMatrix;
use ferro_trade::strategy::TickContext;
use ferro_trade::broker::policies::RiskPolicy;

/// THỰC THI CHIẾN THUẬT (Cấp độ 4: Sniper Tactical)
/// Nơi Sniper soi "Kính ngắm Raw" và bóp cò.
pub fn execute_tactical(
    config: &HeliosConfig,
    memory: &mut TacticalMemory, // [FIXED Task 3.2]: Bỏ gạch dưới để sử dụng bộ nhớ
    playbook: TacticalPlaybook,
    scores: &MarketRegimeScores, 
    tick: &TickContext,
    sensors: &HeliosSensors,
    matrix: &MarketMatrix,
    env: MarketEnvironment, // [NEW]: Nhận thêm môi trường để log
) {
    let i = tick.i;
    let risk = &config.risk_management.stop_management;
    
    // --- 💎 GIÁ TRỊ TỨC THỜI (CURRENT TICK DATA) ---
    let current_close = tick.closes()[i];
    let atr_raw = sensors.atr_sl[i];
    let matrix_rsi = matrix.momentum[i];
    let volume_force = matrix.volume_force[i];
    let price_z_score = matrix.price_z_score[i];
    let kama_mid = sensors.kama_mid[i];
    let stoch_k = sensors.stoch_k[i] / 100.0;
    let macd_hist = sensors.macd_hist[i];
    let sl_min_pct = config.risk_management.risk_base.sl_min_dist_pct; // [FIXED]: Lấy đúng trường sl_min_dist_pct
    let bb_upper = sensors.bb_upper[i]; 
    let bb_lower = sensors.bb_lower[i]; 

    // --- 💰 QUẢN LÝ VỐN "NHIỀU MẠNG" (MULTI-LIFE SIZING) ---
    // [FIXED Task 3.1]: Loại bỏ Dynamic Sizing theo Score để bảo toàn tài nguyên.
    // Helios sử dụng size cố định để chịu được chuỗi thua lỗ (như số mạng trong game).
    let equity = tick.equity;
    
    let size_fraction = if equity < 100.0 {
        // Cố định 1/5 vốn (5 mạng) cho tài khoản nhỏ
        Some(0.2)
    } else {
        // Cố định 1/10 vốn (10 mạng) cho tài khoản lớn, đảm bảo lệnh tối thiểu ~20$
        Some(0.1)
    };

    // Lấy Raw Score của Playbook đang chạy để gắn vào tag
    let raw_score = match playbook {
        TacticalPlaybook::SqueezeAggressive | TacticalPlaybook::SqueezeConfirmed => scores.squeeze,
        TacticalPlaybook::TrendPullback => scores.trend,
        TacticalPlaybook::MomentumIgnition => scores.trend.max(scores.parabolic),
        TacticalPlaybook::RangeOscillator => scores.ranging,
        TacticalPlaybook::ClimaxReversal => scores.parabolic,
        TacticalPlaybook::AbsorptionScout => scores.absorption,
        TacticalPlaybook::AnomalyFade => scores.anomaly,
        _ => 1.0,
    };
    
    let final_score = raw_score * env.stability_index;
    let score_tag = format!("S{:.0}I{:.0}", final_score * 100.0, env.stability_index * 100.0);

    match playbook {
        
        // 🌪️ Đội Squeeze SWAT
        TacticalPlaybook::SqueezeAggressive | TacticalPlaybook::SqueezeConfirmed => {
            let sl_factor = risk.squeeze_sl_factor;
            let tag = if playbook == TacticalPlaybook::SqueezeAggressive { "Sqz_Agg" } else { "Sqz_Conf" };
            let score_tag = format!("S{:.0}", scores.squeeze * 100.0);
            let sqz_cfg = &config.tactical_sniper.squeeze;

            // --- 📏 ĐO LƯỜNG SỨC BUNG (VOLATILITY EXPANSION) ---
            // Thay vì nhìn vị trí giá so với Band (dễ bị trễ), ta nhìn vào Tốc độ nở của dải băng.
            // BB_Width = (Upper - Lower) / Middle.
            if i < 1 { return; }
            
            // Tính độ rộng nến hiện tại và nến trước đó
            let curr_u = sensors.bb_upper[i];
            let curr_l = sensors.bb_lower[i];
            let prev_u = sensors.bb_upper[i-1];
            let prev_l = sensors.bb_lower[i-1];
            
            let current_w = (curr_u - curr_l) / ((curr_u + curr_l) / 2.0);
            let prev_w = (prev_u - prev_l) / ((prev_u + prev_l) / 2.0);
            
            // ĐIỀU KIỆN 1: Biến động phải đang nở rộng vượt ngưỡng (Volatility Expansion)
            // Lò xo bắt đầu bung ra mạnh mẽ.
            let is_expanding = current_w > prev_w * sqz_cfg.vol_expansion_thresh;
            
            // ĐIỀU KIỆN 2: Dòng tiền phải tham gia (Volume Confirmation)
            // Tránh bẫy phá vỡ giả khi không có thanh khoản.
            let is_volume_strong = volume_force > sqz_cfg.volume_confirm_thresh;

            // --- 🏹 BÓP CÒ (TRIGGER LOGIC) ---
            
            // LONG: Biến động nở + Volume mạnh + Giá nằm trên trục KAMA + Động lượng MACD không quá xấu
            if is_expanding && is_volume_strong && current_close > kama_mid && macd_hist > -0.01 && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("{}_L_{}", tag, score_tag))
                );
                // println!("[TACTICAL] {} LONG Triggered at bar {}", tag, i);
            } 
            // SHORT: Biến động nở + Volume mạnh + Giá nằm dưới trục KAMA + Động lượng MACD không quá xấu
            else if is_expanding && is_volume_strong && current_close < kama_mid && macd_hist < 0.01 && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("{}_S_{}", tag, score_tag))
                );
                // println!("[TACTICAL] {} SHORT Triggered at bar {}", tag, i);
            }
        },

        // 🌊 Đội Trend SWAT
        TacticalPlaybook::TrendPullback => {
            let sl_factor = risk.trend_sl_factor;
            let pb_cfg = &config.tactical_sniper.pullback;

            // Xác nhận động lượng quay đầu (MACD Histogram Pivot)
            // Lấy nến i-N để so sánh (N = macd_pivot_lookback)
            let lookback_n = pb_cfg.macd_lookback.max(1.0) as usize;
            
            let is_momentum_turning_up = if i >= lookback_n {
                sensors.macd_hist[i] > sensors.macd_hist[i - lookback_n]
            } else {
                false
            };

            let is_momentum_turning_down = if i >= lookback_n {
                sensors.macd_hist[i] < sensors.macd_hist[i - lookback_n]
            } else {
                false
            };

            // LONG: Giá > KAMA AND Stoch < Threshold (Vùng rẻ) AND MACD quay đầu AND Chưa có vị thế
            if current_close > kama_mid && stoch_k < pb_cfg.stoch_pullback_thresh && is_momentum_turning_up && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Trend_PB_L_{}", score_tag))
                );
                // println!("[TACTICAL] Trend_PB LONG Triggered at bar {}", i);
            } 
            // SHORT: Giá < KAMA AND Stoch > Threshold (Vùng đắt) AND MACD quay đầu AND Chưa có vị thế
            else if current_close < kama_mid && stoch_k > (1.0 - pb_cfg.stoch_pullback_thresh) && is_momentum_turning_down && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Trend_PB_S_{}", score_tag))
                );
                // println!("[TACTICAL] Trend_PB SHORT Triggered at bar {}", i);
            }
        },

        // 🚀 Đội Momentum SWAT
        TacticalPlaybook::MomentumIgnition => {
            let sl_factor = risk.momentum_sl_factor;
            let mom_cfg = &config.tactical_sniper.momentum;
            
            // --- 🏎️ XÁC NHẬN GIA TỐC (ACCELERATION) ---
            let lookback_n = mom_cfg.macd_lookback.max(1.0) as usize;
            
            let is_accelerating_up = if i >= lookback_n {
                sensors.macd_hist[i] > sensors.macd_hist[i - lookback_n] * mom_cfg.accel_factor
            } else {
                false
            };

            let is_accelerating_down = if i >= lookback_n {
                sensors.macd_hist[i] < sensors.macd_hist[i - lookback_n] * mom_cfg.accel_factor
            } else {
                false
            };

            // --- 🛑 MÀNG LỌC QUÁ NHIỆT (CLIMAX PREVENTER) ---
            let is_rsi_safe_long = matrix_rsi > mom_cfg.rsi_threshold && matrix_rsi < mom_cfg.rsi_climax;
            let is_rsi_safe_short = matrix_rsi < (1.0 - mom_cfg.rsi_threshold) && matrix_rsi > (1.0 - mom_cfg.rsi_climax);

            // --- 🏹 BÓP CÒ (TRIGGER LOGIC) ---
            
            // LONG: ... AND Chưa có vị thế
            if is_rsi_safe_long && is_accelerating_up && volume_force > mom_cfg.volume_spike_min && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Mom_L_{}", score_tag))
                );
                // println!("[TACTICAL] Momentum LONG Triggered at bar {}", i);
            } 
            // SHORT: ... AND Chưa có vị thế
            else if is_rsi_safe_short && is_accelerating_down && volume_force > mom_cfg.volume_spike_min && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Mom_S_{}", score_tag))
                );
                // println!("[TACTICAL] Momentum SHORT Triggered at bar {}", i);
            }
        },

        // 🏓 Đội Range SWAT
        TacticalPlaybook::RangeOscillator => {
            let sl_factor = risk.range_sl_factor;
            let range_cfg = &config.tactical_sniper.range;
            
            // --- 📏 KIỂM TRA BIÊN ĐỘ (SAFETY CHECK) ---
            let range_width = bb_upper - bb_lower;
            if range_width / current_close < range_cfg.min_width_pct {
                return; // Bỏ qua nếu hành lang quá hẹp (không đủ bù spread/phí)
            }

            // --- 🧪 XÁC NHẬN KIỆT SỨC (EXHAUSTION) ---
            let is_exhausted = volume_force < range_cfg.volume_max_threshold;
            
            // --- 🏹 NGÒI NỔ ĐỘNG LƯỢNG (MOMENTUM PIVOT) ---
            let is_rebounding_up = if i >= 1 { sensors.macd_hist[i] > sensors.macd_hist[i-1] } else { false };
            let is_rebounding_down = if i >= 1 { sensors.macd_hist[i] < sensors.macd_hist[i-1] } else { false };

            // LONG: ... AND Chưa có vị thế
            if current_close < bb_lower && is_exhausted && stoch_k < range_cfg.stoch_oversold && is_rebounding_up && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = (bb_upper + bb_lower) / 2.0; // Chốt lời tại trục giữa hành lang
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Range_L_{}", score_tag))
                );
                // println!("[TACTICAL] Range_Osc LONG Triggered at bar {}", i);
            } 
            // SHORT: ... AND Chưa có vị thế
            else if current_close > bb_upper && is_exhausted && stoch_k > range_cfg.stoch_overbought && is_rebounding_down && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = (bb_upper + bb_lower) / 2.0; // Chốt lời tại trục giữa hành lang
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Range_S_{}", score_tag))
                );
                // println!("[TACTICAL] Range_Osc SHORT Triggered at bar {}", i);
            }
        },

        // 🌋 Đội Reversal SWAT
        TacticalPlaybook::ClimaxReversal => {
            let sl_factor = risk.reversal_sl_factor;
            let rev_cfg = &config.tactical_sniper.reversal;
            let score = scores.parabolic;
            
            // --- 🏹 NGÒI NỔ ĐIỂM GÃY (MOMENTUM COLLAPSE) ---
            let lookback_n = rev_cfg.macd_lookback.max(1.0) as usize;

            let is_momentum_collapsing_up = if i >= lookback_n {
                sensors.macd_hist[i] < sensors.macd_hist[i - lookback_n]
            } else {
                false
            };

            let is_momentum_collapsing_down = if i >= lookback_n {
                sensors.macd_hist[i] > sensors.macd_hist[i - lookback_n]
            } else {
                false
            };

            // CƠ CHẾ EMERGENCY OVERRIDE: Cho phép lật vị thế nếu Score > 0.98
            let can_trigger = !tick.position().is_open() || score > 0.98;

            // LONG (Bắt đáy)
            if can_trigger && matrix_rsi < (1.0 - rev_cfg.rsi_extreme) && price_z_score.abs() > rev_cfg.z_score_threshold && is_momentum_collapsing_down {
                if tick.position().is_open() {
                    // println!("[TACTICAL] EMERGENCY OVERRIDE: Closing old position for Climax LONG");
                    tick.close(None);
                }
                
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr * 2.0);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Climax_Rev_L_{}", score_tag))
                );
            }
            // SHORT (Bắt đỉnh)
            else if can_trigger && matrix_rsi > rev_cfg.rsi_extreme && price_z_score.abs() > rev_cfg.z_score_threshold && is_momentum_collapsing_up {
                if tick.position().is_open() {
                    // println!("[TACTICAL] EMERGENCY OVERRIDE: Closing old position for Climax SHORT");
                    tick.close(None);
                }

                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr * 2.0);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Climax_Rev_S_{}", score_tag))
                );
            }
        },

        // 🧽 Đội Absorption Scout
        TacticalPlaybook::AbsorptionScout => {
            let abs_cfg = &config.tactical_sniper.absorption;
            let risk_cfg = &config.risk_management.stop_management;

            // --- 🐳 XÁC NHẬN DẤU VẾT CÁ MẬP (SHARK TRACKS) ---
            let is_shark_present = volume_force > abs_cfg.vol_min_threshold;

            // --- 🚀 NGÒI NỔ BỨT PHÁ (THE LAUNCH TRIGGER) ---
            let lookback_n = abs_cfg.macd_lookback.max(1.0) as usize;
            
            let is_breaking_out_up = if i >= lookback_n {
                sensors.macd_hist[i] > sensors.macd_hist[i - lookback_n]
            } else {
                false
            };

            let is_breaking_out_down = if i >= lookback_n {
                sensors.macd_hist[i] < sensors.macd_hist[i - lookback_n]
            } else {
                false
            };

            // LONG: ... AND Chưa có vị thế
            if is_shark_present && is_breaking_out_up && matrix_rsi > config.market_regime.absorption.bias_threshold && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk_cfg.base_sl_atr * 0.7).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk_cfg.base_tp_rr * 1.5);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Absorp_L_{}", score_tag))
                );
                // println!("[TACTICAL] Absorption LONG Triggered at bar {}", i);
            } 
            // SHORT: ... AND Chưa có vị thế
            else if is_shark_present && is_breaking_out_down && matrix_rsi < (1.0 - config.market_regime.absorption.bias_threshold) && !tick.position().is_open() {
                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk_cfg.base_sl_atr * 0.7).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk_cfg.base_tp_rr * 1.5);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Absorp_S_{}", score_tag))
                );
                // println!("[TACTICAL] Absorption SHORT Triggered at bar {}", i);
            }
        },

        // ⚡ Đội Anomaly Fade
        TacticalPlaybook::AnomalyFade => {
            let sl_factor = risk.anomaly_sl_factor;
            let ano_cfg = &config.tactical_sniper.anomaly;
            let score = scores.anomaly;
            
            // --- 🏹 NGÒI NỔ CÚ NẢY (THE BOUNCE TRIGGER) ---
            // Đợi động lượng xác nhận giá đã ngừng lao dốc/phi mã và bắt đầu hồi.
            let i = tick.i;
            let lookback_n = ano_cfg.macd_lookback.max(1.0) as usize;
            let hist_series = sensors.macd_hist.as_ref();
            
            let is_momentum_turning_up = if i >= lookback_n {
                hist_series[i] > hist_series[i - lookback_n]
            } else {
                false
            };

            let is_momentum_turning_down = if i >= lookback_n {
                hist_series[i] < hist_series[i - lookback_n]
            } else {
                false
            };

            // --- 🛡️ XÁC NHẬN RÂU NẾN (REJECTION CHECK) ---
            let high = tick.highs()[i];
            let low = tick.lows()[i];
            let close = tick.closes()[i];
            let body_range = high - low;
            
            let is_bullish_rejection = if body_range > 0.0 {
                (close - low) / body_range > ano_cfg.min_rejection_pct
            } else {
                false
            };

            let is_bearish_rejection = if body_range > 0.0 {
                (high - close) / body_range > ano_cfg.min_rejection_pct
            } else {
                false
            };

            // CƠ CHẾ EMERGENCY OVERRIDE: Cho phép lật vị thế nếu Score > 0.98
            let can_trigger = !tick.position().is_open() || score > 0.98;

            // LONG: Z-Score cực thấp AND RSI cực thấp AND MACD hồi lên AND Có râu nến dưới
            if can_trigger && price_z_score.abs() > ano_cfg.z_score_threshold && matrix_rsi < (1.0 - ano_cfg.rsi_climax) 
               && is_momentum_turning_up && is_bullish_rejection 
            {
                if tick.position().is_open() {
                    // println!("[TACTICAL] EMERGENCY OVERRIDE: Closing old position for Anomaly LONG");
                    tick.close(None);
                }

                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * 2.0); 
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Ano_Fade_L_{}", score_tag))
                );
            } 
            // SHORT: Z-Score cực cao AND RSI cực cao AND MACD hồi xuống AND Có râu nến trên
            else if can_trigger && price_z_score.abs() > ano_cfg.z_score_threshold && matrix_rsi > ano_cfg.rsi_climax 
                    && is_momentum_turning_down && is_bearish_rejection 
            {
                if tick.position().is_open() {
                    // println!("[TACTICAL] EMERGENCY OVERRIDE: Closing old position for Anomaly SHORT");
                    tick.close(None);
                }

                memory.opening_playbook = playbook; // [FIXED Task 3.2]
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * 2.0);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Ano_Fade_S_{}", score_tag))
                );
            }
        },

        // 🧱 Đội Phòng thủ
        TacticalPlaybook::Wait => {}
    }
}
