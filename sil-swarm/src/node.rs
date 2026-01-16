//! Implementação do nó de swarm

use crate::behavior::SwarmBehavior;
use crate::error::{SwarmError, SwarmResult};
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::SwarmAgent;
use std::collections::HashMap;

/// Nó individual de swarm
#[derive(Debug, Clone)]
pub struct SwarmNode {
    /// ID único do nó
    id: u64,
    /// IDs dos vizinhos conhecidos
    neighbors: Vec<u64>,
    /// Distâncias para cada vizinho
    distances: HashMap<u64, f32>,
    /// Estado local atual
    state: SilState,
    /// Configuração
    config: SwarmConfig,
    /// Comportamento ativo
    behavior: SwarmBehavior,
}

/// Configuração do swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    /// Distância padrão entre vizinhos
    pub default_distance: f32,
    /// Peso para alinhamento (flocking)
    pub alignment_weight: f32,
    /// Peso para coesão (flocking)
    pub cohesion_weight: f32,
    /// Peso para separação (flocking)
    pub separation_weight: f32,
    /// Threshold para consenso
    pub consensus_threshold: f32,
    /// Taxa de aprendizado
    pub learning_rate: f32,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            default_distance: 1.0,
            alignment_weight: 0.33,
            cohesion_weight: 0.33,
            separation_weight: 0.34,
            consensus_threshold: 0.8,
            learning_rate: 0.1,
        }
    }
}

impl SwarmNode {
    /// Cria novo nó de swarm
    pub fn new(id: u64) -> Self {
        Self::with_config(id, SwarmConfig::default())
    }

    /// Cria nó com configuração customizada
    pub fn with_config(id: u64, config: SwarmConfig) -> Self {
        Self {
            id,
            neighbors: Vec::new(),
            distances: HashMap::new(),
            state: SilState::neutral(),
            config,
            behavior: SwarmBehavior::Flocking,
        }
    }

    /// Retorna o ID único do nó
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Adiciona vizinho
    pub fn add_neighbor(&mut self, neighbor_id: u64) -> SwarmResult<()> {
        if !self.neighbors.contains(&neighbor_id) {
            self.neighbors.push(neighbor_id);
            self.distances
                .insert(neighbor_id, self.config.default_distance);
        }
        Ok(())
    }

    /// Remove vizinho
    pub fn remove_neighbor(&mut self, neighbor_id: u64) -> SwarmResult<()> {
        if let Some(pos) = self.neighbors.iter().position(|&id| id == neighbor_id) {
            self.neighbors.remove(pos);
            self.distances.remove(&neighbor_id);
            Ok(())
        } else {
            Err(SwarmError::NeighborNotFound(neighbor_id))
        }
    }

    /// Define distância para vizinho
    pub fn set_distance(&mut self, neighbor_id: u64, distance: f32) -> SwarmResult<()> {
        if self.neighbors.contains(&neighbor_id) {
            self.distances.insert(neighbor_id, distance);
            Ok(())
        } else {
            Err(SwarmError::NeighborNotFound(neighbor_id))
        }
    }

    /// Define comportamento ativo
    pub fn set_behavior(&mut self, behavior: SwarmBehavior) {
        self.behavior = behavior;
    }

    /// Estado local atual
    pub fn local_state(&self) -> &SilState {
        &self.state
    }

    /// Atualiza estado local
    pub fn update_state(&mut self, state: SilState) {
        self.state = state;
    }

    /// Comportamento de flocking (boids)
    fn flocking_behavior(&self, local: &SilState, neighbor_states: &[SilState]) -> SilState {
        if neighbor_states.is_empty() {
            return local.clone();
        }

        let mut result = SilState::neutral();

        // Alinhamento: média das direções dos vizinhos
        let mut alignment = 0.0;
        // Coesão: mover em direção ao centro de massa
        let mut cohesion = 0.0;
        // Separação: evitar colisões
        let mut separation = 0.0;

        for (i, state) in neighbor_states.iter().enumerate() {
            let neighbor_id = self.neighbors[i];
            let distance = self.distances.get(&neighbor_id).copied().unwrap_or(1.0);

            // Alinhamento baseado nas camadas
            for layer_idx in 0..local.layers.len() {
                let n_val = state.get(layer_idx).to_complex().norm() as f32;
                alignment += n_val;
            }

            // Coesão: média dos valores dos vizinhos
            for layer_idx in 0..local.layers.len() {
                let n_val = state.get(layer_idx).to_complex().norm() as f32;
                cohesion += n_val / distance;
            }

            // Separação: inverso da distância
            if distance < 1.0 {
                separation += 1.0 / (distance + 0.01);
            }
        }

        let n = neighbor_states.len() as f32;
        alignment /= n;
        cohesion /= n;
        separation /= n;

        // Combina os três comportamentos
        for layer_idx in 0..local.layers.len() {
            let local_val = local.get(layer_idx).to_complex().norm() as f32;
            let new_val = local_val
                * (1.0 - self.config.alignment_weight - self.config.cohesion_weight)
                + alignment * self.config.alignment_weight
                + cohesion * self.config.cohesion_weight
                - separation * self.config.separation_weight;

            // Converte o valor de volta para ByteSil
            let new_bytesil = ByteSil::from_complex(Complex::from_polar(new_val as f64, 0.0));
            result = result.with_layer(layer_idx, new_bytesil);
        }

        result
    }

    /// Comportamento de consenso (média ponderada)
    fn consensus_behavior(&self, local: &SilState, neighbor_states: &[SilState]) -> SilState {
        if neighbor_states.is_empty() {
            return local.clone();
        }

        let mut result = SilState::neutral();

        for layer_idx in 0..local.layers.len() {
            let local_val = local.get(layer_idx).to_complex().norm() as f32;
            let mut weighted_sum = local_val;
            let mut total_weight = 1.0;

            for (i, state) in neighbor_states.iter().enumerate() {
                let neighbor_val = state.get(layer_idx).to_complex().norm() as f32;
                let neighbor_id = self.neighbors[i];
                let distance = self.distances.get(&neighbor_id).copied().unwrap_or(1.0);
                let weight = 1.0 / (distance + 0.1);

                weighted_sum += neighbor_val * weight;
                total_weight += weight;
            }

            let consensus = weighted_sum / total_weight;
            let consensus_bytesil = ByteSil::from_complex(Complex::from_polar(consensus as f64, 0.0));
            result = result.with_layer(layer_idx, consensus_bytesil);
        }

        result
    }

    /// Comportamento emergente (não-linear)
    fn emergent_behavior(&self, local: &SilState, neighbor_states: &[SilState]) -> SilState {
        if neighbor_states.is_empty() {
            return local.clone();
        }

        let mut result = SilState::neutral();

        for layer_idx in 0..local.layers.len() {
            let local_val = local.get(layer_idx).to_complex().norm() as f32;

            // Coleta valores dos vizinhos
            let neighbor_values: Vec<f32> = neighbor_states
                .iter()
                .map(|s| s.get(layer_idx).to_complex().norm() as f32)
                .collect();

            if neighbor_values.is_empty() {
                result = result.with_layer(layer_idx, local.get(layer_idx));
                continue;
            }

            // Padrão emergente: combinação não-linear
            let mean = neighbor_values.iter().sum::<f32>() / neighbor_values.len() as f32;
            let variance = neighbor_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f32>()
                / neighbor_values.len() as f32;

            // Valor emergente combina média e diversidade
            let diversity_factor = variance.sqrt();
            let emergent_val = mean + diversity_factor * self.config.learning_rate;

            // Mix com valor local
            let new_val = local_val * 0.7 + emergent_val * 0.3;
            let new_bytesil = ByteSil::from_complex(Complex::from_polar(new_val as f64, 0.0));
            result = result.with_layer(layer_idx, new_bytesil);
        }

        result
    }

    /// Retorna número de vizinhos
    pub fn neighbor_count(&self) -> usize {
        self.neighbors.len()
    }

    /// Verifica se tem vizinho específico
    pub fn has_neighbor(&self, neighbor_id: u64) -> bool {
        self.neighbors.contains(&neighbor_id)
    }
}

impl SwarmAgent for SwarmNode {
    type NodeId = u64;

    fn neighbors(&self) -> Vec<Self::NodeId> {
        self.neighbors.clone()
    }

    fn behavior(&mut self, local: &SilState, neighbor_states: &[SilState]) -> SilState {
        let result = match self.behavior {
            SwarmBehavior::Flocking => self.flocking_behavior(local, neighbor_states),
            SwarmBehavior::Consensus => self.consensus_behavior(local, neighbor_states),
            SwarmBehavior::Emergent => self.emergent_behavior(local, neighbor_states),
        };

        // Atualiza estado local
        self.state = result.clone();
        result
    }

    fn distance_to(&self, neighbor: &Self::NodeId) -> f32 {
        self.distances
            .get(neighbor)
            .copied()
            .unwrap_or(self.config.default_distance)
    }
}

impl SilComponent for SwarmNode {
    fn name(&self) -> &str {
        "SwarmNode"
    }

    fn layers(&self) -> &[LayerId] {
        &[11] // LB - Sinérgico
    }

    fn version(&self) -> &str {
        "2026.1.11"
    }

    fn is_ready(&self) -> bool {
        !self.neighbors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = SwarmNode::new(1);
        assert_eq!(node.id, 1);
        assert_eq!(node.neighbor_count(), 0);
    }

    #[test]
    fn test_add_remove_neighbors() {
        let mut node = SwarmNode::new(1);
        node.add_neighbor(2).unwrap();
        node.add_neighbor(3).unwrap();
        assert_eq!(node.neighbor_count(), 2);

        node.remove_neighbor(2).unwrap();
        assert_eq!(node.neighbor_count(), 1);
        assert!(!node.has_neighbor(2));
    }

    #[test]
    fn test_set_distance() {
        let mut node = SwarmNode::new(1);
        node.add_neighbor(2).unwrap();
        node.set_distance(2, 2.5).unwrap();
        assert_eq!(node.distance_to(&2), 2.5);
    }
}
