//! Erros específicos do módulo háptico

use thiserror::Error;
use sil_core::traits::SensorError;

pub type HapticResult<T> = Result<T, HapticError>;

#[derive(Debug, Error, Clone)]
pub enum HapticError {
    #[error("Pressure sensor initialization failed: {0}")]
    PressureInitFailed(String),

    #[error("Touch sensor initialization failed: {0}")]
    TouchInitFailed(String),

    #[error("Sensor read failed: {0}")]
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

    #[error("Out of range: {0}")]
    OutOfRange(String),

    #[error("Invalid pressure value: {0}")]
    InvalidPressure(f32),

    #[error("Invalid temperature value: {0}")]
    InvalidTemperature(f32),
}

// Conversão para SensorError do core
impl From<HapticError> for SensorError {
    fn from(err: HapticError) -> Self {
        match err {
            HapticError::NotReady => SensorError::NotInitialized,
            HapticError::Timeout(ms) => SensorError::Timeout(ms),
            HapticError::Hardware(msg) => SensorError::Hardware(msg),
            HapticError::InvalidConfig(msg) => SensorError::InvalidConfig(msg),
            other => SensorError::ReadFailed(other.to_string()),
        }
    }
}
