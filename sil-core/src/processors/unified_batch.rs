//! # Unified Batch Executor — GPU/NPU Integration
//!
//! Executor unificado que roteia operações em batch para GPU ou NPU
//! baseado em capacidades e tipo de operação.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────────┐
//! │                     UnifiedBatchExecutor                           │
//! │  ┌──────────────────────────────────────────────────────────────┐  │
//! │  │  BatchRouter                                                 │  │
//! │  │    ├─ Gradientes      → GPU (WGSL shaders)                   │  │
//! │  │    ├─ Interpolação    → GPU (WGSL shaders)                   │  │
//! │  │    ├─ Gates Quânticas → GPU (hadamard.wgsl, quantum_gates.wgsl)│
//! │  │    ├─ Inferência ML   → NPU (CoreML/NNAPI)                   │  │
//! │  │    └─ Fallback        → CPU                                  │  │
//! │  └──────────────────────────────────────────────────────────────┘  │
//! └────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_core::processors::unified_batch::{UnifiedBatchExecutor, UnifiedBatchConfig};
//!
//! let executor = UnifiedBatchExecutor::new(UnifiedBatchConfig::default())?;
//! let gradients = executor.compute_gradients(&states).await?;
//! let results = executor.apply_hadamard(&quantum_states).await?;
//! ```

use thiserror::Error;

#[cfg(feature = "gpu")]
use super::gpu::{
    GpuContext,
    quantum::{QuantumGpuExecutor, GpuQuantumState, GateMatrix},
};

#[cfg(feature = "npu")]
use super::npu::NpuContext;

/// Erros do executor unificado
#[derive(Debug, Error)]
pub enum UnifiedBatchError {
    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("NPU error: {0}")]
    Npu(String),

    #[error("No backend available for operation: {0}")]
    NoBackend(String),

    #[error("Backend not initialized")]
    NotInitialized,

    #[error("Invalid batch size: {0}")]
    InvalidBatchSize(usize),
}

pub type UnifiedBatchResult<T> = Result<T, UnifiedBatchError>;

/// Configuração do executor unificado
#[derive(Debug, Clone)]
pub struct UnifiedBatchConfig {
    /// Tamanho máximo do batch
    pub max_batch_size: usize,
    /// Tempo máximo de espera antes de flush (ms)
    pub max_wait_ms: u64,
    /// Preferir NPU para inferência
    pub prefer_npu_for_inference: bool,
    /// Preferir GPU para gradientes
    pub prefer_gpu_for_gradients: bool,
    /// Preferir GPU para operações quânticas
    pub prefer_gpu_for_quantum: bool,
    /// Threshold mínimo para usar GPU (estados)
    pub gpu_threshold: usize,
}

impl Default for UnifiedBatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1024,
            max_wait_ms: 5,
            prefer_npu_for_inference: true,
            prefer_gpu_for_gradients: true,
            prefer_gpu_for_quantum: true,
            gpu_threshold: 100, // Usar GPU apenas para batches > 100
        }
    }
}

/// Backend selecionado para operação
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedBackend {
    Gpu,
    Npu,
    Cpu,
}

/// Tipo de operação
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Gradient,
    Interpolation,
    QuantumGate,
    Inference,
    Custom,
}

/// Executor unificado GPU/NPU
pub struct UnifiedBatchExecutor {
    config: UnifiedBatchConfig,

    #[cfg(feature = "gpu")]
    gpu_ctx: Option<Arc<GpuContext>>,

    #[cfg(feature = "gpu")]
    quantum_executor: Option<QuantumGpuExecutor>,

    #[cfg(feature = "npu")]
    npu_ctx: Option<Arc<NpuContext>>,
}

impl UnifiedBatchExecutor {
    /// Cria novo executor com configuração padrão
    pub fn new(config: UnifiedBatchConfig) -> UnifiedBatchResult<Self> {
        let mut executor = Self {
            config,
            #[cfg(feature = "gpu")]
            gpu_ctx: None,
            #[cfg(feature = "gpu")]
            quantum_executor: None,
            #[cfg(feature = "npu")]
            npu_ctx: None,
        };

        // Inicializa backends disponíveis
        executor.init_backends()?;

        Ok(executor)
    }

    /// Inicializa backends disponíveis
    fn init_backends(&mut self) -> UnifiedBatchResult<()> {
        #[cfg(feature = "gpu")]
        {
            if super::gpu::GpuContext::is_available() {
                match GpuContext::new_sync() {
                    Ok(ctx) => {
                        let ctx = Arc::new(ctx);
                        // Criar executor quântico
                        match QuantumGpuExecutor::new(&ctx) {
                            Ok(qe) => {
                                self.quantum_executor = Some(qe);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to create quantum executor: {}", e);
                            }
                        }
                        self.gpu_ctx = Some(ctx);
                    }
                    Err(e) => {
                        eprintln!("Warning: GPU unavailable: {}", e);
                    }
                }
            }
        }

        #[cfg(feature = "npu")]
        {
            if super::npu::NpuContext::is_available() {
                match NpuContext::new() {
                    Ok(ctx) => {
                        self.npu_ctx = Some(Arc::new(ctx));
                    }
                    Err(e) => {
                        eprintln!("Warning: NPU unavailable: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Seleciona backend para operação
    pub fn select_backend(&self, op: OperationType, batch_size: usize) -> SelectedBackend {
        // Se batch pequeno, usar CPU
        if batch_size < self.config.gpu_threshold {
            return SelectedBackend::Cpu;
        }

        match op {
            OperationType::Gradient | OperationType::Interpolation => {
                #[cfg(feature = "gpu")]
                if self.config.prefer_gpu_for_gradients && self.gpu_ctx.is_some() {
                    return SelectedBackend::Gpu;
                }
                SelectedBackend::Cpu
            }

            OperationType::QuantumGate => {
                #[cfg(feature = "gpu")]
                if self.config.prefer_gpu_for_quantum && self.quantum_executor.is_some() {
                    return SelectedBackend::Gpu;
                }
                SelectedBackend::Cpu
            }

            OperationType::Inference => {
                #[cfg(feature = "npu")]
                if self.config.prefer_npu_for_inference && self.npu_ctx.is_some() {
                    return SelectedBackend::Npu;
                }

                #[cfg(feature = "gpu")]
                if self.gpu_ctx.is_some() {
                    return SelectedBackend::Gpu;
                }

                SelectedBackend::Cpu
            }

            OperationType::Custom => {
                #[cfg(feature = "gpu")]
                if self.gpu_ctx.is_some() {
                    return SelectedBackend::Gpu;
                }
                SelectedBackend::Cpu
            }
        }
    }

    /// Verifica se GPU está disponível
    pub fn has_gpu(&self) -> bool {
        #[cfg(feature = "gpu")]
        {
            self.gpu_ctx.is_some()
        }
        #[cfg(not(feature = "gpu"))]
        {
            false
        }
    }

    /// Verifica se NPU está disponível
    pub fn has_npu(&self) -> bool {
        #[cfg(feature = "npu")]
        {
            self.npu_ctx.is_some()
        }
        #[cfg(not(feature = "npu"))]
        {
            false
        }
    }

    /// Verifica se executor quântico está disponível
    pub fn has_quantum_executor(&self) -> bool {
        #[cfg(feature = "gpu")]
        {
            self.quantum_executor.is_some()
        }
        #[cfg(not(feature = "gpu"))]
        {
            false
        }
    }

    /// Aplica Hadamard em batch via GPU
    #[cfg(feature = "gpu")]
    pub async fn apply_hadamard_gpu(
        &self,
        states: &[GpuQuantumState],
    ) -> UnifiedBatchResult<Vec<GpuQuantumState>> {
        let backend = self.select_backend(OperationType::QuantumGate, states.len());

        match backend {
            SelectedBackend::Gpu => {
                let ctx = self.gpu_ctx.as_ref().ok_or(UnifiedBatchError::NotInitialized)?;
                let executor = self
                    .quantum_executor
                    .as_ref()
                    .ok_or(UnifiedBatchError::NotInitialized)?;

                executor
                    .apply_hadamard(ctx, states)
                    .await
                    .map_err(|e| UnifiedBatchError::Gpu(e.to_string()))
            }
            _ => {
                // Fallback CPU usando sil-quantum
                Ok(apply_hadamard_cpu(states))
            }
        }
    }

    /// Aplica gate por tipo via GPU
    #[cfg(feature = "gpu")]
    pub async fn apply_gate_gpu(
        &self,
        states: &[GpuQuantumState],
        gate_type: u32,
        theta: f32,
    ) -> UnifiedBatchResult<Vec<GpuQuantumState>> {
        let backend = self.select_backend(OperationType::QuantumGate, states.len());

        match backend {
            SelectedBackend::Gpu => {
                let ctx = self.gpu_ctx.as_ref().ok_or(UnifiedBatchError::NotInitialized)?;
                let executor = self
                    .quantum_executor
                    .as_ref()
                    .ok_or(UnifiedBatchError::NotInitialized)?;

                executor
                    .apply_gate(ctx, states, gate_type, theta)
                    .await
                    .map_err(|e| UnifiedBatchError::Gpu(e.to_string()))
            }
            _ => {
                // Fallback CPU
                Ok(apply_gate_cpu(states, gate_type, theta))
            }
        }
    }

    /// Aplica matriz customizada via GPU
    #[cfg(feature = "gpu")]
    pub async fn apply_matrix_gpu(
        &self,
        states: &[GpuQuantumState],
        matrix: GateMatrix,
    ) -> UnifiedBatchResult<Vec<GpuQuantumState>> {
        let backend = self.select_backend(OperationType::QuantumGate, states.len());

        match backend {
            SelectedBackend::Gpu => {
                let ctx = self.gpu_ctx.as_ref().ok_or(UnifiedBatchError::NotInitialized)?;
                let executor = self
                    .quantum_executor
                    .as_ref()
                    .ok_or(UnifiedBatchError::NotInitialized)?;

                executor
                    .apply_matrix(ctx, states, matrix)
                    .await
                    .map_err(|e| UnifiedBatchError::Gpu(e.to_string()))
            }
            _ => {
                // Fallback CPU
                Ok(apply_matrix_cpu(states, &matrix))
            }
        }
    }

    /// Retorna referência ao contexto GPU (se disponível)
    #[cfg(feature = "gpu")]
    pub fn gpu_context(&self) -> Option<&Arc<GpuContext>> {
        self.gpu_ctx.as_ref()
    }

    /// Retorna referência ao contexto NPU (se disponível)
    #[cfg(feature = "npu")]
    pub fn npu_context(&self) -> Option<&Arc<NpuContext>> {
        self.npu_ctx.as_ref()
    }

    /// Retorna configuração
    pub fn config(&self) -> &UnifiedBatchConfig {
        &self.config
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  Fallbacks CPU
// ─────────────────────────────────────────────────────────────────────────

#[cfg(feature = "gpu")]
fn apply_hadamard_cpu(states: &[GpuQuantumState]) -> Vec<GpuQuantumState> {
    let h = std::f32::consts::FRAC_1_SQRT_2;

    states
        .iter()
        .map(|s| {
            GpuQuantumState {
                alpha_re: (s.alpha_re + s.beta_re) * h,
                alpha_im: (s.alpha_im + s.beta_im) * h,
                beta_re: (s.alpha_re - s.beta_re) * h,
                beta_im: (s.alpha_im - s.beta_im) * h,
            }
        })
        .collect()
}

#[cfg(feature = "gpu")]
fn apply_gate_cpu(states: &[GpuQuantumState], gate_type: u32, theta: f32) -> Vec<GpuQuantumState> {
    use super::gpu::gate_types::*;

    states
        .iter()
        .map(|s| match gate_type {
            HADAMARD => {
                let h = std::f32::consts::FRAC_1_SQRT_2;
                GpuQuantumState {
                    alpha_re: (s.alpha_re + s.beta_re) * h,
                    alpha_im: (s.alpha_im + s.beta_im) * h,
                    beta_re: (s.alpha_re - s.beta_re) * h,
                    beta_im: (s.alpha_im - s.beta_im) * h,
                }
            }
            PAULI_X => GpuQuantumState {
                alpha_re: s.beta_re,
                alpha_im: s.beta_im,
                beta_re: s.alpha_re,
                beta_im: s.alpha_im,
            },
            PAULI_Y => GpuQuantumState {
                alpha_re: s.beta_im,
                alpha_im: -s.beta_re,
                beta_re: -s.alpha_im,
                beta_im: s.alpha_re,
            },
            PAULI_Z => GpuQuantumState {
                alpha_re: s.alpha_re,
                alpha_im: s.alpha_im,
                beta_re: -s.beta_re,
                beta_im: -s.beta_im,
            },
            ROTATION_X => {
                let c = (theta * 0.5).cos();
                let sin = (theta * 0.5).sin();
                GpuQuantumState {
                    alpha_re: c * s.alpha_re + sin * s.beta_im,
                    alpha_im: c * s.alpha_im - sin * s.beta_re,
                    beta_re: sin * s.alpha_im + c * s.beta_re,
                    beta_im: -sin * s.alpha_re + c * s.beta_im,
                }
            }
            ROTATION_Y => {
                let c = (theta * 0.5).cos();
                let sin = (theta * 0.5).sin();
                GpuQuantumState {
                    alpha_re: c * s.alpha_re - sin * s.beta_re,
                    alpha_im: c * s.alpha_im - sin * s.beta_im,
                    beta_re: sin * s.alpha_re + c * s.beta_re,
                    beta_im: sin * s.alpha_im + c * s.beta_im,
                }
            }
            ROTATION_Z => {
                let half = theta * 0.5;
                let c_neg = (-half).cos();
                let s_neg = (-half).sin();
                let c_pos = half.cos();
                let s_pos = half.sin();
                GpuQuantumState {
                    alpha_re: c_neg * s.alpha_re - s_neg * s.alpha_im,
                    alpha_im: c_neg * s.alpha_im + s_neg * s.alpha_re,
                    beta_re: c_pos * s.beta_re - s_pos * s.beta_im,
                    beta_im: c_pos * s.beta_im + s_pos * s.beta_re,
                }
            }
            _ => *s, // Identity para tipos desconhecidos
        })
        .collect()
}

#[cfg(feature = "gpu")]
fn apply_matrix_cpu(states: &[GpuQuantumState], m: &GateMatrix) -> Vec<GpuQuantumState> {
    states
        .iter()
        .map(|s| {
            // α' = m00 * α + m01 * β
            let alpha_new_re = (m.m00_re * s.alpha_re - m.m00_im * s.alpha_im)
                + (m.m01_re * s.beta_re - m.m01_im * s.beta_im);
            let alpha_new_im = (m.m00_re * s.alpha_im + m.m00_im * s.alpha_re)
                + (m.m01_re * s.beta_im + m.m01_im * s.beta_re);

            // β' = m10 * α + m11 * β
            let beta_new_re = (m.m10_re * s.alpha_re - m.m10_im * s.alpha_im)
                + (m.m11_re * s.beta_re - m.m11_im * s.beta_im);
            let beta_new_im = (m.m10_re * s.alpha_im + m.m10_im * s.alpha_re)
                + (m.m11_re * s.beta_im + m.m11_im * s.beta_re);

            GpuQuantumState {
                alpha_re: alpha_new_re,
                alpha_im: alpha_new_im,
                beta_re: beta_new_re,
                beta_im: beta_new_im,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = UnifiedBatchConfig::default();
        assert_eq!(config.max_batch_size, 1024);
        assert_eq!(config.gpu_threshold, 100);
        assert!(config.prefer_gpu_for_quantum);
    }

    #[test]
    fn test_backend_selection_small_batch() {
        let config = UnifiedBatchConfig::default();
        let executor = UnifiedBatchExecutor::new(config).unwrap();

        // Batch pequeno deve usar CPU
        let backend = executor.select_backend(OperationType::QuantumGate, 50);
        assert_eq!(backend, SelectedBackend::Cpu);
    }

    #[cfg(feature = "gpu")]
    #[test]
    fn test_hadamard_cpu_fallback() {
        let states = vec![GpuQuantumState::zero(); 10];
        let result = apply_hadamard_cpu(&states);

        // H|0⟩ = |+⟩
        for s in &result {
            assert!((s.prob_zero() - 0.5).abs() < 1e-6);
            assert!((s.prob_one() - 0.5).abs() < 1e-6);
        }
    }

    #[cfg(feature = "gpu")]
    #[test]
    fn test_pauli_x_cpu_fallback() {
        use super::super::gpu::gate_types::PAULI_X;

        let states = vec![GpuQuantumState::zero(); 10];
        let result = apply_gate_cpu(&states, PAULI_X, 0.0);

        // X|0⟩ = |1⟩
        for s in &result {
            assert!(s.prob_zero().abs() < 1e-6);
            assert!((s.prob_one() - 1.0).abs() < 1e-6);
        }
    }
}
