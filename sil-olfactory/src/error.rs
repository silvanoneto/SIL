//! Erros específicos do módulo olfativo

use thiserror::Error;
use sil_core::traits::SensorError;

pub type OlfactoryResult<T> = Result<T, OlfactoryError>;

#[derive(Debug, Error, Clone)]
pub enum OlfactoryError {
    #[error("Gas sensor initialization failed: {0}")]
    SensorInitFailed(String),

    #[error("Gas reading failed: {0}")]
    ReadFailed(String),

    #[error("Calibration failed: {0}")]
    CalibrationFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Sensor not ready")]
    NotReady,

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Compound not found: {0}")]
    CompoundNotFound(String),

    #[error("Concentration out of range: {0}")]
    ConcentrationOutOfRange(String),
}

// Conversão para SensorError do core
impl From<OlfactoryError> for SensorError {
    fn from(err: OlfactoryError) -> Self {
        match err {
            OlfactoryError::NotReady => SensorError::NotInitialized,
            OlfactoryError::Timeout(ms) => SensorError::Timeout(ms),
            OlfactoryError::Hardware(msg) => SensorError::Hardware(msg),
            OlfactoryError::InvalidConfig(msg) => SensorError::InvalidConfig(msg),
            OlfactoryError::CalibrationFailed(msg) => SensorError::CalibrationFailed(msg),
            other => SensorError::ReadFailed(other.to_string()),
        }
    }
}
