use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Cấu trúc nến 1 phút cơ bản
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Các loại hành động giao dịch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    OpenLong,
    CloseLong,
    OpenShort,
    CloseShort,
}

/// Tín hiệu giao dịch được tạo ra từ Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub symbol: String,
    pub action: TradeAction,
    pub price: f64,
    pub params: Option<serde_json::Value>,
}

/// Các trạng thái của một tài sản (Finite State Machine)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetState {
    InitialSync,    // Đang tải dữ liệu 30 ngày
    Optimizing,     // Đang chạy GA để tìm Params
    Trading,        // Đang trade real-time
    Observing,      // Chế độ an toàn (chỉ xem, không trade)
    Stopped,        // Dừng hoạt động
}

/// Các lệnh khẩn cấp từ hệ thống hoặc người dùng
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyCmd {
    PanicCloseAll,
    StopAsset(String),
    SystemShutdown,
}

/// Tin nhắn báo cáo trạng thái dành cho UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub symbol: String,
    pub state: AssetState,
    pub message: String,
}

/// Dữ liệu DNA đã được tối ưu hóa cho một cặp tiền và chiến lược cụ thể
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedDna {
    pub symbol: String,
    pub strategy_id: String,
    pub dna: Vec<f64>,
    pub mapping: HashMap<String, usize>,
    pub fitness: f64,
    pub stats: HashMap<String, f64>,
    pub updated_at: i64,
}

/// [THE BRAIN] Quản lý dữ liệu OHLCV và các chỉ báo của một tài sản
pub struct Asset {
    pub symbol: String,
    /// Bộ nhớ đệm nến 1m (Max: 43200 nến ~ 30 ngày)
    pub ohlcv: VecDeque<Candle>,
    /// Kho chứa các chỉ báo kỹ thuật đã tính toán (Key: Tên chỉ báo)
    pub indicators: HashMap<String, Arc<Vec<f64>>>,
}

impl Asset {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            ohlcv: VecDeque::with_capacity(43200),
            indicators: HashMap::new(),
        }
    }

    /// Thêm nến mới và tự động xóa nến cũ nếu vượt quá 30 ngày
    pub fn push_candle(&mut self, candle: Candle) {
        if self.ohlcv.len() >= 43200 {
            self.ohlcv.pop_front();
        }
        self.ohlcv.push_back(candle);
    }

    /// Lấy toàn bộ dữ liệu dưới dạng Vec (phục vụ HeliosBridge)
    pub fn get_candles_vec(&self) -> Vec<Candle> {
        self.ohlcv.iter().cloned().collect()
    }
}
