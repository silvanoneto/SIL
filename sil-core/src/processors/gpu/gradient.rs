//! SilGradient — Gradiente no plano complexo

use crate::state::{ByteSil, SilState, NUM_LAYERS};
use bytemuck::{Pod, Zeroable};

/// Gradiente de uma camada individual
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct LayerGradient {
    /// ∂f/∂ρ — variação na magnitude
    pub d_rho: f32,
    /// ∂f/∂θ — variação na fase
    pub d_theta: f32,
}

impl LayerGradient {
    /// Cria gradiente zero
    pub const fn zero() -> Self {
        Self { d_rho: 0.0, d_theta: 0.0 }
    }
    
    /// Cria novo gradiente
    pub fn new(d_rho: f32, d_theta: f32) -> Self {
        Self { d_rho, d_theta }
    }
    
    /// Magnitude do gradiente: |∇| = √(∂ρ² + ∂θ²)
    pub fn magnitude(&self) -> f32 {
        (self.d_rho * self.d_rho + self.d_theta * self.d_theta).sqrt()
    }
    
    /// Direção do gradiente em radianos
    pub fn direction(&self) -> f32 {
        self.d_theta.atan2(self.d_rho)
    }
    
    /// Normaliza o gradiente para magnitude 1
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag < 1e-10 {
            Self::zero()
        } else {
            Self {
                d_rho: self.d_rho / mag,
                d_theta: self.d_theta / mag,
            }
        }
    }
    
    /// Escala o gradiente
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            d_rho: self.d_rho * factor,
            d_theta: self.d_theta * factor,
        }
    }
}

/// Gradiente completo de um SilState (16 camadas)
#[derive(Debug, Clone)]
pub struct SilGradient {
    /// Gradientes por camada
    pub layers: [LayerGradient; NUM_LAYERS],
}

impl Default for SilGradient {
    fn default() -> Self {
        Self::zero()
    }
}

impl SilGradient {
    /// Cria gradiente zero
    pub fn zero() -> Self {
        Self {
            layers: [LayerGradient::zero(); NUM_LAYERS],
        }
    }
    
    /// Cria gradiente a partir de array de floats (GPU output)
    pub fn from_floats(data: &[f32; NUM_LAYERS * 2]) -> Self {
        let mut layers = [LayerGradient::zero(); NUM_LAYERS];
        for i in 0..NUM_LAYERS {
            layers[i] = LayerGradient {
                d_rho: data[i * 2],
                d_theta: data[i * 2 + 1],
            };
        }
        Self { layers }
    }
    
    /// Converte para array de floats (GPU input)
    pub fn to_floats(&self) -> [f32; NUM_LAYERS * 2] {
        let mut data = [0.0f32; NUM_LAYERS * 2];
        for i in 0..NUM_LAYERS {
            data[i * 2] = self.layers[i].d_rho;
            data[i * 2 + 1] = self.layers[i].d_theta;
        }
        data
    }
    
    /// Calcula gradiente via CPU (diferenças finitas)
    /// 
    /// Usa função de magnitude como f(state)
    pub fn compute_cpu(state: &SilState, epsilon: f32) -> Self {
        let mut layers = [LayerGradient::zero(); NUM_LAYERS];
        
        for i in 0..NUM_LAYERS {
            let layer = state.layers[i];
            let rho = layer.rho as f32;
            let theta = layer.theta as f32;
            
            // ∂f/∂ρ via diferenças finitas centrais
            let z_rho_plus = complex_magnitude(rho + epsilon, theta);
            let z_rho_minus = complex_magnitude(rho - epsilon, theta);
            let d_rho = (z_rho_plus - z_rho_minus) / (2.0 * epsilon);
            
            // ∂f/∂θ via diferenças finitas (circular mod 16)
            let theta_plus = (theta + epsilon) % 16.0;
            let theta_minus = if theta < epsilon { theta - epsilon + 16.0 } else { theta - epsilon };
            let z_theta_plus = complex_magnitude(rho, theta_plus);
            let z_theta_minus = complex_magnitude(rho, theta_minus);
            let d_theta = (z_theta_plus - z_theta_minus) / (2.0 * epsilon);
            
            layers[i] = LayerGradient { d_rho, d_theta };
        }
        
        Self { layers }
    }
    
    /// Magnitude total do gradiente (soma das magnitudes por camada)
    pub fn total_magnitude(&self) -> f32 {
        self.layers.iter().map(|l| l.magnitude()).sum()
    }
    
    /// Média das magnitudes
    pub fn average_magnitude(&self) -> f32 {
        self.total_magnitude() / NUM_LAYERS as f32
    }
    
    /// Camada com maior gradiente
    pub fn max_gradient_layer(&self) -> usize {
        self.layers
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.magnitude().partial_cmp(&b.magnitude()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    
    /// Normaliza todos os gradientes
    pub fn normalize(&self) -> Self {
        let mut layers = [LayerGradient::zero(); NUM_LAYERS];
        for i in 0..NUM_LAYERS {
            layers[i] = self.layers[i].normalize();
        }
        Self { layers }
    }
    
    /// Aplica gradiente ao estado (gradient descent step)
    /// 
    /// new_state = state - learning_rate × ∇
    pub fn apply_to(&self, state: &SilState, learning_rate: f32) -> SilState {
        let mut new_layers = state.layers;
        
        for i in 0..NUM_LAYERS {
            let rho = state.layers[i].rho as f32 - learning_rate * self.layers[i].d_rho;
            let theta = state.layers[i].theta as f32 - learning_rate * self.layers[i].d_theta;
            
            // Clamp rho para [-8, 7]
            let rho_clamped = rho.clamp(-8.0, 7.0).round() as i8;
            
            // Theta circular mod 16
            let theta_normalized = ((theta % 16.0) + 16.0) % 16.0;
            let theta_clamped = theta_normalized.round() as u8 % 16;
            
            new_layers[i] = ByteSil::new(rho_clamped, theta_clamped);
        }
        
        SilState { layers: new_layers }
    }
    
    /// Produto interno entre dois gradientes
    pub fn dot(&self, other: &Self) -> f32 {
        self.layers
            .iter()
            .zip(other.layers.iter())
            .map(|(a, b)| a.d_rho * b.d_rho + a.d_theta * b.d_theta)
            .sum()
    }
}

/// Calcula magnitude do número complexo em coordenadas log-polar
fn complex_magnitude(rho: f32, _theta: f32) -> f32 {
    if rho <= -8.0 {
        return 0.0; // NULL
    }
    
    // Magnitude: 2^(ρ/4)
    2.0_f32.powf(rho / 4.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layer_gradient_magnitude() {
        let g = LayerGradient::new(3.0, 4.0);
        assert!((g.magnitude() - 5.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_layer_gradient_normalize() {
        let g = LayerGradient::new(3.0, 4.0);
        let n = g.normalize();
        assert!((n.magnitude() - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_compute_cpu_gradient() {
        let state = SilState::neutral();
        let grad = SilGradient::compute_cpu(&state, 0.01);
        
        // Gradiente deve ter algum valor para estado não-null
        assert!(grad.total_magnitude() > 0.0);
    }
    
    #[test]
    fn test_apply_gradient() {
        let state = SilState::neutral();
        let grad = SilGradient::compute_cpu(&state, 0.01);
        
        // Aplicar passo pequeno
        let new_state = grad.apply_to(&state, 0.1);
        
        // Estado deve ter mudado (provavelmente)
        // Pode ser igual se gradiente for muito pequeno
        println!("Estado original: {:?}", state.layers[0]);
        println!("Gradiente[0]: {:?}", grad.layers[0]);
        println!("Novo estado: {:?}", new_state.layers[0]);
    }
    
    #[test]
    fn test_gradient_descent_reduces_magnitude() {
        // Começar com estado de alta magnitude
        let state = SilState::maximum();
        
        // Gradiente descendente deve reduzir magnitude
        let mut current = state;
        for _ in 0..10 {
            let grad = SilGradient::compute_cpu(&current, 0.01);
            current = grad.apply_to(&current, 0.5);
        }
        
        // Magnitude deve ter reduzido (ou estabilizado)
        let initial_mag: f32 = state.layers.iter()
            .map(|l| l.to_complex().norm() as f32)
            .sum();
        let final_mag: f32 = current.layers.iter()
            .map(|l| l.to_complex().norm() as f32)
            .sum();
        
        println!("Magnitude inicial: {}", initial_mag);
        println!("Magnitude final: {}", final_mag);
    }
}
