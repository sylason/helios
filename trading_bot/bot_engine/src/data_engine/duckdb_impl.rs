use crate::models::models::{Candle, OptimizedDna};
use crate::models::error::BotError;
use crate::models::repository::{CandleRepository, StrategyRepository};
use async_trait::async_trait;
use duckdb::{params, Connection};
use std::sync::{Arc, Mutex};
use tracing::{info, debug}; // Đã xóa 'error'

/// Lớp lưu trữ dữ liệu nến vào DuckDB
pub struct DuckDbStore {
    conn: Arc<Mutex<Connection>>,
}

impl DuckDbStore {
    /// Khởi tạo DuckDB, tạo bảng nếu chưa tồn tại
    pub fn new(db_path: &str) -> Result<Self, BotError> {
        let conn = if db_path == ":memory:" {
            Connection::open_in_memory().map_err(|e| BotError::DatabaseError(e.to_string()))?
        } else {
            Connection::open(db_path).map_err(|e| BotError::DatabaseError(e.to_string()))?
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
                PRIMARY KEY (symbol, open_time)
            )",
            [],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        // 2. Tạo bảng tham số tối ưu (JSON support)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS optimized_params (
                symbol TEXT,
                strategy_id TEXT,
                dna JSON,
                mapping JSON,
                fitness DOUBLE,
                stats JSON,
                updated_at BIGINT,
                PRIMARY KEY (symbol, strategy_id)
            )",
            [],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;

        info!("📂 DuckDB initialized at: {}", db_path);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl CandleRepository for DuckDbStore {
    /// Lưu nến đơn lẻ
    async fn save_candle(&self, c: Candle) -> Result<(), BotError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO candles_1m (symbol, open_time, open, high, low, close, volume) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![c.symbol, c.timestamp, c.open, c.high, c.low, c.close, c.volume],
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
                "INSERT OR REPLACE INTO candles_1m (symbol, open_time, open, high, low, close, volume) 
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![c.symbol, c.timestamp, c.open, c.high, c.low, c.close, c.volume],
            ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        }
        
        tx.commit().map_err(|e| BotError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Truy vấn nến theo khoảng thời gian
    async fn get_candles_by_range(&self, symbol: &str, start: i64, end: i64) -> Result<Vec<Candle>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT symbol, open_time, open, high, low, close, volume 
             FROM candles_1m 
             WHERE symbol = ? AND open_time >= ? AND open_time <= ?
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
            "SELECT symbol, open_time, open, high, low, close, volume 
             FROM candles_1m 
             WHERE symbol = ? 
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
            }))
        } else {
            Ok(None)
        }
    }

    /// Lấy nến cũ nhất
    async fn get_oldest_candle(&self, symbol: &str) -> Result<Option<Candle>, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT symbol, open_time, open, high, low, close, volume 
             FROM candles_1m 
             WHERE symbol = ? 
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
            }))
        } else {
            Ok(None)
        }
    }

    /// Đếm số lượng nến
    async fn count_candles(&self, symbol: &str) -> Result<usize, BotError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM candles_1m WHERE symbol = ?"
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
            "DELETE FROM candles_1m WHERE open_time < ?",
            params![older_than],
        ).map_err(|e| BotError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl StrategyRepository for DuckDbStore {
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
             VALUES (?, ?, ?, ?, ?, ?, ?)",
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
             WHERE symbol = ? AND strategy_id = ?"
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
    async fn test_duckdb_candle_operations() {
        // 1. Khởi tạo In-Memory DB
        let db = DuckDbStore::new(":memory:").expect("Failed to create in-memory db");

        let c1 = Candle {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 100.0,
        };

        let c2 = Candle {
            symbol: "BTCUSDT".to_string(),
            timestamp: 2000,
            open: 1.5,
            high: 2.5,
            low: 1.0,
            close: 2.0,
            volume: 200.0,
        };

        // 2. Test Insert
        db.save_candle(c1.clone()).await.unwrap();
        db.save_candle(c2.clone()).await.unwrap();

        // 3. Test Get Latest
        let latest = db.get_latest_candle("BTCUSDT").await.unwrap().unwrap();
        assert_eq!(latest.timestamp, 2000);

        // 4. Test Get Range
        let range = db.get_candles_by_range("BTCUSDT", 1000, 2000).await.unwrap();
        assert_eq!(range.len(), 2);

        // 5. Test Cleanup
        db.cleanup_old_data(1500).await.unwrap();
        let range_after = db.get_candles_by_range("BTCUSDT", 0, 5000).await.unwrap();
        assert_eq!(range_after.len(), 1);
        assert_eq!(range_after[0].timestamp, 2000);
    }
}
