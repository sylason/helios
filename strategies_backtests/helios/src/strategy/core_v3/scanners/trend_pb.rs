//! # TREND PULLBACK SCANNER (TREND_PB)
//!
//! ## 1. Mục đích:
//! Bắt các nhịp hồi quy có kiểm soát (Pullback / Retracement) thuận theo trục xu hướng vĩ mô
//! được dẫn dắt bởi KAMA (Kaufman Adaptive Moving Average). Mục tiêu là "Mua khi giá giảm trong Trend tăng,
//! Bán khi giá tăng trong Trend giảm" tại các vùng hỗ trợ/kháng cự động.
//!
//! ## 2. Logic Hoạt Động & Bộ Lọc 3 Tầng An Toàn:
//! - **Chỉ báo sử dụng:**
//!   + `kama_slope_atr`: Độ dốc chuẩn hóa của KAMA làm la bàn xu hướng.
//!   + `kama_mid`: Đường trung vị KAMA đóng vai trò hỗ trợ/kháng cự động.
//!   + `stoch_trend_k`: Stochastic nhận diện nhịp hạ nhiệt tạm thời của giá.
//!   + `rsi_trend`: Trần/Sàn RSI bảo vệ chống mua đu đỉnh hoặc bán đu đáy.
//!   + `clv` (Close Location Value): Đạo hàm vị trí nến xác nhận nến phản ứng thuận chiều.
//! - **Quy trình Chấm Điểm & Phê Duyệt:**
//!   + Bước 1: Kiểm tra độ dốc KAMA đủ mạnh (`|kama_slope| >= slope_thresh`).
//!   + Bước 2: Kiểm tra khoảng cách `delta_kama` nằm trong hộp an toàn `[-inval_floor, max_dist]`.
//!   + Bước 3: Stochastic đã hạ nhiệt + RSI chưa chạm trần cực đại.
//!   + Bước 4: CLV xác nhận đóng nến thuận chiều (Long cấm nến đỏ, Short cấm nến xanh).
//! - **Thiết lập Vị thế:**
//!   + Long: `suggested_sl` neo dưới đáy nhịp hồi hoặc hỗ trợ KAMA: `low.min(kama_mid) - buffer`.
//!   + Short: `suggested_sl` neo trên đỉnh nhịp hồi hoặc kháng cự KAMA: `high.max(kama_mid) + buffer`.
//!   + Take Profit: `None` (để Trailing Stop Loss trong `RiskManager` tự động thả diều lợi nhuận).

use ferro_trade::strategy::TickContext;
use crate::strategy::sensors::HeliosSensors;
use crate::strategy::core_v3::matrix::MarketMatrixV3;
use crate::strategy::core_v3::arbiter::{TacticalScanner, Proposal, SignalType, ScannerId};
use crate::strategy::core_v3::config::HeliosConfigV3;
use crate::strategy::core_v3::normalization::{sat_unbounded, sat_proximity, convex_combine};

pub struct TrendPBScanner;

impl TacticalScanner for TrendPBScanner {
    fn id(&self) -> ScannerId {
        ScannerId::TrendPullback
    }

    fn clone_box(&self) -> Box<dyn TacticalScanner> {
        Box::new(TrendPBScanner)
    }

    fn scan(&self, tick: &TickContext, sensors: &HeliosSensors, matrix: &MarketMatrixV3, config: &HeliosConfigV3) -> Option<Proposal> {
        let i = tick.i;
        let pb_cfg = &config.tactical_sniper.pullback;
        
        let kama_slope = matrix.kama_slope_atr.get(i).copied()
            .unwrap_or_else(|| sensors.kama_slope_adj.get(i).copied().unwrap_or(0.0));
        let stoch_k = sensors.stoch_trend_k.get(i).copied().unwrap_or(f64::NAN);
        let rsi = sensors.rsi_trend.get(i).copied().unwrap_or(f64::NAN);
        let z_score = sensors.z_score.get(i).copied().unwrap_or(0.0);
        let squeeze = matrix.squeeze_intensity.get(i).copied().unwrap_or(0.0);

        if kama_slope.is_nan() || stoch_k.is_nan() || rsi.is_nan() || z_score.is_nan() || squeeze.is_nan() {
            return None;
        }

        // 🧱 PHÒNG THỦ CHOP ZONE: Chặn đứng khi thị trường đang co thắt nén chặt BB/KC
        // Chop Zone là đất diễn của range_osc; TrendPB chỉ săn sóng khi thị trường đã bung nén.
        if squeeze > pb_cfg.max_squeeze_intensity {
            return None;
        }

        let slope_thresh = pb_cfg.kama_slope_threshold.max(1e-4);
        if kama_slope.abs() < slope_thresh {
            return None; // Độ dốc KAMA chưa đủ mạnh để hình thành xu hướng
        }

        let close = tick.closes()[i];
        let open = tick.opens()[i];
        let low = tick.lows()[i];
        let high = tick.highs()[i];

        let current_atr = sensors.atr_sl.get(i).copied().unwrap_or(1.0).max(1e-5);
        let kama_mid = sensors.kama_mid.get(i).copied().unwrap_or(close);
        let buffer = current_atr * pb_cfg.struct_sl_buffer_atr.max(0.05);

        // Khoảng cách chuẩn hóa không gian giá so với KAMA qua ATR
        let delta_kama = (close - kama_mid) / current_atr;
        let max_dist = pb_cfg.max_pullback_dist_atr.max(0.20);
        let inval_floor = pb_cfg.invalidation_buffer_atr.max(0.05);

        // Đạo hàm vi phân gradient nến: Close Location Value (CLV) [-1.0, +1.0]
        let tr_span = (high - low).max(1e-6);
        let clv = (2.0 * close - (high + low)) / tr_span;

        let f_slope = sat_unbounded(kama_slope.abs(), slope_thresh);

        // -------------------------------------------------------------------------
        // 1. CHIẾN ĐỘI PULLBACK LONG (KAMA Dốc Lên Rõ Rệt + Hồi Quy An Toàn + Gradient Dương)
        // -------------------------------------------------------------------------
        if kama_slope > slope_thresh {
            // 🛡️ CHỐT CHẶN BIÊN ĐỘ (BOUNDARY GUARD): Chống mua đu đỉnh phân phối
            // Pullback lành mạnh bắt buộc giá phải nằm trong vùng giá trị lân cận trung bình.
            // Cấm tuyệt đối Long khi Z > +max_boundary_z (vùng bùng nổ của momentum_ignition hoặc đỉnh phân phối).
            if z_score > pb_cfg.max_boundary_z {
                return None;
            }

            // Chốt chặn 1: Hộp không gian hồi quy (-inval <= delta_kama <= max_dist)
            if delta_kama < -inval_floor || delta_kama > max_dist {
                return None; // Thủng KAMA gãy trend hoặc bay quá xa KAMA mua đu đỉnh
            }

            // Chốt chặn 2: Vùng hạ nhiệt Stoch & Trần RSI an toàn (tránh hưng phấn đỉnh sóng)
            let stoch_thresh = pb_cfg.stoch_pullback_thresh.max(1.0);
            if stoch_k > stoch_thresh || rsi > pb_cfg.rsi_max_cap {
                return None;
            }

            // Chốt chặn 3: Gradient vận tốc nến xác nhận lực cầu (cấm nến đỏ Marubozu)
            if clv < 0.15 || close < open {
                return None;
            }

            let f_dist = sat_proximity(delta_kama.abs(), max_dist);
            let f_stoch = sat_proximity(stoch_k, stoch_thresh);
            let score_long = convex_combine(&[f_slope, f_dist, f_stoch], &[1.0, 1.0, 1.0]);

            if score_long >= pb_cfg.min_activation_score {
                // 🛡️ Structural SL neo dưới đáy nhịp hồi hoặc đường hỗ trợ KAMA
                let base_support = low.min(kama_mid);
                let mut sl = base_support - buffer;
                if sl >= close {
                    sl = close - buffer.max(close * 0.002);
                }

                return Some(Proposal {
                    signal_type: SignalType::Long,
                    raw_score: score_long,
                    scanner_id: self.id(),
                    suggested_sl: Some(sl),
                    suggested_tp: None,
                });
            }
        }
        // -------------------------------------------------------------------------
        // 2. CHIẾN ĐỘI PULLBACK SHORT (KAMA Dốc Xuống Rõ Rệt + Hồi Quy An Toàn + Gradient Âm)
        // -------------------------------------------------------------------------
        else if kama_slope < -slope_thresh {
            // 🛡️ CHỐT CHẶN BIÊN ĐỘ (BOUNDARY GUARD): Chống bán đuổi đáy hoảng loạn
            // Cấm tuyệt đối Short khi Z < -max_boundary_z (vùng quá bán cực đại, dễ bị rút chân quét SL).
            if z_score < -pb_cfg.max_boundary_z {
                return None;
            }

            // Chốt chặn 1: Hộp không gian hồi quy (-max_dist <= delta_kama <= inval)
            if delta_kama > inval_floor || delta_kama < -max_dist {
                return None; // Vượt thủng KAMA gãy trend hoặc rơi quá xa KAMA bán đuổi đáy
            }

            // Chốt chặn 2: Vùng hạ nhiệt Stoch & Sàn RSI an toàn
            let stoch_thresh = pb_cfg.stoch_pullback_thresh.max(1.0);
            if stoch_k < (100.0 - stoch_thresh) || rsi < (100.0 - pb_cfg.rsi_max_cap) {
                return None;
            }

            // Chốt chặn 3: Gradient vận tốc nến xác nhận lực cung (cấm nến xanh bốc đầu)
            if clv > -0.15 || close > open {
                return None;
            }

            let f_dist = sat_proximity(delta_kama.abs(), max_dist);
            let stoch_dist = (100.0 - stoch_k).max(0.0);
            let f_stoch = sat_proximity(stoch_dist, stoch_thresh);
            let score_short = convex_combine(&[f_slope, f_dist, f_stoch], &[1.0, 1.0, 1.0]);

            if score_short >= pb_cfg.min_activation_score {
                // 🛡️ Structural SL neo trên đỉnh nhịp hồi hoặc đường kháng cự KAMA
                let base_resistance = high.max(kama_mid);
                let mut sl = base_resistance + buffer;
                if sl <= close {
                    sl = close + buffer.max(close * 0.002);
                }

                return Some(Proposal {
                    signal_type: SignalType::Short,
                    raw_score: score_short,
                    scanner_id: self.id(),
                    suggested_sl: Some(sl),
                    suggested_tp: None,
                });
            }
        }

        None
    }
}
