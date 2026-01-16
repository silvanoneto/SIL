//! # ⚡ SIL-Energy — Medição de Consumo Energético em Joules
//!
//! Módulo para detecção e rastreamento de consumo energético no processamento SIL,
//! permitindo otimização de energia em tempo real.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      SIL ENERGY SYSTEM                          │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
//! │  │ EnergyMeter │  │ EnergyModel │  │    EnergyOptimizer      │ │
//! │  │ (medição)   │  │ (estimativa)│  │ (redução automática)    │ │
//! │  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
//! │         │                │                      │               │
//! │         ▼                ▼                      ▼               │
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │                   EnergySnapshot                            ││
//! │  │  timestamp | stage | joules | watts | utilization          ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! │                            │                                    │
//! │         ┌──────────────────┴──────────────────┐                │
//! │         ▼                                      ▼                │
//! │  ┌─────────────┐                      ┌─────────────┐          │
//! │  │   Events    │                      │   Metrics   │          │
//! │  │ (threshold) │                      │ (histórico) │          │
//! │  └─────────────┘                      └─────────────┘          │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Uso
//!
//! ```ignore
//! use sil_energy::{EnergyMeter, EnergyModel, CpuEnergyModel};
//!
//! // Criar medidor com modelo de CPU
//! let model = CpuEnergyModel::detect();
//! let mut meter = EnergyMeter::new(Box::new(model));
//!
//! // Medir execução
//! meter.begin_measurement();
//! // ... processamento SIL ...
//! let snapshot = meter.end_measurement(1000); // 1000 operações
//!
//! println!("Energia consumida: {:.6} J", snapshot.joules);
//! println!("Potência média: {:.2} W", snapshot.watts);
//! ```

pub mod error;
pub mod model;
pub mod meter;
pub mod optimizer;
pub mod snapshot;
pub mod sampler;

// Re-exports
pub use error::{EnergyError, EnergyResult};
pub use model::{EnergyModel, CpuEnergyModel, GpuEnergyModel, NpuEnergyModel, HybridEnergyModel};
pub use meter::{EnergyMeter, MeterConfig};
pub use optimizer::{EnergyOptimizer, OptimizationStrategy, PowerBudget};
pub use snapshot::{EnergySnapshot, EnergyStats, EnergyReport};
pub use sampler::{EnergySampler, SamplingMode};

/// Constantes físicas e de referência para cálculo de energia
pub mod constants {
    /// Joules por ciclo de CPU típico (ARM Cortex-M4 @ 1.8V, 100MHz)
    pub const JOULES_PER_CYCLE_ARM_M4: f64 = 1.8e-9;

    /// Joules por ciclo de CPU típico (Apple M1 @ 0.9V, 3GHz performance core)
    pub const JOULES_PER_CYCLE_M1_PERF: f64 = 0.3e-9;

    /// Joules por ciclo de CPU típico (Apple M1 @ 0.6V, 2GHz efficiency core)
    pub const JOULES_PER_CYCLE_M1_EFF: f64 = 0.08e-9;

    /// Joules por ciclo de CPU típico (x86-64 @ 1.2V, 4GHz)
    pub const JOULES_PER_CYCLE_X86: f64 = 0.5e-9;

    /// Watts típicos de GPU móvel em idle
    pub const GPU_IDLE_WATTS: f64 = 0.5;

    /// Watts típicos de GPU móvel em carga máxima
    pub const GPU_MAX_WATTS: f64 = 15.0;

    /// Joules por operação SIMD (AVX-512)
    pub const JOULES_PER_SIMD_AVX512: f64 = 2.0e-9;

    /// Joules por operação SIMD (NEON)
    pub const JOULES_PER_SIMD_NEON: f64 = 0.5e-9;

    /// Joules por inferência NPU típica (1 TOPS)
    pub const JOULES_PER_NPU_INFERENCE: f64 = 1.0e-6;

    /// Joules por operação de memória (cache L1 hit)
    pub const JOULES_PER_L1_ACCESS: f64 = 0.5e-12;

    /// Joules por operação de memória (cache L2 hit)
    pub const JOULES_PER_L2_ACCESS: f64 = 2.0e-12;

    /// Joules por operação de memória (DRAM access)
    pub const JOULES_PER_DRAM_ACCESS: f64 = 20.0e-12;

    /// Eficiência típica de conversão de energia (DC-DC)
    pub const DC_DC_EFFICIENCY: f64 = 0.92;

    /// Fator de overhead do sistema operacional
    pub const OS_OVERHEAD_FACTOR: f64 = 1.05;
}

/// Tipos de processador para seleção de modelo de energia
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProcessorType {
    /// CPU genérica
    Cpu,
    /// GPU (integrada ou discreta)
    Gpu,
    /// NPU / Neural Engine
    Npu,
    /// FPGA
    Fpga,
    /// Híbrido (múltiplos tipos)
    Hybrid,
}

impl ProcessorType {
    /// Retorna o modelo de energia padrão para este processador
    pub fn default_model(&self) -> Box<dyn EnergyModel> {
        match self {
            ProcessorType::Cpu => Box::new(CpuEnergyModel::detect()),
            ProcessorType::Gpu => Box::new(GpuEnergyModel::default()),
            ProcessorType::Npu => Box::new(NpuEnergyModel::default()),
            ProcessorType::Fpga => Box::new(CpuEnergyModel::detect()), // Fallback
            ProcessorType::Hybrid => Box::new(HybridEnergyModel::detect()),
        }
    }
}

/// Estágio do pipeline para rastreamento de energia por fase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PipelineStage {
    /// Percepção (L0-L4): sensores
    Sense,
    /// Processamento (L5, L7): CPU/GPU/NPU
    Process,
    /// Atuação (L6): output
    Actuate,
    /// Rede (L8): comunicação P2P
    Network,
    /// Governança (L9-LA): consenso distribuído
    Govern,
    /// Swarm (LB): comportamento emergente
    Swarm,
    /// Quantum (LC-LF): superposição/colapso
    Quantum,
}

impl PipelineStage {
    /// Retorna o fator de peso energético para este estágio
    pub fn energy_weight(&self) -> f64 {
        match self {
            PipelineStage::Sense => 0.8,    // I/O leve
            PipelineStage::Process => 1.5,  // Computação pesada
            PipelineStage::Actuate => 0.6,  // Output simples
            PipelineStage::Network => 1.2,  // Comunicação
            PipelineStage::Govern => 1.0,   // Consenso
            PipelineStage::Swarm => 1.3,    // Emergência
            PipelineStage::Quantum => 0.9,  // Colapso/checkpoint
        }
    }

    /// Retorna o processador preferido para este estágio
    pub fn preferred_processor(&self) -> ProcessorType {
        match self {
            PipelineStage::Sense => ProcessorType::Cpu,
            PipelineStage::Process => ProcessorType::Hybrid,
            PipelineStage::Actuate => ProcessorType::Cpu,
            PipelineStage::Network => ProcessorType::Cpu,
            PipelineStage::Govern => ProcessorType::Cpu,
            PipelineStage::Swarm => ProcessorType::Npu,
            PipelineStage::Quantum => ProcessorType::Cpu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_type_default_model() {
        let cpu_model = ProcessorType::Cpu.default_model();
        assert!(cpu_model.joules_per_operation() > 0.0);
    }

    #[test]
    fn test_pipeline_stage_energy_weight() {
        assert!(PipelineStage::Process.energy_weight() > PipelineStage::Sense.energy_weight());
    }

    #[test]
    fn test_constants() {
        assert!(constants::JOULES_PER_CYCLE_M1_EFF < constants::JOULES_PER_CYCLE_X86);
        assert!(constants::DC_DC_EFFICIENCY > 0.0 && constants::DC_DC_EFFICIENCY <= 1.0);
    }
}
