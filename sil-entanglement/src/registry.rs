//! Registro de pares emaranhados

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sil_core::prelude::*;

/// Informação de par emaranhado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairInfo {
    /// ID do par
    pub pair_id: u64,
    /// Estado correlacionado
    pub correlated_state: SilState,
    /// Força da correlação (0.0 = perdida, 1.0 = forte)
    pub correlation: f32,
    /// Timestamp do emaranhamento
    pub timestamp: u64,
}

/// Registro de emaranhamentos
#[derive(Debug, Clone, Default)]
pub struct EntanglementRegistry {
    /// Pares ativos
    pairs: HashMap<u64, PairInfo>,
    /// Contador de IDs
    next_id: u64,
}

impl EntanglementRegistry {
    /// Cria novo registro
    pub fn new() -> Self {
        Self::default()
    }

    /// Adiciona novo par
    pub fn add_pair(&mut self, state: SilState, correlation: f32) -> u64 {
        let pair_id = self.next_id;
        self.next_id += 1;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let info = PairInfo {
            pair_id,
            correlated_state: state,
            correlation,
            timestamp,
        };

        self.pairs.insert(pair_id, info);
        pair_id
    }

    /// Remove par
    pub fn remove_pair(&mut self, pair_id: u64) -> bool {
        self.pairs.remove(&pair_id).is_some()
    }

    /// Obtém informação do par
    pub fn get_pair(&self, pair_id: u64) -> Option<&PairInfo> {
        self.pairs.get(&pair_id)
    }

    /// Obtém informação mutável do par
    pub fn get_pair_mut(&mut self, pair_id: u64) -> Option<&mut PairInfo> {
        self.pairs.get_mut(&pair_id)
    }

    /// Verifica se par existe
    pub fn has_pair(&self, pair_id: u64) -> bool {
        self.pairs.contains_key(&pair_id)
    }

    /// Lista todos os pares
    pub fn all_pairs(&self) -> Vec<u64> {
        self.pairs.keys().copied().collect()
    }

    /// Número de pares ativos
    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }

    /// Limpa todos os pares
    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    /// Reduz correlação de um par
    pub fn reduce_correlation(&mut self, pair_id: u64, amount: f32) {
        if let Some(info) = self.pairs.get_mut(&pair_id) {
            info.correlation = (info.correlation - amount).max(0.0);
        }
    }
}
