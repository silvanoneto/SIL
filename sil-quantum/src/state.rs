//! Estado quântico interno

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;

/// Dados do estado quântico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumStateData {
    /// Estados em superposição
    pub states: Vec<SilState>,
    /// Pesos de cada estado
    pub weights: Vec<f32>,
    /// Coerência atual (0.0 = decoerido, 1.0 = coerente)
    pub coherence: f32,
    /// Estado está em superposição?
    pub is_superposed: bool,
    /// Número de colapsos
    pub collapse_count: u64,
}

impl Default for QuantumStateData {
    fn default() -> Self {
        Self {
            states: Vec::new(),
            weights: Vec::new(),
            coherence: 1.0,
            is_superposed: false,
            collapse_count: 0,
        }
    }
}

impl QuantumStateData {
    /// Cria novo estado quântico
    pub fn new() -> Self {
        Self::default()
    }

    /// Limpa o estado
    pub fn clear(&mut self) {
        self.states.clear();
        self.weights.clear();
        self.coherence = 1.0;
        self.is_superposed = false;
    }

    /// Reduz coerência
    pub fn reduce_coherence(&mut self, amount: f32) {
        self.coherence = (self.coherence - amount).max(0.0);
    }

    /// Restaura coerência
    pub fn restore_coherence(&mut self) {
        self.coherence = 1.0;
    }

    /// Número de estados em superposição
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
}
