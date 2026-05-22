// helios/examples/common.rs

use std::collections::HashMap;
use ferro_trade::optimizer::params::ParamValue;

// Đường dẫn đến dữ liệu thật (ETHUSDT 30 ngày)
// Sử dụng forward slashes để an toàn
pub const DATA_PATH: &str = "E:/Projects/Automations/trading_backtest/data/crypto/11-02-25/FAPI_ETHUSDT_30d_1m.csv";

// Cấu hình Backtest chung
pub const INITIAL_CASH: f64 = 60.0;     // Vốn thực tế của bạn
pub const COMMISSION: f64 = 0.0004;    // 0.04%
pub const MARGIN: f64 = 1.0 / 20.0;    // Đòn bẩy 20x
pub const FRACTIONAL_UNIT: f64 = 0.0001;

// --- QUY TẮC QUẢN LÝ VỐN "20/10" ---
pub const RISK_SIZE_RATIO: f64 = 0.2;  // Vào lệnh 1/5 tổng tài sản
pub const MIN_TRADE_AMOUNT: f64 = 10.0; // Tiền vào lệnh tối thiểu $10

/// Tạo bộ tham số mặc định cho Helios (Dymate)
pub fn get_default_params() -> HashMap<String, ParamValue> {
    let mut p = HashMap::new();
    
    // --- Indicator Params ---
    p.insert("conf_kama_short_timeframe".to_string(), ParamValue::String("3m".to_string()));
    p.insert("conf_kama_short_period".to_string(), ParamValue::Int(20));
    p.insert("conf_kama_long_timeframe".to_string(), ParamValue::String("15m".to_string()));
    p.insert("conf_kama_long_period".to_string(), ParamValue::Int(100));
    p.insert("conf_rsi_timeframe".to_string(), ParamValue::String("3m".to_string()));
    p.insert("conf_rsi_period".to_string(), ParamValue::Int(14));
    p.insert("conf_adx_timeframe".to_string(), ParamValue::String("15m".to_string()));
    p.insert("conf_adx_period".to_string(), ParamValue::Int(14));
    p.insert("conf_atr_timeframe".to_string(), ParamValue::String("5m".to_string()));
    p.insert("conf_atr_period".to_string(), ParamValue::Int(14));
    p.insert("conf_bbands_timeframe".to_string(), ParamValue::String("5m".to_string()));
    p.insert("conf_bbands_period".to_string(), ParamValue::Int(20));
    p.insert("conf_bbands_nbdev".to_string(), ParamValue::Float(2.0));
    p.insert("conf_bbands_matype".to_string(), ParamValue::String("SMA".to_string()));
    p.insert("conf_volume_ma_timeframe".to_string(), ParamValue::String("5m".to_string()));
    p.insert("conf_volume_ma_period".to_string(), ParamValue::Int(20));
    p.insert("conf_kc_timeframe".to_string(), ParamValue::String("5m".to_string()));
    p.insert("conf_kc_ema_period".to_string(), ParamValue::Int(20));
    p.insert("conf_kc_atr_period".to_string(), ParamValue::Int(10));
    p.insert("conf_kc_atr_mult".to_string(), ParamValue::Float(2.0));
    p.insert("conf_macd_timeframe".to_string(), ParamValue::String("5m".to_string()));
    p.insert("conf_macd_fast".to_string(), ParamValue::Int(12));
    p.insert("conf_macd_slow".to_string(), ParamValue::Int(26));
    p.insert("conf_macd_signal".to_string(), ParamValue::Int(9));
    p.insert("conf_momentum_timeframe".to_string(), ParamValue::String("1m".to_string()));
    p.insert("conf_momentum_period".to_string(), ParamValue::Int(5));
    p.insert("conf_stochrsi_timeframe".to_string(), ParamValue::String("3m".to_string()));
    p.insert("conf_stochrsi_period".to_string(), ParamValue::Int(14));
    p.insert("conf_stochrsi_k_period".to_string(), ParamValue::Int(3));
    p.insert("conf_stochrsi_d_period".to_string(), ParamValue::Int(3));

    // --- Analyzer Params ---
    p.insert("conf_percentile_lookback".to_string(), ParamValue::Int(50));
    p.insert("conf_min_points_for_percentile".to_string(), ParamValue::Int(50));
    p.insert("conf_trend_adx_strong_pct".to_string(), ParamValue::Int(80));
    p.insert("conf_trend_adx_moderate_pct".to_string(), ParamValue::Int(50));
    p.insert("conf_vol_atr_norm_vhigh_pct".to_string(), ParamValue::Int(95));
    p.insert("conf_vol_atr_norm_high_pct".to_string(), ParamValue::Int(80));
    p.insert("conf_vol_atr_norm_low_pct".to_string(), ParamValue::Int(20));
    p.insert("conf_vol_atr_norm_vlow_pct".to_string(), ParamValue::Int(5));
    p.insert("conf_anomaly_tr_atr_ratio".to_string(), ParamValue::Float(6.0));
    p.insert("conf_anomaly_vol_ma_ratio".to_string(), ParamValue::Float(12.0));
    p.insert("conf_momentum_rsi_aligned_thresh".to_string(), ParamValue::Int(60));
    p.insert("conf_momentum_rsi_counter_thresh".to_string(), ParamValue::Int(40));
    p.insert("conf_divergence_pivot_window".to_string(), ParamValue::Int(5));
    p.insert("conf_score_base".to_string(), ParamValue::Float(0.5));
    p.insert("conf_score_add_trending_vol_confirm".to_string(), ParamValue::Float(0.15));
    p.insert("conf_score_add_trending_mom_aligned".to_string(), ParamValue::Float(0.10));
    p.insert("conf_score_add_squeeze_very_low_vol".to_string(), ParamValue::Float(0.15));
    p.insert("conf_score_sub_trending_divergence".to_string(), ParamValue::Float(-0.25));
    p.insert("conf_score_sub_trending_vol_diverge".to_string(), ParamValue::Float(-0.15));
    p.insert("conf_score_sub_ranging_high_vol".to_string(), ParamValue::Float(-0.10));
    p.insert("conf_confidence_min_threshold".to_string(), ParamValue::Float(0.5));

    // --- Engine Params (Triggers) ---
    p.insert("conf_tpb_rsi_buy_thresh".to_string(), ParamValue::Float(45.0));
    p.insert("conf_tpb_kama_tolerance_pct".to_string(), ParamValue::Float(0.005));
    p.insert("conf_mr_rsi_oversold_thresh".to_string(), ParamValue::Float(30.0));
    p.insert("conf_mr_rsi_overbought_thresh".to_string(), ParamValue::Float(70.0));
    p.insert("conf_vcr_kc_tolerance_pct".to_string(), ParamValue::Float(0.005));
    p.insert("conf_vcr_stoch_oversold".to_string(), ParamValue::Float(20.0));
    p.insert("conf_vcr_stoch_overbought".to_string(), ParamValue::Float(80.0));
    p.insert("conf_br_rejection_body_to_wick_ratio".to_string(), ParamValue::Float(1.5));
    p.insert("conf_br_rejection_upper_wick_limit".to_string(), ParamValue::Float(0.4));

    // --- Risk Params ---
    p.insert("conf_risk_base_risk_percent".to_string(), ParamValue::Float(0.01));
    p.insert("conf_risk_confidence_low_thresh".to_string(), ParamValue::Float(0.45));
    p.insert("conf_risk_confidence_high_thresh".to_string(), ParamValue::Float(0.75));
    p.insert("conf_risk_factor_low_conf".to_string(), ParamValue::Float(0.5));
    p.insert("conf_risk_factor_normal_conf".to_string(), ParamValue::Float(1.0));
    p.insert("conf_risk_factor_high_conf".to_string(), ParamValue::Float(1.5));
    p.insert("conf_sl_atr_multiplier".to_string(), ParamValue::Float(5.0));
    p.insert("conf_sl_min_dist_pct".to_string(), ParamValue::Float(0.005));
    p.insert("conf_anchor_tpb_lookback".to_string(), ParamValue::Int(3));
    p.insert("conf_system_min_atr_value".to_string(), ParamValue::Float(1e-8));
    p.insert("conf_system_min_risk_per_unit".to_string(), ParamValue::Float(1e-8));
    p.insert("conf_max_leverage_exposure_factor".to_string(), ParamValue::Float(3.0));
    p.insert("conf_system_min_tradeable_size".to_string(), ParamValue::Float(1e-5));
    p.insert("conf_tp_default_rr_ratio".to_string(), ParamValue::Float(2.0));
    p.insert("conf_tsl_atr_mult_tight".to_string(), ParamValue::Float(1.5));
    p.insert("conf_tsl_atr_mult_normal".to_string(), ParamValue::Float(2.5));
    p.insert("conf_tsl_atr_mult_loose".to_string(), ParamValue::Float(3.5));
    p.insert("conf_pyr_min_profit_ratio_vs_1R".to_string(), ParamValue::Float(0.6));
    p.insert("conf_pyr_risk_budget_factor".to_string(), ParamValue::Float(0.5));

    p
}
