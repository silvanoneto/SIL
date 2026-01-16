//! Erros de orquestração

use thiserror::Error;
use sil_core::traits::ComponentError;

pub type OrchestrationResult<T> = Result<T, OrchestrationError>;

/// Erros de orquestração
#[derive(Debug, Error, Clone)]
pub enum OrchestrationError {
    /// Componente não encontrado
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Componente já registrado
    #[error("Component already registered: {0}")]
    ComponentAlreadyRegistered(String),

    /// Erro de componente
    #[error("Component error: {0}")]
    ComponentError(#[from] ComponentError),

    /// Pipeline inválido
    #[error("Invalid pipeline: {0}")]
    InvalidPipeline(String),

    /// Falha de execução
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Evento inválido
    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    /// Camada inválida
    #[error("Invalid layer: {0}")]
    InvalidLayer(u8),

    /// Configuração inválida
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Timeout
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Lock poison
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}

impl<T> From<std::sync::PoisonError<T>> for OrchestrationError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        OrchestrationError::LockPoisoned(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OrchestrationError::ComponentNotFound("test".into());
        assert!(err.to_string().contains("Component not found"));
    }

    #[test]
    fn test_component_error_conversion() {
        let comp_err = ComponentError::Other("test".into());
        let orch_err: OrchestrationError = comp_err.into();
        assert!(orch_err.to_string().contains("Component error"));
    }
}
