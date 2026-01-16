//! Tensor NPU — Representação de dados para inferência

use super::{NpuError, NpuResult};
use crate::state::SilState;
use crate::processors::npu::context::Precision;

/// Layout de memória do tensor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TensorLayout {
    /// Row-major (C-style): [batch, channels, height, width]
    #[default]
    NCHW,
    /// Column-major: [batch, height, width, channels]
    NHWC,
    /// SIL: [batch, layers, (rho, theta)]
    SilState,
}

/// Tipo de dados do tensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Float32,
    Float16,
    Int8,
    Int4,
    UInt8,
}

/// Tensor para NPU
#[derive(Debug, Clone)]
pub struct NpuTensor {
    /// Dados em bytes
    data: Vec<u8>,
    /// Tipo de dados
    dtype: DataType,
    /// Shape do tensor
    shape: Vec<usize>,
    /// Layout de memória
    layout: TensorLayout,
}

impl NpuTensor {
    /// Cria tensor vazio
    pub fn new(shape: &[usize], dtype: DataType) -> Self {
        let size: usize = shape.iter().product();
        let byte_size = size * Self::dtype_size(dtype);
        
        Self {
            data: vec![0u8; byte_size],
            dtype,
            shape: shape.to_vec(),
            layout: TensorLayout::default(),
        }
    }
    
    /// Cria tensor a partir de SilState
    pub fn from_state(state: &SilState, precision: Precision) -> Self {
        let dtype = match precision {
            Precision::FP32 => DataType::Float32,
            Precision::FP16 => DataType::Float16,
            Precision::INT8 => DataType::Int8,
            Precision::INT4 => DataType::Int4,
        };
        
        // Shape: [1, 16, 2] = [batch, layers, (rho, theta)]
        let shape = vec![1, 16, 2];
        
        let data = match dtype {
            DataType::Float32 => {
                let floats: Vec<f32> = state.layers.iter()
                    .flat_map(|l| [l.rho as f32, l.theta as f32])
                    .collect();
                floats.iter()
                    .flat_map(|f| f.to_le_bytes())
                    .collect()
            }
            DataType::Float16 => {
                let halfs: Vec<u16> = state.layers.iter()
                    .flat_map(|l| {
                        let rho = half::f16::from_f32(l.rho as f32).to_bits();
                        let theta = half::f16::from_f32(l.theta as f32).to_bits();
                        [rho, theta]
                    })
                    .collect();
                halfs.iter()
                    .flat_map(|h| h.to_le_bytes())
                    .collect()
            }
            DataType::Int8 => {
                state.layers.iter()
                    .flat_map(|l| [l.rho as u8, l.theta])
                    .collect()
            }
            DataType::Int4 => {
                // Pack 2 valores em 1 byte
                state.layers.iter()
                    .map(|l| {
                        let rho_u4 = (l.rho as u8) & 0x0F;
                        let theta_u4 = l.theta & 0x0F;
                        (rho_u4 << 4) | theta_u4
                    })
                    .collect()
            }
            DataType::UInt8 => {
                state.layers.iter()
                    .flat_map(|l| [l.to_u8(), 0])
                    .collect()
            }
        };
        
        Self {
            data,
            dtype,
            shape,
            layout: TensorLayout::SilState,
        }
    }
    
    /// Cria tensor a partir de múltiplos estados (batch)
    pub fn from_states(states: &[SilState], precision: Precision) -> Self {
        if states.is_empty() {
            return Self::new(&[0, 16, 2], DataType::Float32);
        }
        
        let batch_size = states.len();
        let single = Self::from_state(&states[0], precision);
        
        let mut data = Vec::with_capacity(single.data.len() * batch_size);
        for state in states {
            let tensor = Self::from_state(state, precision);
            data.extend(tensor.data);
        }
        
        Self {
            data,
            dtype: single.dtype,
            shape: vec![batch_size, 16, 2],
            layout: TensorLayout::SilState,
        }
    }
    
    /// Cria tensor a partir de slice f32
    pub fn from_f32(data: &[f32], shape: &[usize]) -> Self {
        let expected_size: usize = shape.iter().product();
        assert_eq!(data.len(), expected_size, "Shape mismatch");
        
        let bytes: Vec<u8> = data.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        
        Self {
            data: bytes,
            dtype: DataType::Float32,
            shape: shape.to_vec(),
            layout: TensorLayout::default(),
        }
    }
    
    /// Converte para f32 (dequantiza se necessário)
    pub fn as_f32(&self) -> Vec<f32> {
        match self.dtype {
            DataType::Float32 => {
                self.data.chunks(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect()
            }
            DataType::Float16 => {
                self.data.chunks(2)
                    .map(|chunk| {
                        let bits = u16::from_le_bytes([chunk[0], chunk[1]]);
                        half::f16::from_bits(bits).to_f32()
                    })
                    .collect()
            }
            DataType::Int8 => {
                self.data.iter()
                    .map(|&b| b as i8 as f32)
                    .collect()
            }
            DataType::Int4 => {
                self.data.iter()
                    .flat_map(|&b| {
                        let high = ((b >> 4) as i8) as f32;
                        let low = ((b & 0x0F) as i8) as f32;
                        [high, low]
                    })
                    .collect()
            }
            DataType::UInt8 => {
                self.data.iter()
                    .map(|&b| b as f32)
                    .collect()
            }
        }
    }
    
    /// Converte para SilState (primeiro do batch)
    pub fn to_state(&self) -> NpuResult<SilState> {
        use crate::state::ByteSil;
        
        let floats = self.as_f32();
        if floats.len() < 32 {
            return Err(NpuError::TensorSizeMismatch {
                expected: 32,
                actual: floats.len(),
            });
        }
        
        let mut layers = [ByteSil::NULL; 16];
        for i in 0..16 {
            let rho = floats[i * 2].round().clamp(-8.0, 7.0) as i8;
            let theta = (floats[i * 2 + 1].round() as u8) % 16;
            layers[i] = ByteSil::new(rho, theta);
        }
        
        Ok(SilState { layers })
    }
    
    /// Divide tensor batch em tensores individuais
    pub fn split_batch(&self, batch_size: usize) -> NpuResult<Vec<Self>> {
        if batch_size == 0 || self.data.is_empty() {
            return Ok(vec![]);
        }
        
        let item_size = self.data.len() / batch_size;
        
        if item_size == 0 {
            return Err(super::NpuError::TensorSizeMismatch {
                expected: batch_size,
                actual: self.data.len(),
            });
        }
        
        Ok(self.data.chunks(item_size)
            .map(|chunk| Self {
                data: chunk.to_vec(),
                dtype: self.dtype,
                shape: if self.shape.len() > 1 { self.shape[1..].to_vec() } else { vec![item_size] },
                layout: self.layout,
            })
            .collect())
    }
    
    /// Shape do tensor
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
    
    /// Tipo de dados
    pub fn dtype(&self) -> DataType {
        self.dtype
    }
    
    /// Tamanho em bytes de cada tipo
    fn dtype_size(dtype: DataType) -> usize {
        match dtype {
            DataType::Float32 => 4,
            DataType::Float16 => 2,
            DataType::Int8 | DataType::UInt8 => 1,
            DataType::Int4 => 1, // 2 valores por byte
        }
    }
    
    /// Número total de elementos
    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }
    
    /// Dados brutos
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tensor_from_state() {
        let state = SilState::neutral();
        let tensor = NpuTensor::from_state(&state, Precision::FP32);
        
        assert_eq!(tensor.shape(), &[1, 16, 2]);
        assert_eq!(tensor.dtype(), DataType::Float32);
    }
    
    #[test]
    fn test_tensor_roundtrip() {
        let original = SilState::neutral();
        let tensor = NpuTensor::from_state(&original, Precision::FP32);
        let recovered = tensor.to_state().unwrap();
        
        for i in 0..16 {
            assert_eq!(original.layers[i].rho, recovered.layers[i].rho);
            assert_eq!(original.layers[i].theta, recovered.layers[i].theta);
        }
    }
    
    #[test]
    fn test_tensor_batch() {
        let states = vec![SilState::neutral(), SilState::maximum(), SilState::vacuum()];
        let tensor = NpuTensor::from_states(&states, Precision::FP32);
        
        assert_eq!(tensor.shape(), &[3, 16, 2]);
        
        let split = tensor.split_batch(3).unwrap();
        assert_eq!(split.len(), 3);
    }
    
    #[test]
    fn test_tensor_quantization() {
        let state = SilState::neutral();
        
        // FP32
        let t32 = NpuTensor::from_state(&state, Precision::FP32);
        assert_eq!(t32.as_bytes().len(), 128); // 32 floats × 4 bytes
        
        // FP16
        let t16 = NpuTensor::from_state(&state, Precision::FP16);
        assert_eq!(t16.as_bytes().len(), 64); // 32 halfs × 2 bytes
        
        // INT8
        let t8 = NpuTensor::from_state(&state, Precision::INT8);
        assert_eq!(t8.as_bytes().len(), 32); // 32 int8s × 1 byte
        
        // INT4
        let t4 = NpuTensor::from_state(&state, Precision::INT4);
        assert_eq!(t4.as_bytes().len(), 16); // 16 packed bytes
    }
}
