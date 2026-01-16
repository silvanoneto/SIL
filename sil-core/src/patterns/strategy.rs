//! # ⚙️ Strategy Pattern — Processamento
//!
//! Diferentes estratégias de processamento intercambiáveis.
//!
//! ## Uso
//!
//! ```
//! use sil_core::patterns::strategy::*;
//! use sil_core::state::SilState;
//!
//! // Criar contexto com estratégia
//! let context = ProcessingContext::new(Box::new(WeightedAverageStrategy::default()));
//!
//! let result = context.execute(&SilState::neutral());
//! ```

use crate::state::{ByteSil, SilState, layers};
use crate::transforms::SilTransform;

/// **ProcessingStrategy** — Trait para estratégias de processamento L(5-7)
///
/// Segue Pattern 2 (Strategy) do SIL_CODE.md
///
/// # Exemplo
///
/// ```
/// use sil_core::patterns::strategy::ProcessingStrategy;
/// use sil_core::state::ByteSil;
///
/// struct NeuralStrategy;
///
/// impl ProcessingStrategy for NeuralStrategy {
///     fn process(&self, perception: &[ByteSil; 5]) -> [ByteSil; 3] {
///         // Processar com rede neural...
///         [
///             ByteSil::new(2, 8),  // L5 Eletrônico
///             ByteSil::new(1, 4),  // L6 Psicomotor
///             ByteSil::new(0, 0),  // L7 Ambiental
///         ]
///     }
/// }
/// ```
pub trait ProcessingStrategy: Send + Sync {
    /// Processa camadas de percepção L(0-4) → L(5-7)
    fn process(&self, perception: &[ByteSil; 5]) -> [ByteSil; 3];
}

/// Trait para estratégias genéricas — compatibilidade com código existente
pub trait Strategy: Send + Sync {
    /// Executa estratégia sobre o estado
    fn execute(&self, state: &SilState) -> SilState;
    
    /// Nome da estratégia
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Contexto que usa uma estratégia
pub struct ProcessingContext {
    strategy: Box<dyn Strategy>,
}

impl ProcessingContext {
    pub fn new(strategy: Box<dyn Strategy>) -> Self {
        Self { strategy }
    }
    
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategy = strategy;
    }
    
    pub fn execute(&self, state: &SilState) -> SilState {
        self.strategy.execute(state)
    }
}

impl SilTransform for ProcessingContext {
    fn transform(&self, state: &SilState) -> SilState {
        self.strategy.execute(state)
    }
    
    fn name(&self) -> &'static str {
        "ProcessingContext"
    }
}

// =============================================================================
// Estratégias Concretas
// =============================================================================

/// Estratégia de média ponderada
#[derive(Debug, Clone)]
pub struct WeightedAverageStrategy {
    /// Pesos para cada camada de percepção
    weights: [f64; 5],
}

impl WeightedAverageStrategy {
    pub fn new(weights: [f64; 5]) -> Self {
        // Normaliza pesos
        let sum: f64 = weights.iter().sum();
        let normalized = if sum > 0.0 {
            weights.map(|w| w / sum)
        } else {
            [0.2; 5]
        };
        Self { weights: normalized }
    }
}

impl Default for WeightedAverageStrategy {
    fn default() -> Self {
        Self { weights: [0.2; 5] }
    }
}

impl Strategy for WeightedAverageStrategy {
    fn execute(&self, state: &SilState) -> SilState {
        let perception = state.perception();
        
        // Calcula média ponderada de ρ e θ
        let mut sum_rho = 0.0;
        let mut sum_theta = 0.0;
        
        for (i, byte) in perception.iter().enumerate() {
            sum_rho += byte.rho as f64 * self.weights[i];
            sum_theta += byte.theta as f64 * self.weights[i];
        }
        
        let avg = ByteSil::new(sum_rho.round() as i8, sum_theta.round() as u8);
        
        // Aplica em todas as camadas de processamento
        state
            .with_layer(layers::ELECTRONIC, avg)
            .with_layer(layers::PSYCHOMOTOR, avg)
            .with_layer(layers::ENVIRONMENTAL, avg)
    }
    
    fn name(&self) -> &'static str {
        "WeightedAverageStrategy"
    }
}

/// Estratégia de máximo: pega o maior valor
#[derive(Debug, Clone, Copy, Default)]
pub struct MaxStrategy;

impl Strategy for MaxStrategy {
    fn execute(&self, state: &SilState) -> SilState {
        let perception = state.perception();
        
        // Encontra o byte com maior norma
        let max_byte = perception.iter()
            .max_by_key(|b| b.norm())
            .copied()
            .unwrap_or(ByteSil::NULL);
        
        state
            .with_layer(layers::ELECTRONIC, max_byte)
            .with_layer(layers::PSYCHOMOTOR, max_byte)
            .with_layer(layers::ENVIRONMENTAL, max_byte)
    }
    
    fn name(&self) -> &'static str {
        "MaxStrategy"
    }
}

/// Estratégia de mínimo: pega o menor valor
#[derive(Debug, Clone, Copy, Default)]
pub struct MinStrategy;

impl Strategy for MinStrategy {
    fn execute(&self, state: &SilState) -> SilState {
        let perception = state.perception();
        
        // Encontra o byte com menor norma (ignora nulls)
        let min_byte = perception.iter()
            .filter(|b| !b.is_null())
            .min_by_key(|b| b.norm())
            .copied()
            .unwrap_or(ByteSil::NULL);
        
        state
            .with_layer(layers::ELECTRONIC, min_byte)
            .with_layer(layers::PSYCHOMOTOR, min_byte)
            .with_layer(layers::ENVIRONMENTAL, min_byte)
    }
    
    fn name(&self) -> &'static str {
        "MinStrategy"
    }
}

/// Estratégia de diferenciação: detecta mudanças
#[derive(Debug, Clone)]
pub struct DifferentialStrategy {
    previous: Option<[ByteSil; 5]>,
}

impl DifferentialStrategy {
    pub fn new() -> Self {
        Self { previous: None }
    }
    
    pub fn with_baseline(baseline: [ByteSil; 5]) -> Self {
        Self { previous: Some(baseline) }
    }
}

impl Default for DifferentialStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DifferentialStrategy {
    fn execute(&self, state: &SilState) -> SilState {
        let perception = state.perception();
        
        let diff = if let Some(prev) = &self.previous {
            // Calcula diferença XOR
            let mut result = [ByteSil::NULL; 3];
            for i in 0..3 {
                let idx = i.min(4);
                result[i] = perception[idx].xor(&prev[idx]);
            }
            result
        } else {
            // Sem baseline, retorna percepção como está
            [perception[0], perception[1], perception[2]]
        };
        
        state
            .with_layer(layers::ELECTRONIC, diff[0])
            .with_layer(layers::PSYCHOMOTOR, diff[1])
            .with_layer(layers::ENVIRONMENTAL, diff[2])
    }
    
    fn name(&self) -> &'static str {
        "DifferentialStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weighted_average_strategy() {
        let state = SilState::neutral();
        let strategy = WeightedAverageStrategy::default();
        
        let result = strategy.execute(&state);
        
        // Média de ONEs = ONE
        assert_eq!(result.layers[layers::ELECTRONIC], ByteSil::new(0, 0));
    }
    
    #[test]
    fn test_max_strategy() {
        let state = SilState::vacuum()
            .with_layer(layers::PHOTONIC, ByteSil::new(5, 0))
            .with_layer(layers::ACOUSTIC, ByteSil::new(2, 0));
        
        let strategy = MaxStrategy;
        let result = strategy.execute(&state);
        
        // Deve pegar L0 (maior)
        assert_eq!(result.layers[layers::ELECTRONIC].rho, 5);
    }
    
    #[test]
    fn test_processing_context() {
        let context = ProcessingContext::new(Box::new(MaxStrategy));
        let state = SilState::neutral();
        
        let result = context.transform(&state);
        
        // Deve ter processado
        assert!(!result.layers[layers::ELECTRONIC].is_null());
    }
}
