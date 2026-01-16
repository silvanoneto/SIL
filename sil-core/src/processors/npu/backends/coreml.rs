//! Core ML Backend (macOS/iOS)
//!
//! Implementação do backend Core ML para Apple Neural Engine.
//!
//! ## Requisitos
//!
//! - macOS 10.13+ ou iOS 11+
//! - Para ANE (Neural Engine): A11+ chip (iPhone X+, M1+)
//!
//! ## Formatos Suportados
//!
//! - .mlmodel (Core ML Model)
//! - .mlpackage (Core ML Package)
//! - Conversão de ONNX via coremltools
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                     CoreMLBackend                        │
//! │  ┌─────────────────────────────────────────────────────┐ │
//! │  │ MLModel (Objective-C via objc2)                     │ │
//! │  │  - predictionFromFeatures:options:error:            │ │
//! │  │  - modelDescription                                  │ │
//! │  └─────────────────────────────────────────────────────┘ │
//! │  ┌─────────────────────────────────────────────────────┐ │
//! │  │ MLMultiArray                                        │ │
//! │  │  - SilState → [1, 16, 2] tensor                     │ │
//! │  │  - Quantização FP16/INT8                            │ │
//! │  └─────────────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────┘
//! ```

use super::NpuBackendImpl;

/// Backend Core ML
pub struct CoreMLBackend {
    /// Versão do Core ML
    version: String,
    /// ANE disponível
    ane_available: bool,
    /// Compute units preferidos
    compute_units: ComputeUnits,
}

/// Unidades de computação para Core ML
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComputeUnits {
    /// Apenas CPU
    CpuOnly,
    /// CPU + GPU
    CpuAndGpu,
    /// Tudo (CPU + GPU + ANE)
    #[default]
    All,
    /// Apenas CPU e Neural Engine (melhor eficiência energética)
    CpuAndNeuralEngine,
}

impl CoreMLBackend {
    /// Cria novo backend Core ML
    pub fn new() -> Option<Self> {
        let ane_available = Self::check_ane();
        let compute_units = if ane_available {
            ComputeUnits::All
        } else {
            ComputeUnits::CpuAndGpu
        };

        Some(Self {
            version: Self::detect_version(),
            ane_available,
            compute_units,
        })
    }

    /// Cria backend com unidades de computação específicas
    pub fn with_compute_units(units: ComputeUnits) -> Option<Self> {
        let ane_available = Self::check_ane();

        // Se pediu ANE mas não está disponível, falha
        if matches!(units, ComputeUnits::All | ComputeUnits::CpuAndNeuralEngine) && !ane_available {
            return None;
        }

        Some(Self {
            version: Self::detect_version(),
            ane_available,
            compute_units: units,
        })
    }

    /// Detecta versão do Core ML
    fn detect_version() -> String {
        // Core ML versions by macOS:
        // - macOS 10.13 (High Sierra): Core ML 1
        // - macOS 10.14 (Mojave): Core ML 2
        // - macOS 10.15 (Catalina): Core ML 3
        // - macOS 11 (Big Sur): Core ML 4
        // - macOS 12 (Monterey): Core ML 5
        // - macOS 13 (Ventura): Core ML 6
        // - macOS 14 (Sonoma): Core ML 7
        // - macOS 15 (Sequoia): Core ML 8

        #[cfg(target_os = "macos")]
        {
            // Usa sysctlbyname para detectar versão do macOS
            use std::ffi::CStr;
            use std::os::raw::{c_char, c_int, c_void};

            unsafe extern "C" {
                fn sysctlbyname(
                    name: *const c_char,
                    oldp: *mut c_void,
                    oldlenp: *mut usize,
                    newp: *mut c_void,
                    newlen: usize,
                ) -> c_int;
            }

            let name = c"kern.osproductversion";
            let mut version = [0u8; 32];
            let mut len = version.len();

            unsafe {
                if sysctlbyname(
                    name.as_ptr(),
                    version.as_mut_ptr() as *mut c_void,
                    &mut len,
                    std::ptr::null_mut(),
                    0,
                ) == 0 {
                    if let Ok(s) = CStr::from_bytes_until_nul(&version) {
                        let os_version = s.to_string_lossy();
                        // Parse major version
                        if let Some(major) = os_version.split('.').next().and_then(|v| v.parse::<u32>().ok()) {
                            return match major {
                                10 => "3.0".to_string(), // Catalina (10.15)
                                11 => "4.0".to_string(), // Big Sur
                                12 => "5.0".to_string(), // Monterey
                                13 => "6.0".to_string(), // Ventura
                                14 => "7.0".to_string(), // Sonoma
                                15 => "8.0".to_string(), // Sequoia
                                _ => format!("{}.0", major.saturating_sub(7)), // Future versions
                            };
                        }
                    }
                }
            }
        }

        "5.0".to_string() // Default
    }

    /// Verifica disponibilidade do Apple Neural Engine
    fn check_ane() -> bool {
        // ANE disponível em:
        // - iPhone X+ (A11+)
        // - iPad Pro 2018+ (A12X+)
        // - Mac M1+

        #[cfg(target_arch = "aarch64")]
        {
            #[cfg(target_os = "macos")]
            {
                // Detecta se é Apple Silicon via sysctl
                use std::os::raw::{c_char, c_int, c_void};

                unsafe extern "C" {
                    fn sysctlbyname(
                        name: *const c_char,
                        oldp: *mut c_void,
                        oldlenp: *mut usize,
                        newp: *mut c_void,
                        newlen: usize,
                    ) -> c_int;
                }

                let name = c"hw.optional.arm64";
                let mut value: i32 = 0;
                let mut len = std::mem::size_of::<i32>();

                unsafe {
                    if sysctlbyname(
                        name.as_ptr(),
                        &mut value as *mut i32 as *mut c_void,
                        &mut len,
                        std::ptr::null_mut(),
                        0,
                    ) == 0 {
                        return value == 1;
                    }
                }

                // Fallback: se estamos rodando em aarch64 macOS, provavelmente é Apple Silicon
                true
            }

            #[cfg(target_os = "ios")]
            {
                // Em iOS aarch64, sempre tem ANE (A11+)
                true
            }

            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            {
                false
            }
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            // Intel Mac não tem ANE
            false
        }
    }

    /// ANE está disponível?
    pub fn has_ane(&self) -> bool {
        self.ane_available
    }

    /// Retorna as unidades de computação configuradas
    pub fn compute_units(&self) -> ComputeUnits {
        self.compute_units
    }

    /// Define unidades de computação
    pub fn set_compute_units(&mut self, units: ComputeUnits) {
        self.compute_units = units;
    }

    /// Versão do Core ML
    pub fn coreml_version(&self) -> &str {
        &self.version
    }

    /// Informações detalhadas do backend
    pub fn info(&self) -> CoreMLInfo {
        CoreMLInfo {
            version: self.version.clone(),
            ane_available: self.ane_available,
            compute_units: self.compute_units,
            metal_available: Self::check_metal(),
        }
    }

    /// Verifica se Metal está disponível
    fn check_metal() -> bool {
        #[cfg(target_os = "macos")]
        {
            // Metal disponível em macOS 10.11+ com GPU compatível
            true
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

/// Informações do backend Core ML
#[derive(Debug, Clone)]
pub struct CoreMLInfo {
    pub version: String,
    pub ane_available: bool,
    pub compute_units: ComputeUnits,
    pub metal_available: bool,
}

impl Default for CoreMLBackend {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            version: "unknown".to_string(),
            ane_available: false,
            compute_units: ComputeUnits::CpuOnly,
        })
    }
}

impl NpuBackendImpl for CoreMLBackend {
    fn name(&self) -> &str {
        match (self.ane_available, self.compute_units) {
            (true, ComputeUnits::All) => "Core ML (ANE+GPU+CPU)",
            (true, ComputeUnits::CpuAndNeuralEngine) => "Core ML (ANE+CPU)",
            (_, ComputeUnits::CpuAndGpu) => "Core ML (GPU+CPU)",
            (_, ComputeUnits::CpuOnly) => "Core ML (CPU)",
            _ => "Core ML",
        }
    }

    fn is_available(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            true // Core ML sempre disponível em macOS 10.13+
        }

        #[cfg(target_os = "ios")]
        {
            true // Core ML sempre disponível em iOS 11+
        }

        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            false
        }
    }

    fn version(&self) -> Option<String> {
        Some(self.version.clone())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE ML MODEL WRAPPER (quando feature npu está habilitada)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(all(target_os = "macos", feature = "npu"))]
pub mod runtime {
    //! Runtime Core ML via objc2
    //!
    //! Fornece wrappers seguros para a API Objective-C do Core ML.

    use crate::processors::npu::{NpuError, NpuResult, NpuTensor};
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;
    use std::path::Path;
    use std::ptr;

    /// Wrapper para MLModel
    pub struct MLModelWrapper {
        inner: Retained<AnyObject>,
        input_name: String,
        output_name: String,
    }

    impl MLModelWrapper {
        /// Carrega modelo de arquivo
        pub fn load(path: &Path) -> NpuResult<Self> {
            let path_str = path.to_string_lossy();

            unsafe {
                // Cria NSURL
                let url_string = NSString::from_str(&path_str);
                let url: *mut AnyObject = msg_send![
                    class!(NSURL),
                    fileURLWithPath: &*url_string
                ];

                if url.is_null() {
                    return Err(NpuError::InvalidModel("Caminho inválido".to_string()));
                }

                // Carrega modelo
                let mut error: *mut AnyObject = ptr::null_mut();
                let model: *mut AnyObject = msg_send![
                    class!(MLModel),
                    modelWithContentsOfURL: url,
                    error: &mut error
                ];

                if model.is_null() || !error.is_null() {
                    let error_msg = if !error.is_null() {
                        let desc: *mut AnyObject = msg_send![error, localizedDescription];
                        if !desc.is_null() {
                            let ns_str: &NSString = &*(desc as *const NSString);
                            ns_str.to_string()
                        } else {
                            "Erro desconhecido".to_string()
                        }
                    } else {
                        "Falha ao carregar modelo".to_string()
                    };
                    return Err(NpuError::InvalidModel(error_msg));
                }

                // Obtém nomes de input/output do modelo
                let (input_name, output_name) = Self::get_io_names(model)?;

                Ok(Self {
                    inner: Retained::from_raw(model).unwrap(),
                    input_name,
                    output_name,
                })
            }
        }

        /// Obtém nomes de input/output do modelo
        unsafe fn get_io_names(model: *mut AnyObject) -> NpuResult<(String, String)> {
            let description: *mut AnyObject = msg_send![model, modelDescription];
            if description.is_null() {
                return Err(NpuError::InvalidModel("Sem descrição de modelo".to_string()));
            }

            // Input
            let input_desc: *mut AnyObject = msg_send![description, inputDescriptionsByName];
            let input_keys: *mut AnyObject = msg_send![input_desc, allKeys];
            let input_count: usize = msg_send![input_keys, count];

            let input_name = if input_count > 0 {
                let first_key: *mut AnyObject = msg_send![input_keys, objectAtIndex: 0usize];
                let ns_str: &NSString = unsafe { &*(first_key as *const NSString) };
                ns_str.to_string()
            } else {
                "input".to_string()
            };

            // Output
            let output_desc: *mut AnyObject = msg_send![description, outputDescriptionsByName];
            let output_keys: *mut AnyObject = msg_send![output_desc, allKeys];
            let output_count: usize = msg_send![output_keys, count];

            let output_name = if output_count > 0 {
                let first_key: *mut AnyObject = msg_send![output_keys, objectAtIndex: 0usize];
                let ns_str: &NSString = unsafe { &*(first_key as *const NSString) };
                ns_str.to_string()
            } else {
                "output".to_string()
            };

            Ok((input_name, output_name))
        }

        /// Executa inferência
        pub fn predict(&self, input: &NpuTensor) -> NpuResult<NpuTensor> {
            let input_data = input.as_f32();
            let shape = input.shape();

            unsafe {
                // Cria MLMultiArray para input
                let multi_array = Self::create_multi_array(&input_data, shape)?;

                // Cria feature provider
                let input_name = NSString::from_str(&self.input_name);
                let feature_value: *mut AnyObject = msg_send![
                    class!(MLFeatureValue),
                    featureValueWithMultiArray: multi_array
                ];

                let keys: *mut AnyObject = msg_send![
                    class!(NSArray),
                    arrayWithObject: &*input_name
                ];
                let values: *mut AnyObject = msg_send![
                    class!(NSArray),
                    arrayWithObject: feature_value
                ];
                let provider: *mut AnyObject = msg_send![
                    class!(NSDictionary),
                    dictionaryWithObjects: values,
                    forKeys: keys
                ];

                // Cria MLDictionaryFeatureProvider
                let mut error: *mut AnyObject = ptr::null_mut();
                let feature_provider: *mut AnyObject = msg_send![
                    class!(MLDictionaryFeatureProvider),
                    alloc
                ];
                let feature_provider: *mut AnyObject = msg_send![
                    feature_provider,
                    initWithDictionary: provider,
                    error: &mut error
                ];

                if feature_provider.is_null() {
                    return Err(NpuError::InferenceError("Falha ao criar feature provider".to_string()));
                }

                // Executa predição
                let mut pred_error: *mut AnyObject = ptr::null_mut();
                let prediction: *mut AnyObject = msg_send![
                    &*self.inner,
                    predictionFromFeatures: feature_provider,
                    error: &mut pred_error
                ];

                if prediction.is_null() || !pred_error.is_null() {
                    let error_msg = if !pred_error.is_null() {
                        let desc: *mut AnyObject = msg_send![pred_error, localizedDescription];
                        if !desc.is_null() {
                            let ns_str: &NSString = &*(desc as *const NSString);
                            ns_str.to_string()
                        } else {
                            "Erro de inferência".to_string()
                        }
                    } else {
                        "Predição retornou null".to_string()
                    };
                    return Err(NpuError::InferenceError(error_msg));
                }

                // Extrai output
                let output_name = NSString::from_str(&self.output_name);
                let output_feature: *mut AnyObject = msg_send![
                    prediction,
                    featureValueForName: &*output_name
                ];

                if output_feature.is_null() {
                    return Err(NpuError::InferenceError("Output não encontrado".to_string()));
                }

                let output_array: *mut AnyObject = msg_send![output_feature, multiArrayValue];
                Self::multi_array_to_tensor(output_array)
            }
        }

        /// Cria MLMultiArray a partir de dados f32
        unsafe fn create_multi_array(data: &[f32], shape: &[usize]) -> NpuResult<*mut AnyObject> {
            // Cria NSArray de dimensões
            let dims: Vec<*mut AnyObject> = shape.iter().map(|&d| {
                let num: *mut AnyObject = msg_send![
                    class!(NSNumber),
                    numberWithUnsignedInteger: d
                ];
                num
            }).collect();

            let dims_array: *mut AnyObject = msg_send![
                class!(NSArray),
                arrayWithObjects: dims.as_ptr(),
                count: dims.len()
            ];

            // Cria MLMultiArray
            let mut error: *mut AnyObject = ptr::null_mut();
            let array: *mut AnyObject = msg_send![
                class!(MLMultiArray),
                alloc
            ];

            // MLMultiArrayDataTypeFloat32 = 65568
            let dtype: i32 = 65568;

            let array: *mut AnyObject = msg_send![
                array,
                initWithShape: dims_array,
                dataType: dtype,
                error: &mut error
            ];

            if array.is_null() || !error.is_null() {
                return Err(NpuError::InferenceError("Falha ao criar MLMultiArray".to_string()));
            }

            // Copia dados
            let data_ptr: *mut f32 = msg_send![array, dataPointer];
            if !data_ptr.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len()) };
            }

            Ok(array)
        }

        /// Converte MLMultiArray para NpuTensor
        unsafe fn multi_array_to_tensor(array: *mut AnyObject) -> NpuResult<NpuTensor> {
            if array.is_null() {
                return Err(NpuError::InferenceError("Array nulo".to_string()));
            }

            // Obtém shape
            let shape_ns: *mut AnyObject = msg_send![array, shape];
            let shape_count: usize = msg_send![shape_ns, count];

            let mut shape = Vec::with_capacity(shape_count);
            for i in 0..shape_count {
                let num: *mut AnyObject = msg_send![shape_ns, objectAtIndex: i];
                let dim: usize = msg_send![num, unsignedIntegerValue];
                shape.push(dim);
            }

            // Obtém dados
            let total: usize = shape.iter().product();
            let data_ptr: *const f32 = msg_send![array, dataPointer];

            let data = if !data_ptr.is_null() {
                unsafe { std::slice::from_raw_parts(data_ptr, total) }.to_vec()
            } else {
                vec![0.0; total]
            };

            Ok(NpuTensor::from_f32(&data, &shape))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coreml_backend() {
        let backend = CoreMLBackend::default();
        println!("Core ML: {}", backend.name());
        println!("Version: {}", backend.coreml_version());
        println!("ANE: {}", backend.has_ane());

        #[cfg(target_os = "macos")]
        assert!(backend.is_available());
    }

    #[test]
    fn test_compute_units() {
        let backend = CoreMLBackend::with_compute_units(ComputeUnits::CpuOnly);
        assert!(backend.is_some());

        let backend = backend.unwrap();
        assert_eq!(backend.compute_units(), ComputeUnits::CpuOnly);
    }

    #[test]
    fn test_coreml_info() {
        let backend = CoreMLBackend::default();
        let info = backend.info();

        println!("CoreML Info: {:?}", info);
        assert!(!info.version.is_empty());
    }
}
