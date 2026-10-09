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
    /// Sàn biến động chết / tắt thanh khoản (NATR < min_natr_threshold)
    VolatilityFlatline,
    /// Bất thường dữ liệu (NaN, Inf, hoặc chưa đủ warmup)
    DataAnomaly,
}

impl GateVetoReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateVetoReason::ChaosWhipsaw => "CHAOS_WHIPSAW",
            GateVetoReason::VolatilityFlatline => "VOLATILITY_FLATLINE",
            GateVetoReason::DataAnomaly => "DATA_ANOMALY",
        }
    }
}

/// Kết quả thẩm định trạng thái thị trường của Gatekeeper
#[derive(Debug, Clone, Default)]
pub struct GateEvaluation {
    /// Cho phép Arbiter tiếp tục quét tín hiệu hay không
    pub passed: bool,
    /// Lý do phủ quyết (nếu passed == false)
    pub veto_reason: Option<GateVetoReason>,
    /// Giá trị Chaos Index hiện tại
    pub current_chaos: f64,
    /// Giá trị NATR đo đạc thực tế của thị trường (%) từ sensors.natr MTF
    pub market_natr_pct: f64,
    /// Trạng thái khóa trễ Hysteresis có đang hoạt động hay không
    pub is_hysteresis_active: bool,
    /// 📉 Ngưỡng NATR tối thiểu yêu cầu (%) = fee_floor_pct + min_alpha_pct
    pub required_min_natr_pct: f64,
    /// 💰 Sàn chi phí thực tế (%) = 2*taker_fee + slippage
    pub fee_floor_pct: f64,
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
        if matrix.chaos_index.len() <= i || sensors.natr.len() <= i || sensors.atr_sl.len() <= i {
            return GateEvaluation {
                passed: false,
                veto_reason: Some(GateVetoReason::DataAnomaly),
                current_chaos: 0.0,
                market_natr_pct: 0.0,
                is_hysteresis_active: false,
                required_min_natr_pct: 0.0,
                fee_floor_pct: 0.0,
                min_alpha_pct: 0.0,
                chaos_threshold: 0.0,
            };
        }

        let current_chaos = matrix.chaos_index.get(i).copied().unwrap_or(f64::NAN);
        let market_natr_pct = sensors.natr.get(i).copied().unwrap_or(f64::NAN);
        let current_atr_sl = sensors.atr_sl.get(i).copied().unwrap_or(f64::NAN);
        let current_price = tick.closes().get(i).copied().unwrap_or(0.0);

        if current_chaos.is_nan()
            || market_natr_pct.is_nan()
            || current_atr_sl.is_nan()
            || current_price <= 1e-9
            || current_atr_sl <= 0.0
        {
            return GateEvaluation {
                passed: false,
                veto_reason: Some(GateVetoReason::DataAnomaly),
                current_chaos: 0.0,
                market_natr_pct: 0.0,
                is_hysteresis_active: false,
                required_min_natr_pct: 0.0,
                fee_floor_pct: 0.0,
                min_alpha_pct: 0.0,
                chaos_threshold: 0.0,
            };
        }

        // 2. Chốt chặn Sàn Biến Động theo Sàn Chi Phí Sinh Tồn (Physical Cost Floor) + DNA Alpha
        // sensors.natr lưu giá trị phần trăm (%) từ khung MTF chiến thuật (vd: 0.45 tức 0.45%)

        // Sàn chi phí sinh tồn = 2 * taker_fee_rate + slippage_allowance_pct (quy đổi sang %)
        let risk_base = &config.risk_management.risk_base;
        let roundtrip_fee_pct = (risk_base.taker_fee_rate * 2.0 + risk_base.slippage_allowance_pct) * 100.0;
        let min_alpha_pct = config.arbiter.min_natr_threshold * 100.0;
        let required_min_natr_pct = roundtrip_fee_pct + min_alpha_pct;
        let t_enter = config.arbiter.chaos_veto_threshold;

        if market_natr_pct < required_min_natr_pct {
            return GateEvaluation {
                passed: false,
                veto_reason: Some(GateVetoReason::VolatilityFlatline),
                current_chaos,
                market_natr_pct,
                is_hysteresis_active: false,
                required_min_natr_pct,
                fee_floor_pct: roundtrip_fee_pct,
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
                passed: false,
                veto_reason: Some(GateVetoReason::ChaosWhipsaw),
                current_chaos,
                market_natr_pct,
                is_hysteresis_active: true,
                required_min_natr_pct,
                fee_floor_pct: roundtrip_fee_pct,
                min_alpha_pct,
                chaos_threshold: t_enter,
            };
        }

        // Trường hợp 3.2: Nến hiện tại nằm trong dải trễ [t_exit, t_enter]
        // Quét ngược từ i-1 về start_idx để xác định trạng thái Hysteresis FSM:
        // - Nếu gặp nến < t_exit trước: cơn bão đã hạ nhiệt hoàn toàn trước đó -> Mở khóa
        // - Nếu gặp nến >= t_enter trước: bão chưa từng hạ nhiệt dưới t_exit -> Vẫn bị khóa
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
                    // Đã hạ nhiệt hoàn toàn trước đó
                    break;
                }
            }

            if storm_locked {
                return GateEvaluation {
                    passed: false,
                    veto_reason: Some(GateVetoReason::ChaosWhipsaw),
                    current_chaos,
                    market_natr_pct,
                    is_hysteresis_active: true,
                    required_min_natr_pct,
                    fee_floor_pct: roundtrip_fee_pct,
                    min_alpha_pct,
                    chaos_threshold: t_enter,
                };
            }
        }

        // Vượt qua toàn bộ các chốt chặn -> Trạng thái thị trường có thể giao dịch
        GateEvaluation {
            passed: true,
            veto_reason: None,
            current_chaos,
            market_natr_pct,
            is_hysteresis_active: false,
            required_min_natr_pct,
            fee_floor_pct: roundtrip_fee_pct,
            min_alpha_pct,
            chaos_threshold: t_enter,
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
        sensors.atr_sl = Arc::new(vec![10.0; 20]);
        matrix.chaos_index = Arc::new(vec![0.30; 20]); // 0.30 < 0.65 threshold

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(eval.passed);
        assert_eq!(eval.veto_reason, None);
        assert!(!eval.is_hysteresis_active);
        assert!((eval.market_natr_pct - 0.45).abs() < 1e-4, "Market NATR phải là 0.45%");
    }

    #[test]
    fn test_gatekeeper_volatility_flatline_veto() {
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.10; 20]); // Macro NATR 0.10% < 0.22% (Sàn chi phí 0.10% + DNA Alpha 0.12%)
        sensors.atr_sl = Arc::new(vec![2.0; 20]);
        matrix.chaos_index = Arc::new(vec![0.30; 20]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::VolatilityFlatline));
        assert!((eval.market_natr_pct - 0.10).abs() < 1e-4);
    }

    #[test]
    fn test_gatekeeper_immune_to_micro_candle_range() {
        // 🛡️ CHỐNG BẪY LỆCH NGỮ NGHĨA: Nến 1m tích lũy co hẹp (ATR 1m = 1.0 / 2000.0 = 0.05%)
        // nhưng nếu phiên thị trường NATR MTF lành mạnh (0.45%) thì KHÔNG ĐƯỢC VETO!
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(20);
        let tick = env.tick(10);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.45; 20]); // Macro NATR 0.45% > 0.22%
        sensors.atr_sl = Arc::new(vec![1.0; 20]); // Micro ATR chỉ 1.0 (nến 1m co hẹp 0.05%)
        matrix.chaos_index = Arc::new(vec![0.30; 20]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(eval.passed, "Nến 1m co hẹp nhưng phiên có biến động lành mạnh thì phải PASS!");
        assert_eq!(eval.veto_reason, None);
        assert!((eval.market_natr_pct - 0.45).abs() < 1e-4);
    }

    #[test]
    fn test_gatekeeper_micro_data_anomaly_zero_atr() {
        // Nếu nến vi mô 1m chết thanh khoản hoàn toàn (ATR_SL <= 0.0), phải trả về DataAnomaly
        let config = HeliosConfigV3::default();
        let mut env = MockTickEnv::new(10);
        let tick = env.tick(5);
        let mut sensors = HeliosSensors::default();
        let mut matrix = MarketMatrixV3::default();

        sensors.natr = Arc::new(vec![0.45; 10]);
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
        sensors.atr_sl = Arc::new(vec![10.0; 20]);
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
        sensors.atr_sl = Arc::new(vec![10.0; 10]);
        matrix.chaos_index = Arc::new(vec![0.30; 10]);

        let eval = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval.passed);
        assert_eq!(eval.veto_reason, Some(GateVetoReason::DataAnomaly));

        // Test ATR_SL chứa NaN
        sensors.natr = Arc::new(vec![0.25; 10]);
        sensors.atr_sl = Arc::new(vec![f64::NAN; 10]);
        let eval2 = RegimeGatekeeper::evaluate(&tick, &sensors, &matrix, &config);
        assert!(!eval2.passed);
        assert_eq!(eval2.veto_reason, Some(GateVetoReason::DataAnomaly));
    }
}
