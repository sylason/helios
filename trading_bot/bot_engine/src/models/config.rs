use std::env;
use dotenv::dotenv;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ExecutionMode {
    Paper,       // Giả lập 100% qua PaperTrader ở máy local (Zero-delay hoặc real-time)
    Testnet,     // Kết nối Binance Futures Testnet (Gửi lệnh thật lên Testnet API)
    RealTrade,   // Kết nối Binance Futures Mainnet (Gửi lệnh thật tiền thật)
    DryRun,      // Nhận nến real-time, tính tín hiệu nhưng CHỈ LOG chứ KHÔNG gửi HTTP API
    Diagnostic,  // Chỉ chạy Backtest chẩn đoán 1:1 đo đạc PnL/DD
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub api_secret: String,
    pub use_testnet: bool,
    pub opt_num_threads: usize,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        let use_testnet = env::var("BINANCE_USE_TESTNET").unwrap_or_else(|_| "false".to_string()) == "true";
        
        let api_key = if use_testnet {
            env::var("BINANCE_TESTNET_API_KEY").unwrap_or_default()
        } else {
            env::var("BINANCE_API_KEY").unwrap_or_default()
        };

        let api_secret = if use_testnet {
            env::var("BINANCE_TESTNET_SECRET_KEY").unwrap_or_default()
        } else {
            env::var("BINANCE_API_SECRET").unwrap_or_default()
        };

        Self {
            api_key,
            api_secret,
            use_testnet,
            opt_num_threads: env::var("OPT_NUM_THREADS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
        }
    }
}
