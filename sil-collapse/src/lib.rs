//! # 💫 sil-collapse — LF Checkpoint/Restore
//!
//! Implementa checkpoint e restauração de estados para resiliência.
//! Representa o fim de um ciclo — reset, checkpoint, EOF.
//!
//! ## Computational Complexity
//!
//! **Checkpoint Operations — O(1):**
//! - Creation: O(16) to copy SIL (Symbolic Information Lattice) state
//! - Addition: O(1) with VecDeque (optimized 2026-01-12)
//! - Trimming: O(1) per trim using `VecDeque::pop_front()`
//! - Latest/Oldest access: O(1) with `back()`/`front()`
//!
//! **Retrieval — O(h):**
//! - Linear search through h checkpoints
//! - Typical h < 100, acceptable performance
//!
//! **Finality Calculation — O(16):**
//! - Fixed iteration over 16 layers
//!
//! **Recent Optimization (2026-01-12):**
//! - Changed from `Vec::remove(0)` [O(h)] to `VecDeque::pop_front()` [O(1)]
//! - Eliminates O(h²) behavior for repeated trimming
//!
//! **Scalability:** ✓ Excellent after VecDeque optimization
//!
//! See [COMPUTATIONAL_COMPLEXITY.md](../docs/COMPUTATIONAL_COMPLEXITY.md) for detailed analysis.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │         CollapseManager                         │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Checkpoint Storage                       │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Restore Logic                            │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Collapse Detection (L(F) threshold)      │  │
//! │  └───────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_collapse::CollapseManager;
//! use sil_core::prelude::*;
//!
//! let mut manager = CollapseManager::new(initial_state);
//!
//! // Criar checkpoint
//! let checkpoint_id = manager.checkpoint()?;
//!
//! // Modificar estado...
//!
//! // Restaurar
//! manager.restore(&checkpoint_id)?;
//! ```

pub mod manager;
pub mod checkpoint;
pub mod error;
pub mod finality;

pub use manager::{CollapseManager, CollapsibleState};
pub use checkpoint::{Checkpoint, CheckpointId};
pub use error::{CollapseError, CollapseResult};
pub use finality::{
    CollapseType, CollapseConfig, CollapseDecision, CollapseStats,
    COLLAPSE_LAYER, DEFAULT_COLLAPSE_THRESHOLD,
    check_collapse, finality, prepare_collapse, reset_collapse,
};

#[cfg(test)]
mod tests;
