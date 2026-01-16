//! # 🔀 sil-superposition — LD Fork/Merge
//!
//! Implementa fork e merge de estados para experimentação paralela.
//! Permite criar branches de estado e reconciliar com diferentes estratégias.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │           StateManager                          │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Fork: Clone + Track                      │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Merge Strategies:                        │  │
//! │  │  - XOR, Max, Weighted, Average            │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Divergence Detection                     │  │
//! │  └───────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_superposition::{StateManager, MergeStrategy};
//! use sil_core::prelude::*;
//!
//! let mut manager = StateManager::new(initial_state);
//! let fork = manager.fork();
//!
//! // Modifica fork...
//! manager.merge(&fork, MergeStrategy::Average)?;
//! ```

pub mod manager;
pub mod strategy;
pub mod error;
pub mod multiway;

pub use manager::{StateManager, ForkableState};
pub use strategy::MergeStrategy;
pub use error::{SuperpositionError, SuperpositionResult};
pub use multiway::{SuperOp, SuperStrategy, BranchId, Branch, MultiwayGraph};

#[cfg(test)]
mod tests;
