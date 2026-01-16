//! # 🎭 sil-orchestration — Orquestração Central
//!
//! Coordenador central do ecossistema SIL que gerencia componentes de todas
//! as 16 camadas, eventos, pipeline de execução e comunicação entre módulos.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Orchestrator                             │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │            Component Registry                         │  │
//! │  │  Sensors | Processors | Actuators | NetworkNodes     │  │
//! │  │  Governors | SwarmAgents | Quantum | Meta            │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │            Event Bus                                  │  │
//! │  │  StateChange | Threshold | Error | Custom            │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │            Execution Pipeline                         │  │
//! │  │  Sense → Process → Actuate → Network → Govern        │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_orchestration::Orchestrator;
//! use sil_core::prelude::*;
//!
//! let mut orch = Orchestrator::new();
//!
//! // Registrar componentes
//! orch.register_sensor(my_camera);
//! orch.register_processor(my_processor);
//! orch.register_actuator(my_motor);
//!
//! // Executar pipeline
//! orch.run()?;
//! ```

pub mod orchestrator;
pub mod registry;
pub mod events;
pub mod pipeline;
pub mod scheduler;
pub mod error;
pub mod lockfree;
pub mod distributed;

pub use orchestrator::{Orchestrator, OrchestratorConfig};
pub use registry::{ComponentRegistry, ComponentId};
pub use events::{EventBus, EventFilter, EventHandler};
pub use pipeline::{Pipeline, PipelineStage};
pub use scheduler::{Scheduler, SchedulerConfig, SchedulerMode, SchedulerStats};
pub use error::{OrchestrationError, OrchestrationResult};
pub use lockfree::{LockFreeEventBus, Subscription};
pub use distributed::{
    DistributedOrchestrator, ClusterConfig, ClusterMode,
    ClusterState, NodeState, NodeStatus, NodeCapacity,
    CoordinationMessage, DistributedStats,
};

// Re-exporta traits do core
pub use sil_core::prelude::*;
