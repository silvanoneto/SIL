//! # 🌍 sil-environment — L7 Camada Ambiental
//!
//! Implementa a camada L7 (Ambiente/Ambiental) do sistema SIL, atuando tanto
//! como **Sensor** quanto como **Processor**. Esta camada integra dados
//! ambientais e realiza fusão sensorial de múltiplas fontes.
//!
//! ## Arquitetura L7
//!
//! L7 é única porque opera em dois modos:
//!
//! ### Modo Sensor: ClimateSensor
//!
//! Captura dados ambientais através de sensores climáticos:
//! - Temperatura, umidade, pressão
//! - Qualidade do ar (AQI)
//! - CO2, VOC, PM2.5, PM10
//!
//! **Codificação ByteSil:**
//! - `ρ` (magnitude): Score de conforto normalizado [-8, 7]
//! - `θ` (fase): Índice de qualidade do ar [0, 255]
//!
//! ### Modo Processor: SensorFusion
//!
//! Funde dados das camadas de percepção (L0-L4) com contexto ambiental (L7):
//! - Combina múltiplas fontes sensoriais
//! - Aplica ponderação adaptativa
//! - Gera contexto ambiental enriquecido
//!
//! ## Exemplo de Uso
//!
//! ```rust
//! use sil_environment::{ClimateSensor, SensorFusion};
//! use sil_core::traits::{Sensor, Processor};
//! use sil_core::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Modo Sensor: Capturar dados ambientais
//! let mut climate = ClimateSensor::new()?;
//! climate.calibrate()?;
//! let update = climate.sense()?;
//!
//! // Modo Processor: Fusão sensorial
//! let mut fusion = SensorFusion::new()?;
//! let state = SilState::neutral();
//! let enriched_state = fusion.execute(&state)?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Módulos
//!
//! - [`climate`] - Sensor climático (modo Sensor)
//! - [`fusion`] - Fusão sensorial (modo Processor)
//! - [`types`] - Tipos de dados ambientais
//! - [`error`] - Tratamento de erros

pub mod error;
pub mod types;
pub mod climate;
pub mod fusion;

// Re-exportar tipos principais
pub use error::{EnvironmentError, EnvironmentResult};
pub use types::{EnvironmentData, EnvironmentLimits};
pub use climate::{ClimateSensor, ClimateConfig};
pub use fusion::{SensorFusion, FusionConfig, FusionResult};

// Re-exportar traits do core
pub use sil_core::prelude::*;
pub use sil_core::traits::{Sensor, Processor, SilComponent};

#[cfg(test)]
mod tests;
