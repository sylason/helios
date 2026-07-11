// src/strategy/core_v3/risk.rs

use ferro_trade::strategy::TickContext;
use crate::strategy::config::HeliosConfig;
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
        config: &HeliosConfig,
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

        // Kiểm tra biến động (Anomaly)
        let anomaly_threshold = config.market_regime.anomaly.z_score_threshold;
        let current_anomaly = matrix.anomaly_multiplier[i]; 

        let is_volatile = current_anomaly > anomaly_threshold;
        
        // Nới lỏng (loose) khi bão tố (breakout/crash) để tránh cắn SL sớm
        let multiplier = if is_volatile {
            risk_cfg.tsl_atr_loose
        } else {
            risk_cfg.tsl_atr_normal
        };

        for trade in tick.trades.iter().filter(|t| t.is_open()) {
            let entry_bar = trade.entry_bar;
            
            let highs = tick.highs();
            let lows = tick.lows();
            
            let peak = if trade.is_long() {
                // Peak là High cao nhất từ lúc vào lệnh
                let mut max_high = highs[entry_bar];
                for j in entry_bar..=i {
                    if highs[j] > max_high { max_high = highs[j]; }
                }
                max_high
            } else {
                // Peak là Low thấp nhất từ lúc vào lệnh
                let mut min_low = lows[entry_bar];
                for j in entry_bar..=i {
                    if lows[j] < min_low { min_low = lows[j]; }
                }
                min_low
            };

            let new_sl = if trade.is_long() {
                peak - (current_atr * multiplier)
            } else {
                peak + (current_atr * multiplier)
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
