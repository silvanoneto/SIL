//! # 🔗 sil-entanglement — LE State Entanglement
//!
//! Implementa emaranhamento de estados que mantêm correlação distribuída.
//! Estados emaranhados sincronizam mesmo quando separados.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │         EntangledState                          │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  State + Pair Registry                    │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Sync Mechanism                           │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Correlation Tracking                     │  │
//! │  └───────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_entanglement::EntangledState;
//! use sil_core::prelude::*;
//!
//! let mut state1 = EntangledState::new(SilState::neutral());
//! let mut state2 = EntangledState::new(SilState::neutral());
//!
//! let pair_id = state1.entangle(&mut state2)?;
//! state1.sync(&pair_id)?;
//! ```

pub mod state;
pub mod error;
pub mod registry;
pub mod bell;

pub use state::EntangledState;
pub use error::{EntanglementError, EntanglementResult};
pub use registry::EntanglementRegistry;
pub use bell::{CorrelationType, BellState, EntangledPair, teleport, teleport_full};

#[cfg(test)]
mod tests;
