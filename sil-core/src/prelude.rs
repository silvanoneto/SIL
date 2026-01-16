//! # Prelude — Re-exportações Convenientes
//!
//! Importação única para usar o SIL-Core:
//!
//! ```
//! use sil_core::prelude::*;
//! ```

// Estado
pub use crate::state::{
    ByteSil,
    SilState,
    CollapseStrategy,
    NUM_LAYERS,
    PHI,
    layers,
};

// Traits fundamentais
pub use crate::traits::{
    // Base
    SilComponent,
    SilUpdate,
    SilEvent,
    LayerId,
    Timestamp,
    Confidence,
    ThresholdDirection,
    // Percepção (L0-L4)
    Sensor,
    SensorError,
    // Processamento (L5-L7)
    Processor as ProcessorTrait,
    ProcessorError,
    Actuator,
    ActuatorError,
    ActuatorStatus,
    // Interação (L8-LA)
    NetworkNode,
    NetworkError,
    PeerInfo,
    Governor,
    GovernanceError,
    Vote,
    ProposalStatus,
    // Emergência (LB-LC)
    SwarmAgent,
    QuantumState,
    // Meta (LD-LF)
    Forkable,
    MergeError,
    Entangled,
    EntanglementError,
    Collapsible,
    CollapseError,
    // Genéricos
    ComponentError,
    ComponentResult,
};

// Transformações
pub use crate::transforms::{
    SilTransform,
    Pipeline,
    PhaseShift,
    MagnitudeScale,
    LayerSwap,
    LayerXor,
    Identity,
};

// Transformações por fase
pub use crate::transforms::perception::{
    SilSensor,
    SensorTransform,
    NullSensor,
    NeutralSensor,
    ConstantSensor,
    PerceptionAmplify,
    PerceptionNormalize,
};

pub use crate::transforms::processing::{
    ProcessingStrategy,
    ProcessingTransform,
    PassthroughStrategy,
    AggregateStrategy,
    ThresholdStrategy,
    ProcessingAmplify,
    ProcessingRotate,
};

pub use crate::transforms::interaction::{
    SilMediator,
    MediatorTransform,
    ConsensusMediator,
    LocalFirstMediator,
    RemoteFirstMediator,
    InternalFeedback,
    InteractionAmplify,
};

pub use crate::transforms::emergence::{
    EmergenceDetector,
    EmergenceTransform,
    EntropyDetector,
    PeriodicityDetector,
    ConstantDetector,
    InternalSynergy,
    InternalCoherence,
};

pub use crate::transforms::meta::{
    MetaController,
    MetaAction,
    MetaTransform,
    AlwaysContinue,
    CollapseOnNull,
    AdaptiveController,
    CycleCounter,
    PrepareCollapse,
    SetSuperposition,
    SetEntanglement,
};

// Patterns (DEPRECATED — use traits:: instead)
// Mantido para retrocompatibilidade, será removido em versão futura
#[allow(deprecated)]
pub use crate::patterns::{
    observer::{Observer, CompositeObserver, LightObserver, SoundObserver, TouchObserver},
    strategy::{Strategy, ProcessingContext, WeightedAverageStrategy, MaxStrategy, MinStrategy},
    mediator::{Mediator, ConsensusMediatorPattern, VotingMediator, XorMediator, MediatorHub},
    emergent::{EmergentSystem, EmergentPattern, PatternKind},
};

// Ciclo
pub use crate::cycle::{
    sil_loop,
    sil_loop_with_config,
    CycleConfig,
    CycleResult,
    CycleRunner,
    StopReason,
};

// Processadores
pub use crate::processors::{
    ProcessorType,
    ProcessorCapability,
    ProcessorInfo,
    Processor,
    GradientProcessor,
    InterpolationProcessor,
    Quantizable,
};

// CPU sempre disponível
pub use crate::processors::cpu::{CpuContext, CpuResult, CpuGradient};

// GPU (opcional)
#[cfg(feature = "gpu")]
pub use crate::processors::gpu::{
    GpuContext,
    GpuError,
    GpuResult,
    SilGradient,
    LayerGradient,
    lerp_states,
    slerp_states,
    interpolate_sequence,
    bezier_quadratic,
    bezier_cubic,
    state_distance,
    geodesic_distance,
};

// NPU (opcional)
#[cfg(feature = "npu")]
pub use crate::processors::npu::{
    NpuContext,
    NpuError,
    NpuResult,
    NpuModel,
    NpuTensor,
    NpuBackend,
    InferenceResult,
};
