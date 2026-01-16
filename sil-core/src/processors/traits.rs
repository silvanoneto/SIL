//! Traits comuns para processadores

use crate::state::SilState;
use std::future::Future;

/// Trait base para todos os processadores
pub trait Processor: Send + Sync {
    /// Tipo de erro do processador
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Nome do processador
    fn name(&self) -> &str;
    
    /// Verifica se está pronto para uso
    fn is_ready(&self) -> bool;
    
    /// Lista capacidades suportadas
    fn capabilities(&self) -> &[super::ProcessorCapability];
    
    /// Verifica se suporta uma capacidade específica
    fn supports(&self, cap: super::ProcessorCapability) -> bool {
        self.capabilities().contains(&cap)
    }
}

/// Processador com suporte a gradientes
pub trait GradientProcessor: Processor {
    /// Tipo do gradiente produzido
    type Gradient;
    
    /// Calcula gradiente de um estado
    fn compute_gradient(&self, state: &SilState) -> Result<Self::Gradient, Self::Error>;
    
    /// Calcula gradientes em batch
    fn compute_gradients_batch(&self, states: &[SilState]) -> Result<Vec<Self::Gradient>, Self::Error>;
}

/// Processador com suporte a inferência (NPU)
pub trait InferenceProcessor: Processor {
    /// Tipo do modelo
    type Model;
    /// Tipo do tensor de entrada
    type Input;
    /// Tipo do tensor de saída
    type Output;
    
    /// Carrega um modelo
    fn load_model(&self, path: &std::path::Path) -> Result<Self::Model, Self::Error>;
    
    /// Executa inferência
    fn infer(&self, model: &Self::Model, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Processador com suporte a interpolação
pub trait InterpolationProcessor: Processor {
    /// Interpolação linear entre estados
    fn lerp(&self, a: &SilState, b: &SilState, t: f32) -> Result<SilState, Self::Error>;
    
    /// Interpolação esférica entre estados
    fn slerp(&self, a: &SilState, b: &SilState, t: f32) -> Result<SilState, Self::Error>;
    
    /// Gera sequência interpolada
    fn interpolate_sequence(
        &self, 
        a: &SilState, 
        b: &SilState, 
        steps: usize,
        use_slerp: bool,
    ) -> Result<Vec<SilState>, Self::Error>;
}

/// Processador assíncrono (para GPU/NPU)
pub trait AsyncProcessor: Processor {
    /// Calcula gradiente assincronamente
    fn compute_gradient_async<'a>(
        &'a self, 
        state: &'a SilState
    ) -> impl Future<Output = Result<Vec<f32>, Self::Error>> + Send + 'a;
    
    /// Executa batch assincronamente
    fn process_batch_async<'a>(
        &'a self,
        states: &'a [SilState],
    ) -> impl Future<Output = Result<Vec<SilState>, Self::Error>> + Send + 'a;
}

/// Quantização para NPU
pub trait Quantizable {
    /// Converte para INT8 quantizado
    fn to_int8(&self) -> Vec<i8>;
    
    /// Reconstrói a partir de INT8
    fn from_int8(data: &[i8]) -> Self;
    
    /// Converte para FP16
    fn to_fp16(&self) -> Vec<u16>;
    
    /// Reconstrói a partir de FP16
    fn from_fp16(data: &[u16]) -> Self;
}

impl Quantizable for SilState {
    fn to_int8(&self) -> Vec<i8> {
        // Cada ByteSil já é essencialmente int8 (rho: i4, theta: u4)
        // Expandimos para int8 para compatibilidade com NPU
        self.layers.iter()
            .flat_map(|l| [l.rho, l.theta as i8])
            .collect()
    }
    
    fn from_int8(data: &[i8]) -> Self {
        use crate::state::ByteSil;
        let mut layers = [ByteSil::NULL; 16];
        for (i, chunk) in data.chunks(2).enumerate().take(16) {
            if chunk.len() == 2 {
                layers[i] = ByteSil::new(chunk[0], chunk[1] as u8 & 0x0F);
            }
        }
        Self { layers }
    }
    
    fn to_fp16(&self) -> Vec<u16> {
        // Converte para half-precision float
        self.layers.iter()
            .flat_map(|l| {
                let rho_f16 = half::f16::from_f32(l.rho as f32);
                let theta_f16 = half::f16::from_f32(l.theta as f32);
                [rho_f16.to_bits(), theta_f16.to_bits()]
            })
            .collect()
    }
    
    fn from_fp16(data: &[u16]) -> Self {
        use crate::state::ByteSil;
        let mut layers = [ByteSil::NULL; 16];
        for (i, chunk) in data.chunks(2).enumerate().take(16) {
            if chunk.len() == 2 {
                let rho = half::f16::from_bits(chunk[0]).to_f32().round() as i8;
                let theta = half::f16::from_bits(chunk[1]).to_f32().round() as u8 & 0x0F;
                layers[i] = ByteSil::new(rho, theta);
            }
        }
        Self { layers }
    }
}
