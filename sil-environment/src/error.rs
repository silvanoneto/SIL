//! Erros da camada ambiental (L7)

use thiserror::Error;
use sil_core::traits::{SensorError, ProcessorError};

pub type EnvironmentResult<T> = Result<T, EnvironmentError>;

/// Erros do módulo de ambiente
#[derive(Debug, Error, Clone)]
pub enum EnvironmentError {
    #[error("Sensor read failed: {0}")]
    SensorReadFailed(String),

    #[error("Sensor not initialized")]
    NotInitialized,

    #[error("Calibration failed: {0}")]
    CalibrationFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Fusion failed: {0}")]
    FusionFailed(String),

    #[error("Invalid sensor data: {0}")]
    InvalidData(String),

    #[error("Out of range: {0}")]
    OutOfRange(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

// Conversão para SensorError do core
impl From<EnvironmentError> for SensorError {
    fn from(err: EnvironmentError) -> Self {
        match err {
            EnvironmentError::NotInitialized => SensorError::NotInitialized,
            EnvironmentError::Timeout(ms) => SensorError::Timeout(ms),
            EnvironmentError::Hardware(msg) => SensorError::Hardware(msg),
            EnvironmentError::InvalidConfig(msg) => SensorError::InvalidConfig(msg),
            EnvironmentError::CalibrationFailed(msg) => SensorError::CalibrationFailed(msg),
            other => SensorError::ReadFailed(other.to_string()),
        }
    }
}

// Conversão para ProcessorError do core
impl From<EnvironmentError> for ProcessorError {
    fn from(err: EnvironmentError) -> Self {
        match err {
            EnvironmentError::FusionFailed(msg) => ProcessorError::ExecutionFailed(msg),
            EnvironmentError::ProcessingError(msg) => ProcessorError::ExecutionFailed(msg),
            EnvironmentError::InvalidData(msg) => ProcessorError::InvalidInput(msg),
            EnvironmentError::InsufficientData(msg) => ProcessorError::InvalidInput(msg),
            other => ProcessorError::ExecutionFailed(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EnvironmentError::SensorReadFailed("test".into());
        assert!(err.to_string().contains("Sensor read failed"));
    }

    #[test]
    fn test_error_conversion_to_sensor_error() {
        let err = EnvironmentError::NotInitialized;
        let sensor_err: SensorError = err.into();
        assert!(matches!(sensor_err, SensorError::NotInitialized));
    }

    #[test]
    fn test_error_conversion_to_processor_error() {
        let err = EnvironmentError::FusionFailed("test".into());
        let proc_err: ProcessorError = err.into();
        match proc_err {
            ProcessorError::ExecutionFailed(msg) => assert!(msg.contains("test")),
            _ => panic!("Expected ExecutionFailed"),
        }
    }

    #[test]
    fn test_timeout_error() {
        let err = EnvironmentError::Timeout(5000);
        assert_eq!(err.to_string(), "Timeout after 5000ms");
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            EnvironmentError::SensorReadFailed("test".into()),
            EnvironmentError::NotInitialized,
            EnvironmentError::CalibrationFailed("test".into()),
            EnvironmentError::InvalidConfig("test".into()),
            EnvironmentError::Hardware("test".into()),
            EnvironmentError::Timeout(1000),
            EnvironmentError::FusionFailed("test".into()),
            EnvironmentError::InvalidData("test".into()),
            EnvironmentError::OutOfRange("test".into()),
            EnvironmentError::ProcessingError("test".into()),
            EnvironmentError::InsufficientData("test".into()),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
