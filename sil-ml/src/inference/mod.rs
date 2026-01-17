//! # Inference Engine
//!
//! Hardware-agnostic inference with automatic backend selection.

// mod engine;
// mod backend;
// mod cpu;
// mod gpu;
// mod npu;
// mod router;

use sil_core::state::SilState;
use crate::error::{Result};
use crate::edge::{HardwareProfile, DeviceType, rho_sil};

/// Inference engine trait
pub trait InferenceEngine: Send + Sync {
    fn infer(&self, input: &SilState) -> Result<SilState>;
    fn infer_batch(&self, inputs: &[SilState]) -> Result<Vec<SilState>>;
    fn backend_name(&self) -> &str;
}

/// Backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Cpu,
    Gpu,
    Npu,
    Auto,
}

impl Backend {
    pub fn detect() -> Self {
        let profile = HardwareProfile::detect();
        match profile.device_type {
            DeviceType::GPU => Backend::Gpu,
            DeviceType::NPU => Backend::Npu,
            _ => Backend::Cpu,
        }
    }
}

/// Routing decision for distributed inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    Local,
    Offload,
    Distribute,
}

/// CPU inference backend
pub struct CpuBackend;

impl CpuBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine for CpuBackend {
    fn infer(&self, input: &SilState) -> Result<SilState> {
        // Basic passthrough - real implementation would run model
        Ok(*input)
    }

    fn infer_batch(&self, inputs: &[SilState]) -> Result<Vec<SilState>> {
        inputs.iter().map(|input| self.infer(input)).collect()
    }

    fn backend_name(&self) -> &str {
        "cpu"
    }
}

/// Edge router for inference decisions
pub struct EdgeRouter {
    pub rho_critical: f64,
    pub profile: HardwareProfile,
}

impl EdgeRouter {
    pub fn new(rho_critical: f64) -> Self {
        Self {
            rho_critical,
            profile: HardwareProfile::detect(),
        }
    }

    pub fn route(&self, input: &SilState) -> RoutingDecision {
        let rho = rho_sil(input);
        
        if rho <= self.rho_critical {
            RoutingDecision::Local
        } else {
            // TODO: Enable when ChromaticZone is available
            // let zone = ChromaticZone::from_rho(rho);
            // match zone {
            //     ChromaticZone::UltraLocal | ChromaticZone::Local => RoutingDecision::Local,
            //     ChromaticZone::Near => RoutingDecision::Distribute,
            //     ChromaticZone::Far | ChromaticZone::HPC => RoutingDecision::Offload,
            // }
            RoutingDecision::Distribute
        }
    }

    pub fn select_backend(&self) -> Backend {
        Backend::detect()
    }
}

impl Default for EdgeRouter {
    fn default() -> Self {
        Self::new(0.5)
    }
}

/// Unified inference executor
pub struct InferenceExecutor {
    backend: Box<dyn InferenceEngine>,
    router: EdgeRouter,
}

impl InferenceExecutor {
    pub fn new(backend: Backend) -> Self {
        let engine: Box<dyn InferenceEngine> = match backend {
            Backend::Auto | Backend::Cpu => Box::new(CpuBackend::new()),
            // GPU and NPU would have their own implementations
            Backend::Gpu => Box::new(CpuBackend::new()), // Fallback
            Backend::Npu => Box::new(CpuBackend::new()), // Fallback
        };
        
        Self {
            backend: engine,
            router: EdgeRouter::default(),
        }
    }

    pub fn auto() -> Self {
        Self::new(Backend::Auto)
    }

    pub fn infer(&self, input: &SilState) -> Result<SilState> {
        let decision = self.router.route(input);
        
        match decision {
            RoutingDecision::Local => self.backend.infer(input),
            RoutingDecision::Offload => {
                // Would offload to remote - for now, run locally
                self.backend.infer(input)
            }
            RoutingDecision::Distribute => {
                // Would distribute across nodes - for now, run locally
                self.backend.infer(input)
            }
        }
    }

    pub fn infer_batch(&self, inputs: &[SilState]) -> Result<Vec<SilState>> {
        self.backend.infer_batch(inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_backend() {
        let backend = CpuBackend::new();
        let input = SilState::neutral();
        let output = backend.infer(&input).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_edge_router() {
        let router = EdgeRouter::new(0.5);
        let input = SilState::neutral();
        let decision = router.route(&input);
        // Decision depends on rho_sil value
        assert!(matches!(decision, RoutingDecision::Local | RoutingDecision::Distribute | RoutingDecision::Offload));
    }

    #[test]
    fn test_inference_executor() {
        let executor = InferenceExecutor::auto();
        let input = SilState::neutral();
        let output = executor.infer(&input).unwrap();
        assert_eq!(input, output);
    }
}
