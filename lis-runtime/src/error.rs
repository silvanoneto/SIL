//! Error types for LIS runtime

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("LIS compilation error: {0}")]
    CompilationError(String),

    #[error("VSP assembly error: {0}")]
    AssemblyError(String),

    #[error("VSP execution error: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid program format: {0}")]
    InvalidFormat(String),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl From<lis_core::Error> for RuntimeError {
    fn from(err: lis_core::Error) -> Self {
        RuntimeError::CompilationError(err.to_string())
    }
}
