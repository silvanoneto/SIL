//! # 🤚 sil-haptic — L4 Percepção Háptica
//!
//! Implementa sensores hápticos (toque, pressão, temperatura, vibração) usando o trait `Sensor`.
//! Responsável pela captura e conversão de informação tátil para o estado SIL.
//!
//! ## Camada L4: Háptica/Dérmica
//!
//! - **ρ (magnitude)**: Intensidade de pressão normalizada (-8 a +7)
//! - **θ (fase)**: Temperatura, área de contato ou frequência de vibração (0 a 255)
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_haptic::PressureSensor;
//! use sil_core::traits::Sensor;
//!
//! let mut sensor = PressureSensor::new()?;
//! let state = sensor.read_to_state()?;
//! ```

pub mod error;
pub mod pressure;
pub mod touch;
pub mod types;

pub use error::{HapticError, HapticResult};
pub use pressure::{PressureSensor, PressureConfig};
pub use touch::{TouchSensor, HapticConfig};
pub use types::{Pressure, Temperature, Vibration, HapticData, HapticReading};

// Re-export core types
pub use sil_core::prelude::*;

#[cfg(test)]
mod tests;
