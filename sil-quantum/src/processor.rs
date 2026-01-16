//! Processador de estados quânticos

use num_complex::Complex;
use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::QuantumState;
use crate::error::{QuantumError, QuantumResult};
use crate::state::QuantumStateData;

/// Processador de estados quânticos
#[derive(Debug, Clone)]
pub struct QuantumProcessor {
    /// Dados do estado quântico
    data: QuantumStateData,
    /// Configuração
    config: QuantumConfig,
}

/// Configuração do processador quântico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumConfig {
    /// Taxa de decoerência por operação
    pub decoherence_rate: f32,
    /// Threshold mínimo de coerência
    pub min_coherence: f32,
    /// Normalizar pesos automaticamente
    pub auto_normalize: bool,
    /// Permitir pesos negativos (fase)
    pub allow_negative_weights: bool,
}

impl Default for QuantumConfig {
    fn default() -> Self {
        Self {
            decoherence_rate: 0.01,
            min_coherence: 0.1,
            auto_normalize: true,
            allow_negative_weights: false,
        }
    }
}

impl QuantumProcessor {
    /// Cria novo processador quântico
    pub fn new() -> Self {
        Self::with_config(QuantumConfig::default())
    }

    /// Cria processador com configuração customizada
    pub fn with_config(config: QuantumConfig) -> Self {
        Self {
            data: QuantumStateData::new(),
            config,
        }
    }

    /// Valida e normaliza pesos
    fn validate_weights(&self, weights: &[f32]) -> QuantumResult<Vec<f32>> {
        if weights.is_empty() {
            return Err(QuantumError::NoStates);
        }

        // Verifica pesos negativos
        if !self.config.allow_negative_weights && weights.iter().any(|&w| w < 0.0) {
            return Err(QuantumError::InvalidWeights(-1.0));
        }

        if self.config.auto_normalize {
            let sum: f32 = weights.iter().sum();
            if sum == 0.0 {
                return Err(QuantumError::InvalidWeights(0.0));
            }
            Ok(weights.iter().map(|w| w / sum).collect())
        } else {
            let sum: f32 = weights.iter().sum();
            if (sum - 1.0).abs() > 0.01 {
                return Err(QuantumError::InvalidWeights(sum));
            }
            Ok(weights.to_vec())
        }
    }

    /// Gera número pseudo-aleatório a partir de seed
    fn pseudo_random(&self, seed: u64) -> f32 {
        // Simple LCG for deterministic randomness
        let a = 1664525u64;
        let c = 1013904223u64;
        let m = 2u64.pow(32);
        let val = (a.wrapping_mul(seed).wrapping_add(c)) % m;
        (val as f32) / (m as f32)
    }

    /// Retorna dados do estado
    pub fn state_data(&self) -> &QuantumStateData {
        &self.data
    }

    /// Número de estados em superposição
    pub fn state_count(&self) -> usize {
        self.data.state_count()
    }

    /// Limpa o estado
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Aplica decoerência
    pub fn apply_decoherence(&mut self) {
        self.data.reduce_coherence(self.config.decoherence_rate);
    }

    /// Verifica se coerência está acima do mínimo
    pub fn is_coherent(&self) -> bool {
        self.data.coherence >= self.config.min_coherence
    }
}

impl Default for QuantumProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumState for QuantumProcessor {
    fn superpose(&self, states: &[SilState], weights: &[f32]) -> SilState {
        if states.is_empty() {
            return SilState::neutral();
        }

        if states.len() != weights.len() {
            return states[0].clone();
        }

        // Valida e normaliza pesos
        let normalized_weights = match self.validate_weights(weights) {
            Ok(w) => w,
            Err(_) => return states[0].clone(),
        };

        // Cria estado superposto como média ponderada
        let mut result = SilState::neutral();

        // Para cada camada, calcula média ponderada
        for layer_idx in 0..states[0].layers.len() {
            let mut weighted_sum = 0.0;
            let mut total_weight = 0.0;

            for (i, state) in states.iter().enumerate() {
                let value = state.get(layer_idx).to_complex().norm() as f32;
                weighted_sum += value * normalized_weights[i];
                total_weight += normalized_weights[i];
            }

            if total_weight > 0.0 {
                let avg = weighted_sum / total_weight;
                let bytesil = ByteSil::from_complex(Complex::from_polar(avg as f64, 0.0));
                result = result.with_layer(layer_idx, bytesil);
            }
        }

        result
    }

    fn collapse(&mut self, seed: u64) -> SilState {
        if !self.data.is_superposed || self.data.states.is_empty() {
            return SilState::neutral();
        }

        // Usa seed para escolher estado de forma determinística
        let random_val = self.pseudo_random(seed);

        let mut cumulative = 0.0;
        let mut selected_idx = 0;

        for (i, &weight) in self.data.weights.iter().enumerate() {
            cumulative += weight;
            if random_val <= cumulative {
                selected_idx = i;
                break;
            }
        }

        let collapsed_state = self.data.states[selected_idx].clone();

        // Atualiza estado interno
        self.data.is_superposed = false;
        self.data.collapse_count += 1;
        self.apply_decoherence();

        collapsed_state
    }

    fn coherence(&self) -> f32 {
        self.data.coherence
    }

    fn is_superposed(&self) -> bool {
        self.data.is_superposed && self.data.coherence > 0.5
    }
}

impl SilComponent for QuantumProcessor {
    fn name(&self) -> &str {
        "QuantumProcessor"
    }

    fn layers(&self) -> &[LayerId] {
        &[12] // LC - Quântico
    }

    fn version(&self) -> &str {
        "2026.1.11"
    }

    fn is_ready(&self) -> bool {
        self.is_coherent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_processor() {
        let qp = QuantumProcessor::new();
        assert_eq!(qp.coherence(), 1.0);
        assert!(!qp.is_superposed());
    }

    #[test]
    fn test_superpose_single() {
        let qp = QuantumProcessor::new();
        let states = vec![SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(5.0, 0.0)))];
        let weights = vec![1.0];

        let result = qp.superpose(&states, &weights);
        let val = result.get(0).to_complex().norm() as f32;
        assert!(val > 0.0);
    }

    #[test]
    fn test_superpose_weighted() {
        let qp = QuantumProcessor::new();
        let states = vec![
            SilState::neutral().with_layer(0, ByteSil::NULL),
            SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
        ];
        let weights = vec![0.5, 0.5];

        let result = qp.superpose(&states, &weights);
        let val = result.get(0).to_complex().norm() as f32;
        assert!(val > 0.0);
    }
}
