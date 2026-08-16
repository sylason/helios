// strategies_backtests/helios/src/strategy/mod.rs

pub mod config;
pub mod core_v2;
pub mod core_v3;
pub mod memory;
pub mod sensors;
pub mod matrix;

use std::collections::HashMap;

use ferro_trade::optimizer::params::ParamValue;
use ferro_trade::strategy::{InitContext, Strategy, TickContext};
use ferro_trade::indicators::factory::BulkIndicatorFactory;
use crate::strategy::config::HeliosConfig;
use crate::strategy::memory::TacticalMemory;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::matrix::MarketMatrix;

use ferro_trade::data::Timeframe;
use ferro_trade::optimizer::DnaMappable;

/// Chiến lược Helios - Nhạc trưởng điều phối 4 cấp độ chiến thuật.
/// [CLEAN ARCHITECTURE]: Phân tách Parameters (Config) -> Data (Sensors) -> Perception (Matrix).
#[derive(Clone, Debug)]
pub struct HeliosStrategy {
    pub config: HeliosConfig,
    pub memory: TacticalMemory,
    pub sensors: HeliosSensors,
    pub matrix: MarketMatrix,
    pub debug_mode: bool,
    pub cooldown_bars_remaining: usize,
    pub was_position_open: bool,
    pub equity_at_entry: f64,
}

impl HeliosStrategy {
    /// [DEBUG] In toàn bộ cấu hình để kiểm tra DNA nạp vào có đúng không.
    #[allow(dead_code)]
    fn print_full_config(&self) {
        println!("\n==================================================");
        println!("      [HELIOS STRATEGY CONFIG DIAGNOSTIC]");
        println!("==================================================");
        println!("{:#?}", self.config);
        println!("--------------------------------------------------\n");
    }

    /// [DEBUG] Hệ thống chẩn đoán tập trung cho giai đoạn Init.
    /// Gom tất cả các lệnh trace_v và in ấn vào đây để dễ dàng bật/tắt.
    #[allow(dead_code)]
    fn run_init_diagnostics(&mut self, ctx: &InitContext, raw_closes: &[f64], _raw_volumes: &[f64]) {
        let trace_v = |category: &str, name: &str, data: &[f64]| {
            let len = data.len();
            if len == 0 {
                println!("[INIT_TRACE] [{:<15}] {:<20} | EMPTY", category, name);
                return;
            }
            let last_start = len.saturating_sub(10);
            println!("[INIT_TRACE] [{:<15}] {:<20} | len:{:<6} | first10:{:?}\n             {:<38} | last10:{:?}", 
                category, name, len, &data[0..10.min(len)], "", &data[last_start..len]);
        };

        let ind = &self.config.indicators;
        println!("\n=== [HELIOS INIT DIAGNOSTIC: START] ===");
        println!("Config Params: NATR_P={}, MFI_P={}, VOL_MA_P={}, KAMA_S={}, KAMA_L={}, ADX_P={}, LR_P={}", 
            ind.natr.period, ind.mfi.period, ind.vol_ma.period, ind.kama.short_period, ind.kama.long_period, ind.adx.period, ind.linreg.period);

        // --- STEP 0: INITIAL DATA (POINT ZERO) ---
        trace_v("INITIAL", "Closes", &ctx.closes());
        trace_v("INITIAL", "Volumes", &ctx.volumes());

        // --- 1. RAW SENSORS ---
        let s = &self.sensors;
        trace_v("ENV_CHAOS", "NATR", &s.natr);
        trace_v("ENV_CHAOS", "ChaosRatio", &s.chaos_ratio);
        trace_v("ENV_CHAOS", "VOL_SMA", &s.vol_sma);
        trace_v("ENV_CHAOS", "ER", &s.efficiency_ratio);
        trace_v("ENV_CHAOS", "MFI", &s.mfi);
        trace_v("TREND_REGIME", "ADX", &s.adx);
        trace_v("TREND_REGIME", "KAMA_Slope", &s.kama_slope);
        trace_v("TREND_REGIME", "KAMA_Mid", &s.kama_mid);
        trace_v("VOLATILITY", "BB_Upper", &s.bb_upper);
        trace_v("VOLATILITY", "BB_Lower", &s.bb_lower);
        trace_v("VOLATILITY", "KC_Upper", &s.kc_upper);
        trace_v("VOLATILITY", "KC_Lower", &s.kc_lower);
        trace_v("MOMENTUM", "RSI_Trend", &s.rsi_trend);
        trace_v("SNIPER", "Stoch_Trend_K", &s.stoch_trend_k);
        trace_v("SNIPER", "Stoch_Trend_D", &s.stoch_trend_d);
        trace_v("RISK", "ATR_SL", &s.atr_sl);

        // --- 2. VERIFICATIONS ---
        trace_v("VERIFY_POLLUTION", "ctx.closes()", &ctx.closes());
        trace_v("VERIFY_POLLUTION", "ctx.volumes()", &ctx.volumes());
        trace_v("VERIFY_CLEAN", "raw_closes", raw_closes);

        // --- 3. MATRIX PERCEPTION ---
        println!("\n--- [MATRIX PERCEPTION RESULTS] ---");
        trace_v("MATRIX", "TrendDir", &self.matrix.trend_dir);
        trace_v("MATRIX", "TrendStrength", &self.matrix.trend_strength);
        trace_v("MATRIX", "TrendAngleAdj", &self.matrix.trend_angle_adj);
        trace_v("MATRIX", "SqueezeIntensity", &self.matrix.squeeze_intensity);
        trace_v("MATRIX", "Liquidity", &self.matrix.liquidity);
        trace_v("MATRIX", "VolumeForce", &self.matrix.volume_force);
        trace_v("MATRIX", "PriceZScore", &self.matrix.price_z_score);
        trace_v("MATRIX", "ChaosRatio", &self.matrix.chaos_ratio);

        // --- ⚖️ [FINAL COUNCIL VERDICT] ---
        let last_idx = self.sensors.closes.len().saturating_sub(1);
        if last_idx > 0 {
            println!("\n--- [HELIOS FINAL VERDICT (Bar {})] ---", last_idx);
            
            // 1. Kiểm tra Cầu chì
            let env = core_v2::sensor::analyze_environment(
                &self.config.environment_fuse,
                &mut self.memory,
                self.matrix.chaos[last_idx],
                self.matrix.volume_force[last_idx],
                self.matrix.entropy[last_idx],
            );
            println!("   ENVIRONMENT: StabilityIndex={:.4} (StateCounter: {})", env.stability_index, self.memory.state_duration_counter);
            println!("   Fuse Inputs -> Chaos:{:.4} (Thresh:<{:.4}) | VolForce:{:.4} (Thresh:>{:.4}) | Entropy:{:.4} (Thresh:>{:.4})",
                self.matrix.chaos[last_idx], self.config.environment_fuse.chaos_z_threshold,
                self.matrix.volume_force[last_idx], self.config.environment_fuse.min_liquidity_ratio,
                self.matrix.entropy[last_idx], self.config.environment_fuse.er_threshold
            );

            // 2. Kiểm tra Điểm số Thế trận
            let (_, scores) = core_v2::regime::analyze_regime(
                &self.config,
                &mut self.memory,
                self.matrix.squeeze_intensity[last_idx],
                self.matrix.trend_strength[last_idx],
                self.matrix.trend_angle_adj[last_idx],
                self.matrix.volume_force[last_idx],
                self.matrix.bb_width_norm[last_idx],
                self.matrix.price_z_score[last_idx],
                self.matrix.chaos_ratio[last_idx],
            );
            
            // 3. Quyết định Playbook
            let trend_dir_enum = core_v2::regime::detect_trend_direction(self.matrix.trend_dir[last_idx]);
            let playbook = core_v2::playbook::select_playbook(
                &self.config,
                env,
                &scores,
                trend_dir_enum,
                self.matrix.trend_angle_adj[last_idx],
                self.matrix.momentum[last_idx],
            );
            println!("   DECISION -> Playbook: {:?} | MinScoreThresh: {:.4}", playbook, self.config.market_regime.priority.min_score_threshold);
        }

        println!("=== [HELIOS INIT DIAGNOSTIC: END] ===\n");

        // --- 4. INDICATORS SAMPLES ---
        println!("--- [HELIOS INDICATORS SAMPLES] ---");
        self.print_indicator_samples("Closes", &self.sensors.closes, 0, 5);
        self.print_indicator_samples("NATR", &self.sensors.natr, 0, 10);
        self.print_indicator_samples("KAMA_Mid", &self.sensors.kama_mid, 0, 10);
        self.print_indicator_samples("ADX", &self.sensors.adx, 0, 10);
        self.print_indicator_samples("RSI_Trend", &self.sensors.rsi_trend, 0, 10);
        self.print_indicator_samples("ATR_SL", &self.sensors.atr_sl, 0, 10);
        println!("------------------------------------\n");
    }

    /// [DEBUG] In mẫu dữ liệu indicator: Tên, Độ dài, Vị trí bắt đầu, Số lượng, và Giá trị.
    #[allow(dead_code)]
    fn print_indicator_samples(&self, name: &str, data: &[f64], start: usize, count: usize) {
        let len = data.len();
        let end = (start + count).min(len);
        let sample = if start < len { &data[start..end] } else { &[] };
        println!("[DEBUG_IND] {:<15} | Len: {:<5} | Range [{}..{}]: {:?}", name, len, start, end, sample);
    }
}

impl Strategy for HeliosStrategy {
    fn new(_params: &HashMap<String, ParamValue>) -> Self {
        Self {
            config: HeliosConfig::default(),
            memory: TacticalMemory::default(),
            sensors: HeliosSensors::default(),
            matrix: MarketMatrix::default(),
            debug_mode: std::env::var("HELIOS_DEBUG").is_ok(),
            cooldown_bars_remaining: 0,
            was_position_open: false,
            equity_at_entry: 0.0,
        }
    }


    /// [NEW VERSION] Nạp cấu hình từ DNA thông qua Macro (Zero-Allocation, siêu tốc).
    fn update_from_dna(&mut self, dna: &[f64]) {
        // Gọi thẳng hàm được sinh ra từ #[derive(DnaMappableMacro)] trong config.rs
        self.config.update_from_dna(dna);
    }



    /// [ELITE] Khởi tạo hệ thống Giác quan (Sensors) & Nhận thức (Matrix).
    fn init(&mut self, ctx: &mut InitContext) {
        // [CAPTURE DATA SẠCH]: Lấy con trỏ (Arc) dữ liệu 1m ngay lập tức, KHÔNG CLONE.
        let raw_closes = ctx.closes();
        let raw_volumes = ctx.volumes();

        let ind = &self.config.indicators;

        // --- STEP 0: INITIAL DATA (POINT ZERO) ---
        // [SAFETY] Ngăn chặn panic nếu bất kỳ period nào bị bind lỗi về 0
        if ind.natr.period == 0 || ind.mfi.period == 0 || ind.vol_ma.period == 0 || 
           ind.kama.short_period == 0 || ind.kama.long_period == 0 ||
           ind.adx.period == 0 || ind.bbands.period == 0 || ind.atr.period == 0 {
            println!("!!! [SAFETY] CRITICAL PERIOD IS ZERO, ABORTING INIT !!!");
            return;
        }

        // --- 1. RAW SENSORS (HỨNG DỮ LIỆU THÔ) ---
        let mut s = HeliosSensors::default();
        
        s.natr = ctx.natr(ind.natr.timeframe, "NATR", ind.natr.period);
        s.chaos_ratio = ctx.chaos_ratio(ind.natr.timeframe, "ChaosRatio", ind.natr.period);
        s.vol_sma = ctx.volume_sma(ind.vol_ma.timeframe, "VOL_SMA", ind.vol_ma.period);
        s.efficiency_ratio = ctx.efficiency_ratio(ind.kama.timeframe, "ER", ind.kama.short_period);

        let (mfi, _tp, _mf, _pmf, _nmf) = ctx.mfi(ind.mfi.timeframe, "MFI", ind.mfi.period);
        s.mfi = mfi;

        let (adx, _pdi, _mdi, _adxr) = ctx.adx(ind.adx.timeframe, "ADX", ind.adx.period);
        s.adx = adx;

        s.kama_mid = ctx.kama(ind.kama.timeframe, "KAMA", ind.kama.short_period, 2, ind.kama.long_period);
        let (k_slope, k_slope_adj) = ctx.kama_slope(
            ind.kama.timeframe, "KAMA_Slope", 
            ind.kama.short_period, 2, ind.kama.long_period, 
            ind.kama.short_period, ind.atr.period
        );
        s.kama_slope = k_slope;
        s.kama_slope_adj = k_slope_adj;

        let (bb_u, bb_m, bb_l, _bb_w, _bb_wp, _bb_sw, _bb_sq) = ctx.bbands(ind.bbands.timeframe, "BB", ind.bbands.period, ind.bbands.nbdev);
        s.bb_upper = bb_u;
        s.bb_middle = bb_m;
        s.bb_lower = bb_l;

        let (kc_u, _kc_m, kc_l) = ctx.keltner(ind.keltner.timeframe, "KC", ind.keltner.period as u32, ind.keltner.multiplier, ind.keltner.ma_type.clone());
        s.kc_upper = kc_u;
        s.kc_lower = kc_l;

        let (rsi_t, _, _) = ctx.rsi(ind.rsi.timeframe, "RSI_Trend", ind.rsi.period);
        s.rsi_trend = rsi_t;

        let (st_t_k, st_t_d, _, _, _) = ctx.stoch(
            ind.stochastic.timeframe, "STOCH_Trend", 
            ind.stochastic.period as u32, 
            ind.stochastic.k_period as u32, ind.stochastic.k_ma_type.clone(),
            ind.stochastic.d_period as u32, ind.stochastic.d_ma_type.clone(),
            ind.stochastic.zone
        );
        s.stoch_trend_k = st_t_k;
        s.stoch_trend_d = st_t_d;

        s.atr_sl = ctx.atr(ind.atr.timeframe, "ATR_SL", ind.atr.period);

        // --- 2. MARKET MATRIX (CHUYỂN HÓA NHẬN THỨC) ---
        let vol_ratio = match ind.vol_ma.timeframe {
            Timeframe::T5m => 5.0, Timeframe::T15m => 15.0, Timeframe::T1h => 60.0, Timeframe::T4h => 240.0, _ => 1.0,
        };

        // --- 3. GÁN SENSORS ---
        self.sensors = s;
        self.sensors.closes = raw_closes.clone(); // Chỉ clone Arc (Pointer 8 bytes), Zero-Allocation!

        // --- 4. TÍNH TOÁN MARKET MATRIX ---
        self.matrix = MarketMatrix::from_sensors(&self.config, &self.sensors, &raw_volumes, vol_ratio);

        // --- 5. CHẨN ĐOÁN ---
        // self.run_init_diagnostics(ctx, &raw_closes, &raw_volumes);
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
        let prev_atr = self.memory.volatility_benchmark;
        if prev_atr == 0.0 {
            self.memory.volatility_benchmark = current_atr;
        } else {
            self.memory.volatility_benchmark = prev_atr * 0.9 + current_atr * 0.1;
        }

        // Theo dõi Peak Price cho Trailing Stop (FIX #16)
        let pos = tick.position();
        if pos.is_open() {
            let current_peak = self.memory.peak_price.unwrap_or(current_close);
            if pos.is_long {
                self.memory.peak_price = Some(current_peak.max(current_close));
            } else {
                self.memory.peak_price = Some(current_peak.min(current_close));
            }
            self.memory.holding_bars_counter += 1;
        } else {
            self.memory.peak_price = None;
            self.memory.is_breakeven_active = false;
            self.memory.holding_bars_counter = 0;
        }

        // Đếm ngược thời gian hồi phục (Boxing Recovery)
        if self.memory.recovery_period_counter > 0 {
            self.memory.recovery_period_counter -= 1;
        }


        // 1. Quản lý lệnh Trailing sẽ được thực hiện SAU khi phân tích Regime (ở bước 3 bên dưới).
        // Lý do: analyze_regime cập nhật bộ nhớ (bars_since_regime_change), gọi 2 lần sẽ sai lệch.

        // --- 🏛️ QUY TRÌNH HỘI ĐỒNG (COUNCIL WORKFLOW) ---

        // 1. Nhận thức môi trường (Level 1: Environment Fuse)
        let env = core_v2::sensor::analyze_environment(
            &self.config.environment_fuse,
            &mut self.memory,
            self.matrix.chaos[i],
            self.matrix.volume_force[i],
            self.matrix.entropy[i],
        );

        // 2. Phân tích thế trận & Tính điểm (Level 2: Market Regime Scoring)
        // [CHỈ GỌI 1 LẦN]: Để tránh sai lệch bars_since_regime_change
        let (primary_regime, scores) = core_v2::regime::analyze_regime(
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
        core_v2::risk::manage_trailing_stop(
            &self.config,
            &mut self.memory,
            tick,
            primary_regime,
            current_atr,
        );
        
        // 4. Lựa chọn chiến thuật (Level 3: Playbook Arbitrator)
        // [FIXED Task 2.2]: Loại bỏ 'Hard Stop'. Tầng Playbook sẽ tự 'Wait' nếu 
        // Stability Index thấp làm Final Score không đủ ngưỡng.
        let trend_dir_enum = core_v2::regime::detect_trend_direction(self.matrix.trend_dir[i]);

        let playbook = core_v2::playbook::select_playbook(
            &self.config,
            env,
            &scores,
            trend_dir_enum,
            self.matrix.trend_angle_adj[i], // FIX #7
            self.matrix.momentum[i], 
        );

        // 5. Kích nổ Sniper (Level 4: Tactical Execution)
        if playbook != crate::types::TacticalPlaybook::Wait {
            core_v2::tactical::execute_tactical(
                &self.config,
                &mut self.memory,
                playbook,
                &scores,
                tick,
                &self.sensors,
                &self.matrix,
                env, // [NEW Task 3.2]: Truyền môi trường để log thẻ điểm
            );
        }
    }
}

impl Default for HeliosStrategy {
    fn default() -> Self {
        use std::collections::HashMap;
        Self::new(&HashMap::new())
    }
}
