//! Erros específicos do módulo acústico

use thiserror::Error;
use sil_core::traits::SensorError;

pub type AcousticResult<T> = Result<T, AcousticError>;

#[derive(Debug, Error, Clone)]
pub enum AcousticError {
    #[error("Microphone initialization failed: {0}")]
    MicrophoneInitFailed(String),

    #[error("Audio capture failed: {0}")]
    CaptureFailed(String),

    #[error("Audio processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Sensor not ready")]
    NotReady,

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),

    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),
}

// Conversão para SensorError do core
impl From<AcousticError> for SensorError {
    fn from(err: AcousticError) -> Self {
        match err {
            AcousticError::NotReady => SensorError::NotInitialized,
            AcousticError::Timeout(ms) => SensorError::Timeout(ms),
            AcousticError::Hardware(msg) => SensorError::Hardware(msg),
            AcousticError::InvalidConfig(msg) => SensorError::InvalidConfig(msg),
            AcousticError::InvalidSampleRate(rate) => {
                SensorError::InvalidConfig(format!("Invalid sample rate: {}", rate))
            }
            other => SensorError::ReadFailed(other.to_string()),
        }
    }
}
