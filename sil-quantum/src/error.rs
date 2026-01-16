//! Tipos de erro para sil-quantum

use thiserror::Error;

/// Resultado customizado para operações quânticas
pub type QuantumResult<T> = Result<T, QuantumError>;

/// Erros que podem ocorrer em operações quânticas
#[derive(Debug, Clone, Error)]
pub enum QuantumError {
    #[error("Invalid weights: sum must equal 1.0, got {0}")]
    InvalidWeights(f32),

    #[error("Weight mismatch: {states} states but {weights} weights")]
    WeightMismatch { states: usize, weights: usize },

    #[error("No states to superpose")]
    NoStates,

    #[error("Cannot collapse: not in superposition")]
    NotSuperposed,

    #[error("Decoherence detected: coherence {0}")]
    Decoherence(f32),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}
