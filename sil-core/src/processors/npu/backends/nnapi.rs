//! NNAPI Backend (Android)
//!
//! Implementação do backend Android Neural Networks API.
//!
//! ## Requisitos
//!
//! - Android 8.1+ (API 27+)
//! - Para NPU dedicado: Dispositivos com aceleradores neurais (Hexagon DSP, Samsung NPU, etc.)
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                      NNAPIBackend                        │
//! │  ┌─────────────────────────────────────────────────────┐ │
//! │  │ ANeuralNetworksModel                                │ │
//! │  │  - addOperand / addOperation                        │ │
//! │  │  - finish / compile                                 │ │
//! │  └─────────────────────────────────────────────────────┘ │
//! │  ┌─────────────────────────────────────────────────────┐ │
//! │  │ ANeuralNetworksCompilation                          │ │
//! │  │  - setPreference (low power, fast, etc.)            │ │
//! │  │  - finish                                           │ │
//! │  └─────────────────────────────────────────────────────┘ │
//! │  ┌─────────────────────────────────────────────────────┐ │
//! │  │ ANeuralNetworksExecution                            │ │
//! │  │  - setInput / setOutput                             │ │
//! │  │  - compute / startCompute                           │ │
//! │  └─────────────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Dispositivos Suportados
//!
//! | Chipset | Acelerador | API Level |
//! |---------|------------|-----------|
//! | Snapdragon 845+ | Hexagon DSP | 27+ |
//! | Exynos 9820+ | Samsung NPU | 27+ |
//! | Kirin 970+ | NPU | 27+ |
//! | MediaTek Dimensity | APU | 27+ |
//! | Google Tensor | Google TPU | 30+ |

use super::NpuBackendImpl;

/// Backend Android NNAPI
pub struct NNAPIBackend {
    /// Versão da API
    api_level: u32,
    /// Dispositivos disponíveis
    devices: Vec<NNAPIDevice>,
    /// Preferência de execução
    preference: ExecutionPreference,
}

/// Dispositivo NNAPI
#[derive(Debug, Clone)]
pub struct NNAPIDevice {
    /// Nome do dispositivo
    pub name: String,
    /// Tipo do dispositivo
    pub device_type: DeviceType,
    /// Versão do driver
    pub version: String,
    /// Feature level
    pub feature_level: u32,
}

/// Tipo de dispositivo NNAPI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// CPU genérico
    Cpu,
    /// GPU
    Gpu,
    /// DSP (Digital Signal Processor)
    Dsp,
    /// NPU dedicado
    Accelerator,
    /// Desconhecido
    Unknown,
}

/// Preferência de execução
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionPreference {
    /// Baixo consumo de energia (melhor para bateria)
    LowPower,
    /// Execução rápida (melhor latência)
    FastSingleAnswer,
    /// Throughput sustentado (melhor para workloads contínuos)
    #[default]
    SustainedSpeed,
}

impl NNAPIBackend {
    /// Cria novo backend NNAPI
    pub fn new() -> Option<Self> {
        #[cfg(target_os = "android")]
        {
            let api_level = Self::detect_api_level();
            if api_level < 27 {
                return None; // NNAPI requer API 27+
            }

            let devices = Self::enumerate_devices();

            Some(Self {
                api_level,
                devices,
                preference: ExecutionPreference::default(),
            })
        }

        #[cfg(not(target_os = "android"))]
        {
            None
        }
    }

    /// Cria backend com preferência específica
    pub fn with_preference(preference: ExecutionPreference) -> Option<Self> {
        let mut backend = Self::new()?;
        backend.preference = preference;
        Some(backend)
    }

    /// Detecta API level do Android
    #[cfg(target_os = "android")]
    fn detect_api_level() -> u32 {
        // Em produção, usaria android_get_device_api_level() via NDK
        // Por agora, assume API 30+ (Android 11+)
        30
    }

    #[cfg(not(target_os = "android"))]
    fn detect_api_level() -> u32 {
        0
    }

    /// Enumera dispositivos NNAPI disponíveis
    #[cfg(target_os = "android")]
    fn enumerate_devices() -> Vec<NNAPIDevice> {
        // Em produção, usaria ANeuralNetworks_getDeviceCount + getDevice
        // Por agora, retorna lista simulada
        vec![
            NNAPIDevice {
                name: "nnapi-reference".to_string(),
                device_type: DeviceType::Cpu,
                version: "1.0".to_string(),
                feature_level: 5,
            }
        ]
    }

    #[cfg(not(target_os = "android"))]
    fn enumerate_devices() -> Vec<NNAPIDevice> {
        vec![]
    }

    /// API level atual
    pub fn api_level(&self) -> u32 {
        self.api_level
    }

    /// Dispositivos disponíveis
    pub fn devices(&self) -> &[NNAPIDevice] {
        &self.devices
    }

    /// Verifica se tem acelerador dedicado (NPU/DSP)
    pub fn has_accelerator(&self) -> bool {
        self.devices.iter().any(|d| {
            matches!(d.device_type, DeviceType::Accelerator | DeviceType::Dsp)
        })
    }

    /// Define preferência de execução
    pub fn set_preference(&mut self, preference: ExecutionPreference) {
        self.preference = preference;
    }

    /// Preferência atual
    pub fn preference(&self) -> ExecutionPreference {
        self.preference
    }

    /// Informações detalhadas
    pub fn info(&self) -> NNAPIInfo {
        NNAPIInfo {
            api_level: self.api_level,
            device_count: self.devices.len(),
            has_accelerator: self.has_accelerator(),
            preference: self.preference,
        }
    }
}

/// Informações do backend NNAPI
#[derive(Debug, Clone)]
pub struct NNAPIInfo {
    pub api_level: u32,
    pub device_count: usize,
    pub has_accelerator: bool,
    pub preference: ExecutionPreference,
}

impl Default for NNAPIBackend {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            api_level: 0,
            devices: vec![],
            preference: ExecutionPreference::default(),
        })
    }
}

impl NpuBackendImpl for NNAPIBackend {
    fn name(&self) -> &str {
        if self.has_accelerator() {
            "NNAPI (NPU)"
        } else if self.devices.iter().any(|d| d.device_type == DeviceType::Gpu) {
            "NNAPI (GPU)"
        } else {
            "NNAPI (CPU)"
        }
    }

    fn is_available(&self) -> bool {
        #[cfg(target_os = "android")]
        {
            self.api_level >= 27
        }

        #[cfg(not(target_os = "android"))]
        {
            false
        }
    }

    fn version(&self) -> Option<String> {
        if self.api_level > 0 {
            Some(format!("API {}", self.api_level))
        } else {
            None
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NNAPI RUNTIME (Android only)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "android")]
pub mod runtime {
    //! Runtime NNAPI via FFI
    //!
    //! Bindings para a API C do Android Neural Networks.

    use super::*;
    use crate::processors::npu::{NpuError, NpuResult, NpuTensor};
    use std::ptr;

    // NNAPI types (from NeuralNetworks.h)
    type ANeuralNetworksModel = std::ffi::c_void;
    type ANeuralNetworksCompilation = std::ffi::c_void;
    type ANeuralNetworksExecution = std::ffi::c_void;

    // NNAPI result codes
    const ANEURALNETWORKS_NO_ERROR: i32 = 0;

    // Operand types
    const ANEURALNETWORKS_FLOAT32: i32 = 0;
    const ANEURALNETWORKS_TENSOR_FLOAT32: i32 = 3;

    // Execution preferences
    const ANEURALNETWORKS_PREFER_LOW_POWER: i32 = 0;
    const ANEURALNETWORKS_PREFER_FAST_SINGLE_ANSWER: i32 = 1;
    const ANEURALNETWORKS_PREFER_SUSTAINED_SPEED: i32 = 2;

    #[link(name = "neuralnetworks")]
    extern "C" {
        fn ANeuralNetworksModel_create(model: *mut *mut ANeuralNetworksModel) -> i32;
        fn ANeuralNetworksModel_free(model: *mut ANeuralNetworksModel);
        fn ANeuralNetworksModel_finish(model: *mut ANeuralNetworksModel) -> i32;

        fn ANeuralNetworksCompilation_create(
            model: *mut ANeuralNetworksModel,
            compilation: *mut *mut ANeuralNetworksCompilation,
        ) -> i32;
        fn ANeuralNetworksCompilation_free(compilation: *mut ANeuralNetworksCompilation);
        fn ANeuralNetworksCompilation_setPreference(
            compilation: *mut ANeuralNetworksCompilation,
            preference: i32,
        ) -> i32;
        fn ANeuralNetworksCompilation_finish(compilation: *mut ANeuralNetworksCompilation) -> i32;

        fn ANeuralNetworksExecution_create(
            compilation: *mut ANeuralNetworksCompilation,
            execution: *mut *mut ANeuralNetworksExecution,
        ) -> i32;
        fn ANeuralNetworksExecution_free(execution: *mut ANeuralNetworksExecution);
        fn ANeuralNetworksExecution_setInput(
            execution: *mut ANeuralNetworksExecution,
            index: i32,
            type_: *const std::ffi::c_void,
            buffer: *const std::ffi::c_void,
            length: usize,
        ) -> i32;
        fn ANeuralNetworksExecution_setOutput(
            execution: *mut ANeuralNetworksExecution,
            index: i32,
            type_: *const std::ffi::c_void,
            buffer: *mut std::ffi::c_void,
            length: usize,
        ) -> i32;
        fn ANeuralNetworksExecution_compute(execution: *mut ANeuralNetworksExecution) -> i32;
    }

    /// Modelo NNAPI compilado
    pub struct NNAPIModel {
        model: *mut ANeuralNetworksModel,
        compilation: *mut ANeuralNetworksCompilation,
        input_size: usize,
        output_size: usize,
    }

    impl NNAPIModel {
        /// Cria modelo simples (matmul)
        pub fn create_matmul(input_size: usize, output_size: usize, preference: ExecutionPreference) -> NpuResult<Self> {
            unsafe {
                let mut model: *mut ANeuralNetworksModel = ptr::null_mut();

                if ANeuralNetworksModel_create(&mut model) != ANEURALNETWORKS_NO_ERROR {
                    return Err(NpuError::InferenceError("Falha ao criar modelo".to_string()));
                }

                // TODO: Adicionar operandos e operações
                // Por agora, apenas cria modelo vazio

                if ANeuralNetworksModel_finish(model) != ANEURALNETWORKS_NO_ERROR {
                    ANeuralNetworksModel_free(model);
                    return Err(NpuError::InferenceError("Falha ao finalizar modelo".to_string()));
                }

                // Compila
                let mut compilation: *mut ANeuralNetworksCompilation = ptr::null_mut();
                if ANeuralNetworksCompilation_create(model, &mut compilation) != ANEURALNETWORKS_NO_ERROR {
                    ANeuralNetworksModel_free(model);
                    return Err(NpuError::InferenceError("Falha ao criar compilação".to_string()));
                }

                let pref = match preference {
                    ExecutionPreference::LowPower => ANEURALNETWORKS_PREFER_LOW_POWER,
                    ExecutionPreference::FastSingleAnswer => ANEURALNETWORKS_PREFER_FAST_SINGLE_ANSWER,
                    ExecutionPreference::SustainedSpeed => ANEURALNETWORKS_PREFER_SUSTAINED_SPEED,
                };
                ANeuralNetworksCompilation_setPreference(compilation, pref);

                if ANeuralNetworksCompilation_finish(compilation) != ANEURALNETWORKS_NO_ERROR {
                    ANeuralNetworksCompilation_free(compilation);
                    ANeuralNetworksModel_free(model);
                    return Err(NpuError::InferenceError("Falha ao compilar".to_string()));
                }

                Ok(Self {
                    model,
                    compilation,
                    input_size,
                    output_size,
                })
            }
        }

        /// Executa inferência
        pub fn predict(&self, input: &NpuTensor) -> NpuResult<NpuTensor> {
            let input_data = input.as_f32();
            let mut output_data = vec![0.0f32; self.output_size];

            unsafe {
                let mut execution: *mut ANeuralNetworksExecution = ptr::null_mut();

                if ANeuralNetworksExecution_create(self.compilation, &mut execution) != ANEURALNETWORKS_NO_ERROR {
                    return Err(NpuError::InferenceError("Falha ao criar execução".to_string()));
                }

                // Set input
                if ANeuralNetworksExecution_setInput(
                    execution,
                    0,
                    ptr::null(),
                    input_data.as_ptr() as *const _,
                    input_data.len() * std::mem::size_of::<f32>(),
                ) != ANEURALNETWORKS_NO_ERROR {
                    ANeuralNetworksExecution_free(execution);
                    return Err(NpuError::InferenceError("Falha ao definir input".to_string()));
                }

                // Set output
                if ANeuralNetworksExecution_setOutput(
                    execution,
                    0,
                    ptr::null(),
                    output_data.as_mut_ptr() as *mut _,
                    output_data.len() * std::mem::size_of::<f32>(),
                ) != ANEURALNETWORKS_NO_ERROR {
                    ANeuralNetworksExecution_free(execution);
                    return Err(NpuError::InferenceError("Falha ao definir output".to_string()));
                }

                // Execute
                if ANeuralNetworksExecution_compute(execution) != ANEURALNETWORKS_NO_ERROR {
                    ANeuralNetworksExecution_free(execution);
                    return Err(NpuError::InferenceError("Falha na execução".to_string()));
                }

                ANeuralNetworksExecution_free(execution);
            }

            Ok(NpuTensor::from_f32(&output_data, &[1, self.output_size]))
        }
    }

    impl Drop for NNAPIModel {
        fn drop(&mut self) {
            unsafe {
                if !self.compilation.is_null() {
                    ANeuralNetworksCompilation_free(self.compilation);
                }
                if !self.model.is_null() {
                    ANeuralNetworksModel_free(self.model);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nnapi_backend_creation() {
        // Em ambiente não-Android, deve retornar None
        #[cfg(not(target_os = "android"))]
        {
            let backend = NNAPIBackend::new();
            assert!(backend.is_none());
        }

        // Teste o default
        let backend = NNAPIBackend::default();
        assert!(!backend.is_available());
    }

    #[test]
    fn test_execution_preference() {
        let mut backend = NNAPIBackend::default();

        backend.set_preference(ExecutionPreference::LowPower);
        assert_eq!(backend.preference(), ExecutionPreference::LowPower);

        backend.set_preference(ExecutionPreference::FastSingleAnswer);
        assert_eq!(backend.preference(), ExecutionPreference::FastSingleAnswer);
    }

    #[test]
    fn test_nnapi_info() {
        let backend = NNAPIBackend::default();
        let info = backend.info();

        #[cfg(not(target_os = "android"))]
        assert_eq!(info.api_level, 0);
    }
}
