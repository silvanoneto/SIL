//! # 🎤 sil-acoustic — L1 Percepção Acústica
//!
//! Implementa sensores acústicos (microfones, ultrassom) usando o trait `Sensor`.
//! Responsável pela captura e conversão de informação sonora para o estado SIL.
//!
//! ## Camada L1: Acústica
//!
//! - **ρ (magnitude)**: Amplitude/volume normalizado (-8 a +7)
//! - **θ (fase)**: Frequência dominante (0 a 255)
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_acoustic::MicrophoneSensor;
//! use sil_core::traits::Sensor;
//!
//! let mut microphone = MicrophoneSensor::new()?;
//! let state = microphone.read_to_state()?;
//! ```

pub mod error;
pub mod microphone;
pub mod types;

pub use error::{AcousticError, AcousticResult};
pub use microphone::{MicrophoneSensor, AudioConfig};
pub use types::{AudioSample, Frequency, Amplitude, AudioData};

// Re-export core types
pub use sil_core::prelude::*;

#[cfg(test)]
mod tests;
