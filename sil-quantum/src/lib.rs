//! # ⚛️ sil-quantum — LC Quantum States
//!
//! Implementa simulação de estados quânticos com superposição e colapso.
//! Permite representação de múltiplos estados simultâneos e coerência.
//!
//! ## Computational Complexity
//!
//! **Superposition — O(S × 16):**
//! - S = number of superposed states
//! - 16 = fixed SIL (Symbolic Information Lattice) layers
//! - Calculates weighted sum across all states and layers
//!
//! **Collapse — O(S):**
//! - Weighted random selection from S states
//! - Linear search for cumulative probability
//!
//! **Coherence — O(S × 16):**
//! - Pairwise comparison across states and layers
//!
//! **Scalability:**
//! - Small superpositions (S < 10): ✓ Excellent
//! - Medium superpositions (10 < S < 100): △ Good
//! - Large superpositions (S > 1000): Monitor performance
//!
//! **Note:** Quantum systems typically have S < 100 in practice
//!
//! See [COMPUTATIONAL_COMPLEXITY.md](../docs/COMPUTATIONAL_COMPLEXITY.md) for detailed analysis.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │          QuantumProcessor                       │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Superposed States + Weights              │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Coherence Tracker                        │  │
//! │  └───────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────┐  │
//! │  │  Collapse Mechanism                       │  │
//! │  └───────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_quantum::QuantumProcessor;
//! use sil_core::prelude::*;
//!
//! let mut qp = QuantumProcessor::new();
//! let states = vec![state1, state2, state3];
//! let weights = vec![0.5, 0.3, 0.2];
//!
//! let superposed = qp.superpose(&states, &weights);
//! let collapsed = qp.collapse(12345);
//! ```

pub mod processor;
pub mod error;
pub mod state;
pub mod simd;
pub mod gates;
pub mod gpu_gates;

pub use processor::{QuantumProcessor, QuantumConfig};
pub use simd::{superpose_simd, superpose_scalar, superpose_auto};
pub use error::{QuantumError, QuantumResult};
pub use state::QuantumStateData;
pub use gates::{
    QuantumRegime, QuantumGate, Matrix2x2, Complex,
    Hadamard, PauliX, PauliY, PauliZ, SGate, TGate,
    RotationX, RotationY, RotationZ, Phase,
};
pub use gpu_gates::{
    GpuQuantumGate, GpuGateType, GpuQuantumState,
    GateParamsUniform, GateMatrixUniform,
    CustomGate, QuantumCircuit, gpu_utils,
};

#[cfg(test)]
mod tests;
