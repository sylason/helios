use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Cấu trúc nến 1 phút cơ bản
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Candle {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    #[serde(default)]
    pub quote_volume: f64,
}

impl Candle {
    /// Tính toán giá trị thanh khoản danh nghĩa hiệu dụng (Quote Asset Volume - USDT).
    /// Nếu sàn Binance đã cấp quote_volume gốc (> 0.0) thì dùng trực tiếp 1:1.
    /// Nếu nến lịch sử từ CSV cũ hoặc test mock có quote_volume == 0.0, tự động fallback về close * volume.
    #[inline]
    pub fn effective_quote_volume(&self) -> f64 {
        if self.quote_volume > 0.0 {
            self.quote_volume
        } else {
            self.close * self.volume
        }
    }
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
    /// Bộ nhớ đệm nến 1m (Co giãn linh động theo cấu hình)
    pub ohlcv: VecDeque<Candle>,
    /// Kho chứa các chỉ báo kỹ thuật đã tính toán (Key: Tên chỉ báo)
    pub indicators: HashMap<String, Arc<Vec<f64>>>,
    /// [TODO-AUDIT-NEW-15] Dung lượng bộ đệm RAM tối đa linh động (mặc định: 43,200 nến = 30 ngày)
    pub max_capacity: usize,
}

impl Asset {
    pub fn new(symbol: String) -> Self {
        Self::new_with_capacity(symbol, 43200)
    }

    /// [TODO-AUDIT-NEW-15] Khởi tạo Asset với dung lượng bộ đệm RAM tùy biến theo cấu hình
    pub fn new_with_capacity(symbol: String, capacity: usize) -> Self {
        let cap = capacity.max(1000);
        Self {
            symbol,
            ohlcv: VecDeque::with_capacity(cap),
            indicators: HashMap::new(),
            max_capacity: cap,
        }
    }

    /// Thêm nến mới và tự động xóa nến cũ nếu vượt quá dung lượng max_capacity. Chống trùng lặp.
    pub fn push_candle(&mut self, candle: Candle) {
        // Kiểm tra chống trùng lặp / nến cũ
        if let Some(last) = self.ohlcv.back() {
            if candle.timestamp <= last.timestamp {
                return; // Bỏ qua nếu là nến cũ hoặc trùng lặp
            }
        }
        
        if self.ohlcv.len() >= self.max_capacity {
            self.ohlcv.pop_front();
        }
        self.ohlcv.push_back(candle);
    }

    /// Lấy toàn bộ dữ liệu dưới dạng Vec (phục vụ HeliosBridge)
    pub fn get_candles_vec(&self) -> Vec<Candle> {
        self.ohlcv.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_filler_deduplication() {
        let mut asset = Asset::new("BTCUSDT".to_string());
        
        // Tạo hàm helper để tạo nến nhanh
        let make_candle = |ts: i64| Candle {
            symbol: "BTCUSDT".to_string(),
            timestamp: ts,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 10.0,
            quote_volume: 1000.0,
        };

        // 1. Khởi tạo mảng nến có khoảng trống (Gap) ở giữa (Mất mạng phút 2, 3)
        asset.push_candle(make_candle(1000)); // Phút 1
        asset.push_candle(make_candle(4000)); // Phút 4 (Có mạng lại)
        asset.push_candle(make_candle(5000)); // Phút 5
        
        // Assert kiểm tra sự tồn tại của khoảng trống (Before)
        let vec_before = asset.get_candles_vec();
        assert_eq!(vec_before.len(), 3);
        assert_eq!(vec_before[0].timestamp, 1000);
        assert_eq!(vec_before[1].timestamp, 4000); // Rõ ràng mất 2000 và 3000
        
        // 2. Chạy hàm Gap Filler mô phỏng
        // Gap Filler lấy được dữ liệu từ phút 1 đến phút 5 (có thể bao gồm cả nến trùng lặp)
        let gap_candles = vec![
            make_candle(1000), // Trùng
            make_candle(2000), // Bù
            make_candle(3000), // Bù
            make_candle(4000), // Trùng
            make_candle(5000), // Trùng
        ];
        
        // Để trám nến đúng cách khi có gap ở giữa, push_candle truyền thống sẽ không tự sort.
        // Tuy nhiên, AssetActor nhận nến từ data_tx theo thứ tự. 
        // Khi Gap Filler chạy lúc reconnect, nó sẽ được bơm TRƯỚC KHI nến hiện tại (Phút 4, 5) đến!
        // Vậy nên ta cần test đúng kịch bản thực tế: 
        // Lúc mất mạng (sau 1000), khi có mạng lại, WS Manager CHƯA cho WS push vội, 
        // mà nó gọi Gap Filler để đẩy 2000, 3000 trước.
        
        let mut real_asset = Asset::new("BTCUSDT".to_string());
        real_asset.push_candle(make_candle(1000)); // Phút 1
        // Đứt mạng
        // Có mạng lại -> Chạy Gap Filler bơm nến cũ
        for c in gap_candles {
            real_asset.push_candle(c);
        }
        // WS nhận nến mới của hiện tại
        real_asset.push_candle(make_candle(4000));
        real_asset.push_candle(make_candle(5000));

        // 3. Assert kiểm tra dữ liệu nến đã được lấp đầy và liên tục (After)
        let vec_after = real_asset.get_candles_vec();
        assert_eq!(vec_after.len(), 5);
        assert_eq!(vec_after[0].timestamp, 1000);
        assert_eq!(vec_after[1].timestamp, 2000);
        assert_eq!(vec_after[2].timestamp, 3000);
        assert_eq!(vec_after[3].timestamp, 4000);
        assert_eq!(vec_after[4].timestamp, 5000);
    }

    #[test]
    fn test_asset_dynamic_capacity() {
        // Khởi tạo Asset với dung lượng tùy biến 1000 nến
        let mut asset = Asset::new_with_capacity("ETHUSDT".to_string(), 1000);
        assert_eq!(asset.max_capacity, 1000);

        // Bơm 1005 nến -> Buffer phải tự động pop 5 nến đầu và duy trì đúng 1000 nến
        for i in 1..=1005 {
            asset.push_candle(Candle {
                symbol: "ETHUSDT".to_string(),
                timestamp: i * 60_000,
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 50.0,
                quote_volume: 5100.0,
            });
        }

        assert_eq!(asset.ohlcv.len(), 1000);
        assert_eq!(asset.ohlcv.front().unwrap().timestamp, 6 * 60_000); // 5 nến đầu (1..5) đã bị pop
        assert_eq!(asset.ohlcv.back().unwrap().timestamp, 1005 * 60_000);
    }
}

