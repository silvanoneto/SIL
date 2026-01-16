//! # 👅 sil-gustatory — L3 Percepção Gustativa
//!
//! Implementa sensores gustativos (pH, gostos básicos, salinidade) usando o trait `Sensor`.
//! Responsável pela captura e conversão de informação química de gosto para o estado SIL.
//!
//! ## Camada L3: Gustativa
//!
//! - **ρ (magnitude)**: pH level normalizado (-8 a +7, mapeando pH 0-14)
//! - **θ (fase)**: Tipo de gosto dominante (0 a 255, dividido em 5 faixas)
//!
//! ## Os Cinco Gostos Básicos
//!
//! | Gosto | Substância | θ Range | Exemplo |
//! |:------|:-----------|:--------|:--------|
//! | Sweet | Açúcares | 0-50 | Glicose, sacarose |
//! | Sour | Ácidos | 51-101 | Ácido cítrico |
//! | Salty | Sais | 102-152 | NaCl |
//! | Bitter | Alcaloides | 153-203 | Cafeína, quinino |
//! | Umami | Glutamato | 204-254 | MSG, tomate |
//!
//! ## Parâmetros Medidos
//!
//! - **pH**: 0.0 (ácido) a 14.0 (básico), neutro = 7.0
//! - **Salinidade**: partes por milhão (ppm) ou mg/L
//! - **Condutividade**: µS/cm (correlaciona com TDS)
//! - **TDS**: Total Dissolved Solids em ppm
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_gustatory::{TasteSensor, TasteType};
//! use sil_core::traits::Sensor;
//!
//! let mut sensor = TasteSensor::new()?;
//! let taste_data = sensor.read()?;
//!
//! println!("pH: {}", taste_data.ph.value);
//! println!("Dominant taste: {:?}", taste_data.dominant_taste());
//! println!("Salinity: {} ppm", taste_data.salinity.ppm);
//!
//! let state = sensor.sense()?;
//! // state.byte.rho = pH normalizado [-8, 7]
//! // state.byte.theta = tipo de gosto [0, 255]
//! ```

pub mod error;
pub mod taste_sensor;
pub mod types;

pub use error::{GustatoryError, GustatoryResult};
pub use taste_sensor::{TasteSensor, TasteSensorConfig};
pub use types::{PhLevel, TasteProfile, TasteType, Salinity, TasteData};

// Re-export core types
pub use sil_core::prelude::*;

#[cfg(test)]
mod tests;
