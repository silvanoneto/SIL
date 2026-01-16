//! # 🎭 Patterns — Design Patterns SIL
//!
//! ⚠️ **DEPRECATED**: Este módulo será removido em versão futura.
//!
//! Use os traits de `sil_core::traits` em vez disso:
//!
//! | Antigo (patterns) | Novo (traits) |
//! |:------------------|:--------------|
//! | `patterns::observer::SilSensor` | `traits::Sensor` |
//! | `patterns::strategy::ProcessingStrategy` | `traits::Processor` |
//! | `patterns::mediator::SilMediator` | `traits::NetworkNode` |
//! | `patterns::emergent::EmergenceDetector` | `traits::SwarmAgent` |
//!
//! ## Migração
//!
//! ```ignore
//! // ANTES:
//! use sil_core::patterns::observer::SilSensor;
//!
//! // DEPOIS:
//! use sil_core::traits::Sensor;
//! ```
//!
//! Os traits em `transforms/` (SilSensor, ProcessingStrategy, etc.) ainda são válidos
//! para uso com o sistema de transformações. Este módulo `patterns/` contém apenas
//! implementações concretas que serão movidas para os crates específicos.

#[deprecated(
    since = "2026.1.12",
    note = "Use sil_core::traits::{Sensor, Processor, NetworkNode, SwarmAgent} instead. \
            Este módulo será movido para os crates específicos (sil-photonic, sil-network, etc.)"
)]
pub mod observer;

#[deprecated(
    since = "2026.1.12",
    note = "Use sil_core::traits::Processor instead. \
            Este módulo será movido para sil-electronic."
)]
pub mod strategy;

#[deprecated(
    since = "2026.1.12",
    note = "Use sil_core::traits::{NetworkNode, Governor} instead. \
            Este módulo será movido para sil-network."
)]
pub mod mediator;

#[deprecated(
    since = "2026.1.12",
    note = "Use sil_core::traits::{SwarmAgent, QuantumState} instead. \
            Este módulo será movido para sil-swarm."
)]
pub mod emergent;

// Re-exportações (também deprecated)
#[allow(deprecated)]
pub use observer::*;
#[allow(deprecated)]
pub use strategy::*;
#[allow(deprecated)]
pub use mediator::*;
#[allow(deprecated)]
pub use emergent::*;
