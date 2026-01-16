//! # ⚡ Processadores — GPU, NPU, FPGA e CPU
//!
//! Abstração unificada para diferentes backends de processamento acelerado.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        Processor Trait                               │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────┐ │
//! │  │   GPU    │  │   NPU    │  │   FPGA   │  │   CPU    │  │Hybrid │ │
//! │  │  wgpu    │  │ CoreML   │  │ Xilinx/  │  │  SIMD    │  │ Auto- │ │
//! │  │ compute  │  │ ANE/NNAPI│  │  Intel   │  │  rayon   │  │select │ │
//! │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └───────┘ │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Processadores Disponíveis
//!
//! - **GPU** (`gpu`): Computação geral via wgpu (Metal/Vulkan/DX12)
//! - **NPU** (`npu`): Inferência neural via CoreML (Apple) / NNAPI (Android)
//! - **FPGA** (`fpga`): Aceleração em hardware reconfigurável (Xilinx/Intel/Lattice)
//! - **CPU** (`cpu`): Fallback com SIMD quando disponível
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::processors::{Processor, ProcessorCapability};
//!
//! // Auto-seleciona o melhor processador disponível
//! let processor = Processor::auto_select(&[
//!     ProcessorCapability::MatrixOps,
//!     ProcessorCapability::Inference,
//! ])?;
//!
//! // Ou seleciona específico
//! let npu = NpuContext::new()?;
//! let result = npu.infer(&model, &input).await?;
//! ```

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "npu")]
pub mod npu;

#[cfg(feature = "fpga")]
pub mod fpga;

pub mod cpu;
mod traits;
pub mod performance_fixes;  // Hot fixes para problemas de performance
pub mod auto;               // Seleção automática de processador
pub mod unified_batch;      // Executor unificado GPU/NPU
#[cfg(feature = "gpu")]
pub mod quantum_bridge;     // Bridge sil-quantum ↔ sil-core GPU

pub use traits::*;
pub use performance_fixes::{ProcessorSelector, available_processors_cached};
pub use unified_batch::{UnifiedBatchExecutor, UnifiedBatchConfig, UnifiedBatchError, SelectedBackend, OperationType};

// Re-exportações condicionais
#[cfg(feature = "gpu")]
pub use gpu::{GpuContext, GpuError, GpuResult, SilGradient, LayerGradient};

#[cfg(feature = "gpu")]
pub use gpu::quantum::{QuantumGpuExecutor, GpuQuantumState, HadamardParams, GateParams, GateMatrix, gate_types};

#[cfg(feature = "gpu")]
pub use quantum_bridge::{QuantumGpuBridge, QuantumBridgeError, QuantumBridgeResult, matrices};

#[cfg(feature = "npu")]
pub use npu::{NpuContext, NpuError, NpuResult, NpuModel, InferenceResult};

#[cfg(feature = "fpga")]
pub use fpga::{FpgaContext, FpgaError, FpgaResult, FpgaConfig, FpgaBackend};

pub use cpu::{CpuContext, CpuResult};

use std::sync::OnceLock;

/// Capacidades de processamento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessorCapability {
    /// Operações matriciais (matmul, conv)
    MatrixOps,
    /// Cálculo de gradientes
    Gradients,
    /// Interpolação de estados
    Interpolation,
    /// Inferência de modelos neurais
    Inference,
    /// Quantização/dequantização
    Quantization,
    /// Operações de redução (sum, mean, max)
    Reduction,
}

/// Tipo de processador
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorType {
    /// GPU via wgpu
    Gpu,
    /// Neural Processing Unit
    Npu,
    /// FPGA acelerador
    Fpga,
    /// CPU (fallback)
    Cpu,
    /// Híbrido (auto-seleção por operação)
    Hybrid,
}

impl ProcessorType {
    /// Verifica se o processador está disponível no sistema
    pub fn is_available(&self) -> bool {
        match self {
            ProcessorType::Cpu => true, // Sempre disponível
            #[cfg(feature = "gpu")]
            ProcessorType::Gpu => gpu::GpuContext::is_available(),
            #[cfg(not(feature = "gpu"))]
            ProcessorType::Gpu => false,
            #[cfg(feature = "npu")]
            ProcessorType::Npu => npu::NpuContext::is_available(),
            #[cfg(not(feature = "npu"))]
            ProcessorType::Npu => false,
            #[cfg(feature = "fpga")]
            ProcessorType::Fpga => fpga::FpgaContext::is_available(),
            #[cfg(not(feature = "fpga"))]
            ProcessorType::Fpga => false,
            ProcessorType::Hybrid => true,
        }
    }

    /// Lista processadores disponíveis
    pub fn available() -> Vec<Self> {
        Self::available_cached().to_vec()
    }

    /// Lista processadores disponíveis (com cache)
    ///
    /// Performance:
    /// - Primeira chamada: ~4.8µs (detecção real)
    /// - Chamadas subsequentes: <1ns (cache lookup)
    ///
    /// FIX: Elimina regressão de +21,310% em available()
    pub fn available_cached() -> &'static [Self] {
        static AVAILABLE: OnceLock<Vec<ProcessorType>> = OnceLock::new();

        AVAILABLE.get_or_init(|| {
            [Self::Gpu, Self::Npu, Self::Fpga, Self::Cpu]
                .into_iter()
                .filter(|p| p.is_available())
                .collect()
        })
    }
}

/// Informações do processador
#[derive(Debug, Clone)]
pub struct ProcessorInfo {
    pub processor_type: ProcessorType,
    pub name: String,
    pub vendor: String,
    pub capabilities: Vec<ProcessorCapability>,
    /// Memória disponível em bytes (se aplicável)
    pub memory_bytes: Option<u64>,
    /// Compute units / cores
    pub compute_units: Option<u32>,
}
