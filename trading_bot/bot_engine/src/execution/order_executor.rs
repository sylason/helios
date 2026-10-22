use tracing::{info, warn, error};
use rust_decimal::prelude::FromPrimitive;

use binance_sdk::derivatives_trading_usds_futures::websocket_api::{
    WebsocketApi,
    NewAlgoOrderParams as WsNewAlgoOrderParams,
    NewAlgoOrderAlgoTypeEnum as WsNewAlgoOrderAlgoTypeEnum,
    NewAlgoOrderSideEnum as WsNewAlgoOrderSideEnum,
    NewAlgoOrderTypeEnum as WsNewAlgoOrderTypeEnum,
    NewAlgoOrderWorkingTypeEnum as WsNewAlgoOrderWorkingTypeEnum,
    NewAlgoOrderClosePositionEnum as WsNewAlgoOrderClosePositionEnum,
    CancelAlgoOrderParams as WsCancelAlgoOrderParams,
    NewOrderParams as WsNewOrderParams,
    NewOrderSideEnum as WsNewOrderSideEnum,
    NewOrderTypeEnum as WsNewOrderTypeEnum,
};

use binance_sdk::derivatives_trading_usds_futures::rest_api::{
    RestApi,
    CancelAllAlgoOpenOrdersParams,
    CurrentAllAlgoOpenOrdersParams,
    NewOrderParams as RestNewOrderParams,
    NewOrderSideEnum as RestNewOrderSideEnum,
    NewOrderTypeEnum as RestNewOrderTypeEnum,
};
use binance_sdk::config::ConfigurationRestApi;
use chrono::Utc;

use crate::models::market_info::MarketFilter;
use crate::error::BotError;
use ferro_trade::broker::actions::Action;
use ferro_trade::broker::policies::RiskPolicy;
use crate::data_engine::sqlite_impl::SqliteStore;
use crate::models::weight_monitor::WeightMonitor;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct OrderExecutor {
    #[allow(dead_code)]
    rest_client: RestApi,
    pub ws_api: Arc<RwLock<Option<WebsocketApi>>>,
    pub active_algo_ids: Arc<RwLock<HashMap<String, i64>>>,
    market_filter: MarketFilter,
    db_store: Arc<SqliteStore>,
    #[allow(dead_code)]
    monitor: Arc<WeightMonitor>,
    pub active_sl_prices: Arc<RwLock<HashMap<String, f64>>>,
}

impl OrderExecutor {
    pub fn new(api_key: String, api_secret: String, use_testnet: bool, market_filter: MarketFilter, db_store: Arc<SqliteStore>, monitor: Arc<WeightMonitor>) -> Self {
        let rest_conf = ConfigurationRestApi::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .build()
            .unwrap();
            
        let rest_client = if use_testnet {
            binance_sdk::derivatives_trading_usds_futures::DerivativesTradingUsdsFuturesRestApi::testnet(rest_conf)
        } else {
            binance_sdk::derivatives_trading_usds_futures::DerivativesTradingUsdsFuturesRestApi::production(rest_conf)
        };

        // 1. Phục hồi active algo orders từ SQLite nếu có (Persistence Guard sau khi restart bot)
        let (initial_algo_ids, initial_sl_prices) = match db_store.get_all_active_algo_orders() {
            Ok(list) => {
                let mut ids = HashMap::new();
                let mut sls = HashMap::new();
                for (sym, algo_id, sl_price) in list {
                    ids.insert(sym.clone(), algo_id);
                    if let Some(sl) = sl_price {
                        sls.insert(sym, sl);
                    }
                }
                (ids, sls)
            }
            Err(e) => {
                tracing::error!("❌ Không thể nạp active_algo_orders từ SQLite: {:?}", e);
                (HashMap::new(), HashMap::new())
            }
        };

        Self {
            rest_client,
            ws_api: Arc::new(RwLock::new(None)),
            active_algo_ids: Arc::new(RwLock::new(initial_algo_ids)),
            market_filter,
            db_store,
            monitor,
            active_sl_prices: Arc::new(RwLock::new(initial_sl_prices)),
        }
    }

    /// Thiết lập hoặc cập nhật kết nối WebSocket API
    pub async fn set_ws_api(&self, client: Option<WebsocketApi>) {
        *self.ws_api.write().await = client;
    }

    /// Lấy algo_id đang hoạt động của một symbol
    pub async fn get_active_algo_id(&self, symbol: &str) -> Option<i64> {
        self.active_algo_ids.read().await.get(symbol).copied()
    }

    /// Lưu algo_id mới của một symbol (vào cả RAM và SQLite)
    pub async fn set_active_algo_id(&self, symbol: &str, algo_id: i64) {
        self.active_algo_ids.write().await.insert(symbol.to_string(), algo_id);
        let sl = self.active_sl_prices.read().await.get(symbol).copied();
        if let Err(e) = self.db_store.save_active_algo_order(symbol, algo_id, sl) {
            error!("❌ [{}] Lỗi lưu active_algo_id vào SQLite: {:?}", symbol, e);
        }
    }

    /// Xóa algo_id của một symbol (khỏi cả RAM và SQLite)
    pub async fn clear_active_algo_id(&self, symbol: &str) {
        self.active_algo_ids.write().await.remove(symbol);
        if let Err(e) = self.db_store.remove_active_algo_order(symbol) {
            error!("❌ [{}] Lỗi xóa active_algo_id khỏi SQLite: {:?}", symbol, e);
        }
    }

    /// Chờ WebSocket API sẵn sàng (In-Flight Retry Await cho Fast Reconnect)
    pub async fn wait_ws_ready(&self, max_wait_ms: u64) -> Option<WebsocketApi> {
        let start = tokio::time::Instant::now();
        let timeout = tokio::time::Duration::from_millis(max_wait_ms);
        while start.elapsed() < timeout {
            if let Some(ws) = self.ws_api.read().await.as_ref().cloned() {
                return Some(ws);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        None
    }

    /// Lấy mức SL đang kích hoạt trên sàn của một symbol
    pub async fn get_active_sl(&self, symbol: &str) -> Option<f64> {
        self.active_sl_prices.read().await.get(symbol).copied()
    }

    /// Đặt mức SL đang kích hoạt trên sàn của một symbol (vào cả RAM và SQLite)
    pub async fn set_active_sl(&self, symbol: &str, sl: f64) {
        self.active_sl_prices.write().await.insert(symbol.to_string(), sl);
        if let Some(algo_id) = self.active_algo_ids.read().await.get(symbol).copied() {
            if let Err(e) = self.db_store.save_active_algo_order(symbol, algo_id, Some(sl)) {
                error!("❌ [{}] Lỗi lưu active_sl vào SQLite: {:?}", symbol, e);
            }
        }
    }

    /// Xóa mức SL đang kích hoạt trên sàn của một symbol
    pub async fn clear_active_sl(&self, symbol: &str) {
        self.active_sl_prices.write().await.remove(symbol);
    }

    /// [TODO-STAB-06-RECONCILE] Truy vấn các lệnh Algo đang mở trên sàn Binance để đồng bộ vào RAM & DB
    /// Sử dụng khi bot khởi động lại (Adopt & Protect) hoặc khi phát hiện RAM bị mất algo_id
    pub async fn sync_active_algo_orders(&self, symbol: &str) -> Result<Option<i64>, BotError> {
        let params = CurrentAllAlgoOpenOrdersParams::builder()
            .symbol(symbol.to_string())
            .build()
            .map_err(|e| BotError::Internal(e.to_string()))?;

        self.monitor.wait_if_needed(1).await;
        match self.rest_client.current_all_algo_open_orders(params).await {
            Ok(resp) => {
                let orders = resp.data().await.map_err(|e| BotError::from(e))?;
                for o in orders {
                    if let Some(algo_id) = o.algo_id {
                        let trigger_price: Option<f64> = o.trigger_price.as_ref().and_then(|p| p.parse().ok());
                        info!("🔍 [{}] Đã đồng bộ Algo SL từ sàn Binance: algo_id={}, trigger_price={:?}", symbol, algo_id, trigger_price);
                        self.set_active_algo_id(symbol, algo_id).await;
                        if let Some(price) = trigger_price {
                            self.set_active_sl(symbol, price).await;
                        }
                        return Ok(Some(algo_id));
                    }
                }
                self.clear_active_algo_id(symbol).await;
                self.clear_active_sl(symbol).await;
                Ok(None)
            }
            Err(e) => {
                warn!("⚠️ [{}] Không thể query open algo orders từ sàn Binance: {:?}", symbol, e);
                Err(BotError::from(e))
            }
        }
    }

    /// 🛡️ MONOTONIC RATCHET GUARD & NAKED POSITION GUARD (EX-11):
    /// Kiểm tra xem mức SL mới có hợp lệ và chặt hơn mức cũ hay không.
    /// Trả về Some(valid_new_sl) nếu cần cập nhật lên sàn Binance, hoặc None nếu bị chặn/không đổi.
    pub fn evaluate_ratchet_sl(
        is_long: bool,
        old_sl: Option<f64>,
        new_sl_price: Option<f64>,
        has_position: bool,
    ) -> Option<f64> {
        // 1. Chống xóa sổ SL (Naked Position Guard): Có vị thế và đã có SL -> CẤM xóa bằng None!
        if has_position && old_sl.is_some() && new_sl_price.is_none() {
            warn!(
                "🛡️ [NAKED POSITION GUARD BLOCKED] Attempted to wipe out SL {:.2} with None! Guarded & kept existing SL.",
                old_sl.unwrap()
            );
            return None;
        }

        match (is_long, old_sl, new_sl_price) {
            // Long: new_sl không được thấp hơn old_sl - 1e-4
            (true, Some(old), Some(new)) => {
                if new < old - 1e-4 {
                    warn!(
                        "🛡️ [MONOTONIC RATCHET BLOCKED] Long SL retracted from {:.2} to {:.2}. Guarded & kept tighter SL: {:.2}",
                        old, new, old
                    );
                    None
                } else if (new - old).abs() <= 1e-4 {
                    // Mức giá không đổi đáng kể -> No-op, tiết kiệm 100% API Weight
                    None
                } else {
                    Some(new)
                }
            }
            // Short: new_sl không được cao hơn old_sl + 1e-4
            (false, Some(old), Some(new)) => {
                if new > old + 1e-4 {
                    warn!(
                        "🛡️ [MONOTONIC RATCHET BLOCKED] Short SL retracted from {:.2} to {:.2}. Guarded & kept tighter SL: {:.2}",
                        old, new, old
                    );
                    None
                } else if (new - old).abs() <= 1e-4 {
                    // Mức giá không đổi đáng kể -> No-op, tiết kiệm 100% API Weight
                    None
                } else {
                    Some(new)
                }
            }
            // Chưa có SL cũ mà có SL mới -> Cho phép đặt
            (_, None, Some(new)) => Some(new),
            _ => None,
        }
    }

    /// [TODO-AUDIT-NEW-10] Trích xuất giá Stop Loss từ Action
    /// Ưu tiên trích xuất từ RiskPolicy::FixedPrice { sl_price, .. } (chuẩn Helios V3),
    /// fallback sang trường stop trực tiếp hoặc new_sl_price của UpdateTradeSlTp.
    pub fn extract_sl_price(action: &Action) -> Option<f64> {
        match action {
            Action::Buy { stop, policy, .. } | Action::Sell { stop, policy, .. } => {
                if let Some(RiskPolicy::FixedPrice { sl_price: Some(sl), .. }) = policy {
                    Some(*sl)
                } else {
                    *stop
                }
            }
            Action::UpdateTradeSlTp { new_sl_price, .. } => *new_sl_price,
            _ => None,
        }
    }

    /// Tính toán khối lượng giao dịch hợp lệ
    pub fn calculate_order_size(&self, risk_usdt: f64, current_price: f64, leverage: f64) -> Result<f64, BotError> {
        if current_price <= 0.0 {
            return Err(BotError::Internal("Giá thị trường không hợp lệ".to_string()));
        }

        // Tính số lượng Coin (Qty)
        let raw_qty = (risk_usdt * leverage) / current_price;
        
        // Làm tròn theo Market Filter step_size
        let qty = self.market_filter.round_qty(raw_qty);

        // Kiểm tra Min Notional (Thường là 5 USDT trên Binance Futures)
        let notional = qty * current_price;
        if notional < 5.0 {
            return Err(BotError::MarketFilterError(format!("Giá trị lệnh ({:.2} USDT) nhỏ hơn Min Notional (5 USDT)", notional)));
        }

        Ok(qty)
    }

    /// Làm tròn giá thành Decimal chuẩn theo tick_size của sàn
    fn format_price_decimal(&self, price: f64) -> rust_decimal::Decimal {
        let rounded = self.market_filter.round_price(price);
        let decimals = MarketFilter::count_decimals(self.market_filter.tick_size);
        rust_decimal::Decimal::from_f64(rounded).unwrap().round_dp(decimals as u32)
    }

    /// Làm tròn khối lượng thành Decimal chuẩn theo step_size của sàn
    fn format_qty_decimal(&self, qty: f64) -> rust_decimal::Decimal {
        let rounded = self.market_filter.round_qty(qty);
        let decimals = MarketFilter::count_decimals(self.market_filter.step_size);
        rust_decimal::Decimal::from_f64(rounded).unwrap().round_dp(decimals as u32)
    }

    /// Thực thi tín hiệu giao dịch
    pub async fn execute_action(
        &self, 
        symbol: &str, 
        action: Action, 
        current_price: f64, 
        available_balance: f64,
        position_amt: f64,
        leverage: f64
    ) -> Result<(), BotError> {
        
        let mut sl_price_to_set = None;

        // Xử lý riêng Action::UpdateTradeSlTp (Trailing Stop Loss)
        if let Action::UpdateTradeSlTp { new_sl_price, .. } = &action {
            let old_sl = self.get_active_sl(symbol).await;
            let has_position = position_amt.abs() > 0.0;
            let is_long = position_amt > 0.0;

            // 🛡️ Đánh giá qua Monotonic Ratchet Guard & Naked Position Guard
            let sl_to_place = match Self::evaluate_ratchet_sl(is_long, old_sl, *new_sl_price, has_position) {
                Some(sl) => sl,
                None => {
                    // Bị Ratchet chặn hoặc mức giá không đổi -> No-op, hoàn tất an toàn
                    return Ok(());
                }
            };

            // 1. Chờ kết nối WebSocket API sẵn sàng (tối đa 1500ms nếu đang Fast Reconnect)
            let ws = match self.wait_ws_ready(1500).await {
                Some(w) => w,
                None => {
                    error!("❌ [{}] Không thể cập nhật Trailing SL: WebSocket API không kết nối sau 1500ms!", symbol);
                    return Err(BotError::Internal("WebSocket Trade API unavailable".to_string()));
                }
            };

            // 2. Hủy lệnh SL cũ qua WebSocket API (nếu có algo_id)
            let mut old_algo_id_opt = self.get_active_algo_id(symbol).await;

            // 🛡️ RECOVERY GUARD: Nếu RAM chưa có algo_id (sau khi bot restart), thử đồng bộ từ sàn Binance
            if old_algo_id_opt.is_none() {
                info!("🔄 [{}] RAM chưa có algo_id, đang đồng bộ Open Algo Orders từ sàn Binance...", symbol);
                if let Ok(Some(synced_id)) = self.sync_active_algo_orders(symbol).await {
                    old_algo_id_opt = Some(synced_id);
                }
            }

            if let Some(old_algo_id) = old_algo_id_opt {
                let p_cancel = WsCancelAlgoOrderParams::builder().algo_id(old_algo_id).build().unwrap();
                match ws.cancel_algo_order(p_cancel).await {
                    Ok(_) => {
                        info!("🧹 [{}] Đã hủy Algo SL cũ (ID: {}) qua WebSocket API.", symbol, old_algo_id);
                        self.clear_active_algo_id(symbol).await;
                    }
                    Err(e) => {
                        warn!("⚠️ [{}] Lỗi hủy Algo SL cũ (ID: {}) qua WS: {}. Tiếp tục đặt SL mới.", symbol, old_algo_id, e);
                    }
                }
            } else {
                // 🛡️ NẾU VẪN KHÔNG CÓ algo_id: Dọn sạch mọi lệnh Algo mồ côi qua REST để phòng ngừa xung đột closePosition
                let p_cancel = CancelAllAlgoOpenOrdersParams::builder(symbol.to_string()).build().unwrap();
                self.monitor.wait_if_needed(1).await;
                let _ = self.rest_client.cancel_all_algo_open_orders(p_cancel).await;
                info!("🧹 [{}] Đã dọn sạch Algo Orders mồ côi qua REST trước khi đặt SL mới.", symbol);
            }

            // 3. Đặt lệnh Trailing SL mới siêu tốc qua WebSocket API
            let sl_side = if position_amt > 0.0 { WsNewAlgoOrderSideEnum::Sell } else { WsNewAlgoOrderSideEnum::Buy };
            let rounded_sl = self.format_price_decimal(sl_to_place);
            let p_sl = WsNewAlgoOrderParams::builder(
                WsNewAlgoOrderAlgoTypeEnum::Conditional,
                symbol.to_string(),
                sl_side,
                WsNewAlgoOrderTypeEnum::StopMarket
            )
            .trigger_price(rounded_sl)
            .close_position(Some(WsNewAlgoOrderClosePositionEnum::True))
            .working_type(WsNewAlgoOrderWorkingTypeEnum::MarkPrice)
            .build().unwrap();

            // Thử lại tối đa 3 lần với WebSocket API
            let mut success = false;
            for attempt in 1..=3 {
                match ws.new_algo_order(p_sl.clone()).await {
                    Ok(resp) => {
                        let old_sl_str = old_sl.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "None".to_string());
                        info!("🛡️ [TRAILING SL UPDATE WS] Symbol: {} | SL Ratchet: {} -> {:.2}", symbol, old_sl_str, sl_to_place);
                        self.set_active_sl(symbol, sl_to_place).await;
                        if let Ok(data) = resp.data() {
                            if let Some(new_id) = data.algo_id {
                                self.set_active_algo_id(symbol, new_id).await;
                                info!("📌 [{}] Đã lưu new algo_id = {} qua WebSocket API", symbol, new_id);
                            }
                        }
                        success = true;
                        break;
                    }
                    Err(e) => {
                        warn!("⚠️ [{}] Thử lần {}/3 đặt Trailing SL qua WS thất bại ({}), thử lại sau 20ms...", symbol, attempt, e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                    }
                }
            }

            if !success {
                error!("❌ [{}] Đã thử 3 lần nhưng không thể đặt Trailing SL mới qua WebSocket tại: {}", symbol, rounded_sl);
                return Err(BotError::Internal(format!("Failed to place Trailing SL at {}", rounded_sl)));
            }
            return Ok(());
        }

        let initial_sl = Self::extract_sl_price(&action);
        let (side, is_close_all, risk_percentage) = match action {
            Action::Buy { size, policy, .. } => {
                // ⚠️ SENSOR PROBE: Bắt thiếu sót đặt lệnh Take Profit trên Binance Live
                if let Some(RiskPolicy::FixedPrice { tp_price: Some(tp), .. }) = &policy {
                    warn!(
                        "⚠️ [LIVE ENGINE GAP: TP NOT PLACED ON EXCHANGE] Symbol: {} | Strategy emitted Take Profit at {:.2}, \
                        but OrderExecutor ONLY places Algo STOP_MARKET for SL. NO Take Profit order submitted to Binance Futures!",
                        symbol, tp
                    );
                }
                sl_price_to_set = initial_sl;
                (WsNewOrderSideEnum::Buy, false, size)
            },
            Action::Sell { size, policy, .. } => {
                // ⚠️ SENSOR PROBE: Bắt thiếu sót đặt lệnh Take Profit trên Binance Live
                if let Some(RiskPolicy::FixedPrice { tp_price: Some(tp), .. }) = &policy {
                    warn!(
                        "⚠️ [LIVE ENGINE GAP: TP NOT PLACED ON EXCHANGE] Symbol: {} | Strategy emitted Take Profit at {:.2}, \
                        but OrderExecutor ONLY places Algo STOP_MARKET for SL. NO Take Profit order submitted to Binance Futures!",
                        symbol, tp
                    );
                }
                sl_price_to_set = initial_sl;
                (WsNewOrderSideEnum::Sell, false, size)
            },
            Action::ClosePosition { .. } | Action::CloseLongs { .. } | Action::CloseShorts { .. } => {
                // Đảo ngược vị thế hiện tại để đóng lệnh
                if position_amt > 0.0 {
                    (WsNewOrderSideEnum::Sell, true, 0.0)
                } else if position_amt < 0.0 {
                    (WsNewOrderSideEnum::Buy, true, 0.0)
                } else {
                    info!("ℹ️ [{}] Nhận tín hiệu Close nhưng đang không có vị thế mở.", symbol);
                    return Ok(());
                }
            },
            _ => {
                info!("ℹ️ [{}] Bỏ qua Action không hỗ trợ: {:?}", symbol, action);
                return Ok(());
            }
        };

        let qty = if is_close_all {
            position_amt.abs()
        } else {
            let risk_usdt = available_balance * risk_percentage;
            let target_qty = self.calculate_order_size(risk_usdt, current_price, leverage)?;
            
            // Binance One-way Mode logic: to reverse a position, we must add the current position size
            match side {
                WsNewOrderSideEnum::Buy => {
                    if position_amt < 0.0 {
                        // Đang Short, muốn Buy -> Phải đóng Short trước rồi mới mở Long
                        target_qty + position_amt.abs()
                    } else {
                        target_qty
                    }
                }
                WsNewOrderSideEnum::Sell => {
                    if position_amt > 0.0 {
                        // Đang Long, muốn Sell -> Phải đóng Long trước rồi mới mở Short
                        target_qty + position_amt
                    } else {
                        target_qty
                    }
                }
            }
        };

        info!("🚀 [{}] Chuẩn bị gửi lệnh: {:?} Qty: {:.4} @ Market (WebSocket API)", symbol, side, qty);

        let dec_qty = self.format_qty_decimal(qty);

        let params = WsNewOrderParams::builder(
            symbol.to_string(),
            side.clone(),
            WsNewOrderTypeEnum::Market
        )
        .quantity(dec_qty)
        .build().map_err(|e| BotError::Internal(e.to_string()))?;

        let ws = match self.wait_ws_ready(1500).await {
            Some(w) => w,
            None => {
                error!("❌ [{}] Lỗi gửi Market Order: WebSocket API không kết nối sau 1500ms!", symbol);
                return Err(BotError::Internal("WebSocket Trade API unavailable".to_string()));
            }
        };

        match ws.new_order(params).await {
            Ok(res) => {
                let data = res.data().map_err(|e| BotError::Internal(e.to_string()))?;
                info!("✅ [{}] Khớp lệnh thành công qua WebSocket API! OrderID: {:?}", symbol, data.order_id);
                
                // Ghi log vào SQLite TradeHistory
                let order_id_str = data.order_id.map_or("UNKNOWN".to_string(), |id| id.to_string());
                let side_str = match side {
                    WsNewOrderSideEnum::Buy => "BUY",
                    WsNewOrderSideEnum::Sell => "SELL",
                };
                let current_time = Utc::now().timestamp_millis();
                
                if let Err(e) = self.db_store.save_trade_history(
                    &order_id_str,
                    symbol,
                    side_str,
                    current_price,
                    qty,
                    current_time
                ) {
                    error!("❌ [{}] Lỗi ghi log giao dịch vào SQLite: {:?}", symbol, e);
                }

                // Nếu Strategy có báo StopLoss ngay lúc vào lệnh, ta đặt luôn Algo Order qua WebSocket
                if let Some(sl) = sl_price_to_set {
                    let sl_side = match side {
                        WsNewOrderSideEnum::Buy => WsNewAlgoOrderSideEnum::Sell,
                        WsNewOrderSideEnum::Sell => WsNewAlgoOrderSideEnum::Buy,
                    };
                    let rounded_sl = self.format_price_decimal(sl);
                    let p_sl = WsNewAlgoOrderParams::builder(
                        WsNewAlgoOrderAlgoTypeEnum::Conditional,
                        symbol.to_string(),
                        sl_side,
                        WsNewAlgoOrderTypeEnum::StopMarket
                    )
                    .trigger_price(rounded_sl)
                    .close_position(Some(WsNewAlgoOrderClosePositionEnum::True))
                    .working_type(WsNewAlgoOrderWorkingTypeEnum::MarkPrice)
                    .build().unwrap();
                    
                    // [TODO-AUDIT-NEW-10] Thử lại tối đa 7 lần cho Hard SL qua WebSocket API với exponential backoff
                    const SL_RETRY_DELAYS_MS: [u64; 7] = [10, 20, 40, 80, 160, 320, 500];
                    let mut success = false;
                    for (attempt_idx, &delay_ms) in SL_RETRY_DELAYS_MS.iter().enumerate() {
                        let attempt = attempt_idx + 1;
                        match ws.new_algo_order(p_sl.clone()).await {
                            Ok(resp) => {
                                info!("🛡️ [{}] Đã đặt Hard SL bảo vệ qua WebSocket tại: {}", symbol, rounded_sl);
                                self.set_active_sl(symbol, sl).await;
                                if let Ok(data) = resp.data() {
                                    if let Some(new_id) = data.algo_id {
                                        self.set_active_algo_id(symbol, new_id).await;
                                    }
                                }
                                success = true;
                                break;
                            },
                            Err(e) => {
                                let err_str = e.to_string();
                                warn!("⚠️ [{}] Thử lần {}/7 đặt Hard SL qua WS thất bại ({}), thử lại sau {}ms...", symbol, attempt, err_str, delay_ms);

                                // Nếu dính lỗi trùng lệnh hoặc đã tồn tại algo order (-2021 hoặc -4061)
                                if err_str.contains("-4061") || err_str.contains("already exists") {
                                    warn!("🔄 [{}] Phát hiện lỗi trùng lệnh Algo (-4061), đồng bộ và hủy lệnh cũ trước khi thử lại...", symbol);
                                    if let Ok(Some(existing_id)) = self.sync_active_algo_orders(symbol).await {
                                        let p_cancel = WsCancelAlgoOrderParams::builder().algo_id(existing_id).build().unwrap();
                                        let _ = ws.cancel_algo_order(p_cancel).await;
                                        self.clear_active_algo_id(symbol).await;
                                    }
                                }

                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                            }
                        }
                    }

                    if !success {
                        error!(
                            "🚨 [CRITICAL ALERT] [{}] Đã thử 7 lần nhưng THẤT BẠI đặt Hard SL tại {}! Kích hoạt EMERGENCY PANIC CLOSE để triệt tiêu Naked Position!",
                            symbol, rounded_sl
                        );

                        // Đảo chiều side để đóng vị thế khẩn cấp
                        let close_side = match side {
                            WsNewOrderSideEnum::Buy => WsNewOrderSideEnum::Sell,
                            WsNewOrderSideEnum::Sell => WsNewOrderSideEnum::Buy,
                        };

                        let close_params = WsNewOrderParams::builder(
                            symbol.to_string(),
                            close_side,
                            WsNewOrderTypeEnum::Market
                        )
                        .quantity(dec_qty)
                        .build().map_err(|e| BotError::Internal(e.to_string()))?;

                        let mut closed_successfully = false;
                        if let Some(ws_panic) = self.wait_ws_ready(1000).await {
                            match ws_panic.new_order(close_params).await {
                                Ok(res) => {
                                    if let Ok(data) = res.data() {
                                        error!("🚨 [{}] EMERGENCY PANIC CLOSE THÀNH CÔNG qua WebSocket! OrderID: {:?}", symbol, data.order_id);
                                        closed_successfully = true;
                                    }
                                }
                                Err(e) => {
                                    warn!("⚠️ [{}] Lỗi gửi Panic Close qua WebSocket: {:?}. Sẵn sàng kích hoạt REST fallback...", symbol, e);
                                }
                            }
                        }

                        // Last-resort fallback: REST API nếu WebSocket thất bại
                        if !closed_successfully {
                            warn!("⚠️ [{}] Kích hoạt Last-Resort REST Panic Close...", symbol);
                            let rest_side = match side {
                                WsNewOrderSideEnum::Buy => RestNewOrderSideEnum::Sell,
                                WsNewOrderSideEnum::Sell => RestNewOrderSideEnum::Buy,
                            };
                            let rest_params = RestNewOrderParams::builder(
                                symbol.to_string(),
                                rest_side,
                                RestNewOrderTypeEnum::Market
                            )
                            .quantity(dec_qty)
                            .build().map_err(|e| BotError::Internal(e.to_string()))?;

                            self.monitor.wait_if_needed(1).await;
                            match self.rest_client.new_order(rest_params).await {
                                Ok(res) => {
                                    if let Ok(data) = res.data().await {
                                        error!("🚨 [{}] LAST-RESORT REST PANIC CLOSE THÀNH CÔNG! OrderID: {:?}", symbol, data.order_id);
                                    }
                                }
                                Err(e) => {
                                    error!("💥 [CATASTROPHIC FAILURE] [{}] KHÔNG THỂ PANIC CLOSE QUA CẢ WS LẪN REST: {:?}", symbol, e);
                                }
                            }
                        }

                        return Err(BotError::Internal(format!(
                            "EMERGENCY PANIC CLOSE TRIGGERED: Failed to place Hard SL for {} at {}",
                            symbol, rounded_sl
                        )));
                    }
                }

                if is_close_all {
                    self.clear_active_sl(symbol).await;
                    if let Some(old_algo_id) = self.get_active_algo_id(symbol).await {
                        let p_cancel = WsCancelAlgoOrderParams::builder().algo_id(old_algo_id).build().unwrap();
                        let _ = ws.cancel_algo_order(p_cancel).await;
                        self.clear_active_algo_id(symbol).await;
                        info!("🧹 [{}] Đã hủy Algo SL cũ (ID: {}) qua WebSocket API khi đóng vị thế.", symbol, old_algo_id);
                    } else {
                        // 🛡️ RAM không có algo_id (sau khi bot restart) -> Dọn sạch toàn bộ Algo Order mồ côi qua REST
                        let p_cancel = CancelAllAlgoOpenOrdersParams::builder(symbol.to_string()).build().unwrap();
                        self.monitor.wait_if_needed(1).await;
                        let _ = self.rest_client.cancel_all_algo_open_orders(p_cancel).await;
                        info!("🧹 [{}] Đã dọn sạch Algo Orders mồ côi qua REST khi đóng vị thế.", symbol);
                    }
                }

                Ok(())
            },
            Err(e) => {
                error!("❌ [{}] Lỗi gửi lệnh qua WebSocket API: {:?}", symbol, e);
                Err(BotError::Internal(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_order_size_btc() {
        let filter = MarketFilter {
            tick_size: 0.1,
            step_size: 0.001,
            min_qty: 0.001,
            max_qty: 1000.0,
            min_notional: 5.0,
            ..Default::default()
        };
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Giả sử có 100 USDT, Risk 1.0 (100%), đòn bẩy x1, giá BTC = 60000.5
        // Qty mong muốn: 100 / 60000.5 = 0.001666...
        // Làm tròn step_size 0.001 -> 0.002
        let qty = executor.calculate_order_size(100.0, 60000.5, 1.0).unwrap();
        assert_eq!(qty, 0.002);
    }

    #[test]
    fn test_calculate_order_size_doge() {
        let filter = MarketFilter {
            tick_size: 0.00001,
            step_size: 1.0,
            min_qty: 1.0,
            max_qty: 10000000.0,
            min_notional: 5.0,
            ..Default::default()
        };
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Giả sử rủi ro 10 USDT, đòn bẩy x10 (thành 100 USDT), giá DOGE = 0.12345
        // Qty mong muốn: 100 / 0.12345 = 810.04455...
        // Làm tròn step_size 1.0 -> 810.0
        let qty = executor.calculate_order_size(10.0, 0.12345, 10.0).unwrap();
        assert_eq!(qty, 810.0);
    }

    #[test]
    fn test_calculate_order_size_min_notional_fail() {
        let filter = MarketFilter::default();
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Rủi ro chỉ 1 USDT, đòn bẩy x1, giá 60000
        // Qty = 0.000016 -> làm tròn thành 0.0 (không đủ min qty) hoặc 0.000
        // Notional = 0 < 5.0 -> lỗi MinNotional
        let res = executor.calculate_order_size(1.0, 60000.0, 1.0);
        assert!(res.is_err());
        match res {
            Err(BotError::MarketFilterError(_)) => (),
            _ => panic!("Phải trả về MarketFilterError"),
        }
    }

    #[tokio::test]
    #[ignore] // Chỉ chạy thủ công khi cần test với Testnet thật
    async fn test_integration_execute_order_testnet() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("BINANCE_TESTNET_API_KEY").unwrap_or_default();
        let api_secret = std::env::var("BINANCE_TESTNET_SECRET_KEY").unwrap_or_default();
        
        if api_key.is_empty() || api_secret.is_empty() {
            println!("Skipping integration test due to missing BINANCE_TESTNET_API_KEY");
            return;
        }

        let filter = MarketFilter {
            tick_size: 0.00001,
            step_size: 1.0,
            min_qty: 1.0,
            max_qty: 10000000.0,
            min_notional: 5.0,
            ..Default::default()
        };
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new(api_key, api_secret, true, filter, db_store, monitor);

        let action = Action::Buy {
            size: 1.0, // Sử dụng 100% của mức rủi ro truyền vào
            limit: None,
            stop: None,
            policy: None,
            tag: None,
        };
        
        // Tăng vốn lên 100 USDT để đảm bảo Notional (Risk) > 5 USDT bất chấp giá Market thay đổi
        let res = executor.execute_action("DOGEUSDT", action, 0.12, 100.0, 0.0, 1.0).await;
        
        assert!(res.is_ok(), "Lỗi gửi lệnh E2E Testnet: {:?}", res.err());
    }

    #[test]
    fn test_monotonic_ratchet_long() {
        // 1. Long: Initial SL 1900.0, new SL 1950.0 (tighter) -> Cho phép
        let res = OrderExecutor::evaluate_ratchet_sl(true, Some(1900.0), Some(1950.0), true);
        assert_eq!(res, Some(1950.0));

        // 2. Long: Retracted SL 1920.0 từ 1950.0 -> Bị Monotonic Ratchet chặn (None)
        let res = OrderExecutor::evaluate_ratchet_sl(true, Some(1950.0), Some(1920.0), true);
        assert_eq!(res, None);

        // 3. Long: Same SL 1950.0 -> No-op (None) để tránh spam API
        let res = OrderExecutor::evaluate_ratchet_sl(true, Some(1950.0), Some(1950.0), true);
        assert_eq!(res, None);
    }

    #[test]
    fn test_monotonic_ratchet_short() {
        // 1. Short: Initial SL 2100.0, new SL 2050.0 (tighter) -> Cho phép
        let res = OrderExecutor::evaluate_ratchet_sl(false, Some(2100.0), Some(2050.0), true);
        assert_eq!(res, Some(2050.0));

        // 2. Short: Retracted SL 2080.0 từ 2050.0 -> Bị Monotonic Ratchet chặn (None)
        let res = OrderExecutor::evaluate_ratchet_sl(false, Some(2050.0), Some(2080.0), true);
        assert_eq!(res, None);

        // 3. Short: Same SL 2050.0 -> No-op (None) để tránh spam API
        let res = OrderExecutor::evaluate_ratchet_sl(false, Some(2050.0), Some(2050.0), true);
        assert_eq!(res, None);
    }

    #[test]
    fn test_naked_position_guard() {
        // Đang có vị thế với SL 1950.0, cố xóa SL bằng None -> Bị Naked Position Guard chặn (None)
        let res = OrderExecutor::evaluate_ratchet_sl(true, Some(1950.0), None, true);
        assert_eq!(res, None);
    }

    #[tokio::test]
    async fn test_order_executor_wait_ws_ready_timeout() {
        let filter = MarketFilter::default();
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // Khi ws_api chưa được gán (None), wait_ws_ready phải trả về None sau thời gian chờ
        let ws = executor.wait_ws_ready(100).await;
        assert!(ws.is_none());
    }

    #[tokio::test]
    async fn test_order_executor_algo_id_tracking() {
        let filter = MarketFilter::default();
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        assert_eq!(executor.get_active_algo_id("BTCUSDT").await, None);

        executor.set_active_algo_id("BTCUSDT", 987654321).await;
        assert_eq!(executor.get_active_algo_id("BTCUSDT").await, Some(987654321));

        executor.clear_active_algo_id("BTCUSDT").await;
        assert_eq!(executor.get_active_algo_id("BTCUSDT").await, None);
    }

    #[tokio::test]
    async fn test_order_executor_persistence_recovery() {
        let filter = MarketFilter::default();
        let db_store = Arc::new(SqliteStore::new(":memory:").unwrap());
        let monitor = Arc::new(crate::models::weight_monitor::WeightMonitor::new(2400));

        // 1. Giả lập phiên chạy trước đã lưu algo_id và SL vào SQLite
        db_store.save_active_algo_order("BTCUSDT", 11223344, Some(58_500.0)).unwrap();
        db_store.save_active_algo_order("ETHUSDT", 55667788, Some(3_100.0)).unwrap();

        // 2. Bot restart: Khởi tạo OrderExecutor mới với cùng db_store
        let executor = OrderExecutor::new("".to_string(), "".to_string(), true, filter, db_store, monitor);

        // 3. Xác minh: Executor mới lập tức khôi phục 100% algo_id và SL từ SQLite vào RAM
        assert_eq!(executor.get_active_algo_id("BTCUSDT").await, Some(11223344));
        assert_eq!(executor.get_active_sl("BTCUSDT").await, Some(58_500.0));

        assert_eq!(executor.get_active_algo_id("ETHUSDT").await, Some(55667788));
        assert_eq!(executor.get_active_sl("ETHUSDT").await, Some(3_100.0));

        // 4. Xóa khi dời hoặc đóng lệnh -> SQLite cũng được xóa
        executor.clear_active_algo_id("BTCUSDT").await;
        assert_eq!(executor.get_active_algo_id("BTCUSDT").await, None);
        assert_eq!(executor.db_store.get_active_algo_order("BTCUSDT").unwrap(), None);
    }

    #[test]
    fn test_extract_sl_price_from_action() {
        // 1. Helios V3 style: Action::Buy với RiskPolicy::FixedPrice mang sl_price, stop là None
        let buy_action_v3 = Action::Buy {
            size: 1.0,
            limit: None,
            stop: None,
            policy: Some(RiskPolicy::FixedPrice {
                sl_price: Some(1950.0),
                tp_price: Some(2100.0),
            }),
            tag: Some("TREND_PB".to_string()),
        };
        assert_eq!(OrderExecutor::extract_sl_price(&buy_action_v3), Some(1950.0));

        // 2. Helios V3 style: Action::Sell với RiskPolicy::FixedPrice mang sl_price
        let sell_action_v3 = Action::Sell {
            size: 1.0,
            limit: None,
            stop: None,
            policy: Some(RiskPolicy::FixedPrice {
                sl_price: Some(2050.0),
                tp_price: None,
            }),
            tag: Some("REVERSAL".to_string()),
        };
        assert_eq!(OrderExecutor::extract_sl_price(&sell_action_v3), Some(2050.0));

        // 3. Fallback style: stop field trực tiếp khi policy là None
        let legacy_action = Action::Buy {
            size: 1.0,
            limit: None,
            stop: Some(1940.0),
            policy: None,
            tag: None,
        };
        assert_eq!(OrderExecutor::extract_sl_price(&legacy_action), Some(1940.0));

        // 4. Trailing SL style: Action::UpdateTradeSlTp
        let update_action = Action::UpdateTradeSlTp {
            trade_id: 1,
            new_sl_price: Some(1980.0),
            new_tp_price: None,
        };
        assert_eq!(OrderExecutor::extract_sl_price(&update_action), Some(1980.0));

        // 5. Action ClosePosition -> None
        let close_action = Action::ClosePosition { portion: 1.0 };
        assert_eq!(OrderExecutor::extract_sl_price(&close_action), None);
    }
}
