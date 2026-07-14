use tracing::{info, debug};
use ferro_trade::broker::actions::Action; // Dùng enum Action từ ferro_trade

#[derive(Debug, Clone, PartialEq)]
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
    pub leverage: f64, // Không dùng đòn bẩy tạm thời, mặc định là 1.0 (hoặc margin rate)
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
            leverage: 1.0, // Tạm thời trade spot hoặc future leverage 1x
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

        // Bỏ qua funding fee tạm thời
        price_diff * self.position_size
    }

    /// Tính toán vốn hiện tại (Bao gồm Unrealized PnL)
    pub fn get_total_equity(&self, current_price: f64) -> f64 {
        self.equity + self.get_unrealized_pnl(current_price)
    }

    /// Đóng vị thế hiện tại
    fn close_position(&mut self, current_price: f64) {
        if self.position_type == PositionType::None {
            return;
        }

        let pnl = self.get_unrealized_pnl(current_price);
        let exit_value = current_price * self.position_size;
        let commission = exit_value * self.commission_rate;
        let old_position = self.position_type.clone();

        // Cập nhật Equity
        self.equity += pnl - commission;
        
        info!(
            "🔴 [PAPER TRADER] CLOSE {}: Exit Price = {:.2}, PnL = {:.2}, New Equity = {:.2}",
            match old_position {
                PositionType::Long => "LONG",
                PositionType::Short => "SHORT",
                PositionType::None => "NONE",
            },
            current_price,
            pnl,
            self.equity
        );

        // Reset trạng thái
        self.position_type = PositionType::None;
        self.entry_price = 0.0;
        self.position_size = 0.0;
    }

    /// Mở vị thế mới (Sử dụng 100% Equity)
    fn open_position(&mut self, pos_type: PositionType, current_price: f64) {
        if self.position_type != PositionType::None {
            self.close_position(current_price);
        }

        // Tính toán size dựa trên 100% equity
        // Value = Equity * Leverage
        let trade_value = self.equity * self.leverage;
        let commission = trade_value * self.commission_rate;
        
        // Trừ phí mở lệnh
        self.equity -= commission;
        
        // Tính size
        self.position_size = trade_value / current_price;
        self.entry_price = current_price;
        self.position_type = pos_type.clone();

        info!("📈 [PaperTrader] MỞ VỊ THẾ {:?} @ {:.2}. Size: {:.4}. Equity còn: {:.2}", 
            self.position_type, self.entry_price, self.position_size, self.equity);
    }

    /// Xử lý tín hiệu (Action) từ Strategy
    pub fn process_action(&mut self, action: Action, current_price: f64) {
        match action {
            Action::Buy { .. } => {
                if self.position_type != PositionType::Long {
                    self.open_position(PositionType::Long, current_price);
                }
            }
            Action::Sell { .. } => {
                if self.position_type != PositionType::Short {
                    self.open_position(PositionType::Short, current_price);
                }
            }
            Action::CloseLongs { .. } => {
                if self.position_type == PositionType::Long {
                    self.close_position(current_price);
                }
            }
            Action::CloseShorts { .. } => {
                if self.position_type == PositionType::Short {
                    self.close_position(current_price);
                }
            }
            Action::ClosePosition { .. } => {
                self.close_position(current_price);
            }
            Action::UpdateTradeSlTp { .. } => {} // Không xử lý SL/TP tạm thời
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_trader_long() {
        let mut trader = PaperTrader::new("BTCUSDT".to_string(), 1000.0, 0.001); // 0.1% fee

        // Enter Long tại giá 100
        trader.process_action(Action::Buy { size: 1.0, limit: None, stop: None, policy: None, tag: None }, 100.0);
        assert_eq!(trader.position_type, PositionType::Long);
        assert_eq!(trader.entry_price, 100.0);
        
        // Phí mở: 1000 * 0.001 = 1.0 => Equity = 999.0
        // Size = 1000 / 100 = 10.0
        assert_eq!(trader.position_size, 10.0);
        assert_eq!(trader.equity, 999.0);

        // Giá tăng lên 110 => Lãi = (110 - 100) * 10 = 100
        assert_eq!(trader.get_unrealized_pnl(110.0), 100.0);
        assert_eq!(trader.get_total_equity(110.0), 1099.0);

        // Exit Long tại 110
        trader.process_action(Action::CloseLongs { portion: 1.0 }, 110.0);
        // Phí đóng: 110 * 10 * 0.001 = 1.1
        // Equity mới = 999.0 + 100 - 1.1 = 1097.9
        assert_eq!(trader.position_type, PositionType::None);
        assert_eq!(trader.equity, 1097.9);
    }

    #[test]
    fn test_paper_trader_short() {
        let mut trader = PaperTrader::new("BTCUSDT".to_string(), 1000.0, 0.0); // 0% fee

        // Enter Short tại 100
        trader.process_action(Action::Sell { size: 1.0, limit: None, stop: None, policy: None, tag: None }, 100.0);
        assert_eq!(trader.position_type, PositionType::Short);
        
        // Giá giảm xuống 90 => Lãi = (100 - 90) * 10 = 100
        assert_eq!(trader.get_unrealized_pnl(90.0), 100.0);
        
        trader.process_action(Action::CloseShorts { portion: 1.0 }, 90.0);
        assert_eq!(trader.equity, 1100.0);
    }
}
