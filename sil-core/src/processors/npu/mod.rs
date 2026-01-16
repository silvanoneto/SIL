//! # 🧠 NPU — Neural Processing Unit
//!
//! Aceleração de inferência neural via backends nativos:
//! - **macOS/iOS**: Core ML (Apple Neural Engine)
//! - **Android**: NNAPI (Neural Networks API)
//! - **Windows**: DirectML
//! - **Linux**: OpenVINO / TensorRT
//!
//! ## Conceito
//!
//! NPU é otimizado para operações de inferência:
//! - Quantização INT8/FP16
//! - Operações de convolução
//! - Transformers e atenção
//! - Batch processing eficiente
//!
//! ## Arquitetura SIL-NPU
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    SilState                         │
//! │  16 camadas × (ρ: i4, θ: u4) = 128 bits            │
//! └────────────────────┬────────────────────────────────┘
//!                      │ Quantize
//!                      ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              NPU Input Tensor                       │
//! │  [batch, 16, 2] float16 ou int8                    │
//! └────────────────────┬────────────────────────────────┘
//!                      │ Inference
//!                      ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              NPU Output Tensor                      │
//! │  Classificação, Embedding, Predição                │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::processors::npu::{NpuContext, NpuModel};
//!
//! let npu = NpuContext::new()?;
//! let model = npu.load_model("sil_classifier.mlmodel")?;
//! let result = npu.infer(&model, &state)?;
//! ```

mod context;
mod model;
mod tensor;
mod backends;

pub use context::{NpuContext, Precision};
pub use model::{NpuModel, ModelFormat};
pub use tensor::{NpuTensor, TensorLayout, DataType};

/// Erro de NPU
#[derive(Debug, thiserror::Error)]
pub enum NpuError {
    #[error("NPU não disponível")]
    NotAvailable,
    
    #[error("Backend não suportado: {0}")]
    UnsupportedBackend(String),
    
    #[error("Modelo inválido: {0}")]
    InvalidModel(String),
    
    #[error("Formato não suportado: {0}")]
    UnsupportedFormat(String),
    
    #[error("Erro de inferência: {0}")]
    InferenceError(String),
    
    #[error("Erro de quantização: {0}")]
    QuantizationError(String),
    
    #[error("Tamanho de tensor inválido: esperado {expected}, recebido {actual}")]
    TensorSizeMismatch { expected: usize, actual: usize },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Resultado NPU
pub type NpuResult<T> = Result<T, NpuError>;

/// Resultado de inferência
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Output tensor
    pub output: NpuTensor,
    /// Tempo de inferência em microsegundos
    pub latency_us: u64,
    /// Backend utilizado
    pub backend: NpuBackend,
}

/// Backend NPU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuBackend {
    /// Apple Neural Engine (via Core ML)
    CoreML,
    /// Android Neural Networks API
    NNAPI,
    /// Microsoft DirectML
    DirectML,
    /// Intel OpenVINO
    OpenVINO,
    /// NVIDIA TensorRT
    TensorRT,
    /// CPU Fallback (usando SIMD)
    CpuFallback,
}

impl NpuBackend {
    /// Detecta o melhor backend disponível
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            return Self::CoreML;
        }
        
        #[cfg(target_os = "ios")]
        {
            return Self::CoreML;
        }
        
        #[cfg(target_os = "android")]
        {
            return Self::NNAPI;
        }
        
        #[cfg(target_os = "windows")]
        {
            return Self::DirectML;
        }
        
        #[cfg(target_os = "linux")]
        {
            // Tenta OpenVINO primeiro, depois TensorRT
            return Self::OpenVINO;
        }
        
        #[allow(unreachable_code)]
        Self::CpuFallback
    }
    
    /// Nome do backend
    pub fn name(&self) -> &'static str {
        match self {
            Self::CoreML => "Core ML",
            Self::NNAPI => "NNAPI",
            Self::DirectML => "DirectML",
            Self::OpenVINO => "OpenVINO",
            Self::TensorRT => "TensorRT",
            Self::CpuFallback => "CPU Fallback",
        }
    }
}
