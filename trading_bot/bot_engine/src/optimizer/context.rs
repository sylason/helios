use ferro_trade::backtest::{InitContext, TickContext};
// Removed Candle

/// LiveInitContext cung cấp một giao diện tương thích với InitContext của ferro_trade
/// nhưng được tối ưu hóa cho môi trường Live.
pub struct LiveInitContext<'a> {
    pub inner: &'a mut InitContext<'a>,
}

/// LiveTickContext cung cấp một giao diện tương thích với TickContext của ferro_trade
/// nhưng có thể mở rộng các tính năng live như logging hoặc phát tín hiệu ra UI.
pub struct LiveTickContext<'a> {
    pub inner: &'a TickContext<'a>,
}

impl<'a> LiveInitContext<'a> {
    pub fn new(inner: &'a mut InitContext<'a>) -> Self {
        Self { inner }
    }
}

impl<'a> LiveTickContext<'a> {
    pub fn new(inner: &'a TickContext<'a>) -> Self {
        Self { inner }
    }
}
