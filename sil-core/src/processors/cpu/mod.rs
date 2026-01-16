//! # 🖥️ CPU Processor — Fallback com SIMD
//!
//! Processamento em CPU com otimizações SIMD quando disponíveis.

use crate::state::{ByteSil, SilState, NUM_LAYERS};
use super::{Processor, ProcessorCapability, GradientProcessor, InterpolationProcessor};

/// Contexto de processamento CPU
#[derive(Debug, Clone)]
pub struct CpuContext {
    /// Número de threads para operações paralelas
    pub num_threads: usize,
    /// SIMD disponível
    pub simd_available: bool,
}

/// Resultado CPU
pub type CpuResult<T> = Result<T, CpuError>;

/// Erro CPU
#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("Operação não suportada: {0}")]
    Unsupported(String),
    
    #[error("Buffer inválido: esperado {expected}, recebido {actual}")]
    InvalidBuffer { expected: usize, actual: usize },
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuContext {
    /// Cria novo contexto CPU
    pub fn new() -> Self {
        Self {
            num_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            simd_available: Self::detect_simd(),
        }
    }
    
    /// Detecta suporte a SIMD
    fn detect_simd() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx2")
        }
        #[cfg(target_arch = "aarch64")]
        {
            // ARM NEON sempre disponível em aarch64
            true
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            false
        }
    }
    
    /// Calcula gradiente via diferenças finitas
    pub fn compute_gradient(&self, state: &SilState, epsilon: f32) -> CpuGradient {
        let mut layers = [LayerGrad::zero(); NUM_LAYERS];
        
        for i in 0..NUM_LAYERS {
            let layer = state.layers[i];
            let rho = layer.rho as f32;
            let theta = layer.theta as f32;
            
            // ∂f/∂ρ
            let z_rho_plus = complex_magnitude(rho + epsilon, theta);
            let z_rho_minus = complex_magnitude(rho - epsilon, theta);
            let d_rho = (z_rho_plus - z_rho_minus) / (2.0 * epsilon);
            
            // ∂f/∂θ (circular)
            let theta_plus = (theta + epsilon) % 16.0;
            let theta_minus = if theta < epsilon { theta - epsilon + 16.0 } else { theta - epsilon };
            let z_theta_plus = complex_magnitude(rho, theta_plus);
            let z_theta_minus = complex_magnitude(rho, theta_minus);
            let d_theta = (z_theta_plus - z_theta_minus) / (2.0 * epsilon);
            
            layers[i] = LayerGrad { d_rho, d_theta };
        }
        
        CpuGradient { layers }
    }
}

impl Processor for CpuContext {
    type Error = CpuError;
    
    fn name(&self) -> &str {
        "CPU"
    }
    
    fn is_ready(&self) -> bool {
        true
    }
    
    fn capabilities(&self) -> &[ProcessorCapability] {
        &[
            ProcessorCapability::MatrixOps,
            ProcessorCapability::Gradients,
            ProcessorCapability::Interpolation,
            ProcessorCapability::Reduction,
        ]
    }
}

impl GradientProcessor for CpuContext {
    type Gradient = CpuGradient;
    
    fn compute_gradient(&self, state: &SilState) -> Result<Self::Gradient, Self::Error> {
        Ok(self.compute_gradient(state, 0.01))
    }
    
    fn compute_gradients_batch(&self, states: &[SilState]) -> Result<Vec<Self::Gradient>, Self::Error> {
        Ok(states.iter().map(|s| self.compute_gradient(s, 0.01)).collect())
    }
}

impl InterpolationProcessor for CpuContext {
    fn lerp(&self, a: &SilState, b: &SilState, t: f32) -> Result<SilState, Self::Error> {
        Ok(lerp_states(a, b, t))
    }
    
    fn slerp(&self, a: &SilState, b: &SilState, t: f32) -> Result<SilState, Self::Error> {
        Ok(slerp_states(a, b, t))
    }
    
    fn interpolate_sequence(
        &self,
        a: &SilState,
        b: &SilState,
        steps: usize,
        use_slerp: bool,
    ) -> Result<Vec<SilState>, Self::Error> {
        if steps == 0 {
            return Ok(vec![]);
        }
        if steps == 1 {
            return Ok(vec![*a]);
        }
        
        let interp_fn = if use_slerp { slerp_states } else { lerp_states };
        
        Ok((0..steps)
            .map(|i| {
                let t = i as f32 / (steps - 1) as f32;
                interp_fn(a, b, t)
            })
            .collect())
    }
}

/// Gradiente de uma camada
#[derive(Debug, Clone, Copy, Default)]
pub struct LayerGrad {
    pub d_rho: f32,
    pub d_theta: f32,
}

impl LayerGrad {
    pub const fn zero() -> Self {
        Self { d_rho: 0.0, d_theta: 0.0 }
    }
    
    pub fn magnitude(&self) -> f32 {
        (self.d_rho * self.d_rho + self.d_theta * self.d_theta).sqrt()
    }
}

/// Gradiente completo CPU
#[derive(Debug, Clone)]
pub struct CpuGradient {
    pub layers: [LayerGrad; NUM_LAYERS],
}

impl CpuGradient {
    pub fn total_magnitude(&self) -> f32 {
        self.layers.iter().map(|l| l.magnitude()).sum()
    }
}

// Helpers

fn complex_magnitude(rho: f32, _theta: f32) -> f32 {
    if rho <= -8.0 {
        return 0.0;
    }
    2.0_f32.powf(rho / 4.0)
}

fn lerp_states(a: &SilState, b: &SilState, t: f32) -> SilState {
    let t = t.clamp(0.0, 1.0);
    let mut layers = [ByteSil::NULL; NUM_LAYERS];
    
    for i in 0..NUM_LAYERS {
        let rho_a = a.layers[i].rho as f32;
        let rho_b = b.layers[i].rho as f32;
        let theta_a = a.layers[i].theta as f32;
        let theta_b = b.layers[i].theta as f32;
        
        let rho = rho_a * (1.0 - t) + rho_b * t;
        let theta = theta_a * (1.0 - t) + theta_b * t;
        
        let rho_i = rho.round().clamp(-8.0, 7.0) as i8;
        let theta_u = (theta.round() as i32).rem_euclid(16) as u8;
        
        layers[i] = ByteSil::new(rho_i, theta_u);
    }
    
    SilState { layers }
}

fn slerp_states(a: &SilState, b: &SilState, t: f32) -> SilState {
    let t = t.clamp(0.0, 1.0);
    let mut layers = [ByteSil::NULL; NUM_LAYERS];
    
    for i in 0..NUM_LAYERS {
        let rho_a = a.layers[i].rho as f32;
        let rho_b = b.layers[i].rho as f32;
        let theta_a = a.layers[i].theta as f32;
        let theta_b = b.layers[i].theta as f32;
        
        let rho = rho_a * (1.0 - t) + rho_b * t;
        let theta = slerp_angle(theta_a, theta_b, t);
        
        let rho_i = rho.round().clamp(-8.0, 7.0) as i8;
        let theta_u = (theta.round() as i32).rem_euclid(16) as u8;
        
        layers[i] = ByteSil::new(rho_i, theta_u);
    }
    
    SilState { layers }
}

fn slerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut delta = b - a;
    
    if delta > 8.0 {
        delta -= 16.0;
    } else if delta < -8.0 {
        delta += 16.0;
    }
    
    let mut result = a + delta * t;
    
    if result < 0.0 {
        result += 16.0;
    } else if result >= 16.0 {
        result -= 16.0;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_context_creation() {
        let ctx = CpuContext::new();
        assert!(ctx.num_threads >= 1);
        println!("CPU threads: {}, SIMD: {}", ctx.num_threads, ctx.simd_available);
    }
    
    #[test]
    fn test_cpu_gradient() {
        let ctx = CpuContext::new();
        let state = SilState::neutral();
        let grad = ctx.compute_gradient(&state, 0.01);
        assert!(grad.total_magnitude() > 0.0);
    }
    
    #[test]
    fn test_cpu_interpolation() {
        let ctx = CpuContext::new();
        let a = SilState::vacuum();
        let b = SilState::maximum();
        
        let mid = ctx.lerp(&a, &b, 0.5).unwrap();
        
        // Deve estar entre a e b
        for i in 0..NUM_LAYERS {
            let rho_a = a.layers[i].rho;
            let rho_b = b.layers[i].rho;
            let rho_mid = mid.layers[i].rho;
            assert!(rho_mid >= rho_a.min(rho_b) && rho_mid <= rho_a.max(rho_b));
        }
    }
}
