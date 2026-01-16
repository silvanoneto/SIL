//! Erros da camada eletrônica

use thiserror::Error;
use sil_core::vsp::VspError;

pub type ElectronicResult<T> = Result<T, ElectronicError>;

/// Erros de processamento eletrônico
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ElectronicError {
    /// Erro do VSP interno
    #[error("VSP error: {0}")]
    VspError(String),

    /// Bytecode inválido
    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),

    /// Estado inválido
    #[error("Invalid processor state: {0}")]
    InvalidState(String),

    /// Limite de execução excedido
    #[error("Execution limit exceeded: {reason}")]
    ExecutionLimitExceeded { reason: String },

    /// Memória insuficiente
    #[error("Insufficient memory: {reason}")]
    InsufficientMemory { reason: String },

    /// Erro de configuração
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Pipeline error
    #[error("Pipeline error: {0}")]
    PipelineError(String),

    /// Compatibilidade
    #[error("Incompatible processor state")]
    IncompatibleState,

    /// Recurso não disponível
    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),
}

impl From<VspError> for ElectronicError {
    fn from(err: VspError) -> Self {
        ElectronicError::VspError(err.to_string())
    }
}

impl From<std::io::Error> for ElectronicError {
    fn from(err: std::io::Error) -> Self {
        ElectronicError::InvalidBytecode(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ElectronicError::InvalidBytecode("test".into());
        assert!(err.to_string().contains("Invalid bytecode"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ElectronicError = io_err.into();
        assert!(err.to_string().contains("Invalid bytecode"));
    }
}
