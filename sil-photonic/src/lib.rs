//! # 📷 sil-photonic — L0 Percepção Fotônica
//!
//! Implementa sensores óticos (câmeras, sensores de luz) usando o trait `Sensor`.
//! Responsável pela captura e conversão de informação luminosa para o estado SIL.
//!
//! ## Camada L0: Fotônica
//!
//! - **ρ (magnitude)**: Intensidade luminosa normalizada (-8 a +7)
//! - **θ (fase)**: Hue/matiz de cor (0 a 255)
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_photonic::CameraSensor;
//! use sil_core::traits::Sensor;
//!
//! let mut camera = CameraSensor::new(640, 480)?;
//! let state = camera.read_to_state()?;
//! ```

pub mod error;
pub mod camera;
pub mod light;
pub mod types;

pub use error::{PhotonicError, PhotonicResult};
pub use camera::{CameraSensor, CameraConfig};
pub use light::{LightSensor, LightConfig};
pub use types::{ImageData, Pixel, Intensity};

// Re-export core types
pub use sil_core::prelude::*;

#[cfg(test)]
mod tests;
