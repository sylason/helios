use std::env;
use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub api_secret: String,
    pub opt_num_threads: usize,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        
        Self {
            api_key: env::var("BINANCE_API_KEY").unwrap_or_default(),
            api_secret: env::var("BINANCE_API_SECRET").unwrap_or_default(),
            opt_num_threads: env::var("OPT_NUM_THREADS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
        }
    }
}
