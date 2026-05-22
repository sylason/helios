// strategies_backtests/helios/src/strategy/logic/tactical.rs

use crate::types::*;
use crate::strategy::config::HeliosConfig;
use crate::strategy::memory::TacticalMemory;
use ferro_trade::strategy::TickContext;
use ferro_trade::broker::policies::RiskPolicy;

/// THỰC THI CHIẾN THUẬT (Cấp độ 4: Sniper Tactical)
/// Nơi Sniper soi "Kính ngắm Raw" và bóp cò.
pub fn execute_tactical(
    config: &HeliosConfig,
    _memory: &mut TacticalMemory,
    playbook: TacticalPlaybook,
    scores: &MarketRegimeScores, // NEW: Bảng điểm để ghi nhãn chẩn đoán
    tick: &TickContext,
    // --- KÍNH NGẮM SNIPER (DỮ LIỆU ĐÃ ĐƯỢC CHUẨN HÓA) ---
    current_close: f64,
    bb_upper: f64,
    bb_lower: f64,
    kama_mid: f64,
    matrix_rsi: f64, 
    stoch_k: f64,
    stoch_cross: f64,  // Tín hiệu cắt: 1.0 (Lên), -1.0 (Xuống)
    _stoch_k_zone: f64, // Tín hiệu vùng
    _macd_main: f64,
    macd_hist: f64,    // Động lượng sớm (Xác nhận Squeeze)
    atr_raw: f64,
    volume_force: f64,   
    price_z_score: f64,  
) {
    let risk = &config.risk_management.stop_management;

    // --- 💰 QUẢN LÝ VỐN LINH HOẠT (DYNAMIC SIZING) ---
    let equity = tick.equity;
    let base_lot_size = config.risk_management.risk_base.base_lot_size;
    
    // Tỉ lệ quy mô dựa trên điểm số (Score Factor)
    // Giúp Helios tự động đánh lớn khi tín hiệu cực mạnh và thu hẹp khi tín hiệu vừa đủ.
    let score_factor = match playbook {
        TacticalPlaybook::SqueezeAggressive | TacticalPlaybook::SqueezeConfirmed => scores.squeeze,
        TacticalPlaybook::TrendPullback => scores.trend,
        TacticalPlaybook::MomentumIgnition => scores.trend.max(scores.parabolic),
        TacticalPlaybook::RangeOscillator => scores.ranging,
        TacticalPlaybook::ClimaxReversal => scores.parabolic,
        TacticalPlaybook::AbsorptionScout => scores.absorption,
        TacticalPlaybook::AnomalyFade => scores.anomaly,
        _ => 1.0,
    }.min(2.0).max(0.2); // Giới hạn từ 0.2x đến 2.0x để đảm bảo an toàn vốn

    let margin_intended = (equity * base_lot_size * score_factor).max(10.0);
    
    if margin_intended > equity {
        return;
    }
    let size_fraction = Some(margin_intended / equity);

    let rsi_thresh = if config.tactical_sniper.pullback.rsi_buy_thresh > 1.0 {
        config.tactical_sniper.pullback.rsi_buy_thresh / 100.0
    } else {
        config.tactical_sniper.pullback.rsi_buy_thresh
    };

    // --- 🧭 SNIPER BÓP CÒ DỰA TRÊN PLAYBOOK ---
    let sl_min_pct = config.risk_management.risk_base.sl_min_dist_pct;

    match playbook {
        
        // 🌪️ Đội Squeeze SWAT
        TacticalPlaybook::SqueezeAggressive | TacticalPlaybook::SqueezeConfirmed => {
            let sl_factor = risk.squeeze_sl_factor;
            let tag = if playbook == TacticalPlaybook::SqueezeAggressive { "Sqz_Agg" } else { "Sqz_Conf" };
            let score_tag = format!("S{:.0}", scores.squeeze * 100.0);

            if current_close > bb_upper && macd_hist > -0.01 && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("{}_L_{}", tag, score_tag))
                );
            } else if current_close < bb_lower && macd_hist < 0.01 && !tick.position().is_short {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("{}_S_{}", tag, score_tag))
                );
            }
        },

        // 🌊 Đội Trend SWAT
        TacticalPlaybook::TrendPullback => {
            let sl_factor = risk.trend_sl_factor;
            let score_tag = format!("S{:.0}", scores.trend * 100.0);

            if current_close > kama_mid && matrix_rsi < rsi_thresh && stoch_cross == 1.0 && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Trend_PB_L_{}", score_tag))
                );
            } 
            else if current_close < kama_mid && matrix_rsi > (1.0 - rsi_thresh) && stoch_cross == -1.0 && !tick.position().is_short {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Trend_PB_S_{}", score_tag))
                );
            }
        },

        // 🚀 Đội Momentum SWAT
        TacticalPlaybook::MomentumIgnition => {
            let sl_factor = risk.momentum_sl_factor;
            let mom_cfg = &config.tactical_sniper.momentum;
            let score_tag = format!("S{:.0}", scores.trend.max(scores.parabolic) * 100.0);
            
            if matrix_rsi > mom_cfg.rsi_threshold && volume_force > mom_cfg.volume_spike_min && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Mom_L_{}", score_tag))
                );
            } 
            else if matrix_rsi < (1.0 - mom_cfg.rsi_threshold) && volume_force > mom_cfg.volume_spike_min && !tick.position().is_short {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Mom_S_{}", score_tag))
                );
            }
        },

        // 🏓 Đội Range SWAT
        TacticalPlaybook::RangeOscillator => {
            let sl_factor = risk.range_sl_factor;
            let range_cfg = &config.tactical_sniper.range;
            let range_width = bb_upper - bb_lower;
            if range_width / current_close < range_cfg.min_width_pct {
                return;
            }

            let is_exhausted = volume_force < 1.0;
            let score_tag = format!("S{:.0}", scores.ranging * 100.0);

            if current_close < bb_lower && is_exhausted && stoch_k < range_cfg.stoch_oversold && stoch_cross == 1.0 && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = (bb_upper + bb_lower) / 2.0; 
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Range_L_{}", score_tag))
                );
            } 
            else if current_close > bb_upper && is_exhausted && stoch_k > range_cfg.stoch_overbought && stoch_cross == -1.0 && !tick.position().is_short {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = (bb_upper + bb_lower) / 2.0;
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Range_S_{}", score_tag))
                );
            }
        },

        // 🌋 Đội Reversal SWAT
        TacticalPlaybook::ClimaxReversal => {
            let sl_factor = risk.reversal_sl_factor;
            let rev_cfg = &config.tactical_sniper.reversal;
            let score_tag = format!("S{:.0}", scores.parabolic * 100.0);
            
            if matrix_rsi < (1.0 - rev_cfg.rsi_extreme) && price_z_score < -2.5 && volume_force < 1.2 && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr * 2.0);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Climax_Rev_L_{}", score_tag))
                );
            }
            else if matrix_rsi > rev_cfg.rsi_extreme && price_z_score > 2.5 && volume_force < 1.2 && !tick.position().is_short {
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
            let score_tag = format!("S{:.0}", scores.absorption * 100.0);
            if matrix_rsi > config.market_regime.absorption.bias_threshold && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * 0.7).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * risk.base_tp_rr * 1.5);
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Absorp_L_{}", score_tag))
                );
            } else if matrix_rsi < (1.0 - config.market_regime.absorption.bias_threshold) && !tick.position().is_short {
                let sl_dist = (atr_raw * risk.base_sl_atr * 0.7).max(current_close * sl_min_pct);
                let sl = current_close + sl_dist;
                let tp = current_close - (sl_dist * risk.base_tp_rr * 1.5);
                
                tick.sell(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Absorp_S_{}", score_tag))
                );
            }
        },

        // ⚡ Đội Anomaly Fade
        TacticalPlaybook::AnomalyFade => {
            let sl_factor = risk.anomaly_sl_factor;
            let ano_cfg = &config.tactical_sniper.anomaly;
            let score_tag = format!("S{:.0}", scores.anomaly * 100.0);
            
            if price_z_score < -config.market_regime.anomaly.z_score_threshold && matrix_rsi > ano_cfg.rsi_climax && !tick.position().is_long {
                let sl_dist = (atr_raw * risk.base_sl_atr * sl_factor).max(current_close * sl_min_pct);
                let sl = current_close - sl_dist;
                let tp = current_close + (sl_dist * 2.0); 
                
                tick.buy(size_fraction, None, None, 
                    Some(RiskPolicy::FixedPrice { sl_price: Some(sl), tp_price: Some(tp) }), 
                    Some(format!("Ano_Fade_L_{}", score_tag))
                );
            } 
            else if price_z_score > config.market_regime.anomaly.z_score_threshold && matrix_rsi < (1.0 - ano_cfg.rsi_climax) && !tick.position().is_short {
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
