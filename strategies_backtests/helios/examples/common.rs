// helios/examples/common.rs

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

