// src/strategy/core_v3/risk.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::sensors::HeliosSensors;

pub struct RiskManager;

impl RiskManager {
    /// Tính toán Size dựa trên Fixed Percentage (Fraction của Ký quỹ) nhưng đảm bảo Min Order Size.
    /// Trả về một giá trị < 1.0 (ví dụ 0.1 = 10%) để `ferro_trade` dùng làm Initial Margin.
    pub fn calculate_size(equity: f64, risk_pct: f64, min_order_usd: f64, leverage: f64) -> f64 {
        // 1. Số tiền ký quỹ mục tiêu (Target Initial Margin)
        let mut target_margin = equity * risk_pct;
        
        // 2. Kích thước lệnh thực tế (Order Size USD)
        let mut order_size_usd = target_margin * leverage;
        
        // 3. Đảm bảo Order Size >= Min Order (Binance yêu cầu ~ 21 USDT)
        if order_size_usd < min_order_usd {
            order_size_usd = min_order_usd;
            target_margin = order_size_usd / leverage;
        }
        
        // 4. Trả về tỷ lệ phần trăm (size_fraction) so với Equity hiện tại
        let size_fraction = target_margin / equity;
        
        // Giới hạn an toàn tối đa 95% vốn để trừ hao phí và trượt giá
        size_fraction.min(0.95)
    }

    /// Tính giá cắt lỗ khởi tạo
    pub fn get_initial_sl(
        tick: &TickContext,
        sensors: &HeliosSensors,
        is_long: bool,
        multiplier: f64
    ) -> f64 {
        let i = tick.i;
        let current_atr = sensors.atr_sl[i];
        let price = tick.closes()[i];
        if is_long {
            price - (current_atr * multiplier)
        } else {
            price + (current_atr * multiplier)
        }
    }

    /// Quản lý Trailing Stop cho tất cả các lệnh đang mở (Adaptive Loose Trailing)
    pub fn manage_trailing_stop(
        config: &HeliosConfigV3,
        matrix: &MarketMatrixV3,
        tick: &TickContext,
        sensors: &HeliosSensors,
        debug_mode: bool,
    ) {
        let i = tick.i;
        let risk_cfg = &config.risk_management.stop_management;
        
        let current_atr = sensors.atr_sl[i];
        if current_atr.is_nan() || current_atr <= 0.0 {
            return;
        }

        let anomaly_threshold = config.tactical_sniper.anomaly.z_score_threshold;
        let current_anomaly = matrix.anomaly_multiplier[i]; 
        let is_volatile = current_anomaly > anomaly_threshold;
        
        let current_squeeze = matrix.squeeze_multiplier[i];
        let is_squeeze = current_squeeze > 0.8; // High squeeze intensity

        for trade in tick.trades.iter().filter(|t| t.is_open()) {
            let entry_bar = trade.entry_bar;
            let duration = i - entry_bar;
            
            // 1. Base Multiplier by Playbook Tag
            let base_multiplier = if let Some(ref tag_str) = trade.tag {
                match tag_str.as_str() {
                    "RANGE_OSC" => risk_cfg.tsl_atr_tight,
                    "MOMENTUM_IGN" | "ANOMALY_BREAKOUT" => risk_cfg.tsl_atr_loose,
                    "TREND_PB" | "ABSORPTION" => risk_cfg.tsl_atr_normal,
                    _ => risk_cfg.tsl_atr_normal,
                }
            } else {
                risk_cfg.tsl_atr_normal
            };

            // Tìm đỉnh lợi nhuận cao nhất (High Watermark) và vị trí nến tạo đỉnh
            let highs = tick.highs();
            let lows = tick.lows();
            
            let (peak_val, peak_bar) = if trade.is_long() {
                let mut max_high = highs[entry_bar];
                let mut max_bar = entry_bar;
                for j in entry_bar..=i {
                    if highs[j] > max_high { 
                        max_high = highs[j]; 
                        max_bar = j;
                    }
                }
                (max_high, max_bar)
            } else {
                let mut min_low = lows[entry_bar];
                let mut min_bar = entry_bar;
                for j in entry_bar..=i {
                    if lows[j] < min_low { 
                        min_low = lows[j]; 
                        min_bar = j;
                    }
                }
                (min_low, min_bar)
            };

            // Đếm số nến đi ngang kể từ đỉnh, TẠM DỪNG đếm nếu thị trường đang nén lò xo (Squeeze)
            let mut effective_stagnant_bars = 0;
            for j in peak_bar..=i {
                if matrix.squeeze_multiplier[j] <= 0.8 {
                    effective_stagnant_bars += 1;
                }
            }

            // Tính số chu kỳ đếm ngược đã trôi qua
            let time_countdown_bars = risk_cfg.time_countdown_bars;
            let decay_cycles = effective_stagnant_bars / time_countdown_bars;
            
            // Lệnh lỗ mà ngâm quá 1 chu kỳ thì cắt luôn
            let current_pnl = trade.pl(Some(tick.closes()[i]));
            if decay_cycles > 0 && current_pnl <= 0.0 {
                tick.close(None);
                continue;
            }

            // Tính toán hệ số nhân Multiplier hiện tại (Bao gồm Progressive Decay)
            let mut current_multiplier = if is_volatile {
                // Rule 1: Explosive market. Give it room to breathe (avoid whipsaw).
                if base_multiplier < risk_cfg.tsl_atr_loose {
                    risk_cfg.tsl_atr_loose
                } else {
                    base_multiplier
                }
            } else {
                base_multiplier
            };

            // Áp dụng Progressive Decay: Lệnh ngâm càng lâu, siết SL càng chặt
            if decay_cycles > 0 {
                // Nhân hệ số siết (ví dụ 0.8) cho mỗi chu kỳ trôi qua
                let progressive_factor = risk_cfg.tighten_step_multiplier.powi(decay_cycles as i32);
                current_multiplier *= progressive_factor;
                
                // Đảm bảo không bóp chặt quá mức tối thiểu
                if current_multiplier < risk_cfg.tsl_atr_tight {
                    current_multiplier = risk_cfg.tsl_atr_tight;
                }
            }

            let new_sl = if trade.is_long() {
                peak_val - (current_atr * current_multiplier)
            } else {
                peak_val + (current_atr * current_multiplier)
            };

            let should_update = if trade.is_long() {
                new_sl > trade.sl_price.unwrap_or(f64::NEG_INFINITY)
            } else {
                new_sl < trade.sl_price.unwrap_or(f64::INFINITY)
            };

            if should_update {
                tick.update_trade_sl_tp(trade.id, Some(new_sl), trade.tp_price);
                if debug_mode {
                    // println!("[Bar {}] Trailing Stop Updated: Trade ID {} | New SL: {:.2} | Peak: {:.2} | Volatile: {}", 
                    //     i, trade.id, new_sl, peak, is_volatile);
                }
            }
        }
    }
}
