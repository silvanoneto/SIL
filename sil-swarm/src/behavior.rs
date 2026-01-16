//! Tipos de comportamento de swarm

use serde::{Deserialize, Serialize};

/// Tipos de comportamento de swarm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmBehavior {
    /// Flocking (boids): alinhamento, coesão, separação
    Flocking,
    /// Consenso: convergência para estado comum
    Consensus,
    /// Emergente: padrões não-lineares
    Emergent,
}

/// Comportamento de flocking (boids)
#[derive(Debug, Clone)]
pub struct FlockingBehavior {
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub separation_weight: f32,
}

impl Default for FlockingBehavior {
    fn default() -> Self {
        Self {
            alignment_weight: 0.33,
            cohesion_weight: 0.33,
            separation_weight: 0.34,
        }
    }
}

/// Comportamento de consenso
#[derive(Debug, Clone)]
pub struct ConsensusBehavior {
    pub threshold: f32,
    pub convergence_rate: f32,
}

impl Default for ConsensusBehavior {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            convergence_rate: 0.1,
        }
    }
}
