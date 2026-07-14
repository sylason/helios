use serde::{Deserialize, Serialize};

/// Thông tin số dư tài khoản
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountState {
    pub wallet_balance: f64,
    pub available_balance: f64,
}

/// Thông tin vị thế của một tài sản
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionState {
    pub symbol: String,
    pub position_amt: f64,
    pub entry_price: f64,
    pub unrealized_profit: f64,
    pub leverage: f64,
    pub isolated_margin: f64,
}

impl PositionState {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            position_amt: 0.0,
            entry_price: 0.0,
            unrealized_profit: 0.0,
            leverage: 1.0,
            isolated_margin: 0.0,
        }
    }
}

/// Các sự kiện cập nhật tài khoản và vị thế truyền qua EventBus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountEvent {
    AccountUpdate {
        wallet_balance: f64,
        available_balance: f64,
    },
    PositionUpdate {
        symbol: String,
        position_amt: f64,
        entry_price: f64,
        unrealized_profit: f64,
    }
}

/// Hàm trợ giúp kiểm tra số dư an toàn trước khi mở lệnh
pub fn can_open_position(available_margin: f64, required_margin: f64) -> bool {
    // Luôn giữ lại 5% buffer an toàn (không dùng hết 100% tiền)
    let safe_margin = available_margin * 0.95;
    safe_margin >= required_margin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_open_position() {
        assert_eq!(can_open_position(100.0, 50.0), true);
        
        // Cần 96 USDT, nhưng do có 5% buffer nên 100 USDT available chỉ an toàn tới 95 USDT.
        assert_eq!(can_open_position(100.0, 96.0), false);
        
        // Vừa đúng 95
        assert_eq!(can_open_position(100.0, 95.0), true);
    }
}
