//! # 🐝 sil-swarm — LB Swarm Intelligence
//!
//! Implementa comportamento coletivo de swarm onde o todo é maior que a soma das partes.
//! Permite coordenação descentralizada, consenso emergente, e padrões de flocking.
//!
//! ## Computational Complexity
//!
//! **Flocking Behavior — O(N × 16):**
//! - N = number of neighbors
//! - 16 = fixed SIL (Signal Intermediate Language) layers
//! - Each neighbor's state is processed across all layers
//!
//! **Consensus — O(N × 16):**
//! - Similar to flocking, linear in neighbor count
//!
//! **Scalability:**
//! - Small swarms (N < 50): ✓ Excellent performance
//! - Medium swarms (50 < N < 500): △ Good performance
//! - Large swarms (N > 1000): Consider spatial partitioning
//!
//! **Optimization opportunities:**
//! - Spatial hashing to limit visible neighbors to fixed k
//! - SIMD vectorization for layer processing
//!
//! See [COMPUTATIONAL_COMPLEXITY.md](../docs/COMPUTATIONAL_COMPLEXITY.md) for detailed analysis.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              SwarmNode                          │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  ID + Neighbors + State                   │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Behaviors: Flocking, Consensus, Emerge   │  │
//! │  └───────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_swarm::SwarmNode;
//! use sil_core::prelude::*;
//!
//! let mut node = SwarmNode::new(0);
//! node.add_neighbor(1);
//! node.add_neighbor(2);
//!
//! let state = node.behavior(&local_state, &neighbor_states);
//! ```

pub mod node;
pub mod behavior;
pub mod error;
pub mod spatial;
pub mod emergence;

pub use node::{SwarmNode, SwarmConfig};
pub use behavior::{SwarmBehavior, FlockingBehavior, ConsensusBehavior};
pub use error::{SwarmError, SwarmResult};
pub use spatial::{SpatialGrid, Position3D, SpatialSwarmConfig};
pub use emergence::{OrgType, EmergenceLevel, ComplexityMetrics};

#[cfg(test)]
mod tests;
