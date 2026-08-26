use tracing::info;
use ferro_trade::broker::actions::Action;
use crate::models::account_state::AccountEvent;
use serde::{Serialize, Deserialize};

/// [TYPE-SAFE EXIT REASON]
/// Phân loại chính xác 100% lý do đóng lệnh cho hệ thống thống kê Quant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    /// 1. Chạm Trailing Stoploss (đã dời theo chiều có lợi)
    TrailingStopLoss,
    /// 2. Chạm Cắt Lỗ Cứng Ban Đầu (vừa vào lệnh bị giật ngược)
    HardStopLoss,
    /// 3. Chạm Chốt Lời Mục Tiêu (Take Profit)
    TakeProfit,
    /// 4. Cắt lỗ sớm do thời gian đi ngang (Stagnant Time Decay)
    StagnantDecay,
    /// 5. Đóng lệnh giải phóng vốn vì ngâm quá 4 chu kỳ (Zombie Trade Timeout)
    ZombieTimeout,
    /// 6. Chiến lược chủ động phát tín hiệu đóng lệnh
    StrategySignalClose,
}

impl ExitReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExitReason::TrailingStopLoss => "TSL",
            ExitReason::HardStopLoss => "HARD_SL",
            ExitReason::TakeProfit => "TP",
            ExitReason::StagnantDecay => "STAGNANT_DECAY",
            ExitReason::ZombieTimeout => "ZOMBIE_TIMEOUT",
            ExitReason::StrategySignalClose => "STRATEGY_CLOSE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradeRecord {
    pub id: usize,
    pub pos_type: PositionType,
    pub entry_price: f64,
    pub exit_price: f64,
    pub entry_timestamp: i64,
    pub exit_timestamp: i64,
    pub bars_held: usize,
    pub exit_reason: ExitReason,
    pub opening_playbook_tag: Option<String>,
    pub net_pnl: f64,
    pub pnl_pct: f64,
    pub equity_after: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionType {
    None,
    Long,
    Short,
}

pub struct PaperTrader {
    pub symbol: String,
    pub equity: f64,
    pub initial_equity: f64,
    pub position_type: PositionType,
    pub entry_price: f64,
    pub position_size: f64, // Số lượng coin đang giữ
    pub commission_rate: f64,
    pub entry_commission: f64, // Phí mở lệnh của vị thế hiện tại
    pub leverage: f64,
    pub current_sl_price: Option<f64>, // Mức giá Trailing StopLoss bảo vệ
    pub current_tp_price: Option<f64>, // Mức giá TakeProfit chốt lời
    pub entry_timestamp: i64,          // Thời điểm mở vị thế (ms)
    pub entry_bar_index: usize,        // Chỉ số nến mở vị thế
    pub opening_playbook_tag: Option<String>, // Tag Scanner mở lệnh
    pub total_trades: usize,           // Tích lũy tổng số giao dịch từ đầu
    pub winning_trades: usize,         // Số giao dịch thắng tích lũy từ đầu
    pub losing_trades: usize,          // Số giao dịch thua tích lũy từ đầu
    pub closed_trades: Vec<PaperTradeRecord>,
}

impl PaperTrader {
    pub fn new(symbol: String, initial_equity: f64, commission_rate: f64) -> Self {
        Self {
            symbol,
            equity: initial_equity,
            initial_equity,
            position_type: PositionType::None,
            entry_price: 0.0,
            position_size: 0.0,
            commission_rate,
            entry_commission: 0.0,
            leverage: 1.0,
            current_sl_price: None,
            current_tp_price: None,
            entry_timestamp: 0,
            entry_bar_index: 0,
            opening_playbook_tag: None,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            closed_trades: Vec::new(),
        }
    }


    /// Cập nhật PnL nếu có vị thế đang mở (Unrealized PnL)
    pub fn get_unrealized_pnl(&self, current_price: f64) -> f64 {
        if self.position_type == PositionType::None {
            return 0.0;
        }

        let price_diff = if self.position_type == PositionType::Long {
            current_price - self.entry_price
        } else {
            self.entry_price - current_price
        };

        price_diff * self.position_size
    }

    /// Tính toán vốn hiện tại (Bao gồm Unrealized PnL)
    pub fn get_total_equity(&self, current_price: f64) -> f64 {
        self.equity + self.get_unrealized_pnl(current_price)
    }

    /// Báo cáo thống kê tích lũy toàn bộ thời gian (Cumulative All-Time Report)
    pub fn log_cumulative_report(&self, current_price: f64, time_label: &str) {
        let total_eq = self.get_total_equity(current_price);
        let ret_pct = if self.initial_equity > 0.0 {
            ((total_eq - self.initial_equity) / self.initial_equity) * 100.0
        } else {
            0.0
        };
        let win_rate = if self.total_trades > 0 {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        };
        info!(
            "[CUMULATIVE PAPER TRADE REPORT] [{}] Time: {} | Initial: ${:.2} -> Current: ${:.2} | Cum. Return: {:.2}% | Total Trades: {} | Win Rate: {:.2}% (W: {}, L: {})",
            self.symbol,
            time_label,
            self.initial_equity,
            total_eq,
            ret_pct,
            self.total_trades,
            win_rate,
            self.winning_trades,
            self.losing_trades
        );
    }

    /// Broadcast sự kiện trạng thái vị thế và tài khoản ra EventBus
    fn broadcast_account_events(&self, account_tx: Option<&tokio::sync::broadcast::Sender<AccountEvent>>) {
        if let Some(tx) = account_tx {
            let position_amt = match self.position_type {
                PositionType::Long => self.position_size,
                PositionType::Short => -self.position_size,
                PositionType::None => 0.0,
            };
            let pos_evt = AccountEvent::PositionUpdate {
                symbol: self.symbol.clone(),
                position_amt,
                entry_price: self.entry_price,
                unrealized_profit: 0.0,
            };
            let acc_evt = AccountEvent::AccountUpdate {
                wallet_balance: self.equity,
                available_balance: self.equity,
            };

            let _ = tx.send(pos_evt);
            let _ = tx.send(acc_evt);
        }
    }

    /// Đóng vị thế hiện tại
    pub fn close_position(
        &mut self, 
        current_price: f64,
        exit_reason: ExitReason,
        exit_timestamp: i64,
        exit_bar_index: usize,
        account_tx: Option<&tokio::sync::broadcast::Sender<AccountEvent>>
    ) {
        if self.position_type == PositionType::None {
            return;
        }

        let pnl = self.get_unrealized_pnl(current_price);
        let exit_value = current_price * self.position_size;
        let exit_commission = exit_value * self.commission_rate;
        let old_position = self.position_type.clone();
        
        // PnL ròng sau khi trừ phí đóng lệnh (Equity đã bị trừ entry_commission trước đó)
        let net_pnl_realized = pnl - exit_commission;
        let total_commission = self.entry_commission + exit_commission;
        let net_pnl_trade = pnl - total_commission; // Net PnL hoàn chỉnh cả 2 chiều để lưu record

        let margin_used = (self.entry_price * self.position_size) / self.leverage;
        let pnl_pct = if margin_used > 0.0 { (net_pnl_trade / margin_used) * 100.0 } else { 0.0 };

        // Cập nhật Equity & Thống kê tích lũy toàn bộ thời gian
        self.equity += net_pnl_realized;
        self.total_trades += 1;
        if net_pnl_trade > 0.0 {
            self.winning_trades += 1;
        } else if net_pnl_trade < 0.0 {
            self.losing_trades += 1;
        }
        
        let bars_held = exit_bar_index.saturating_sub(self.entry_bar_index).max(1);

        self.closed_trades.push(PaperTradeRecord {
            id: self.total_trades,
            pos_type: old_position.clone(),
            entry_price: self.entry_price,
            exit_price: current_price,
            entry_timestamp: self.entry_timestamp,
            exit_timestamp,
            bars_held,
            exit_reason,
            opening_playbook_tag: self.opening_playbook_tag.clone(),
            net_pnl: net_pnl_trade,
            pnl_pct,
            equity_after: self.equity,
        });

        let tag_str = self.opening_playbook_tag.as_deref().unwrap_or("NONE");
        info!(
            "[PAPER TRADER] CLOSE {}: Reason = {}, Tag = {}, Exit Price = {:.2}, Net PnL = ${:+.2} ({:+.2}% ROE), Bars Held = {}, New Equity = ${:.2} (Cum. Trades: {})",
            match old_position {
                PositionType::Long => "LONG",
                PositionType::Short => "SHORT",
                PositionType::None => "NONE",
            },
            exit_reason.as_str(),
            tag_str,
            current_price,
            net_pnl_trade,
            pnl_pct,
            bars_held,
            self.equity,
            self.total_trades
        );

        self.position_type = PositionType::None;
        self.entry_price = 0.0;
        self.position_size = 0.0;
        self.entry_commission = 0.0;
        self.current_sl_price = None;
        self.current_tp_price = None;
        self.entry_timestamp = 0;
        self.entry_bar_index = 0;
        self.opening_playbook_tag = None;

        self.broadcast_account_events(account_tx);
    }

    /// In ra bảng log chi tiết tất cả các lệnh đã thực hiện và vị thế đang mở
    pub fn log_all_trades(&self, current_price: f64) {
        info!("================================================================================");
        info!("📋 [PAPER TRADER] DANH SÁCH TẤT CẢ CÁC GIAO DỊCH ĐÃ ĐÓNG ({})", self.closed_trades.len());
        if self.closed_trades.is_empty() {
            info!("   (Chưa phát sinh giao dịch nào đã đóng)");
        } else {
            for record in &self.closed_trades {
                let side_str = match record.pos_type {
                    PositionType::Long => "LONG ",
                    PositionType::Short => "SHORT",
                    PositionType::None => "NONE ",
                };
                let status_icon = if record.net_pnl >= 0.0 { "🟢" } else { "🔴" };
                let price_pnl_pct = if record.entry_price > 0.0 {
                    let dir = match record.pos_type {
                        PositionType::Long => 1.0,
                        PositionType::Short => -1.0,
                        PositionType::None => 0.0,
                    };
                    ((record.exit_price / record.entry_price - 1.0) * dir) * 100.0
                } else {
                    0.0
                };
                let tag_str = record.opening_playbook_tag.as_deref().unwrap_or("NONE");
                info!("   {} [TRADE #{:03}] {} [{}] Entry: {:.2} -> Exit: {:.2} | Reason: {} ({} bars) | Net PnL: ${:+.2} ({:+.2}% ROE | {:+.2}% Price) | Equity: ${:.2}",
                    status_icon, record.id, side_str, tag_str, record.entry_price, record.exit_price, record.exit_reason.as_str(), record.bars_held, record.net_pnl, record.pnl_pct, price_pnl_pct, record.equity_after);
            }
        }
        
        if self.position_type != PositionType::None {
            let u_pnl = self.get_unrealized_pnl(current_price);
            let margin_used = (self.entry_price * self.position_size) / self.leverage;
            let u_pnl_pct = if margin_used > 0.0 { (u_pnl / margin_used) * 100.0 } else { 0.0 };
            let u_price_pct = if self.entry_price > 0.0 {
                let dir = match self.position_type {
                    PositionType::Long => 1.0,
                    PositionType::Short => -1.0,
                    PositionType::None => 0.0,
                };
                ((current_price / self.entry_price - 1.0) * dir) * 100.0
            } else {
                0.0
            };
            let pos_str = match self.position_type {
                PositionType::Long => "LONG ",
                PositionType::Short => "SHORT",
                PositionType::None => "NONE ",
            };
            let sl_str = self.current_sl_price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
            let tp_str = self.current_tp_price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
            let tag_str = self.opening_playbook_tag.as_deref().unwrap_or("NONE");
            let icon = if u_pnl >= 0.0 { "🟢" } else { "🔴" };
            info!("--------------------------------------------------------------------------------");
            info!("📌 [CURRENT OPEN POSITION] {} {} [{}] Entry: {:.2} | Current: {:.2} | Size: {:.4} | SL: {} | TP: {} | uPnL: ${:+.2} ({:+.2}% ROE | {:+.2}% Price)",
                  icon, pos_str, tag_str, self.entry_price, current_price, self.position_size, sl_str, tp_str, u_pnl, u_pnl_pct, u_price_pct);
        } else {
            info!("--------------------------------------------------------------------------------");
            info!("📌 [CURRENT OPEN POSITION] (Không có vị thế nào đang mở)");
        }
        info!("================================================================================");
    }

    /// Mở vị thế mới (Tôn trọng size_pct & policy từ Action của Strategy)
    pub fn open_position(
        &mut self, 
        pos_type: PositionType, 
        current_price: f64, 
        size_pct: Option<f64>,
        initial_sl: Option<f64>,
        initial_tp: Option<f64>,
        tag: Option<String>,
        entry_timestamp: i64,
        entry_bar_index: usize,
        account_tx: Option<&tokio::sync::broadcast::Sender<AccountEvent>>
    ) {
        if self.position_type != PositionType::None {
            self.close_position(current_price, ExitReason::StrategySignalClose, entry_timestamp, entry_bar_index, account_tx);
        }

        let size_ratio = size_pct.filter(|&s| s > 0.0).unwrap_or(1.0);
        let trade_value = self.equity * size_ratio * self.leverage;
        let commission = trade_value * self.commission_rate;
        self.entry_commission = commission;
        
        // Trừ phí mở lệnh
        self.equity -= commission;
        
        // Tính size
        self.position_size = trade_value / current_price;
        self.entry_price = current_price;
        self.position_type = pos_type;
        self.current_sl_price = initial_sl;
        self.current_tp_price = initial_tp;
        self.entry_timestamp = entry_timestamp;
        self.entry_bar_index = entry_bar_index;
        self.opening_playbook_tag = tag;

        let sl_display = self.current_sl_price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
        let tp_display = self.current_tp_price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
        let tag_display = self.opening_playbook_tag.as_deref().unwrap_or("NONE");

        info!("📈 [PaperTrader] MỞ VỊ THẾ {:?} [{}] @ {:.2}. Size: {:.4} (Ratio: {:.1}%). SL: {}, TP: {}. Equity còn: {:.2}", 
            self.position_type, tag_display, self.entry_price, self.position_size, size_ratio * 100.0, sl_display, tp_display, self.equity);

        self.broadcast_account_events(account_tx);
    }

    /// Quét nến real-time xem giá High/Low có chạm Trailing SL hay TP hay không
    pub fn on_candle(
        &mut self, 
        candle: &crate::models::models::Candle, 
        account_tx: Option<&tokio::sync::broadcast::Sender<AccountEvent>>
    ) -> bool {
        if self.position_type == PositionType::None {
            return false;
        }

        let sl_opt = self.current_sl_price;
        let tp_opt = self.current_tp_price;

        let (is_sl_hit, is_tp_hit) = match self.position_type {
            PositionType::Long => (
                sl_opt.map_or(false, |sl| candle.low <= sl),
                tp_opt.map_or(false, |tp| candle.high >= tp),
            ),
            PositionType::Short => (
                sl_opt.map_or(false, |sl| candle.high >= sl),
                tp_opt.map_or(false, |tp| candle.low <= tp),
            ),
            PositionType::None => (false, false),
        };

        if is_sl_hit && is_tp_hit {
            // Nếu cả SL và TP cùng chạm trong 1 cây nến:
            // Ưu tiên cái nào gần giá Open hơn (giống logic ferro_trade::Broker)
            let sl = sl_opt.unwrap();
            let tp = tp_opt.unwrap();
            let dist_sl = (candle.open - sl).abs();
            let dist_tp = (candle.open - tp).abs();
            if dist_sl <= dist_tp {
                let is_tsl = match self.position_type {
                    PositionType::Long => sl > self.entry_price,
                    PositionType::Short => sl < self.entry_price,
                    PositionType::None => false,
                };
                let reason = if is_tsl { ExitReason::TrailingStopLoss } else { ExitReason::HardStopLoss };
                info!("⚡ [PAPER TRADER] {:?} dính {} @ {:.2} (gần Open hơn TP)", self.position_type, reason.as_str(), sl);
                self.close_position(sl, reason, candle.timestamp, 0, account_tx);
            } else {
                info!("🎯 [PAPER TRADER] {:?} chạm TakeProfit @ {:.2} (gần Open hơn SL)", self.position_type, tp);
                self.close_position(tp, ExitReason::TakeProfit, candle.timestamp, 0, account_tx);
            }
            return true;
        } else if is_sl_hit {
            let sl = sl_opt.unwrap();
            let is_tsl = match self.position_type {
                PositionType::Long => sl > self.entry_price,
                PositionType::Short => sl < self.entry_price,
                PositionType::None => false,
            };
            let reason = if is_tsl { ExitReason::TrailingStopLoss } else { ExitReason::HardStopLoss };
            info!("⚡ [PAPER TRADER] {:?} Position dính {} @ {:.2}", self.position_type, reason.as_str(), sl);
            self.close_position(sl, reason, candle.timestamp, 0, account_tx);
            return true;
        } else if is_tp_hit {
            let tp = tp_opt.unwrap();
            info!("🎯 [PAPER TRADER] {:?} Position chạm TakeProfit @ {:.2}", self.position_type, tp);
            self.close_position(tp, ExitReason::TakeProfit, candle.timestamp, 0, account_tx);
            return true;
        }

        false
    }

    /// Xử lý tín hiệu (Action) từ Strategy
    pub fn process_action(
        &mut self, 
        action: Action, 
        current_price: f64, 
        account_tx: Option<&tokio::sync::broadcast::Sender<AccountEvent>>
    ) {
        use ferro_trade::broker::policies::RiskPolicy;

        match action {
            Action::Buy { size, policy, tag, .. } => {
                if self.position_type != PositionType::Long {
                    let (initial_sl, initial_tp) = match policy {
                        Some(RiskPolicy::FixedPrice { sl_price, tp_price }) => (sl_price, tp_price),
                        _ => (None, None),
                    };
                    self.open_position(PositionType::Long, current_price, Some(size), initial_sl, initial_tp, tag, 0, 0, account_tx);
                }
            }
            Action::Sell { size, policy, tag, .. } => {
                if self.position_type != PositionType::Short {
                    let (initial_sl, initial_tp) = match policy {
                        Some(RiskPolicy::FixedPrice { sl_price, tp_price }) => (sl_price, tp_price),
                        _ => (None, None),
                    };
                    self.open_position(PositionType::Short, current_price, Some(size), initial_sl, initial_tp, tag, 0, 0, account_tx);
                }
            }
            Action::CloseLongs { .. } => {
                if self.position_type == PositionType::Long {
                    self.close_position(current_price, ExitReason::StrategySignalClose, 0, 0, account_tx);
                }
            }
            Action::CloseShorts { .. } => {
                if self.position_type == PositionType::Short {
                    self.close_position(current_price, ExitReason::StrategySignalClose, 0, 0, account_tx);
                }
            }
            Action::ClosePosition { .. } => {
                self.close_position(current_price, ExitReason::StrategySignalClose, 0, 0, account_tx);
            }
            Action::UpdateTradeSlTp { new_sl_price, new_tp_price, .. } => {
                if new_sl_price.is_some() || new_tp_price.is_some() || self.current_sl_price.is_some() || self.current_tp_price.is_some() {
                    let sl_str = new_sl_price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
                    let tp_str = new_tp_price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
                    info!("🛡️ [PAPER TRADER] Cập nhật SL/TP mới: SL = {}, TP = {}", sl_str, tp_str);
                    self.current_sl_price = new_sl_price;
                    self.current_tp_price = new_tp_price;
                    self.broadcast_account_events(account_tx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_trader_long() {
        let mut trader = PaperTrader::new("BTCUSDT".to_string(), 1000.0, 0.001);

        trader.process_action(Action::Buy { size: 1.0, limit: None, stop: None, policy: None, tag: None }, 100.0, None);
        assert_eq!(trader.position_type, PositionType::Long);
        assert_eq!(trader.entry_price, 100.0);
        assert_eq!(trader.position_size, 1000.0 / 100.0);
        assert_eq!(trader.equity, 999.0);

        assert_eq!(trader.get_unrealized_pnl(110.0), 100.0);
        assert_eq!(trader.get_total_equity(110.0), 1099.0);

        trader.process_action(Action::CloseLongs { portion: 1.0 }, 110.0, None);
        assert_eq!(trader.position_type, PositionType::None);
        assert_eq!(trader.equity, 1097.9);
    }

    #[test]
    fn test_paper_trader_trailing_sl() {
        let mut trader = PaperTrader::new("ETHUSDT".to_string(), 1000.0, 0.0);

        trader.process_action(Action::Buy { size: 1.0, limit: None, stop: None, policy: None, tag: None }, 2000.0, None);
        assert_eq!(trader.position_type, PositionType::Long);

        trader.process_action(Action::UpdateTradeSlTp { trade_id: 0, new_sl_price: Some(1950.0), new_tp_price: None }, 2000.0, None);
        assert_eq!(trader.current_sl_price, Some(1950.0));

        let candle = crate::models::models::Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp: 1600000000,
            open: 1990.0,
            high: 2010.0,
            low: 1940.0,
            close: 1960.0,
            volume: 10.0,
        };

        let sl_hit = trader.on_candle(&candle, None);
        assert!(sl_hit);
        assert_eq!(trader.position_type, PositionType::None);
        assert_eq!(trader.equity, 975.0);
    }

    #[test]
    fn test_paper_trader_cancel_sl() {
        let mut trader = PaperTrader::new("ETHUSDT".to_string(), 1000.0, 0.0);

        trader.process_action(Action::Buy { size: 1.0, limit: None, stop: None, policy: None, tag: None }, 2000.0, None);
        trader.process_action(Action::UpdateTradeSlTp { trade_id: 0, new_sl_price: Some(1950.0), new_tp_price: None }, 2000.0, None);
        assert_eq!(trader.current_sl_price, Some(1950.0));

        // Hủy SL bằng cách truyền None
        trader.process_action(Action::UpdateTradeSlTp { trade_id: 0, new_sl_price: None, new_tp_price: None }, 2000.0, None);
        assert_eq!(trader.current_sl_price, None);

        let candle = crate::models::models::Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp: 1600000000,
            open: 1990.0,
            high: 2010.0,
            low: 1940.0,
            close: 1960.0,
            volume: 10.0,
        };

        // Khi SL đã bị hủy thì candle.low <= 1950 không làm đóng vị thế
        let sl_hit = trader.on_candle(&candle, None);
        assert!(!sl_hit);
        assert_eq!(trader.position_type, PositionType::Long);
    }

    #[test]
    fn test_paper_trader_take_profit() {
        use ferro_trade::broker::policies::RiskPolicy;
        let mut trader = PaperTrader::new("ETHUSDT".to_string(), 1000.0, 0.0);

        let policy = Some(RiskPolicy::FixedPrice { sl_price: Some(1900.0), tp_price: Some(2100.0) });
        trader.process_action(Action::Buy { size: 1.0, limit: None, stop: None, policy, tag: None }, 2000.0, None);
        assert_eq!(trader.current_sl_price, Some(1900.0));
        assert_eq!(trader.current_tp_price, Some(2100.0));

        let candle = crate::models::models::Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp: 1600000000,
            open: 2050.0,
            high: 2120.0, // Chạm TP 2100
            low: 2040.0,
            close: 2110.0,
            volume: 10.0,
        };

        let tp_hit = trader.on_candle(&candle, None);
        assert!(tp_hit);
        assert_eq!(trader.position_type, PositionType::None);
        assert_eq!(trader.equity, 1050.0); // (2100 - 2000) * 0.5 coin = +50$
    }
}

