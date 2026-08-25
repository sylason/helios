use thiserror::Error;
use binance_sdk::common::errors::ConnectorError;
use crate::data_engine::error::DataError;
use crate::execution::error::ExecutionError;
use crate::optimizer::error::OptimizerError;

/// Enum Lỗi gốc phân cấp toàn hệ thống (Hierarchical Root BotError)
#[derive(Error, Debug)]
pub enum BotError {
    #[error("Data error: {0}")]
    Data(#[from] DataError),

    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),

    #[error("Optimizer error: {0}")]
    Optimizer(#[from] OptimizerError),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("API Error {code}: {msg}")]
    ApiError {
        code: i32,
        msg: String,
    },

    #[error("Network Error: {0}")]
    NetworkError(String),

    #[error("Margin Error: {0}")]
    MarginError(String),

    #[error("Market Filter Error: {0}")]
    MarketFilterError(String),

    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("State Error: {0}")]
    StateError(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Unauthorized access: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal Error: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for BotError {
    fn from(err: rusqlite::Error) -> Self {
        BotError::Data(DataError::from(err))
    }
}

impl From<ConnectorError> for BotError {
    fn from(err: ConnectorError) -> Self {
        BotError::Execution(ExecutionError::from(err))
    }
}

impl From<serde_json::Error> for BotError {
    fn from(err: serde_json::Error) -> Self {
        BotError::Execution(ExecutionError::from(err))
    }
}

impl From<anyhow::Error> for BotError {
    fn from(err: anyhow::Error) -> Self {
        match err.downcast::<ConnectorError>() {
            Ok(conn_err) => BotError::from(conn_err),
            Err(e) => BotError::Internal(e.to_string()),
        }
    }
}

