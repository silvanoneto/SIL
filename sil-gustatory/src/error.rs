//! Erros específicos do módulo gustativo

use thiserror::Error;
use sil_core::traits::SensorError;

pub type GustatoryResult<T> = Result<T, GustatoryError>;

#[derive(Debug, Error, Clone)]
pub enum GustatoryError {
    #[error("Taste sensor initialization failed: {0}")]
    SensorInitFailed(String),

    #[error("Taste reading failed: {0}")]
    ReadFailed(String),

    #[error("pH sensor read failed: {0}")]
    PhReadFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Sensor not ready")]
    NotReady,

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Invalid pH value: {0} (must be 0.0-14.0)")]
    InvalidPh(f32),

    #[error("Calibration failed: {0}")]
    CalibrationFailed(String),
}

// Conversão para SensorError do core
impl From<GustatoryError> for SensorError {
    fn from(err: GustatoryError) -> Self {
        match err {
            GustatoryError::NotReady => SensorError::NotInitialized,
            GustatoryError::Timeout(ms) => SensorError::Timeout(ms),
            GustatoryError::Hardware(msg) => SensorError::Hardware(msg),
            GustatoryError::InvalidConfig(msg) => SensorError::InvalidConfig(msg),
            GustatoryError::CalibrationFailed(msg) => SensorError::CalibrationFailed(msg),
            other => SensorError::ReadFailed(other.to_string()),
        }
    }
}
