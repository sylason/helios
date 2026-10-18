// strategies_backtests/helios/src/strategy/core_v3/gatekeeper.rs

//! # 🛡️ Regime Gatekeeper (Cầu Dao Điều Hướng Chế Độ Thị Trường)
//!
//! ## Bản chất Khoa học Kinh tế lượng & Vi cấu trúc Sổ lệnh:
//! Trong thị trường tài chính, không phải lúc nào thị trường cũng có thể giao dịch sinh lời ($EV > 0$).
//! Có 2 trạng thái thị trường mang tính "tử địa" mà mọi mô hình kỹ thuật đều bị đánh bại:
//! 1. **Bão Biến Động Hỗn Loạn (Chaos Whipsaw):** Biến động bùng nổ không định hướng, râu nến giật 2 chiều
//!    quét sạch Stop Loss trước khi kịp hình thành xu hướng.
//! 2. **Sàn Biến Động Chết / Tê Liệt Thanh Khoản (Volatility Flatline):** Độ biến động thực tế NATR
//!    thấp hơn chi phí sàn và trượt giá ($NATR < 0.12\%$), khiến kỳ vọng toán học $EV < 0$ do "chết vì phí".
//!
//! ## Nguyên tắc Thiết kế (SOLID & Clean Architecture):
//! - **Single Responsibility Principle:** `RegimeGatekeeper` độc quyền quyết định thị trường CÓ ĐƯỢC PHÉP
//!   giao dịch hay không. Không làm thay việc của Scanners hay Risk Manager.
//! - **Stateless Lookback Window (K = 5 bars):** Không lưu trạng thái biến thiên trong RAM bộ nhớ đệm,
//!   đảm bảo thread-safe 100% với Rayon GA Optimizer và miễn nhiễm lỗi mất trạng thái khi restart bot.
//! - **Hysteresis Schmitt Trigger:** Chống hiện tượng rung lắc bật/tắt liên tục quanh ngưỡng kích hoạt.

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::config::HeliosConfigV3;

/// Lý do Cầu Dao Gatekeeper phủ quyết không cho phép mở vị thế
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVetoReason {
    /// Bão biến động hỗn loạn (Chaos Index >= threshold hoặc đang trong dải trễ Hysteresis)
    ChaosWhipsaw,
    /// Sàn biến động vĩ mô không bù nổi chi phí vị thế (Macro NATR < Roundtrip Fee + Min Alpha)
    MacroFlatline,
    /// Sàn thanh khoản vi mô chết / que tăm (Micro NATR < Single-leg Taker Fee)
    MicroFlatline,
    /// Bất thường dữ liệu (NaN, Inf, hoặc chưa đủ warmup)
    DataAnomaly,
}

impl GateVetoReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateVetoReason::ChaosWhipsaw => "CHAOS_WHIPSAW",
            GateVetoReason::MacroFlatline => "MACRO_FLATLINE",
            GateVetoReason::MicroFlatline => "MICRO_FLATLINE",
            GateVetoReason::DataAnomaly => "DATA_ANOMALY",
        }
    }
}

/// [MACRO MARKET REGIME]
/// 6 trạng thái vật lý khép kín của thị trường theo chuẩn phân tích định lượng trực giao.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarketRegime {
    #[default]
    VolatilityFlatline, // 🛑 Tử địa thanh khoản (NATR vi mô/vĩ mô < Sàn chi phí)
    TurbulentChaos,     // 🌪️ Bão biến động hỗn loạn (Chaos Index >= 0.70 hoặc dính Hysteresis)
    DataAnomaly,        // ⚠️ Dữ liệu bất thường hoặc chứa NaN/Inf
    Trending,           // 📈 Xu hướng bền vững có trớn (KER >= 0.35 hoặc ADX >= 25, không nén)
    RangeBound,         // 📦 Vùng dao động đi ngang / nén tích lũy (Squeeze >= 0.65 hoặc KER thấp & ADX < 20)
    Breakout,           // 🚀 Bứt phá nến đầu từ dải nén (Squeeze <= 0.35 và ADX >= 20, nhưng chưa đủ trớn Trend)
}

impl MarketRegime {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketRegime::VolatilityFlatline => "VOLATILITY_FLATLINE",
            MarketRegime::TurbulentChaos => "TURBULENT_CHAOS",
            MarketRegime::DataAnomaly => "DATA_ANOMALY",
            MarketRegime::Trending => "TRENDING",
            MarketRegime::RangeBound => "RANGE_BOUND",
            MarketRegime::Breakout => "BREAKOUT",
        }
    }
}

/// Kết quả thẩm định trạng thái thị trường của Gatekeeper
#[derive(Debug, Clone, Default)]
pub struct GateEvaluation {
    /// 🏛️ Chế độ thị trường vĩ mô được phân loại khách quan
    pub regime: MarketRegime,
    /// Cho phép Arbiter tiếp tục quét tín hiệu hay không
    pub passed: bool,
    /// Lý do phủ quyết (nếu passed == false)
    pub veto_reason: Option<GateVetoReason>,
    /// Giá trị Chaos Index hiện tại
    pub current_chaos: f64,
    /// Giá trị NATR đo đạc thực tế của thị trường (%) từ sensors.natr MTF
    pub market_natr_pct: f64,
    /// 🔬 Giá trị NATR vi mô (%) từ sensors.micro_natr
    pub micro_natr_pct: f64,
    /// Trạng thái khóa trễ Hysteresis có đang hoạt động hay không
    pub is_hysteresis_active: bool,
    /// 📉 Ngưỡng NATR vĩ mô tối thiểu yêu cầu (%) = fee_floor_pct + min_alpha_pct
    pub required_min_natr_pct: f64,
    /// 💰 Sàn chi phí 2 chiều thực tế (%) = 2*taker_fee + slippage
    pub fee_floor_pct: f64,
    /// 🛡️ Sàn chi phí 1 chiều vi mô (%) = taker_fee_rate * 100.0
    pub micro_fee_floor_pct: f64,
    /// 🧬 Ngưỡng Alpha bù đắp từ DNA (%)
    pub min_alpha_pct: f64,
    /// 🌪️ Ngưỡng cắt bão Chaos
    pub chaos_threshold: f64,
}

/// Bộ gác cổng chế độ thị trường vi mô
pub struct RegimeGatekeeper;

impl RegimeGatekeeper {
    /// Đánh giá trạng thái vi mô thị trường trước khi cho phép Arbiter quét tín hiệu.
    /// 
    /// # Tham số:
    /// - `tick`: Ngữ cảnh tick nến hiện tại
    /// - `sensors`: Cảm biến thô từ Factory
    /// - `matrix`: Ma trận dữ liệu chuẩn hóa
    /// - `config`: Cấu hình siêu tham số DNA
    /// 
    /// # Trả về:
    /// `GateEvaluation`: Nếu `passed == false`, Arbiter sẽ thoát sớm $O(1)$ mà không cần chạy bất kỳ Scanner nào.
    pub fn evaluate(
        tick: &TickContext,
        sensors: &HeliosSensors,
        matrix: &MarketMatrixV3,
        config: &HeliosConfigV3,
    ) -> GateEvaluation {
        let i = tick.i;

        // 1. Kiểm tra giới hạn mảng & Dữ liệu bất thường (Data Anomaly Guard)
        if matrix.chaos_index.len() <= i
            || sensors.natr.len() <= i
            || sensors.micro_natr.len() <= i
            || sensors.atr_sl.len() <= i
        {
            return GateEvaluation {
                regime: MarketRegime::DataAnomaly,
                passed: false,
                veto_reason: Some(GateVetoReason::DataAnomaly),
                current_chaos: 0.0,
                market_natr_pct: 0.0,
                micro_natr_pct: 0.0,
                is_hysteresis_active: false,
                required_min_natr_pct: 0.0,
                fee_floor_pct: 0.0,
                micro_fee_floor_pct: 0.0,
                min_alpha_pct: 0.0,
                chaos_threshold: 0.0,
            };
        }

        let current_chaos = matrix.chaos_index.get(i).copied().unwrap_or(f64::NAN);
        let market_natr_pct = sensors.natr.get(i).copied().unwrap_or(f64::NAN);
        let micro_natr_pct = sensors.micro_natr.get(i).copied().unwrap_or(f64::NAN);
        let current_atr_sl = sensors.atr_sl.get(i).copied().unwrap_or(f64::NAN);
        let current_price = tick.closes().get(i).copied().unwrap_or(0.0);

        let risk_base = &config.risk_management.risk_base;
        let roundtrip_fee_pct = (risk_base.taker_fee_rate * 2.0 + risk_base.slippage_allowance_pct) * 100.0;
        let micro_fee_floor_pct = (risk_base.taker_fee_rate * 100.0).max(1e-4);
        let min_alpha_pct = config.arbiter.min_natr_threshold * 100.0;
        let required_min_natr_pct = roundtrip_fee_pct + min_alpha_pct;
        let t_enter = config.arbiter.chaos_veto_threshold;

        if current_chaos.is_nan()
            || market_natr_pct.is_nan()
            || micro_natr_pct.is_nan()
            || current_atr_sl.is_nan()
            || current_price <= 1e-9
            || current_atr_sl <= 0.0
        {
            return GateEvaluation {
                regime: MarketRegime::DataAnomaly,
                passed: false,
                veto_reason: Some(GateVetoReason::DataAnomaly),
                current_chaos: 0.0,
                market_natr_pct: 0.0,
                micro_natr_pct: 0.0,
                is_hysteresis_active: false,
                required_min_natr_pct: 0.0,
                fee_floor_pct: 0.0,
                micro_fee_floor_pct: 0.0,
                min_alpha_pct: 0.0,
                chaos_threshold: 0.0,
            };
        }

        // 2.1. Chốt chặn Sàn Kinh Tế Vĩ Mô (Macro Economic Floor)
        // sensors.natr lưu giá trị phần trăm (%) từ khung MTF chiến thuật (vd: 0.45 tức 0.45%)
        if market_natr_pct < required_min_natr_pct {
            return GateEvaluation {
                regime: MarketRegime::VolatilityFlatline,
                passed: false,
                veto_reason: Some(GateVetoReason::MacroFlatline),
                current_chaos,
                market_natr_pct,
                micro_natr_pct,
                is_hysteresis_active: false,
                required_min_natr_pct,
                fee_floor_pct: roundtrip_fee_pct,
                micro_fee_floor_pct,
                min_alpha_pct,
                chaos_threshold: t_enter,
            };
        }

        // 2.2. Chốt chặn Sàn Thanh Khoản Khớp Lệnh Vi Mô (Micro Execution Liquidity Floor)
        // Nến vi mô không được phép là nến que tăm có biên độ nhỏ hơn phí vào lệnh Taker (0.04%)
        if micro_natr_pct < micro_fee_floor_pct {
            return GateEvaluation {
                regime: MarketRegime::VolatilityFlatline,
                passed: false,
                veto_reason: Some(GateVetoReason::MicroFlatline),
                current_chaos,
                market_natr_pct,
                micro_natr_pct,
                is_hysteresis_active: false,
                required_min_natr_pct,
                fee_floor_pct: roundtrip_fee_pct,
                micro_fee_floor_pct,
                min_alpha_pct,
                chaos_threshold: t_enter,
            };
        }

        // 3. Chốt chặn Bão Chaos Whipsaw (Stateless Hysteresis Lookback K = 5 bars)
        let delta = config.arbiter.chaos_hysteresis_delta;
        let t_exit = (t_enter - delta).max(0.0);

        // Trường hợp 3.1: Nến hiện tại vượt ngưỡng kích hoạt bão
        if current_chaos >= t_enter {
            return GateEvaluation {
                regime: MarketRegime::TurbulentChaos,
                passed: false,
                veto_reason: Some(GateVetoReason::ChaosWhipsaw),
                current_chaos,
                market_natr_pct,
                micro_natr_pct,
                is_hysteresis_active: true,
                required_min_natr_pct,
                fee_floor_pct: roundtrip_fee_pct,
                micro_fee_floor_pct,
                min_alpha_pct,
                chaos_threshold: t_enter,
            };
        }

        // Trường hợp 3.2: Nến hiện tại nằm trong dải trễ [t_exit, t_enter]
        if current_chaos >= t_exit {
            let lookback_bars = 5usize;
            let start_idx = i.saturating_sub(lookback_bars.saturating_sub(1));
            let mut storm_locked = false;

            for j in (start_idx..i).rev() {
                let val = matrix.chaos_index.get(j).copied().unwrap_or(0.0);
                if val >= t_enter {
                    storm_locked = true;
                    break;
                } else if val < t_exit {
                    break;
                }
            }

            if storm_locked {
                return GateEvaluation {
                    regime: MarketRegime::TurbulentChaos,
                    passed: false,
                    veto_reason: Some(GateVetoReason::ChaosWhipsaw),
                    current_chaos,
                    market_natr_pct,
                    micro_natr_pct,
                    is_hysteresis_active: true,
                    required_min_natr_pct,
                    fee_floor_pct: roundtrip_fee_pct,
                    micro_fee_floor_pct,
                    min_alpha_pct,
                    chaos_threshold: t_enter,
                };
            }
        }

        // Vượt qua toàn bộ các chốt chặn -> Phân loại chế độ thị trường giao dịch
        let regime = Self::classify_regime(tick, sensors, matrix, config, None);
        if regime == MarketRegime::DataAnomaly {
            return GateEvaluation {
                regime: MarketRegime::DataAnomaly,
                passed: false,
                veto_reason: Some(GateVetoReason::DataAnomaly),
                current_chaos,
                market_natr_pct,
                micro_natr_pct,
                is_hysteresis_active: false,
                required_min_natr_pct,
                fee_floor_pct: roundtrip_fee_pct,
                micro_fee_floor_pct,
                min_alpha_pct,
                chaos_threshold: t_enter,
            };
        }

        GateEvaluation {
            regime,
            passed: true,
            veto_reason: None,
            current_chaos,
            market_natr_pct,
            micro_natr_pct,
            is_hysteresis_active: false,
            required_min_natr_pct,
            fee_floor_pct: roundtrip_fee_pct,
            micro_fee_floor_pct,
            min_alpha_pct,
            chaos_threshold: t_enter,
        }
    }

    /// Phân loại chế độ thị trường vĩ mô khách quan dựa trên KER, ADX và Squeeze Intensity.
    pub fn classify_regime(
        tick: &TickContext,
        sensors: &HeliosSensors,
        matrix: &MarketMatrixV3,
        config: &HeliosConfigV3,
        veto_reason: Option<GateVetoReason>,
    ) -> MarketRegime {
        if let Some(reason) = veto_reason {
            return match reason {
                GateVetoReason::MacroFlatline | GateVetoReason::MicroFlatline => MarketRegime::VolatilityFlatline,
                GateVetoReason::ChaosWhipsaw => MarketRegime::TurbulentChaos,
                GateVetoReason::DataAnomaly => MarketRegime::DataAnomaly,
            };
        }

        let i = tick.i;
        let opt_er = sensors.efficiency_ratio.get(i).copied();
        let opt_adx = sensors.adx.get(i).copied();
        let opt_sq = matrix.squeeze_intensity.get(i).copied();

        // 🛑 PHÁT HIỆN DỊ TẬT SỐ HỌC: Nếu có mảng cảm biến mà giá trị là NaN -> DataAnomaly
        if opt_er.map(|v| v.is_nan()).unwrap_or(false)
            || opt_adx.map(|v| v.is_nan()).unwrap_or(false)
            || opt_sq.map(|v| v.is_nan()).unwrap_or(false)
        {
            return MarketRegime::DataAnomaly;
        }

        let er = opt_er.unwrap_or(0.0);
        let adx = opt_adx.unwrap_or(0.0);
        let squeeze = opt_sq.unwrap_or(0.5);

        let cfg = &config.macro_regime;
        let is_mature_trend = (er >= cfg.min_trending_er || adx >= cfg.min_trending_adx)
            && squeeze < cfg.range_squeeze_threshold;

        if is_mature_trend {
            MarketRegime::Trending
        } else if squeeze <= cfg.breakout_squeeze_threshold && adx >= cfg.min_breakout_adx {
            MarketRegime::Breakout
        } else {
            MarketRegime::RangeBound
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::collections::HashMap;
    use ferro_trade::data::{DataView, RawDataCache};
    use ferro_trade::backtest::IndicatorManager;
    use ferro_trade::broker::position::Position;
    use ferro_trade::broker::actions::Action;
    use ferro_trade::broker::trade::Trade;
    use smallvec::SmallVec;

    struct MockTickEnv {
        data_view: DataView,
        indicator_manager: IndicatorManager,
        actions: SmallVec<[Action; 4]>,
        trades: Vec<Trade>,
    }

    impl MockTickEnv {
        fn new(len: usize) -> Self {
            let raw_cache = RawDataCache {
                datetime: Arc::new(vec![0i64; len]),
                open: Arc::new(vec![2000.0; len]),
                high: Arc::new(vec![2010.0; len]),
                low: Arc::new(vec![1990.0; len]),
                close: Arc::new(vec![2000.0; len]),
                volume: Arc::new(vec![1000.0; len]),
                custom_columns: HashMap::new(),
            };
            Self {
                data_view: DataView::new(raw_cache),
                indicator_manager: IndicatorManager::new(),
                actions: SmallVec::new(),
                trades: Vec::new(),
            }
        }

        fn tick(&mut self, i: usize) -> TickContext<'_> {
            TickContext {
                i,
                equity: 1000.0,
                data_view: &self.data_view,
                indicator_manager: &self.indicator_manager,
                trades: &self.trades,
                last_closed_trade: None,
                actions: &mut self.actions,
                position: Position::default(),
            }
        }
    }

    #[test]
    fn test_gatekeeper_normal_market_passes() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.45; 20]); // Macro NATR 0.45% > (0.10% fee + 0.12% alpha = 0.22%)
        sensors.micro_natr = Arc::new(vec![0.08; 20]); // Micro NATR 0.08% > 0.04%
        sensors.atr_sl = Arc::new(vec![10.0; 20]);
        sensors.efficiency_ratio = Arc::new(vec![0.50; 20]);
        sensors.adx = Arc::new(vec![30.0; 20]);
        matrix.squeeze_intensity = Arc::new(vec![0.20; 20]);
        matrix.chaos_index = Arc::new(vec![0.30; 20]); // 0.30 < 0.65 threshold

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(eval.passed);
        assert_eq!(eval.veto_reason, None);
        assert_eq!(eval.regime, MarketRegime::Trending);
        assert!(!eval.is_hysteresis_active);
        assert!((eval.market_natr_pct - 0.45).abs() < 1e-4, "Market NATR phải là 0.45%");
        assert!((eval.micro_natr_pct - 0.08).abs() < 1e-4, "Micro NATR phải là 0.08%");
    }

    #[test]
    fn test_gatekeeper_macro_flatline_veto() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.10; 20]); // Macro NATR 0.10% < 0.22% (Sàn chi phí 0.10% + DNA Alpha 0.12%)
        sensors.micro_natr = Arc::new(vec![0.08; 20]);
        sensors.atr_sl = Arc::new(vec![2.0; 20]);
        matrix.chaos_index = Arc::new(vec![0.30; 20]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::MacroFlatline));
        assert_eq!(eval.regime, MarketRegime::VolatilityFlatline);
        assert!((eval.market_natr_pct - 0.10).abs() < 1e-4);
    }

    #[test]
    fn test_gatekeeper_micro_flatline_veto() {
        // 🛡️ CHỐNG BẪY MÙ 4H (NEW-NATR-4H-BLIND):
        // Dù Macro NATR 4h rất cao (1.50%), nhưng nếu nến vi mô phẳng lì que tăm (0.02% < 0.04%) thì PHẢI VETO!
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![1.50; 20]); // Macro NATR 4h 1.50% rất cao
        sensors.micro_natr = Arc::new(vec![0.02; 20]); // Micro NATR que tăm 0.02% < 0.04%
        sensors.atr_sl = Arc::new(vec![0.4; 20]);
        matrix.chaos_index = Arc::new(vec![0.30; 20]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed, "Nến vi mô que tăm không đủ bù phí Taker phải bị VETO!");
        assert_eq!(eval.veto_reason, Some(GateVetoReason::MicroFlatline));
        assert_eq!(eval.regime, MarketRegime::VolatilityFlatline);
        assert!((eval.micro_natr_pct - 0.02).abs() < 1e-4);
    }

    #[test]
    fn test_gatekeeper_immune_to_micro_candle_range() {
        // 🛡️ CHỐNG BẪY LỆCH NGỮ NGHĨA: Nến vi mô tích lũy co hẹp (0.05% >= 0.04% sàn Taker)
        // và phiên thị trường NATR MTF lành mạnh (0.45%) thì KHÔNG ĐƯỢC VETO!
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.45; 20]); // Macro NATR 0.45% > 0.22%
        sensors.micro_natr = Arc::new(vec![0.05; 20]); // Micro NATR 0.05% >= 0.04%
        sensors.atr_sl = Arc::new(vec![1.0; 20]);
        sensors.efficiency_ratio = Arc::new(vec![0.50; 20]);
        sensors.adx = Arc::new(vec![30.0; 20]);
        matrix.squeeze_intensity = Arc::new(vec![0.20; 20]);
        matrix.chaos_index = Arc::new(vec![0.30; 20]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(eval.passed, "Nến vi mô nén tích lũy lành mạnh thì phải PASS!");
        assert_eq!(eval.veto_reason, None);
        assert_eq!(eval.regime, MarketRegime::Trending);
        assert!((eval.market_natr_pct - 0.45).abs() < 1e-4);
    }

    #[test]
    fn test_gatekeeper_micro_data_anomaly_zero_atr() {
        // Nếu nến vi mô chết thanh khoản hoàn toàn (ATR_SL <= 0.0), phải trả về DataAnomaly
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.45; 10]);
        sensors.micro_natr = Arc::new(vec![0.08; 10]);
        sensors.atr_sl = Arc::new(vec![0.0; 10]); // ATR = 0.0 lỗi dữ liệu
        matrix.chaos_index = Arc::new(vec![0.30; 10]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::DataAnomaly));
    }

    #[test]
    fn test_gatekeeper_chaos_immediate_veto() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.25; 20]);
        sensors.micro_natr = Arc::new(vec![0.08; 20]);
        sensors.atr_sl = Arc::new(vec![10.0; 20]);
        matrix.chaos_index = Arc::new(vec![0.75; 20]); // 0.75 >= 0.65 threshold

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::ChaosWhipsaw));
        assert!(eval.is_hysteresis_active);
    }

    #[test]
    fn test_gatekeeper_chaos_hysteresis_lock() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.25; 20]);
        sensors.micro_natr = Arc::new(vec![0.08; 20]);
        sensors.atr_sl = Arc::new(vec![10.0; 20]);
        // Bar 8: 0.70 (kích hoạt bão >= 0.65)
        // Bar 9: 0.62 (chưa hạ dưới t_exit = 0.55)
        // Bar 10: 0.60 (vẫn >= 0.55)
        let mut chaos_vec = vec![0.30; 20];
        chaos_vec[8] = 0.70;
        chaos_vec[9] = 0.62;
        chaos_vec[10] = 0.60;
        matrix.chaos_index = Arc::new(chaos_vec);

        let tick = env.tick(10);
        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::ChaosWhipsaw));
        assert!(eval.is_hysteresis_active);
    }

    #[test]
    fn test_gatekeeper_chaos_hysteresis_release() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.25; 20]);
        sensors.micro_natr = Arc::new(vec![0.08; 20]);
        sensors.atr_sl = Arc::new(vec![10.0; 20]);
        sensors.efficiency_ratio = Arc::new(vec![0.50; 20]);
        sensors.adx = Arc::new(vec![30.0; 20]);
        matrix.squeeze_intensity = Arc::new(vec![0.20; 20]);
        // Bar 7: 0.70 (kích hoạt bão >= 0.65)
        // Bar 8: 0.50 (hạ nhiệt dưới t_exit = 0.55 -> giải phóng)
        // Bar 10: 0.60 (vẫn < 0.65 và bão trước đó đã hạ nhiệt)
        let mut chaos_vec = vec![0.30; 20];
        chaos_vec[7] = 0.70;
        chaos_vec[8] = 0.50;
        chaos_vec[10] = 0.60;
        matrix.chaos_index = Arc::new(chaos_vec);

        let tick = env.tick(10);
        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(eval.passed);
        assert_eq!(eval.veto_reason, None);
        assert_eq!(eval.regime, MarketRegime::Trending);
        assert!(!eval.is_hysteresis_active);
    }

    #[test]
    fn test_gatekeeper_data_anomaly_nan() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![f64::NAN; 10]);
        sensors.micro_natr = Arc::new(vec![0.08; 10]);
        sensors.atr_sl = Arc::new(vec![10.0; 10]);
        matrix.chaos_index = Arc::new(vec![0.30; 10]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::DataAnomaly));
        assert_eq!(eval.regime, MarketRegime::DataAnomaly);

        // Test Micro NATR chứa NaN
        sensors.natr = Arc::new(vec![0.25; 10]);
        sensors.micro_natr = Arc::new(vec![f64::NAN; 10]);
        sensors.atr_sl = Arc::new(vec![10.0; 10]);
        let eval_micro_nan = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval_micro_nan.passed);
        assert_eq!(eval_micro_nan.veto_reason, Some(GateVetoReason::DataAnomaly));
        assert_eq!(eval_micro_nan.regime, MarketRegime::DataAnomaly);

        // Test ATR_SL chứa NaN
        sensors.micro_natr = Arc::new(vec![0.08; 10]);
        sensors.atr_sl = Arc::new(vec![f64::NAN; 10]);
        let eval2 = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval2.passed);
        assert_eq!(eval2.veto_reason, Some(GateVetoReason::DataAnomaly));
        assert_eq!(eval2.regime, MarketRegime::DataAnomaly);
    }

    #[test]
    fn test_regime_classification_trending() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.efficiency_ratio = Arc::new(vec![0.45; 10]); // ER 0.45 >= 0.35
        sensors.adx = Arc::new(vec![28.0; 10]);              // ADX 28 >= 25
        matrix.squeeze_intensity = Arc::new(vec![0.25; 10]); // Squeeze 0.25 < 0.65

        let regime = RegimeGatekeeper::classify_regime(&tick, &sensors, &matrix, &config, None);
        assert_eq!(regime, MarketRegime::Trending);
    }

    #[test]
    fn test_regime_classification_breakout() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.efficiency_ratio = Arc::new(vec![0.20; 10]); // ER thấp 0.20 < 0.35 (chưa thành trend)
        sensors.adx = Arc::new(vec![22.0; 10]);              // ADX 22 >= 20 (xung lực nổ)
        matrix.squeeze_intensity = Arc::new(vec![0.30; 10]); // Squeeze nổ dải <= 0.35

        let regime = RegimeGatekeeper::classify_regime(&tick, &sensors, &matrix, &config, None);
        assert_eq!(regime, MarketRegime::Breakout);
    }

    #[test]
    fn test_regime_classification_range_bound() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        // Kịch bản 1: Squeeze nén chặt
        sensors.efficiency_ratio = Arc::new(vec![0.20; 10]);
        sensors.adx = Arc::new(vec![15.0; 10]);
        matrix.squeeze_intensity = Arc::new(vec![0.75; 10]); // Squeeze >= 0.65
        let regime1 = RegimeGatekeeper::classify_regime(&tick, &sensors, &matrix, &config, None);
        assert_eq!(regime1, MarketRegime::RangeBound);

        // Kịch bản 2: ADX yếu và ER thấp
        matrix.squeeze_intensity = Arc::new(vec![0.50; 10]);
        sensors.efficiency_ratio = Arc::new(vec![0.15; 10]);
        sensors.adx = Arc::new(vec![12.0; 10]);
        let regime2 = RegimeGatekeeper::classify_regime(&tick, &sensors, &matrix, &config, None);
        assert_eq!(regime2, MarketRegime::RangeBound);
    }

    #[test]
    fn test_regime_data_anomaly_on_nan_inputs() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.efficiency_ratio = Arc::new(vec![f64::NAN; 10]);
        sensors.adx = Arc::new(vec![25.0; 10]);
        matrix.squeeze_intensity = Arc::new(vec![0.50; 10]);
        let regime = RegimeGatekeeper::classify_regime(&tick, &sensors, &matrix, &config, None);
        assert_eq!(regime, MarketRegime::DataAnomaly);
    }
}
