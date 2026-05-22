// strategies_backtests/helios/src/strategy/mod.rs

pub mod config;
pub mod logic;
pub mod memory;
pub mod sensors;
pub mod matrix;

use std::collections::HashMap;
use std::sync::Arc;

use ferro_trade::optimizer::params::ParamValue;
use ferro_trade::optimizer::config_parser;
use ferro_trade::strategy::{InitContext, Strategy, TickContext};
use ferro_trade::indicators::factory::BulkIndicatorFactory;

use crate::types::*;
use crate::strategy::config::HeliosConfig;
use crate::strategy::memory::TacticalMemory;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::matrix::MarketMatrix;

/// Chiến lược Helios - Nhạc trưởng điều phối 4 cấp độ chiến thuật.
/// [CLEAN ARCHITECTURE]: Phân tách Parameters (Config) -> Data (Sensors) -> Perception (Matrix).
#[derive(Clone, Debug)]
pub struct HeliosStrategy {
    pub config: HeliosConfig,
    pub memory: TacticalMemory,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrix,
}

impl Strategy for HeliosStrategy {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfig::default(),
            memory: TacticalMemory::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrix::default(),
        }
    }

    /// [ELITE] Tự động hóa DNA-to-Struct mapping.
    fn update_from_dna(&mut self, dna: &[f64], mapping: &HashMap<String, usize>) {
        let tf_options = vec!["1m", "5m", "15m", "1h", "4h"];
        let ma_options = vec![
            "SMA", "EMA", "WMA", "HMA", "RMA", "DMA", "DEMA", "TMA", "TEMA", "WSMA", "SMM", "SWMA", "TRIMA", "LinReg", "Vidya", "T3"
        ];
        
        match config_parser::bind_params(dna, mapping, &tf_options, &ma_options) {
            Ok(new_config) => {
                self.config = new_config;
                // println!("[HELIOS] DNA Bind Success. NATR: {}, MFI: {}", 
                //     self.config.indicators.natr.period, self.config.indicators.mfi.period);
            },
            Err(e) => {
                eprintln!("[HELIOS] BIND ERROR: {}. Please DELETE your .db file to sync new JSON fields!", e);
            }
        }
    }

    /// [ELITE] Khởi tạo hệ thống Giác quan (Sensors) & Nhận thức (Matrix).
    fn init(&mut self, ctx: &mut InitContext) {
        let ind = &self.config.indicators;

        // [SAFETY] Ngăn chặn panic nếu period bị bind lỗi về 0 (thường do DB cũ)
        if ind.natr.period == 0 || ind.mfi.period == 0 || ind.vol_ma.period == 0 {
            return;
        }

        // --- 1. RAW SENSORS (HỨNG DỮ LIỆU THÔ) ---
        let mut s = HeliosSensors::default();
        
        // Nhóm Environment & Chaos
        s.natr = ctx.natr(&ind.natr.timeframe, "NATR", ind.natr.period);
        s.chaos_ratio = ctx.chaos_ratio(&ind.natr.timeframe, "ChaosRatio", ind.natr.period);
        s.vol_ma_series = ctx.volume_sma(&ind.vol_ma.timeframe, "VOL_MA", ind.vol_ma.period);
        s.efficiency_ratio = ctx.efficiency_ratio(&ind.kama.timeframe, "ER", ind.kama.short_period);
        let (mfi, _, _, _, _) = ctx.mfi(&ind.mfi.timeframe, "MFI", ind.mfi.period);
        s.mfi = mfi;

        // Nhóm Trend & Regime
        let (adx, _, _, _) = ctx.adx(&ind.adx.timeframe, "ADX", ind.adx.period);
        s.adx = adx;
        s.linreg_slope = ctx.linreg_slope(&ind.linreg.timeframe, "Slope", ind.linreg.period);
        s.linreg_angle = ctx.linreg_angle(&ind.linreg.timeframe, "Angle", ind.linreg.period);
        s.kama_mid = ctx.kama(&ind.kama.timeframe, "KAMA", ind.kama.short_period, 2, ind.kama.long_period);

        // Nhóm Volatility (Squeeze Intensity)
        let (bb_u, _, bb_l, _, _, _, _) = ctx.bbands(&ind.bbands.timeframe, "BB", ind.bbands.period, ind.bbands.nbdev);
        s.bb_upper = bb_u;
        s.bb_lower = bb_l;
        let (kc_u, _, kc_l) = ctx.keltner(&ind.keltner.timeframe, "KC", ind.keltner.period as u32, ind.keltner.multiplier, ind.keltner.ma_type.clone());
        s.kc_upper = kc_u;
        s.kc_lower = kc_l;

        // Nhóm Momentum (Velocity)
        let (rsi, _, _) = ctx.rsi(&ind.rsi.timeframe, "RSI", ind.rsi.period);
        s.rsi = rsi;
        let (macd_m, macd_s, macd_h, _, _) = ctx.macd(&ind.macd.timeframe, "MACD", ind.macd.fast, ind.macd.slow, ind.macd.signal);
        s.macd_main = macd_m;
        s.macd_signal = macd_s;
        s.macd_hist = macd_h;

        // Nhóm Sniper (Stochastic Triggers)
        let (st_k, st_d, st_cr, st_kz, st_dz) = ctx.stoch(
            &ind.stochastic.timeframe, "STOCH", 
            ind.stochastic.period as u32, 
            ind.stochastic.k_period as u32, ind.stochastic.k_ma_type.clone(),
            ind.stochastic.d_period as u32, ind.stochastic.d_ma_type.clone(),
            ind.stochastic.zone
        );
        s.stoch_k = st_k;
        s.stoch_d = st_d;
        s.stoch_cross = st_cr;
        s.stoch_k_zone = st_kz;
        s.stoch_d_zone = st_dz;

        // Risk Management
        s.atr_sl = ctx.atr(&ind.atr.timeframe, "ATR_SL", ind.atr.period);

        // --- 2. MARKET MATRIX (CHUYỂN HÓA NHẬN THỨC) ---
        // Tính vol_ratio để normalize Volume 1m so với MA khung cao
        let vol_ratio = match ind.vol_ma.timeframe.as_str() {
            "5m" => 5.0,
            "15m" => 15.0,
            "1h" => 60.0,
            "4h" => 240.0,
            _ => 1.0,
        };

        self.sensors = s;
        self.sensors.closes = Arc::new(ctx.closes().to_vec()); // FIX #10
        self.matrix = MarketMatrix::from_sensors(&self.config, &self.sensors, &ctx.volumes(), vol_ratio);
    }

    fn next(&mut self, tick: &TickContext) {
        let i = tick.i;
        
        // --- 🛡️ SAFETY FUSE (Kiểm tra dữ liệu) ---
        if self.sensors.atr_sl.len() <= i || self.matrix.chaos_ratio.len() <= i {
            return;
        }

        let current_close = tick.closes()[i];
        let current_atr = self.sensors.atr_sl[i];

        // --- 🧬 MAHORAGA ADAPTATION (Cập nhật bộ nhớ) ---
        self.memory.volatility_benchmark = current_atr; 

        // Theo dõi Peak Price cho Trailing Stop (FIX #16)
        let pos = tick.position();
        if pos.is_open() {
            let high = tick.highs()[i];
            let low = tick.lows()[i];

            let current_peak = self.memory.peak_price.unwrap_or(current_close);
            if pos.is_long {
                self.memory.peak_price = Some(current_peak.max(high));
            } else {
                self.memory.peak_price = Some(current_peak.min(low));
            }
        } else {
            self.memory.peak_price = None;
            self.memory.is_breakeven_active = false;
        }

        // Đếm ngược thời gian hồi phục (Boxing Recovery)
        if self.memory.recovery_period_counter > 0 {
            self.memory.recovery_period_counter -= 1;
        }


        // // 1. Quản lý lệnh Trailing (Chạy trước để chốt lời/cắt lỗ)
        // // Helios sử dụng cơ chế Trailing Thích nghi (Adaptive) dựa trên Thế trận.
        
        // // Cần tính toán Regime trước để Trailing có thể sử dụng (Hoặc dùng Regime của nến trước)
        // // Tuy nhiên, để tối ưu, ta tính toán các chỉ số cần thiết cho Trailing ngay tại đây:
        // let current_atr = self.sensors.atr_sl[i];
        
        // // 1a. Phân tích thế trận sơ bộ (Sử dụng Trend Angle Adjusted)
        // let (regime_for_trailing, _trailing_scores) = logic::regime::analyze_regime(
        //     &self.config,
        //     &mut self.memory,
        //     self.matrix.squeeze_intensity[i],
        //     self.matrix.trend_strength[i],
        //     self.matrix.trend_angle_adj[i], // FIX #7: Dùng góc hiệu chỉnh
        //     self.matrix.volume_force[i],
        //     self.matrix.bb_width_norm[i],
        //     self.matrix.price_z_score[i],
        //     self.matrix.chaos_ratio[i], // FIX #3, #23: Dùng tỷ lệ biến động
        // );

        // logic::risk::manage_trailing_stop(
        //     &self.config,
        //     &mut self.memory,
        //     tick,
        //     regime_for_trailing,
        //     self.matrix.trend_angle_adj[i], // FIX #7
        //     current_atr,
        // );
        // --- 🏛️ QUY TRÌNH HỘI ĐỒNG (COUNCIL WORKFLOW) ---

        // 1. Nhận thức môi trường (Level 1: Environment Fuse)
        let env = logic::sensor::analyze_environment(
            &self.config.environment_fuse,
            &mut self.memory,
            self.matrix.chaos[i],
            self.matrix.volume_force[i],
            self.matrix.entropy[i],
        );

        // 2. Phân tích thế trận & Tính điểm (Level 2: Market Regime Scoring)
        // [CHỈ GỌI 1 LẦN]: Để tránh sai lệch bars_since_regime_change
        let (primary_regime, scores) = logic::regime::analyze_regime(
            &self.config,
            &mut self.memory,
            self.matrix.squeeze_intensity[i],
            self.matrix.trend_strength[i],
            self.matrix.trend_angle_adj[i], // FIX #7
            self.matrix.volume_force[i],
            self.matrix.bb_width_norm[i],
            self.matrix.price_z_score[i],
            self.matrix.chaos_ratio[i], // FIX #3, #23
        );

        // 3. Quản lý rủi ro & Trailing (Level 3: Risk Management)
        // Luôn chạy để bảo vệ các lệnh đang mở, kể cả khi môi trường bắt đầu xấu đi.
        logic::risk::manage_trailing_stop(
            &self.config,
            &mut self.memory,
            tick,
            primary_regime,
            self.matrix.trend_angle_adj[i], // FIX #7
            current_atr,
        );
        
        // --- 🛑 DỪNG TẤN CÔNG NẾU MÔI TRƯỜNG HỖN LOẠN ---
        if env == crate::types::MarketEnvironment::Chaotic {
            return;
        }

        // 4. Lựa chọn chiến thuật (Level 3: Playbook Arbitrator)
        let trend_dir_enum = logic::regime::detect_trend_direction(self.matrix.trend_dir[i]);

        let playbook = logic::playbook::select_playbook(
            &self.config,
            env,
            &scores,
            trend_dir_enum,
            self.matrix.trend_angle_adj[i], // FIX #7
            self.matrix.momentum[i], 
        );

        // 5. Kích nổ Sniper (Level 4: Tactical Execution)
        if playbook != crate::types::TacticalPlaybook::Wait {
            logic::tactical::execute_tactical(
                &self.config,
                &mut self.memory,
                playbook,
                &scores,
                tick,
                current_close,
                self.sensors.bb_upper[i],
                self.sensors.bb_lower[i],
                self.sensors.kama_mid[i],
                self.matrix.momentum[i],
                self.sensors.stoch_k[i],
                self.sensors.stoch_cross[i],
                self.sensors.stoch_k_zone[i],
                self.sensors.macd_main[i],
                self.sensors.macd_hist[i],
                self.sensors.atr_sl[i],
                self.matrix.volume_force[i],
                self.matrix.price_z_score[i],
            );
        }
    }
}
