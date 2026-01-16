//! Erros específicos do módulo fotônico

use thiserror::Error;
use sil_core::traits::SensorError;

pub type PhotonicResult<T> = Result<T, PhotonicError>;

#[derive(Debug, Error, Clone)]
pub enum PhotonicError {
    #[error("Camera initialization failed: {0}")]
    CameraInitFailed(String),

    #[error("Frame capture failed: {0}")]
    CaptureFailed(String),

    #[error("Light sensor read failed: {0}")]
    LightReadFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Sensor not ready")]
    NotReady,

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Hardware error: {0}")]
    Hardware(String),
}

// Conversão para SensorError do core
impl From<PhotonicError> for SensorError {
    fn from(err: PhotonicError) -> Self {
        match err {
            PhotonicError::NotReady => SensorError::NotInitialized,
            PhotonicError::Timeout(ms) => SensorError::Timeout(ms),
            PhotonicError::Hardware(msg) => SensorError::Hardware(msg),
            PhotonicError::InvalidConfig(msg) => SensorError::InvalidConfig(msg),
            other => SensorError::ReadFailed(other.to_string()),
        }
    }
}
