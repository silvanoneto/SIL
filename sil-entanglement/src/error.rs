//! Tipos de erro para sil-entanglement

use sil_core::traits::EntanglementError as CoreEntanglementError;
use thiserror::Error;

/// Resultado customizado para operações de entanglement
pub type EntanglementResult<T> = Result<T, EntanglementError>;

/// Erros que podem ocorrer em operações de entanglement
#[derive(Debug, Clone, Error)]
pub enum EntanglementError {
    #[error("Core entanglement error: {0}")]
    Core(#[from] CoreEntanglementError),

    #[error("Invalid pair ID: {0}")]
    InvalidPairId(String),

    #[error("Correlation lost: {0}")]
    CorrelationLost(f32),

    #[error("Cannot sync: {0}")]
    CannotSync(String),
}
