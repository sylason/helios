use crate::models::error::BotError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarketFilter {
    pub tick_size: f64,
    pub step_size: f64,
    pub min_qty: f64,
    pub max_qty: f64,
    pub min_notional: f64,
}

impl Default for MarketFilter {
    fn default() -> Self {
        Self {
            tick_size: 0.1,
            step_size: 0.001,
            min_qty: 0.001,
            max_qty: 1000000.0,
            min_notional: 5.0,
        }
    }
}

impl MarketFilter {
    /// Làm tròn giá theo quy định tick_size của sàn.
    pub fn round_price(&self, price: f64) -> f64 {
        if self.tick_size <= 0.0 {
            return price;
        }
        // Để tránh sai số f64 sau khi nhân chia, ta làm tròn đến số thập phân phù hợp
        let rounded = (price / self.tick_size).round() * self.tick_size;
        
        // Cắt bớt phần sai số nhỏ của f64 (ví dụ 1.2340000000000002 -> 1.234)
        // Tìm số chữ số thập phân của tick_size
        let decimals = Self::count_decimals(self.tick_size);
        let factor = 10_f64.powi(decimals as i32);
        (rounded * factor).round() / factor
    }

    /// Làm tròn số lượng theo quy định step_size của sàn.
    pub fn round_qty(&self, qty: f64) -> f64 {
        if self.step_size <= 0.0 {
            return qty;
        }
        let rounded = (qty / self.step_size).round() * self.step_size;
        
        let decimals = Self::count_decimals(self.step_size);
        let factor = 10_f64.powi(decimals as i32);
        (rounded * factor).round() / factor
    }

    /// Kiểm tra lệnh có vi phạm luật của sàn không
    pub fn validate_order(&self, price: f64, qty: f64) -> Result<(), BotError> {
        let rounded_qty = self.round_qty(qty);
        
        if rounded_qty < self.min_qty {
            return Err(BotError::Validation(format!(
                "Quantity {} is less than min_qty {}",
                rounded_qty, self.min_qty
            )));
        }
        if rounded_qty > self.max_qty {
            return Err(BotError::Validation(format!(
                "Quantity {} is greater than max_qty {}",
                rounded_qty, self.max_qty
            )));
        }
        
        let rounded_price = self.round_price(price);
        let notional = rounded_price * rounded_qty;
        if notional < self.min_notional {
            return Err(BotError::Validation(format!(
                "Notional {} is less than min_notional {}",
                notional, self.min_notional
            )));
        }

        Ok(())
    }

    /// Đếm số lượng chữ số thập phân, ví dụ 0.001 -> 3, 0.00000001 -> 8
    pub fn count_decimals(val: f64) -> u32 {
        if val >= 1.0 || val <= 0.0 {
            return 0;
        }
        let s = format!("{:.12}", val); // Đảm bảo hỗ trợ đến 12 chữ số thập phân cho micro-ticks/memecoins
        let trimmed = s.trim_end_matches('0');
        if let Some(pos) = trimmed.find('.') {
            (trimmed.len() - pos - 1) as u32
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_decimals() {
        assert_eq!(MarketFilter::count_decimals(0.1), 1);
        assert_eq!(MarketFilter::count_decimals(0.01), 2);
        assert_eq!(MarketFilter::count_decimals(0.001), 3);
        assert_eq!(MarketFilter::count_decimals(0.0001), 4);
        assert_eq!(MarketFilter::count_decimals(0.00001), 5);
        assert_eq!(MarketFilter::count_decimals(0.00000001), 8);
        assert_eq!(MarketFilter::count_decimals(1.0), 0);
        assert_eq!(MarketFilter::count_decimals(10.0), 0);
    }


    #[test]
    fn test_round_price() {
        let filter = MarketFilter {
            tick_size: 0.1,
            ..Default::default()
        };
        // 1.234 -> 1.2
        assert_eq!(filter.round_price(1.234), 1.2);
        // 1.25 -> 1.3
        assert_eq!(filter.round_price(1.25), 1.3);
        
        let filter_small = MarketFilter {
            tick_size: 0.001,
            ..Default::default()
        };
        assert_eq!(filter_small.round_price(1.2345), 1.235);
        assert_eq!(filter_small.round_price(1.2344), 1.234);
    }

    #[test]
    fn test_round_qty() {
        let filter = MarketFilter {
            step_size: 0.001,
            ..Default::default()
        };
        assert_eq!(filter.round_qty(0.1234), 0.123);
        assert_eq!(filter.round_qty(0.1235), 0.124);
        
        let filter_large = MarketFilter {
            step_size: 1.0,
            ..Default::default()
        };
        assert_eq!(filter_large.round_qty(1.2), 1.0);
        assert_eq!(filter_large.round_qty(1.5), 2.0);
    }

    #[test]
    fn test_validate_order() {
        let filter = MarketFilter {
            tick_size: 0.1,
            step_size: 0.001,
            min_qty: 0.01,
            max_qty: 100.0,
            min_notional: 5.0,
        };

        // Valid order: price 10.0, qty 0.6 -> notional = 6.0 > 5.0
        assert!(filter.validate_order(10.0, 0.6).is_ok());

        // Invalid: qty < min_qty
        assert!(filter.validate_order(10.0, 0.005).is_err());

        // Invalid: qty > max_qty
        assert!(filter.validate_order(10.0, 101.0).is_err());

        // Invalid: notional < min_notional
        // price = 10.0, qty = 0.1 -> notional = 1.0 < 5.0
        assert!(filter.validate_order(10.0, 0.1).is_err());
    }
}
