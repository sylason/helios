use thiserror::Error;

/// Lỗi chuyên biệt cho Tầng Dữ liệu và Lưu trữ (Data Engine Layer)
#[derive(Error, Debug)]
pub enum DataError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("API error {code}: {msg}")]
    Api { code: i32, msg: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Data not found: {0}")]
    NotFound(String),
}

impl From<rusqlite::Error> for DataError {
    fn from(err: rusqlite::Error) -> Self {
        DataError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for DataError {
    fn from(err: serde_json::Error) -> Self {
        DataError::Serialization(err.to_string())
    }
}
