//! # 🔄 Transforms — Transformações SIL
//!
//! Transformações de estado para estado, seguindo o princípio:
//!
//! > *"Transformação é pura — mesma entrada, mesma saída."*
//!
//! ## Estrutura por Fase do Ciclo
//!
//! - [`perception`]: L(0-4) — Sensores
//! - [`processing`]: L(5-7) — Computação local
//! - [`interaction`]: L(8-A) — Comunicação
//! - [`emergence`]: L(B-C) — Padrões emergentes
//! - [`meta`]: L(D-F) — Controle de fluxo

pub mod perception;
pub mod processing;
pub mod interaction;
pub mod emergence;
pub mod meta;

use crate::state::SilState;

/// Trait central: transformação de estado para estado
///
/// # Princípio SOLID
///
/// - **S**: Uma transformação, uma responsabilidade
/// - **O**: Extensível sem modificar (novas impls)
/// - **L**: Qualquer impl substitui outra
/// - **I**: Interface mínima (um método)
/// - **D**: Depende de abstração (SilState)
///
/// # Exemplo
///
/// ```
/// use sil_core::transforms::SilTransform;
/// use sil_core::state::{SilState, ByteSil};
///
/// struct DoublePhase;
///
/// impl SilTransform for DoublePhase {
///     fn transform(&self, state: &SilState) -> SilState {
///         let mut layers = state.layers;
///         for layer in &mut layers {
///             layer.theta = (layer.theta * 2) % 16;
///         }
///         SilState::from_layers(layers)
///     }
/// }
/// ```
pub trait SilTransform: Send + Sync {
    /// Aplica transformação, retornando novo estado
    fn transform(&self, state: &SilState) -> SilState;
    
    /// Nome da transformação (para debug/log)
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Pipeline: sequência de transformações
///
/// Composição de transformações por encadeamento.
///
/// # Exemplo
///
/// ```
/// use sil_core::transforms::{Pipeline, PhaseShift, MagnitudeScale, SilTransform};
/// use sil_core::state::SilState;
///
/// let pipeline = Pipeline::new(vec![
///     Box::new(PhaseShift(4)),
///     Box::new(MagnitudeScale(2)),
/// ]);
///
/// let output = pipeline.transform(&SilState::neutral());
/// ```
pub struct Pipeline {
    transforms: Vec<Box<dyn SilTransform>>,
}

impl Pipeline {
    /// Cria pipeline vazio
    pub fn new(transforms: Vec<Box<dyn SilTransform>>) -> Self {
        Self { transforms }
    }
    
    /// Cria pipeline vazio
    pub fn empty() -> Self {
        Self { transforms: Vec::new() }
    }
    
    /// Adiciona transformação ao pipeline
    pub fn push(&mut self, transform: Box<dyn SilTransform>) {
        self.transforms.push(transform);
    }
    
    /// Número de transformações
    pub fn len(&self) -> usize {
        self.transforms.len()
    }
    
    /// Pipeline vazio?
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
}

impl SilTransform for Pipeline {
    fn transform(&self, state: &SilState) -> SilState {
        self.transforms.iter().fold(
            *state,
            |s, t| t.transform(&s)
        )
    }
    
    fn name(&self) -> &'static str {
        "Pipeline"
    }
}

// =============================================================================
// Transformações Básicas
// =============================================================================

/// Deslocamento de fase em todas as camadas
///
/// θ_new = (θ + shift) % 16
#[derive(Debug, Clone, Copy)]
pub struct PhaseShift(pub u8);

impl SilTransform for PhaseShift {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        for layer in &mut layers {
            layer.theta = (layer.theta + self.0) % 16;
        }
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "PhaseShift"
    }
}

/// Escala de magnitude em todas as camadas
///
/// ρ_new = clamp(ρ + scale, -8, 7)
#[derive(Debug, Clone, Copy)]
pub struct MagnitudeScale(pub i8);

impl SilTransform for MagnitudeScale {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        for layer in &mut layers {
            let new_rho = (layer.rho as i16 + self.0 as i16)
                .clamp(-8, 7) as i8;
            layer.rho = new_rho;
        }
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "MagnitudeScale"
    }
}

/// Troca duas camadas
#[derive(Debug, Clone, Copy)]
pub struct LayerSwap(pub usize, pub usize);

impl SilTransform for LayerSwap {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        if self.0 < 16 && self.1 < 16 {
            layers.swap(self.0, self.1);
        }
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "LayerSwap"
    }
}

/// XOR entre duas camadas, resultado em destino
#[derive(Debug, Clone, Copy)]
pub struct LayerXor {
    pub src_a: usize,
    pub src_b: usize,
    pub dest: usize,
}

impl SilTransform for LayerXor {
    fn transform(&self, state: &SilState) -> SilState {
        if self.src_a >= 16 || self.src_b >= 16 || self.dest >= 16 {
            return *state;
        }
        
        let result = state.layers[self.src_a].xor(&state.layers[self.src_b]);
        state.with_layer(self.dest, result)
    }
    
    fn name(&self) -> &'static str {
        "LayerXor"
    }
}

/// Identidade (não faz nada)
#[derive(Debug, Clone, Copy, Default)]
pub struct Identity;

impl SilTransform for Identity {
    fn transform(&self, state: &SilState) -> SilState {
        *state
    }
    
    fn name(&self) -> &'static str {
        "Identity"
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ByteSil;
    
    #[test]
    fn test_phase_shift() {
        let state = SilState::neutral(); // θ=0 em todas
        let shifted = PhaseShift(4).transform(&state);
        
        for layer in &shifted.layers {
            assert_eq!(layer.theta, 4);
        }
    }
    
    #[test]
    fn test_magnitude_scale() {
        let state = SilState::neutral(); // ρ=0 em todas
        let scaled = MagnitudeScale(3).transform(&state);
        
        for layer in &scaled.layers {
            assert_eq!(layer.rho, 3);
        }
    }
    
    #[test]
    fn test_magnitude_scale_clamp() {
        let state = SilState::maximum(); // ρ=7 em todas
        let scaled = MagnitudeScale(5).transform(&state);
        
        for layer in &scaled.layers {
            assert_eq!(layer.rho, 7); // Clampado em 7
        }
    }
    
    #[test]
    fn test_pipeline() {
        let state = SilState::neutral();
        
        let pipeline = Pipeline::new(vec![
            Box::new(PhaseShift(2)),
            Box::new(MagnitudeScale(1)),
        ]);
        
        let result = pipeline.transform(&state);
        
        for layer in &result.layers {
            assert_eq!(layer.theta, 2);
            assert_eq!(layer.rho, 1);
        }
    }
    
    #[test]
    fn test_identity() {
        let state = SilState::neutral()
            .with_layer(5, ByteSil::new(3, 7));
        
        let result = Identity.transform(&state);
        assert_eq!(state, result);
    }
    
    #[test]
    fn test_layer_swap() {
        let state = SilState::vacuum()
            .with_layer(0, ByteSil::new(5, 10))
            .with_layer(15, ByteSil::new(-3, 2));
        
        let swapped = LayerSwap(0, 15).transform(&state);
        
        assert_eq!(swapped.layers[0], ByteSil::new(-3, 2));
        assert_eq!(swapped.layers[15], ByteSil::new(5, 10));
    }
}
