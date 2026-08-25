use thiserror::Error;

/// Lỗi chuyên biệt cho Tầng Tối ưu hóa & Phân tích Chiến lược (Optimizer & Analysis Layer)
#[derive(Error, Debug)]
pub enum OptimizerError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Calculation error: {0}")]
    Calculation(String),

    #[error("Helios bridge error: {0}")]
    Bridge(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for OptimizerError {
    fn from(err: serde_json::Error) -> Self {
        OptimizerError::Serialization(err.to_string())
    }
}
