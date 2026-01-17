//! Error types for sil-ml

use thiserror::Error;

/// Result type for sil-ml operations
pub type Result<T> = std::result::Result<T, SilMlError>;

/// sil-ml error types
#[derive(Error, Debug)]
pub enum SilMlError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Shape mismatch: expected {expected}, got {actual}")]
    ShapeMismatch { expected: String, actual: String },

    #[error("Hardware not available: {0}")]
    HardwareUnavailable(String),

    #[error("Inference error: {0}")]
    InferenceError(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for SilMlError {
    fn from(err: std::io::Error) -> Self {
        SilMlError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for SilMlError {
    fn from(err: serde_json::Error) -> Self {
        SilMlError::SerializationError(err.to_string())
    }
}
