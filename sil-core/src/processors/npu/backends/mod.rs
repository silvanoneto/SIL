//! Backend-specific implementations
//!
//! Este módulo contém implementações específicas para cada backend NPU.
//!
//! ## Backends Disponíveis
//!
//! | Backend | Plataforma | Acelerador |
//! |---------|------------|------------|
//! | CoreML | macOS/iOS | Apple Neural Engine |
//! | NNAPI | Android | Hexagon DSP, Samsung NPU, etc. |
//! | DirectML | Windows | NPU, GPU |

#[cfg(target_os = "macos")]
pub mod coreml;

#[cfg(target_os = "android")]
pub mod nnapi;

#[cfg(target_os = "windows")]
pub mod directml;

// Re-exports
#[cfg(target_os = "macos")]
pub use coreml::{CoreMLBackend, ComputeUnits, CoreMLInfo};

#[cfg(target_os = "android")]
pub use nnapi::{NNAPIBackend, NNAPIDevice, DeviceType, ExecutionPreference, NNAPIInfo};

/// Trait para backends NPU
pub trait NpuBackendImpl: Send + Sync {
    /// Nome do backend
    fn name(&self) -> &str;

    /// Verifica se está disponível
    fn is_available(&self) -> bool;

    /// Versão do backend
    fn version(&self) -> Option<String>;
}

/// Informações do backend ativo
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Nome do backend
    pub name: String,
    /// Versão
    pub version: Option<String>,
    /// Tem acelerador neural dedicado
    pub has_neural_engine: bool,
    /// Plataforma
    pub platform: &'static str,
}

impl BackendInfo {
    /// Detecta informações do backend atual
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            let backend = coreml::CoreMLBackend::default();
            Self {
                name: backend.name().to_string(),
                version: backend.version(),
                has_neural_engine: backend.has_ane(),
                platform: "macOS",
            }
        }

        #[cfg(target_os = "android")]
        {
            let backend = nnapi::NNAPIBackend::default();
            Self {
                name: backend.name().to_string(),
                version: backend.version(),
                has_neural_engine: backend.has_accelerator(),
                platform: "Android",
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "android")))]
        {
            Self {
                name: "CPU Fallback".to_string(),
                version: None,
                has_neural_engine: false,
                platform: std::env::consts::OS,
            }
        }
    }
}
