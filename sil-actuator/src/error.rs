//! Erros da camada de atuador

use thiserror::Error;
use sil_core::traits::ActuatorError as CoreActuatorError;

pub type ActuatorResult<T> = Result<T, ActuatorError>;

/// Erros de atuador
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ActuatorError {
    /// Comando falhou
    #[error("Command failed: {0}")]
    CommandFailed(String),

    /// Atuador ocupado
    #[error("Actuator busy")]
    Busy,

    /// Falha no atuador
    #[error("Actuator fault: {0}")]
    Fault(String),

    /// Fora de alcance
    #[error("Out of range: {0}")]
    OutOfRange(String),

    /// Estado inválido
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Configuração inválida
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Hardware não inicializado
    #[error("Hardware not initialized")]
    NotInitialized,

    /// Timeout de operação
    #[error("Operation timeout after {0}ms")]
    Timeout(u64),

    /// Calibração falhou
    #[error("Calibration failed: {0}")]
    CalibrationFailed(String),

    /// Comunicação falhou
    #[error("Communication failed: {0}")]
    CommunicationFailed(String),

    /// Erro de conversão
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

impl From<ActuatorError> for CoreActuatorError {
    fn from(err: ActuatorError) -> Self {
        match err {
            ActuatorError::CommandFailed(msg) => CoreActuatorError::CommandFailed(msg),
            ActuatorError::Busy => CoreActuatorError::Busy,
            ActuatorError::Fault(msg) => CoreActuatorError::Fault(msg),
            ActuatorError::OutOfRange(msg) => CoreActuatorError::OutOfRange(msg),
            ActuatorError::InvalidState(msg) => CoreActuatorError::CommandFailed(format!("Invalid state: {}", msg)),
            ActuatorError::InvalidConfig(msg) => CoreActuatorError::CommandFailed(format!("Invalid config: {}", msg)),
            ActuatorError::NotInitialized => CoreActuatorError::Fault("Not initialized".into()),
            ActuatorError::Timeout(ms) => CoreActuatorError::CommandFailed(format!("Timeout after {}ms", ms)),
            ActuatorError::CalibrationFailed(msg) => CoreActuatorError::Fault(format!("Calibration failed: {}", msg)),
            ActuatorError::CommunicationFailed(msg) => CoreActuatorError::CommandFailed(format!("Communication failed: {}", msg)),
            ActuatorError::ConversionError(msg) => CoreActuatorError::CommandFailed(format!("Conversion error: {}", msg)),
        }
    }
}

impl From<CoreActuatorError> for ActuatorError {
    fn from(err: CoreActuatorError) -> Self {
        match err {
            CoreActuatorError::CommandFailed(msg) => ActuatorError::CommandFailed(msg),
            CoreActuatorError::Busy => ActuatorError::Busy,
            CoreActuatorError::Fault(msg) => ActuatorError::Fault(msg),
            CoreActuatorError::OutOfRange(msg) => ActuatorError::OutOfRange(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ActuatorError::CommandFailed("test".into());
        assert!(err.to_string().contains("Command failed"));
    }

    #[test]
    fn test_error_busy() {
        let err = ActuatorError::Busy;
        assert_eq!(err.to_string(), "Actuator busy");
    }

    #[test]
    fn test_error_out_of_range() {
        let err = ActuatorError::OutOfRange("0-180°".into());
        assert!(err.to_string().contains("Out of range"));
    }

    #[test]
    fn test_error_conversion_to_core() {
        let err = ActuatorError::CommandFailed("test".into());
        let core_err: CoreActuatorError = err.into();
        assert!(core_err.to_string().contains("Command failed"));
    }

    #[test]
    fn test_error_conversion_from_core() {
        let core_err = CoreActuatorError::Busy;
        let err: ActuatorError = core_err.into();
        assert_eq!(err, ActuatorError::Busy);
    }

    #[test]
    fn test_timeout_error() {
        let err = ActuatorError::Timeout(5000);
        assert!(err.to_string().contains("5000ms"));
    }

    #[test]
    fn test_fault_error() {
        let err = ActuatorError::Fault("motor stalled".into());
        assert!(err.to_string().contains("motor stalled"));
    }
}
