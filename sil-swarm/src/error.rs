//! Tipos de erro para sil-swarm

use thiserror::Error;

/// Resultado customizado para operações de swarm
pub type SwarmResult<T> = Result<T, SwarmError>;

/// Erros que podem ocorrer em operações de swarm
#[derive(Debug, Clone, Error)]
pub enum SwarmError {
    #[error("Neighbor not found: {0}")]
    NeighborNotFound(u64),

    #[error("Invalid behavior: {0}")]
    InvalidBehavior(String),

    #[error("Insufficient neighbors: need {need}, have {have}")]
    InsufficientNeighbors { need: usize, have: usize },

    #[error("Convergence failed: {0}")]
    ConvergenceFailed(String),

    #[error("State mismatch: {0}")]
    StateMismatch(String),
}
