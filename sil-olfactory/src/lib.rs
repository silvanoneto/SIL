//! # sil-olfactory — L2 Percepção Olfativa
//!
//! Implementa sensores de gás e detecção química usando o trait `Sensor`.
//! Responsável pela captura e conversão de informação química para o estado SIL.
//!
//! ## Camada L2: Olfativa
//!
//! - **ρ (magnitude)**: Concentração composta normalizada (-8 a +7)
//! - **θ (fase)**: ID do composto dominante ou assinatura de odor (0 a 255)
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_olfactory::GasSensor;
//! use sil_core::traits::Sensor;
//!
//! let mut sensor = GasSensor::new()?;
//! let state = sensor.sense()?;
//! ```

pub mod error;
pub mod gas_sensor;
pub mod types;

pub use error::{OlfactoryError, OlfactoryResult};
pub use gas_sensor::{GasSensor, GasSensorConfig};
pub use types::{
    GasConcentration, GasData, GasType, OdorProfile, OdorClass,
    ChemicalSignature,
};

// Re-export core types
pub use sil_core::prelude::*;

#[cfg(test)]
mod tests;
