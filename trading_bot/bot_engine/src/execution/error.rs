use thiserror::Error;
use binance_sdk::common::errors::ConnectorError;

/// Lỗi chuyên biệt cho Tầng Thực thi (Execution Engine Layer)
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("API error {code}: {msg}")]
    Api { code: i32, msg: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Margin error: {0}")]
    Margin(String),

    #[error("Market filter error: {0}")]
    MarketFilter(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<ConnectorError> for ExecutionError {
    fn from(err: ConnectorError) -> Self {
        match err {
            ConnectorError::NetworkError(msg) => ExecutionError::Network(msg),
            ConnectorError::ConnectorClientError { msg, code } |
            ConnectorError::UnauthorizedError { msg, code } |
            ConnectorError::ForbiddenError { msg, code } |
            ConnectorError::TooManyRequestsError { msg, code, .. } |
            ConnectorError::RateLimitBanError { msg, code, .. } |
            ConnectorError::NotFoundError { msg, code } |
            ConnectorError::BadRequestError { msg, code } => ExecutionError::Api {
                code: code.unwrap_or(-1) as i32,
                msg,
            },
            ConnectorError::ServerError { msg, status_code } => ExecutionError::Api {
                code: status_code.unwrap_or(500) as i32,
                msg,
            },
        }
    }
}

impl From<serde_json::Error> for ExecutionError {
    fn from(err: serde_json::Error) -> Self {
        ExecutionError::Serialization(err.to_string())
    }
}
