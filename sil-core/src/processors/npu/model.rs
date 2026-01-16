//! Modelo NPU — Carregamento e representação

use super::{NpuError, NpuResult, NpuBackend};
use std::path::Path;
use std::fs;

/// Formato de modelo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    /// Apple Core ML (.mlmodel, .mlpackage)
    CoreML,
    /// ONNX (.onnx)
    ONNX,
    /// TensorFlow Lite (.tflite)
    TFLite,
    /// PyTorch TorchScript (.pt)
    TorchScript,
    /// OpenVINO IR (.xml + .bin)
    OpenVINO,
    /// TensorRT Engine (.engine, .plan)
    TensorRT,
    /// SIL Native (.silmodel)
    SilNative,
}

impl ModelFormat {
    /// Detecta formato pela extensão
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "mlmodel" | "mlpackage" => Some(Self::CoreML),
            "onnx" => Some(Self::ONNX),
            "tflite" => Some(Self::TFLite),
            "pt" | "pth" => Some(Self::TorchScript),
            "xml" => Some(Self::OpenVINO),
            "engine" | "plan" => Some(Self::TensorRT),
            "silmodel" => Some(Self::SilNative),
            _ => None,
        }
    }
    
    /// Backends compatíveis
    pub fn compatible_backends(&self) -> &[NpuBackend] {
        match self {
            Self::CoreML => &[NpuBackend::CoreML, NpuBackend::CpuFallback],
            Self::ONNX => &[
                NpuBackend::DirectML, 
                NpuBackend::OpenVINO, 
                NpuBackend::TensorRT,
                NpuBackend::CpuFallback,
            ],
            Self::TFLite => &[NpuBackend::NNAPI, NpuBackend::CpuFallback],
            Self::TorchScript => &[NpuBackend::CpuFallback],
            Self::OpenVINO => &[NpuBackend::OpenVINO, NpuBackend::CpuFallback],
            Self::TensorRT => &[NpuBackend::TensorRT, NpuBackend::CpuFallback],
            Self::SilNative => &[
                NpuBackend::CoreML,
                NpuBackend::NNAPI,
                NpuBackend::DirectML,
                NpuBackend::CpuFallback,
            ],
        }
    }
}

/// Modelo carregado para NPU
pub struct NpuModel {
    /// Nome do modelo
    pub name: String,
    /// Formato original
    pub format: ModelFormat,
    /// Shape de entrada [batch, channels, ...]
    pub input_shape: Vec<usize>,
    /// Shape de saída
    pub output_shape: Vec<usize>,
    /// Pesos (para CPU fallback)
    weights: Vec<f32>,
    /// Dados do modelo compilado (para NPU)
    compiled_data: Vec<u8>,
}

impl NpuModel {
    /// Carrega modelo de arquivo
    pub fn load(path: &Path, backend: NpuBackend) -> NpuResult<Self> {
        let format = ModelFormat::from_path(path)
            .ok_or_else(|| NpuError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ))?;
        
        // Verifica compatibilidade
        if !format.compatible_backends().contains(&backend) {
            return Err(NpuError::UnsupportedBackend(format!(
                "{} não suporta {}",
                backend.name(),
                path.display()
            )));
        }
        
        // Carrega dados do arquivo
        let data = fs::read(path)?;
        
        // Parse básico baseado no formato
        let (input_shape, output_shape, weights) = Self::parse_model(&data, format)?;
        
        Ok(Self {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("model")
                .to_string(),
            format,
            input_shape,
            output_shape,
            weights,
            compiled_data: data,
        })
    }
    
    /// Cria modelo SIL nativo para classificação de estados
    pub fn sil_classifier(num_classes: usize) -> Self {
        // Modelo simples: 32 inputs (16 camadas × 2) → num_classes outputs
        let input_size = 32;
        
        // Inicializa pesos aleatórios (em produção, seriam treinados)
        let weights = (0..input_size * num_classes)
            .map(|i| (i as f32 * 0.1).sin() * 0.5)
            .collect();
        
        Self {
            name: "sil_classifier".to_string(),
            format: ModelFormat::SilNative,
            input_shape: vec![1, input_size],
            output_shape: vec![1, num_classes],
            weights,
            compiled_data: vec![],
        }
    }
    
    /// Cria modelo SIL para embedding de estados
    pub fn sil_embedding(embedding_dim: usize) -> Self {
        let input_size = 32;
        
        let weights = (0..input_size * embedding_dim)
            .map(|i| (i as f32 * 0.05).cos() * 0.3)
            .collect();
        
        Self {
            name: "sil_embedding".to_string(),
            format: ModelFormat::SilNative,
            input_shape: vec![1, input_size],
            output_shape: vec![1, embedding_dim],
            weights,
            compiled_data: vec![],
        }
    }
    
    /// Cria modelo SIL para predição de próximo estado
    pub fn sil_predictor() -> Self {
        let input_size = 32;
        let output_size = 32; // Mesmo tamanho (prediz próximo estado)
        
        let weights = (0..input_size * output_size)
            .map(|i| {
                // Matriz quase-identidade com pequenas perturbações
                let row = i / output_size;
                let col = i % output_size;
                if row == col { 0.9 } else { 0.01 }
            })
            .collect();
        
        Self {
            name: "sil_predictor".to_string(),
            format: ModelFormat::SilNative,
            input_shape: vec![1, input_size],
            output_shape: vec![1, output_size],
            weights,
            compiled_data: vec![],
        }
    }
    
    fn parse_model(data: &[u8], format: ModelFormat) -> NpuResult<(Vec<usize>, Vec<usize>, Vec<f32>)> {
        match format {
            ModelFormat::SilNative => Self::parse_sil_native(data),
            ModelFormat::ONNX => Self::parse_onnx(data),
            _ => {
                // Fallback: assume modelo simples 32→16
                Ok((vec![1, 32], vec![1, 16], vec![0.0; 32 * 16]))
            }
        }
    }
    
    fn parse_sil_native(data: &[u8]) -> NpuResult<(Vec<usize>, Vec<usize>, Vec<f32>)> {
        // Formato SIL Native (simplificado):
        // [4 bytes: input_size] [4 bytes: output_size] [weights as f32...]
        
        if data.len() < 8 {
            return Err(NpuError::InvalidModel("Dados muito curtos".to_string()));
        }
        
        let input_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let output_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        
        let weights_bytes = &data[8..];
        let expected_len = input_size * output_size * 4;
        
        if weights_bytes.len() < expected_len {
            return Err(NpuError::InvalidModel(format!(
                "Pesos incompletos: esperado {}, encontrado {}",
                expected_len,
                weights_bytes.len()
            )));
        }
        
        let weights: Vec<f32> = weights_bytes
            .chunks(4)
            .take(input_size * output_size)
            .map(|chunk| {
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            })
            .collect();
        
        Ok((vec![1, input_size], vec![1, output_size], weights))
    }
    
    fn parse_onnx(_data: &[u8]) -> NpuResult<(Vec<usize>, Vec<usize>, Vec<f32>)> {
        // TODO: Implementar parser ONNX real
        // Por agora, retorna estrutura dummy
        Ok((vec![1, 32], vec![1, 16], vec![0.0; 32 * 16]))
    }
    
    /// Pesos como f32 (para CPU fallback)
    pub fn weights_f32(&self) -> &[f32] {
        &self.weights
    }
    
    /// Tamanho do output
    pub fn output_size(&self) -> usize {
        self.output_shape.iter().product()
    }
    
    /// Tamanho do input
    pub fn input_size(&self) -> usize {
        self.input_shape.iter().product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_format_detection() {
        assert_eq!(
            ModelFormat::from_path(Path::new("model.mlmodel")),
            Some(ModelFormat::CoreML)
        );
        assert_eq!(
            ModelFormat::from_path(Path::new("model.onnx")),
            Some(ModelFormat::ONNX)
        );
        assert_eq!(
            ModelFormat::from_path(Path::new("model.silmodel")),
            Some(ModelFormat::SilNative)
        );
    }
    
    #[test]
    fn test_sil_classifier() {
        let model = NpuModel::sil_classifier(10);
        assert_eq!(model.input_size(), 32);
        assert_eq!(model.output_size(), 10);
    }
    
    #[test]
    fn test_sil_predictor() {
        let model = NpuModel::sil_predictor();
        assert_eq!(model.input_size(), 32);
        assert_eq!(model.output_size(), 32);
    }
}
