//! # ⚙️ Processing — Transformações de Processamento L(5-7)
//!
//! Camadas de computação local: Eletrônico, Psicomotor, Ambiental.
//!
//! ## Pattern: Strategy
//!
//! Diferentes estratégias de processamento podem ser trocadas.

use crate::state::{ByteSil, SilState, layers};
use super::SilTransform;

/// Trait para estratégias de processamento
pub trait ProcessingStrategy: Send + Sync {
    /// Processa percepção, retorna camadas L(5-7)
    fn process(&self, perception: &[ByteSil; 5]) -> [ByteSil; 3];
    
    /// Nome da estratégia
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Transformação que aplica estratégia de processamento
pub struct ProcessingTransform<P: ProcessingStrategy> {
    strategy: P,
}

impl<P: ProcessingStrategy> ProcessingTransform<P> {
    pub fn new(strategy: P) -> Self {
        Self { strategy }
    }
}

impl<P: ProcessingStrategy> SilTransform for ProcessingTransform<P> {
    fn transform(&self, state: &SilState) -> SilState {
        let perception = state.perception();
        let processed = self.strategy.process(&perception);
        
        state
            .with_layer(layers::ELECTRONIC, processed[0])
            .with_layer(layers::PSYCHOMOTOR, processed[1])
            .with_layer(layers::ENVIRONMENTAL, processed[2])
    }
    
    fn name(&self) -> &'static str {
        "ProcessingTransform"
    }
}

// =============================================================================
// Estratégias Básicas
// =============================================================================

/// Estratégia passthrough: copia percepção diretamente
#[derive(Debug, Clone, Copy, Default)]
pub struct PassthroughStrategy;

impl ProcessingStrategy for PassthroughStrategy {
    fn process(&self, perception: &[ByteSil; 5]) -> [ByteSil; 3] {
        // L5 = média de L0-L1 (visual + auditivo)
        let l5 = perception[0].mix(&perception[1]);
        
        // L6 = média de L2-L3 (olfato + gustação)
        let l6 = perception[2].mix(&perception[3]);
        
        // L7 = L4 (dérmico → ambiental)
        let l7 = perception[4];
        
        [l5, l6, l7]
    }
    
    fn name(&self) -> &'static str {
        "PassthroughStrategy"
    }
}

/// Estratégia de agregação: XOR de todas as percepções
#[derive(Debug, Clone, Copy, Default)]
pub struct AggregateStrategy;

impl ProcessingStrategy for AggregateStrategy {
    fn process(&self, perception: &[ByteSil; 5]) -> [ByteSil; 3] {
        let aggregate = perception.iter()
            .fold(ByteSil::NULL, |a, b| a.xor(b));
        
        [aggregate, aggregate, aggregate]
    }
    
    fn name(&self) -> &'static str {
        "AggregateStrategy"
    }
}

/// Estratégia baseada em threshold
#[derive(Debug, Clone, Copy)]
pub struct ThresholdStrategy {
    pub threshold: i8,
}

impl ThresholdStrategy {
    pub fn new(threshold: i8) -> Self {
        Self { threshold }
    }
}

impl ProcessingStrategy for ThresholdStrategy {
    fn process(&self, perception: &[ByteSil; 5]) -> [ByteSil; 3] {
        // Conta quantas percepções estão acima do threshold
        let count = perception.iter()
            .filter(|b| b.rho >= self.threshold)
            .count();
        
        // L5: Nível de ativação
        let l5 = ByteSil::new(count as i8 - 3, 0);
        
        // L6: Média das que passaram
        let active: Vec<_> = perception.iter()
            .filter(|b| b.rho >= self.threshold)
            .collect();
        
        let l6 = if active.is_empty() {
            ByteSil::NULL
        } else {
            let sum_rho: i16 = active.iter().map(|b| b.rho as i16).sum();
            let sum_theta: u16 = active.iter().map(|b| b.theta as u16).sum();
            ByteSil::new(
                (sum_rho / active.len() as i16) as i8,
                (sum_theta / active.len() as u16) as u8,
            )
        };
        
        // L7: XOR das que passaram (entropia)
        let l7 = active.iter()
            .fold(ByteSil::NULL, |a, b| a.xor(b));
        
        [l5, l6, l7]
    }
    
    fn name(&self) -> &'static str {
        "ThresholdStrategy"
    }
}

// =============================================================================
// Transformações específicas de processamento
// =============================================================================

/// Amplifica camadas de processamento
#[derive(Debug, Clone, Copy)]
pub struct ProcessingAmplify(pub i8);

impl SilTransform for ProcessingAmplify {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        
        for i in 5..8 {
            let new_rho = (layers[i].rho as i16 + self.0 as i16)
                .clamp(-8, 7) as i8;
            layers[i].rho = new_rho;
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "ProcessingAmplify"
    }
}

/// Rotaciona fase das camadas de processamento
#[derive(Debug, Clone, Copy)]
pub struct ProcessingRotate(pub u8);

impl SilTransform for ProcessingRotate {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        
        for i in 5..8 {
            layers[i].theta = (layers[i].theta + self.0) % 16;
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "ProcessingRotate"
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_passthrough_strategy() {
        let perception = [
            ByteSil::new(2, 0),  // L0
            ByteSil::new(4, 4),  // L1
            ByteSil::new(-1, 8), // L2
            ByteSil::new(1, 12), // L3
            ByteSil::new(3, 2),  // L4
        ];
        
        let strategy = PassthroughStrategy;
        let [_l5, _l6, l7] = strategy.process(&perception);
        
        // L7 deve ser L4
        assert_eq!(l7, perception[4]);
    }
    
    #[test]
    fn test_processing_transform() {
        let state = SilState::neutral();
        let transform = ProcessingTransform::new(AggregateStrategy);
        
        let result = transform.transform(&state);
        
        // AggregateStrategy faz: fold(NULL, XOR(perception))
        // = NULL ^ ONE ^ ONE ^ ONE ^ ONE ^ ONE
        // = (-8,0) ^ (0,0) ^ ... (5 vezes)
        // = -8 XOR 0 XOR 0 XOR 0 XOR 0 XOR 0 = -8
        for i in 5..8 {
            assert_eq!(result.layers[i].rho, -8);
            assert_eq!(result.layers[i].theta, 0);
        }
    }
    
    #[test]
    fn test_processing_amplify() {
        let state = SilState::neutral();
        let amplified = ProcessingAmplify(3).transform(&state);
        
        // Processamento amplificado
        for i in 5..8 {
            assert_eq!(amplified.layers[i].rho, 3);
        }
        
        // Resto inalterado
        for i in 0..5 {
            assert_eq!(amplified.layers[i].rho, 0);
        }
        for i in 8..16 {
            assert_eq!(amplified.layers[i].rho, 0);
        }
    }
}
