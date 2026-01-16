//! Tipos de erro para sil-collapse

use sil_core::traits::CollapseError as CoreCollapseError;
use thiserror::Error;

/// Resultado customizado para operações de collapse
pub type CollapseResult<T> = Result<T, CollapseError>;

/// Erros que podem ocorrer em operações de collapse
#[derive(Debug, Clone, Error)]
pub enum CollapseError {
    #[error("Core collapse error: {0}")]
    Core(#[from] CoreCollapseError),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Checkpoint limit reached: {0}")]
    LimitReached(usize),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}
