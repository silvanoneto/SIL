//! Tipos de erro para sil-superposition

use sil_core::traits::MergeError;
use thiserror::Error;

/// Resultado customizado para operações de superposição
pub type SuperpositionResult<T> = Result<T, SuperpositionError>;

/// Erros que podem ocorrer em operações de fork/merge
#[derive(Debug, Clone, Error)]
pub enum SuperpositionError {
    #[error("Merge error: {0}")]
    MergeError(#[from] MergeError),

    #[error("Invalid fork: {0}")]
    InvalidFork(String),

    #[error("State divergence too large: {0}")]
    DivergenceTooLarge(f32),

    #[error("No states to merge")]
    NoStates,

    #[error("Strategy failed: {0}")]
    StrategyFailed(String),
}
