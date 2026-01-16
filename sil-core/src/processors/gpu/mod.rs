//! # 🎮 GPU Compute — Gradientes no Plano Complexo
//!
//! Aceleração GPU via wgpu para cálculos massivamente paralelos.
//!
//! ## Conceito
//!
//! Cada ByteSil (ρ, θ) vive no plano complexo log-polar:
//! - **ρ ∈ [-8, 7]** → magnitude (log-escala)
//! - **θ ∈ [0, 15]** → fase (16 divisões de 2π)
//!
//! O gradiente ∇ = (∂/∂ρ, ∂/∂θ) indica a direção de maior variação.
//!
//! ## Features
//!
//! - `gradient`: Calcula ∇f para batch de estados
//! - `interpolate`: Lerp/Slerp entre estados via gradiente
//! - `jacobian`: Matriz Jacobiana de transformações
//! - `laplacian`: Difusão no espaço de estados
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::processors::gpu::{GpuContext, SilGradient};
//!
//! let ctx = GpuContext::new().await?;
//! let states: Vec<SilState> = vec![...];
//! let gradients: Vec<SilGradient> = ctx.compute_gradients(&states).await?;
//! ```

mod context;
mod gradient;
mod shaders;
pub mod interpolate;
pub mod batching;
pub mod pipeline_pool;
pub mod quantum;

pub use context::GpuContext;
pub use gradient::{SilGradient, LayerGradient};
pub use interpolate::{lerp_states, slerp_states, interpolate_sequence, bezier_quadratic, bezier_cubic, state_distance, geodesic_distance};
pub use batching::{BatchedGpuExecutor, BatchedGpuHandle, BatchConfig, GpuOp};
pub use pipeline_pool::{GpuPipelinePool, PoolStats};
pub use quantum::{QuantumGpuExecutor, GpuQuantumState, HadamardParams, GateParams, GateMatrix, gate_types};

use std::sync::OnceLock;

/// Erro de GPU
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("GPU não disponível")]
    NoAdapter,
    
    #[error("Falha ao criar device: {0}")]
    DeviceCreation(String),
    
    #[error("Shader inválido: {0}")]
    ShaderCompilation(String),
    
    #[error("Buffer overflow: esperado {expected}, recebido {actual}")]
    BufferOverflow { expected: usize, actual: usize },
    
    #[error("Timeout na execução GPU")]
    Timeout,
}

/// Resultado GPU
pub type GpuResult<T> = Result<T, GpuError>;

/// Cache estático de disponibilidade de GPU (FIX: regressão +1,551,665%)
static GPU_AVAILABLE: OnceLock<bool> = OnceLock::new();

impl GpuContext {
    /// Verifica se GPU está disponível (com cache)
    /// 
    /// Performance:
    /// - Primeira chamada: ~4.8µs (detecção real)
    /// - Chamadas subsequentes: <1ns (cache lookup)
    pub fn is_available() -> bool {
        *GPU_AVAILABLE.get_or_init(|| {
            // Tenta criar instância para verificar (apenas primeira vez)
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });
            
            pollster::block_on(async {
                instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }).await.is_some()
            })
        })
    }
}
