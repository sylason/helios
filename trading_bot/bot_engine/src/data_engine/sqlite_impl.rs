use crate::models::models::{Candle, OptimizedDna};
use crate::models::error::BotError;
use crate::models::repository::{CandleRepository, StrategyRepository};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use tracing::{info, debug}; 

/// Lớp lưu trữ dữ liệu nến vào SQLite
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    /// Khởi tạo SQLite, tạo bảng nếu chưa tồn tại
    pub fn new(db_path: &str) -> Result<Self, BotError> {
        let conn = if db_path == ":memory:" {
            Connection::open_in_memory().map_err(|e| BotError::DatabaseError(e.to_string()))?
        } else {
            let c = Connection::open(db_path).map_err(|e| BotError::DatabaseError(e.to_string()))?;
            // Kích hoạt WAL Mode và Busy Timeout cho phép đọc ghi đồng thời không khóa chết
            c.execute_batch("
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA busy_timeout = 5000;
            ").map_err(|e| BotError::DatabaseError(e.to_string()))?;
            c
        };
        
        // 1. Tạo bảng nến
        conn.execute(
            "CREATE TABLE IF NOT EXISTS candles_1m (
                symbol TEXT,
                open_time BIGINT,
                open DOUBLE,
                high DOUBLE,
                low DOUBLE,
                close DOUBLE,
                volume DOUBLE,
                quote_volume DOUBLE DEFAULT 0.0,
                PRIMARY KEY (symbol, open_time)
            )",
            [],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        // 2. Tạo bảng tham số tối ưu (JSON support)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS optimized_params (
                symbol TEXT,
                strategy_id TEXT,
                dna TEXT,
                mapping TEXT,
                fitness DOUBLE,
                stats TEXT,
                updated_at BIGINT,
                PRIMARY KEY (symbol, strategy_id)
            )",
            [],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        // 3. Tạo bảng lịch sử giao dịch (Trade History)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trade_history (
                order_id TEXT PRIMARY KEY,
                symbol TEXT,
                side TEXT,
                price DOUBLE,
                qty DOUBLE,
                timestamp BIGINT
            )",
            [],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        // 4. Tạo bảng quản lý Active Algo Orders (Stop Loss)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS active_algo_orders (
                symbol TEXT PRIMARY KEY,
                algo_id BIGINT NOT NULL,
                sl_price DOUBLE,
                updated_at BIGINT NOT NULL
            )",
            [],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        info!("📂 SQLite initialized at: {}", db_path);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Lưu lịch sử giao dịch (Trade History)
    pub fn save_trade_history(&self, order_id: &str, symbol: &str, side: &str, price: f64, qty: f64, timestamp: i64) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO trade_history (order_id, symbol, side, price, qty, timestamp) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![order_id, symbol, side, price, qty, timestamp],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        debug!("💾 [{}] Trade {} saved to DB.", symbol, order_id);
        Ok(())
    }

    /// Lưu active algo order vào SQLite
    pub fn save_active_algo_order(&self, symbol: &str, algo_id: i64, sl_price: Option<f64>) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        let current_time = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT OR REPLACE INTO active_algo_orders (symbol, algo_id, sl_price, updated_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![symbol, algo_id, sl_price, current_time],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        debug!("💾 [{}] Active Algo ID {} (SL: {:?}) saved to DB.", symbol, algo_id, sl_price);
        Ok(())
    }

    /// Lấy active algo order từ SQLite cho 1 symbol
    pub fn get_active_algo_order(&self, symbol: &str) -> Result<Option<(i64, Option<f64>)>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT algo_id, sl_price FROM active_algo_orders WHERE symbol = ?1"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        let mut rows = stmt.query(params![symbol]).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| BotError::DatabaseError(e.to_string()))? {
            let algo_id: i64 = row.get(0).map_err(|e| BotError::DatabaseError(e.to_string()))?;
            let sl_price: Option<f64> = row.get(1).map_err(|e| BotError::DatabaseError(e.to_string()))?;
            Ok(Some((algo_id, sl_price)))
        } else {
            Ok(None)
        }
    }

    /// Lấy tất cả active algo orders từ SQLite
    pub fn get_all_active_algo_orders(&self) -> Result<Vec<(String, i64, Option<f64>)>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT symbol, algo_id, sl_price FROM active_algo_orders"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let mut result = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                result.push(item);
            }
        }
        Ok(result)
    }

    /// Xóa active algo order khỏi SQLite
    pub fn remove_active_algo_order(&self, symbol: &str) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM active_algo_orders WHERE symbol = ?1",
            params![symbol],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        debug!("💾 [{}] Active Algo order removed from DB.", symbol);
        Ok(())
    }
}

#[async_trait]
impl CandleRepository for SqliteStore {
    /// Lưu nến đơn lẻ
    async fn save_candle(&self, c: Candle) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO candles_1m (symbol, open_time, open, high, low, close, volume, quote_volume) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![c.symbol, c.timestamp, c.open, c.high, c.low, c.close, c.volume, c.effective_quote_volume()],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        debug!("💾 [{}] Candle {} saved to DB.", c.symbol, c.timestamp);
        Ok(())
    }

    /// Lưu danh sách nến (Batch insert dùng Transaction)
    async fn save_candles(&self, candles: Vec<Candle>) -> Result<(), BotError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| BotError::DatabaseError(e.to_string()))?;
        
        for c in candles {
            tx.execute(
                "INSERT OR REPLACE INTO candles_1m (symbol, open_time, open, high, low, close, volume, quote_volume) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![c.symbol, c.timestamp, c.open, c.high, c.low, c.close, c.volume, c.effective_quote_volume()],
            ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        }
        
        tx.commit().map_err(|e| BotError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Truy vấn nến theo khoảng thời gian
    async fn get_candles_by_range(&self, symbol: &str, start: i64, end: i64) -> Result<Vec<Candle>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT symbol, open_time, open, high, low, close, volume, quote_volume 
             FROM candles_1m 
             WHERE symbol = ?1 AND open_time >= ?2 AND open_time <= ?3
             ORDER BY open_time ASC"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let candle_iter = stmt.query_map(params![symbol, start, end], |row| {
            Ok(Candle {
                symbol: row.get(0)?,
                timestamp: row.get(1)?,
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                volume: row.get(6)?,
                quote_volume: row.get(7)?,
            })
        }).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for candle in candle_iter {
            results.push(candle.map_err(|e| BotError::DatabaseError(e.to_string()))?);
        }
        Ok(results)
    }

    /// Lấy nến mới nhất
    async fn get_latest_candle(&self, symbol: &str) -> Result<Option<Candle>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT symbol, open_time, open, high, low, close, volume, quote_volume 
             FROM candles_1m 
             WHERE symbol = ?1 
             ORDER BY open_time DESC LIMIT 1"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let mut rows = stmt.query(params![symbol])
            .map_err(|e| BotError::DatabaseError(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| BotError::DatabaseError(e.to_string()))? {
            Ok(Some(Candle {
                symbol: row.get(0)?,
                timestamp: row.get(1)?,
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                volume: row.get(6)?,
                quote_volume: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lấy nến cũ nhất
    async fn get_oldest_candle(&self, symbol: &str) -> Result<Option<Candle>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT symbol, open_time, open, high, low, close, volume, quote_volume 
             FROM candles_1m 
             WHERE symbol = ?1 
             ORDER BY open_time ASC LIMIT 1"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let mut rows = stmt.query(params![symbol])
            .map_err(|e| BotError::DatabaseError(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| BotError::DatabaseError(e.to_string()))? {
            Ok(Some(Candle {
                symbol: row.get(0)?,
                timestamp: row.get(1)?,
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                volume: row.get(6)?,
                quote_volume: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Đếm số lượng nến
    async fn count_candles(&self, symbol: &str) -> Result<usize, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM candles_1m WHERE symbol = ?1"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let mut rows = stmt.query(params![symbol])
            .map_err(|e| BotError::DatabaseError(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| BotError::DatabaseError(e.to_string()))? {
            let count: i64 = row.get(0)?;
            Ok(count as usize)
        } else {
            Ok(0)
        }
    }

    /// Xóa dữ liệu cũ
    async fn cleanup_old_data(&self, older_than: i64) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM candles_1m WHERE open_time < ?1",
            params![older_than],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl StrategyRepository for SqliteStore {
    async fn save_best_dna(&self, d: OptimizedDna) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        
        let dna_json = serde_json::to_string(&d.dna)
            .map_err(|e| BotError::DatabaseError(format!("DNA serialization error: {}", e)))?;
        let mapping_json = serde_json::to_string(&d.mapping)
            .map_err(|e| BotError::DatabaseError(format!("Mapping serialization error: {}", e)))?;
        let stats_json = serde_json::to_string(&d.stats)
            .map_err(|e| BotError::DatabaseError(format!("Stats serialization error: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO optimized_params (symbol, strategy_id, dna, mapping, fitness, stats, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![d.symbol, d.strategy_id, dna_json, mapping_json, d.fitness, stats_json, d.updated_at],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        info!("🧬 [{}] DNA for strategy '{}' saved to DB.", d.symbol, d.strategy_id);
        Ok(())
    }

    async fn get_best_dna(&self, symbol: &str, strategy_id: &str) -> Result<Option<OptimizedDna>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT dna, mapping, fitness, stats, updated_at 
             FROM optimized_params 
             WHERE symbol = ?1 AND strategy_id = ?2"
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        let mut rows = stmt.query(params![symbol, strategy_id])
            .map_err(|e| BotError::DatabaseError(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| BotError::DatabaseError(e.to_string()))? {
            let dna_json: String = row.get(0)?;
            let mapping_json: String = row.get(1)?;
            let stats_json: String = row.get(3)?;

            Ok(Some(OptimizedDna {
                symbol: symbol.to_string(),
                strategy_id: strategy_id.to_string(),
                dna: serde_json::from_str(&dna_json).unwrap_or_default(),
                mapping: serde_json::from_str(&mapping_json).unwrap_or_default(),
                fitness: row.get(2)?,
                stats: serde_json::from_str(&stats_json).unwrap_or_default(),
                updated_at: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_candle_operations() {
        // 1. Khởi tạo In-Memory DB
        let db = SqliteStore::new(":memory:").expect("Failed to create in-memory db");

        let c1 = Candle {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 100.0,
            quote_volume: 150.0,
        };

        let c2 = Candle {
            symbol: "BTCUSDT".to_string(),
            timestamp: 2000,
            open: 1.5,
            high: 2.5,
            low: 1.0,
            close: 2.0,
            volume: 200.0,
            quote_volume: 400.0,
        };

        // 2. Test Insert
        db.save_candle(c1.clone()).await.unwrap();
        db.save_candle(c2.clone()).await.unwrap();

        // 3. Test Get Latest
        let latest = db.get_latest_candle("BTCUSDT").await.unwrap().unwrap();
        assert_eq!(latest.timestamp, 2000);
        assert_eq!(latest.quote_volume, 400.0);

        // 4. Test Get Range
        let range = db.get_candles_by_range("BTCUSDT", 1000, 2000).await.unwrap();
        assert_eq!(range.len(), 2);

        // 5. Test Cleanup
        db.cleanup_old_data(1500).await.unwrap();
        let range_after = db.get_candles_by_range("BTCUSDT", 0, 5000).await.unwrap();
        assert_eq!(range_after.len(), 1);
        assert_eq!(range_after[0].timestamp, 2000);
    }

    #[tokio::test]
    async fn test_sqlite_active_algo_orders() {
        let db = SqliteStore::new(":memory:").expect("Failed to create in-memory db");

        // 1. Kiểm tra ban đầu rỗng
        assert_eq!(db.get_active_algo_order("BTCUSDT").unwrap(), None);
        assert_eq!(db.get_all_active_algo_orders().unwrap().len(), 0);

        // 2. Lưu active algo order cho BTCUSDT và ETHUSDT
        db.save_active_algo_order("BTCUSDT", 12345678, Some(59_000.0)).unwrap();
        db.save_active_algo_order("ETHUSDT", 87654321, Some(3_200.0)).unwrap();

        // 3. Đọc lại kiểm tra
        let btc = db.get_active_algo_order("BTCUSDT").unwrap().unwrap();
        assert_eq!(btc.0, 12345678);
        assert_eq!(btc.1, Some(59_000.0));

        let all = db.get_all_active_algo_orders().unwrap();
        assert_eq!(all.len(), 2);

        // 4. Xóa BTCUSDT
        db.remove_active_algo_order("BTCUSDT").unwrap();
        assert_eq!(db.get_active_algo_order("BTCUSDT").unwrap(), None);
        assert_eq!(db.get_all_active_algo_orders().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_sqlite_candle_storage_with_quote_volume() {
        let db = SqliteStore::new(":memory:").expect("Failed to create in-memory db");

        let c1 = Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp: 1000,
            open: 2000.0,
            high: 2010.0,
            low: 1990.0,
            close: 2005.0,
            volume: 5.0,
            quote_volume: 10025.5,
        };

        // 1. Test single save & get
        db.save_candle(c1.clone()).await.unwrap();
        let latest = db.get_latest_candle("ETHUSDT").await.unwrap().unwrap();
        assert_eq!(latest.symbol, "ETHUSDT");
        assert_eq!(latest.timestamp, 1000);
        assert_eq!(latest.quote_volume, 10025.5);

        // 2. Test batch save & range query
        let c2 = Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp: 2000,
            open: 2005.0,
            high: 2020.0,
            low: 2000.0,
            close: 2015.0,
            volume: 8.0,
            quote_volume: 16120.0,
        };
        let c3 = Candle {
            symbol: "ETHUSDT".to_string(),
            timestamp: 3000,
            open: 2015.0,
            high: 2030.0,
            low: 2010.0,
            close: 2025.0,
            volume: 10.0,
            quote_volume: 20250.0,
        };
        db.save_candles(vec![c2, c3]).await.unwrap();

        let range = db.get_candles_by_range("ETHUSDT", 1000, 3000).await.unwrap();
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].quote_volume, 10025.5);
        assert_eq!(range[1].quote_volume, 16120.0);
        assert_eq!(range[2].quote_volume, 20250.0);

        let oldest = db.get_oldest_candle("ETHUSDT").await.unwrap().unwrap();
        assert_eq!(oldest.timestamp, 1000);
        assert_eq!(oldest.quote_volume, 10025.5);
    }
}
