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

/// Cấu hình tập trung cho toàn bộ Động cơ Bot (EngineConfig)
/// Gom toàn bộ các hằng số hardcoded về một nguồn duy nhất (Single Source of Truth).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    /// Vốn ban đầu (USD / USDT)
    pub initial_cash: f64,
    /// Tỷ lệ phí giao dịch (Ví dụ: 0.0004 = 0.04% Taker fee)
    pub commission_rate: f64,
    /// Tỷ lệ ký quỹ (Ví dụ: 0.05 = đòn bẩy 20x, 1.0 = Spot)
    pub margin_rate: f64,
    /// Đơn vị làm tròn số lượng nhỏ nhất (Fractional Unit)
    pub fractional_unit: Option<f64>,
    /// Số lượng nến 1m tối đa lưu trữ trong Rolling Buffer RAM
    pub max_live_mem_candles: usize,
    /// Số ngày nến lịch sử cần đồng bộ khi khởi động (Ví dụ: 33 ngày)
    pub historical_sync_days: u64,
    /// Số nến đệm thêm để bảo toàn nến vào vị thế (Padding bars)
    pub buffer_padding_bars: usize,
    /// Số luồng tính toán tối ưu hóa đa luồng
    pub opt_num_threads: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            initial_cash: 60.0,
            commission_rate: 0.0004,
            margin_rate: 0.05,
            fractional_unit: Some(0.0001),
            max_live_mem_candles: 15000,
            historical_sync_days: 33,
            buffer_padding_bars: 200,
            opt_num_threads: 4,
        }
    }
}

impl EngineConfig {
    /// Kiểm định tính hợp lệ của cấu hình
    pub fn validate(&self) -> Result<(), String> {
        if self.initial_cash <= 0.0 {
            return Err("initial_cash must be positive".to_string());
        }
        if self.commission_rate < 0.0 {
            return Err("commission_rate cannot be negative".to_string());
        }
        if self.max_live_mem_candles < 1000 {
            return Err("max_live_mem_candles must be at least 1000 for MTF warmup".to_string());
        }
        Ok(())
    }
}

/// Trình phân giải đường dẫn an toàn (PathResolver)
/// Giúp tìm đúng file config/database ở bất kỳ thư mục làm việc (CWD) nào.
pub struct PathResolver;

impl PathResolver {
    /// Tìm file theo đường dẫn tương đối, tự động kiểm tra CWD và Workspace Root
    pub fn resolve(relative_path: &str) -> std::path::PathBuf {
        let p = std::path::Path::new(relative_path);
        if p.exists() {
            return p.to_path_buf();
        }

        // Kiểm tra từ biến môi trường CARGO_MANIFEST_DIR
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let candidate = std::path::Path::new(&manifest_dir).join(relative_path);
            if candidate.exists() {
                return candidate;
            }

            // Kiểm tra lùi về root workspace
            if let Some(parent) = std::path::Path::new(&manifest_dir).parent() {
                let root_candidate = parent.join(relative_path);
                if root_candidate.exists() {
                    return root_candidate;
                }
                if let Some(grandparent) = parent.parent() {
                    let gp_candidate = grandparent.join(relative_path);
                    if gp_candidate.exists() {
                        return gp_candidate;
                    }
                }
            }
        }

        p.to_path_buf()
    }

    /// Tự động tìm kiếm đường dẫn cấu hình chiến lược (Strategy Config Resolver)
    pub fn resolve_strategy_config(strategy_id: &str) -> String {
        let candidates = [
            format!("examples/{}_optimizer_config.json", strategy_id),
            format!("config/{}_optimizer_config.json", strategy_id),
            format!("{}_optimizer_config.json", strategy_id),
            format!("../../strategies_backtests/{}/examples/{}_optimizer_config.json", strategy_id, strategy_id),
            format!("strategies_backtests/{}/examples/{}_optimizer_config.json", strategy_id, strategy_id),
            format!("../strategies_backtests/{}/examples/{}_optimizer_config.json", strategy_id, strategy_id),
        ];

        for cand in &candidates {
            let p = Self::resolve(cand);
            if p.exists() {
                return p.to_string_lossy().into_owned();
            }
        }

        candidates[0].clone()
    }
}
