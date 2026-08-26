use tokio::sync::{broadcast, mpsc};
use crate::models::models::{Candle, TradeSignal, EmergencyCmd, StatusUpdate};
use crate::models::account_state::AccountEvent;

/// "Hệ thần kinh trung ương" của Bot
pub struct EventBus {
    /// Kênh truyền dữ liệu nến 1m (Data Bus)
    pub data_tx: broadcast::Sender<(Candle, bool)>,
    
    /// Kênh truyền tín hiệu giao dịch (Trade Bus)
    pub trade_tx: mpsc::Sender<TradeSignal>,
    
    /// Kênh truyền lệnh khẩn cấp (Emergency Bus)
    pub emergency_tx: mpsc::Sender<EmergencyCmd>,

    /// Kênh báo cáo trạng thái lên UI (Status Bus)
    pub status_tx: mpsc::Sender<StatusUpdate>,
    
    /// Kênh truyền tải trạng thái tài khoản và vị thế
    pub account_tx: broadcast::Sender<AccountEvent>,
}

impl EventBus {
    pub fn new() -> (Self, mpsc::Receiver<TradeSignal>, mpsc::Receiver<EmergencyCmd>, mpsc::Receiver<StatusUpdate>) {
        let (data_tx, _) = broadcast::channel(4096);
        let (trade_tx, trade_rx) = mpsc::channel(50);
        let (emergency_tx, emergency_rx) = mpsc::channel(10);
        let (status_tx, status_rx) = mpsc::channel(100);
        let (account_tx, _) = broadcast::channel(512);

        (
            Self {
                data_tx,
                trade_tx,
                emergency_tx,
                status_tx,
                account_tx,
            },
            trade_rx,
            emergency_rx,
            status_rx,
        )
    }
}
