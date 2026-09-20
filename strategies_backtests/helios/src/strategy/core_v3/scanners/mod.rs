//! # TACTICAL SCANNERS MODULE (HELIOS V3)
//! 
//! Module này chứa 6 Tactical Scanners chuyên trách độc lập, đóng vai trò phát hiện cơ hội
//! vào lệnh theo từng bối cảnh thị trường cụ thể (Regime):
//! 1. `trend_pb`: Bắt điểm hồi quy thuận xu hướng KAMA (Trend Pullback).
//! 2. `momentum_ignition`: Bắt các cú bứt phá xung lực nến mạnh (Momentum Breakout).
//! 3. `range_osc`: Đánh dao động biên trên/dưới trong vùng đi ngang Bollinger Bands (Mean Reversion).
//! 4. `reversal`: Bắt điểm kiệt sức đảo chiều tại các vùng quá độ Z-Score và KAMA (Climax Fade).
//! 5. `anomaly`: Bắt dị biệt biến động giá và râu nến bất thường (Anomaly Wick Fade).
//! 6. `absorption`: Bắt vùng nén thanh khoản và gom hàng/phân phối của dòng tiền lớn (Volume Squeeze).

pub mod trend_pb;
pub mod momentum_ignition;
pub mod range_osc;
pub mod reversal;
pub mod anomaly;
pub mod absorption;
