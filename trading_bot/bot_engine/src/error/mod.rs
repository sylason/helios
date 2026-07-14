use thiserror::Error;
use binance_sdk::common::errors::ConnectorError;

#[derive(Error, Debug)]
pub enum BotError {
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

    #[error("Internal Error: {0}")]
    Internal(String),
}

impl From<ConnectorError> for BotError {
    fn from(err: ConnectorError) -> Self {
        match err {
            ConnectorError::NetworkError(msg) => BotError::NetworkError(msg),
            ConnectorError::ConnectorClientError { msg, code } |
            ConnectorError::UnauthorizedError { msg, code } |
            ConnectorError::ForbiddenError { msg, code } |
            ConnectorError::TooManyRequestsError { msg, code } |
            ConnectorError::RateLimitBanError { msg, code } |
            ConnectorError::NotFoundError { msg, code } |
            ConnectorError::BadRequestError { msg, code } => BotError::ApiError {
                code: code.unwrap_or(-1) as i32,
                msg,
            },
            ConnectorError::ServerError { msg, status_code } => BotError::ApiError {
                code: status_code.unwrap_or(500) as i32,
                msg,
            },
        }
    }
}

impl From<serde_json::Error> for BotError {
    fn from(err: serde_json::Error) -> Self {
        BotError::ParseError(err.to_string())
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
