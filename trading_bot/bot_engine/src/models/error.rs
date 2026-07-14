use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Strategy error: {0}")]
    StrategyError(String),
    
    #[error("Unauthorized access: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl From<rusqlite::Error> for BotError {
    fn from(err: rusqlite::Error) -> Self {
        BotError::DatabaseError(err.to_string())
    }
}
