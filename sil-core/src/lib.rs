//! # 🌀 SIL-Core
//!
//! Implementação do padrão SIL (Signal Intermediate Language).
//!
//! > *"Linguagem intermediária otimizada para processamento de sinais complexos em representação log-polar."*
//!
//! ## O Padrão SIL
//!
//! 1. Todo estado é um **vetor de 16 camadas**
//! 2. Cada camada é um **número complexo** (ρ, θ)
//! 3. O programa é uma **transformação de estados**
//! 4. O ciclo é **fechado** (L(F) → L(0))
//!
//! ## Computational Complexity
//!
//! **Core Operations — O(1):**
//! - ByteSil arithmetic (mul, div, pow, root): O(1) via log-polar representation
//! - SilState access (get, set, with_layer): O(1) array operations
//! - All operations marked `#[inline]` and `const fn` for zero-cost abstractions
//!
//! **Layer Operations — O(16) = O(1):**
//! - tensor, xor, project, collapse: O(16) fixed iterations
//! - SIMD optimizations available (2-8× speedup on AVX2/NEON)
//! - Fixed architecture ensures constant-factor performance
//!
//! **Pipeline Operations — O(t × 16):**
//! - Linear in transform count (t)
//! - Each transform processes 16 layers
//!
//! **Scalability:** ✓ Excellent — Core operations scale to any magnitude
//!
//! See [COMPUTATIONAL_COMPLEXITY.md](../docs/COMPUTATIONAL_COMPLEXITY.md) for detailed analysis.
//!
//! ## Módulos
//!
//! - [`state`]: BitDeSil, ByteSil e SilState — representação do estado
//! - [`semantics`]: LayerSemantics — interpretação semântica por camada
//! - [`traits`]: Traits fundamentais (Sensor, Processor, NetworkNode, etc.)
//! - [`transforms`]: Transformações de estado para estado
//! - [`patterns`]: Design patterns adaptados ao SIL
//! - [`cycle`]: Loop fechado principal
//! - [`vsp`]: Virtual Sil Processor — máquina virtual
//!
//! ## Quick Start
//!
//! ```
//! use sil_core::prelude::*;
//!
//! // Criar estado inicial
//! let state = SilState::neutral();
//!
//! // Criar pipeline de transformações
//! let pipeline = Pipeline::new(vec![
//!     Box::new(PhaseShift(4)),
//!     Box::new(MagnitudeScale(2)),
//! ]);
//!
//! // Executar ciclo
//! let result = sil_loop(state, &pipeline, 100);
//! ```
//!
//! ## Princípios
//!
//! 1. **Estado é sagrado** — Nunca modifique in-place, sempre crie novo
//! 2. **Transformação é pura** — Mesma entrada, mesma saída
//! 3. **Ciclo é fechado** — Todo programa tem feedback L(F) → L(0)
//! 4. **Camadas são ortogonais** — Cada camada tem sua semântica
//! 5. **Colapso é inevitável** — Todo estado eventualmente colapsa

pub mod state;
pub mod semantics;
pub mod traits;
pub mod transforms;

#[deprecated(
    since = "2026.1.12",
    note = "Use sil_core::traits instead. Patterns serão movidos para crates específicos."
)]
pub mod patterns;

pub mod cycle;
pub mod prelude;

// Processadores (GPU, NPU, CPU)
pub mod processors;

// Virtual Sil Processor (máquina virtual)
pub mod vsp;

// I/O nativo para pipelines
pub mod io;

// Re-exportações de nível superior
pub use state::{BitDeSil, ByteSil, SilState, NUM_LAYERS, PHI, PHI_INV};
pub use semantics::{
    LayerSemantics, LayerGroup, RhoInterpretation, ThetaInterpretation,
    ControlMode, GovernanceMode, EthicalMode, OrgType, QuantumRegime,
    SuperStrategy, CorrelationType, CollapseType,
    interpret_rho_for_layer, interpret_theta_for_layer,
};
pub use transforms::{SilTransform, Pipeline};
pub use cycle::{sil_loop, sil_loop_with_config, CycleConfig, CycleResult, StopReason};

// Re-exporta traits fundamentais
pub use traits::{
    SilComponent, SilUpdate, SilEvent,
    Sensor, SensorError,
    Processor, ProcessorError,
    Actuator, ActuatorError, ActuatorStatus,
    NetworkNode, NetworkError, PeerInfo,
    Governor, GovernanceError, Vote, ProposalStatus,
    SwarmAgent, QuantumState,
    Forkable, MergeError,
    Entangled, EntanglementError,
    Collapsible, CollapseError,
    ComponentError, ComponentResult,
};

// Re-exporta processadores
pub use processors::{ProcessorType, ProcessorCapability, ProcessorInfo};

// Re-exporta VSP
pub use vsp::{Vsp, VspConfig, VspError, VspResult};

// Re-exporta ferramentas VSP
pub use vsp::{
    Assembler, assemble, disassemble,
    Repl,
    Debugger, Breakpoint, DebugEvent, DebuggerState,
    EntanglementManager, NodeId, PairId,
};

// Python FFI
#[cfg(feature = "python")]
pub mod python;

// WASM FFI
#[cfg(feature = "wasm")]
pub mod wasm;
