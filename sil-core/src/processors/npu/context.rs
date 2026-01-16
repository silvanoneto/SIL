//! Contexto NPU — Inicialização e gerenciamento

use super::{NpuError, NpuResult, NpuBackend, NpuModel, NpuTensor, InferenceResult};
use crate::state::SilState;
use crate::processors::{Processor, ProcessorCapability, InferenceProcessor};
use std::path::Path;
use std::time::Instant;

/// Contexto de processamento NPU
pub struct NpuContext {
    /// Backend ativo
    backend: NpuBackend,
    /// Pronto para inferência
    ready: bool,
    /// Precisão padrão
    default_precision: Precision,
}

/// Precisão de inferência
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Precision {
    /// Float32 (maior precisão, mais lento)
    FP32,
    /// Float16 (bom equilíbrio)
    #[default]
    FP16,
    /// Int8 quantizado (mais rápido, menor precisão)
    INT8,
    /// Int4 quantizado (experimental)
    INT4,
}

impl NpuContext {
    /// Cria novo contexto NPU
    pub fn new() -> NpuResult<Self> {
        let backend = NpuBackend::detect();
        
        // Verifica se o backend está realmente disponível
        let ready = Self::check_backend_availability(&backend);
        
        if !ready && !matches!(backend, NpuBackend::CpuFallback) {
            // Fallback para CPU se backend nativo não disponível
            return Ok(Self {
                backend: NpuBackend::CpuFallback,
                ready: true,
                default_precision: Precision::FP32,
            });
        }
        
        Ok(Self {
            backend,
            ready,
            default_precision: Precision::FP16,
        })
    }
    
    /// Cria contexto com backend específico
    pub fn with_backend(backend: NpuBackend) -> NpuResult<Self> {
        let ready = Self::check_backend_availability(&backend);
        
        if !ready {
            return Err(NpuError::UnsupportedBackend(backend.name().to_string()));
        }
        
        Ok(Self {
            backend,
            ready,
            default_precision: Precision::FP16,
        })
    }
    
    /// Verifica se NPU está disponível
    pub fn is_available() -> bool {
        let backend = NpuBackend::detect();
        Self::check_backend_availability(&backend)
    }
    
    fn check_backend_availability(backend: &NpuBackend) -> bool {
        match backend {
            NpuBackend::CoreML => {
                #[cfg(target_os = "macos")]
                {
                    // Core ML sempre disponível em macOS 10.13+
                    true
                }
                #[cfg(not(target_os = "macos"))]
                {
                    false
                }
            }
            NpuBackend::NNAPI => {
                #[cfg(target_os = "android")]
                {
                    // Verificar versão do Android (API 27+)
                    true
                }
                #[cfg(not(target_os = "android"))]
                {
                    false
                }
            }
            NpuBackend::DirectML => {
                #[cfg(target_os = "windows")]
                {
                    // DirectML requer Windows 10 1903+
                    true
                }
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            }
            NpuBackend::OpenVINO | NpuBackend::TensorRT => {
                // Requer bibliotecas instaladas
                false
            }
            NpuBackend::CpuFallback => true,
        }
    }
    
    /// Backend ativo
    pub fn backend(&self) -> NpuBackend {
        self.backend
    }
    
    /// Define precisão padrão
    pub fn set_precision(&mut self, precision: Precision) {
        self.default_precision = precision;
    }
    
    /// Carrega modelo de arquivo
    pub fn load_model(&self, path: &Path) -> NpuResult<NpuModel> {
        NpuModel::load(path, self.backend)
    }
    
    /// Executa inferência em SilState
    pub fn infer(&self, model: &NpuModel, state: &SilState) -> NpuResult<InferenceResult> {
        let start = Instant::now();
        
        // Converter estado para tensor
        let input = NpuTensor::from_state(state, self.default_precision);
        
        // Executar inferência
        let output = self.run_inference(model, &input)?;
        
        let latency_us = start.elapsed().as_micros() as u64;
        
        Ok(InferenceResult {
            output,
            latency_us,
            backend: self.backend,
        })
    }
    
    /// Executa inferência em batch
    pub fn infer_batch(&self, model: &NpuModel, states: &[SilState]) -> NpuResult<Vec<InferenceResult>> {
        let start = Instant::now();
        
        // Converter estados para tensor batch
        let input = NpuTensor::from_states(states, self.default_precision);
        
        // Executar inferência
        let output = self.run_inference(model, &input)?;
        
        let latency_us = start.elapsed().as_micros() as u64;
        let per_item_latency = latency_us / states.len().max(1) as u64;
        
        // Dividir output por batch
        let outputs = output.split_batch(states.len())?;
        
        Ok(outputs.into_iter().map(|o| InferenceResult {
            output: o,
            latency_us: per_item_latency,
            backend: self.backend,
        }).collect())
    }
    
    fn run_inference(&self, model: &NpuModel, input: &NpuTensor) -> NpuResult<NpuTensor> {
        match self.backend {
            NpuBackend::CoreML => self.run_coreml(model, input),
            NpuBackend::CpuFallback => self.run_cpu_fallback(model, input),
            _ => Err(NpuError::UnsupportedBackend(self.backend.name().to_string())),
        }
    }
    
    #[cfg(target_os = "macos")]
    fn run_coreml(&self, model: &NpuModel, input: &NpuTensor) -> NpuResult<NpuTensor> {
        // TODO: Implementar via objc2 / core-ml bindings
        // Por agora, usa fallback
        self.run_cpu_fallback(model, input)
    }
    
    #[cfg(not(target_os = "macos"))]
    fn run_coreml(&self, _model: &NpuModel, _input: &NpuTensor) -> NpuResult<NpuTensor> {
        Err(NpuError::UnsupportedBackend("CoreML".to_string()))
    }
    
    fn run_cpu_fallback(&self, model: &NpuModel, input: &NpuTensor) -> NpuResult<NpuTensor> {
        // Implementação CPU básica para testes
        // Em produção, usaria SIMD otimizado
        
        let input_data = input.as_f32();
        let weights = model.weights_f32();
        
        // Simples matmul: output = input × weights
        let output_size = model.output_size();
        let mut output = vec![0.0f32; output_size];
        
        // Naive matmul (seria substituído por BLAS/SIMD)
        for (i, out) in output.iter_mut().enumerate() {
            for (j, &inp) in input_data.iter().enumerate() {
                if let Some(&w) = weights.get(j * output_size + i) {
                    *out += inp * w;
                }
            }
        }
        
        Ok(NpuTensor::from_f32(&output, &[1, output_size]))
    }
}

impl Default for NpuContext {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            backend: NpuBackend::CpuFallback,
            ready: true,
            default_precision: Precision::FP32,
        })
    }
}

impl Processor for NpuContext {
    type Error = NpuError;
    
    fn name(&self) -> &str {
        self.backend.name()
    }
    
    fn is_ready(&self) -> bool {
        self.ready
    }
    
    fn capabilities(&self) -> &[ProcessorCapability] {
        &[
            ProcessorCapability::Inference,
            ProcessorCapability::Quantization,
            ProcessorCapability::MatrixOps,
        ]
    }
}

impl InferenceProcessor for NpuContext {
    type Model = NpuModel;
    type Input = SilState;
    type Output = InferenceResult;
    
    fn load_model(&self, path: &Path) -> Result<Self::Model, Self::Error> {
        self.load_model(path)
    }
    
    fn infer(&self, model: &Self::Model, input: &Self::Input) -> Result<Self::Output, Self::Error> {
        self.infer(model, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_npu_context_creation() {
        let ctx = NpuContext::new().unwrap();
        println!("NPU Backend: {}", ctx.backend().name());
        assert!(ctx.is_ready());
    }
    
    #[test]
    fn test_npu_availability() {
        let available = NpuContext::is_available();
        println!("NPU disponível: {}", available);
    }
}
