//! # Emergence — Métricas de Complexidade Emergente
//!
//! Implementa métricas para medir emergência e complexidade em sistemas multi-agente.
//!
//! ## Teoria
//!
//! Emergência ocorre quando o comportamento coletivo não pode ser reduzido
//! às partes individuais. Métricas incluem:
//!
//! - **Φ (Phi)**: Integrated Information Theory (Tononi)
//! - **Shannon Entropy**: Diversidade de estados
//! - **Kolmogorov Complexity**: Incompressibilidade

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

/// Tipo de organização emergente (interpretação theta de LB)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum OrgType {
    /// Agregado simples (sem emergência)
    Aggregate = 0,
    /// Rebanho (flocking básico)
    Flock = 2,
    /// Colônia (divisão de trabalho)
    #[default]
    Colony = 4,
    /// Rede (conectividade estruturada)
    Network = 6,
    /// Hierarquia (organização em níveis)
    Hierarchy = 8,
    /// Ecossistema (interdependências complexas)
    Ecosystem = 10,
    /// Holobionte (simbiose integrada)
    Holobiont = 12,
    /// Superorganismo (identidade coletiva)
    Superorganism = 14,
}

impl OrgType {
    /// Cria OrgType a partir de theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0b1110 {
            0 => Self::Aggregate,
            2 => Self::Flock,
            4 => Self::Colony,
            6 => Self::Network,
            8 => Self::Hierarchy,
            10 => Self::Ecosystem,
            12 => Self::Holobiont,
            14 => Self::Superorganism,
            _ => Self::Colony,
        }
    }

    /// Converte para theta
    pub fn to_theta(self) -> u8 {
        self as u8
    }

    /// Nível de emergência (0-7)
    pub fn emergence_level(&self) -> u8 {
        (self.to_theta() / 2) as u8
    }

    /// Verifica se há verdadeira emergência
    pub fn is_emergent(&self) -> bool {
        !matches!(self, Self::Aggregate)
    }

    /// Verifica se requer coordenação ativa
    pub fn requires_coordination(&self) -> bool {
        matches!(
            self,
            Self::Colony | Self::Hierarchy | Self::Ecosystem | Self::Holobiont | Self::Superorganism
        )
    }

    /// Nome descritivo
    pub fn name(&self) -> &'static str {
        match self {
            Self::Aggregate => "Aggregate",
            Self::Flock => "Flock",
            Self::Colony => "Colony",
            Self::Network => "Network",
            Self::Hierarchy => "Hierarchy",
            Self::Ecosystem => "Ecosystem",
            Self::Holobiont => "Holobiont",
            Self::Superorganism => "Superorganism",
        }
    }
}

impl fmt::Display for OrgType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Nível de emergência observado
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmergenceLevel {
    /// Sem emergência detectada
    None,
    /// Emergência fraca (correlações simples)
    Weak,
    /// Emergência moderada (padrões locais)
    Moderate,
    /// Emergência forte (comportamento global)
    Strong,
    /// Emergência radical (identidade coletiva)
    Radical,
}

impl EmergenceLevel {
    /// Converte de Φ normalizado (0.0-1.0)
    pub fn from_phi(phi: f64) -> Self {
        if phi < 0.1 {
            Self::None
        } else if phi < 0.3 {
            Self::Weak
        } else if phi < 0.5 {
            Self::Moderate
        } else if phi < 0.7 {
            Self::Strong
        } else {
            Self::Radical
        }
    }

    /// Valor numérico (0-4)
    pub fn value(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Weak => 1,
            Self::Moderate => 2,
            Self::Strong => 3,
            Self::Radical => 4,
        }
    }
}

impl Default for EmergenceLevel {
    fn default() -> Self {
        Self::None
    }
}

/// Métricas de complexidade
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Φ (Phi) - Integrated Information (0.0 - 1.0)
    pub phi: f64,

    /// Entropia de Shannon (bits)
    pub shannon_entropy: f64,

    /// Estimativa de complexidade de Kolmogorov (bits)
    pub kolmogorov_estimate: f64,

    /// Número de agentes
    pub agent_count: usize,

    /// Número de conexões únicas
    pub connection_count: usize,

    /// Grau médio de conectividade
    pub mean_degree: f64,

    /// Coeficiente de clustering
    pub clustering_coefficient: f64,

    /// Nível de emergência derivado
    pub emergence_level: EmergenceLevel,
}

impl ComplexityMetrics {
    /// Cria novas métricas
    pub fn new() -> Self {
        Self::default()
    }

    /// Calcula métricas para um conjunto de estados
    pub fn from_states(states: &[u8], connections: &[(usize, usize)]) -> Self {
        let agent_count = states.len();
        let connection_count = connections.len();

        // Shannon entropy
        let shannon = Self::calculate_shannon(states);

        // Phi aproximado (simplificado)
        let phi = Self::estimate_phi(agent_count, connection_count);

        // Kolmogorov (muito simplificado)
        let kolmogorov = Self::estimate_kolmogorov(states);

        // Grau médio
        let mean_degree = if agent_count > 0 {
            (2.0 * connection_count as f64) / agent_count as f64
        } else {
            0.0
        };

        // Clustering
        let clustering = Self::calculate_clustering(agent_count, connections);

        let emergence_level = EmergenceLevel::from_phi(phi);

        Self {
            phi,
            shannon_entropy: shannon,
            kolmogorov_estimate: kolmogorov,
            agent_count,
            connection_count,
            mean_degree,
            clustering_coefficient: clustering,
            emergence_level,
        }
    }

    /// Calcula entropia de Shannon
    fn calculate_shannon(states: &[u8]) -> f64 {
        if states.is_empty() {
            return 0.0;
        }

        let mut counts: HashMap<u8, usize> = HashMap::new();
        for &s in states {
            *counts.entry(s).or_insert(0) += 1;
        }

        let n = states.len() as f64;
        counts.values()
            .map(|&c| {
                let p = c as f64 / n;
                if p > 0.0 { -p * p.log2() } else { 0.0 }
            })
            .sum()
    }

    /// Estima Φ (versão simplificada)
    fn estimate_phi(agents: usize, connections: usize) -> f64 {
        if agents < 2 {
            return 0.0;
        }

        // Máximo de conexões possíveis
        let max_connections = agents * (agents - 1) / 2;
        if max_connections == 0 {
            return 0.0;
        }

        // Conectividade relativa
        let connectivity = connections as f64 / max_connections as f64;

        // Phi aumenta com conectividade mas satura
        // Fórmula simplificada: Φ ≈ connectivity × (1 - connectivity/2)
        // Máximo em connectivity = 1
        connectivity * (1.0 - connectivity / 2.0)
    }

    /// Estima complexidade de Kolmogorov (compressão simples)
    fn estimate_kolmogorov(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        // Run-length encoding simplificado
        let mut runs = 1;
        for i in 1..data.len() {
            if data[i] != data[i - 1] {
                runs += 1;
            }
        }

        // Bits necessários para representar runs
        let bits_per_run = if data.len() > 1 {
            (data.len() as f64).log2().ceil()
        } else {
            1.0
        };

        runs as f64 * bits_per_run
    }

    /// Calcula coeficiente de clustering
    fn calculate_clustering(agents: usize, connections: &[(usize, usize)]) -> f64 {
        if agents < 3 || connections.is_empty() {
            return 0.0;
        }

        // Constrói adjacência
        let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
        for &(a, b) in connections {
            adjacency.entry(a).or_default().push(b);
            adjacency.entry(b).or_default().push(a);
        }

        // Calcula clustering local para cada nó
        let mut total_clustering = 0.0;
        let mut nodes_counted = 0;

        for (&_node, neighbors) in &adjacency {
            let k = neighbors.len();
            if k < 2 {
                continue;
            }

            // Conta triângulos
            let mut triangles = 0;
            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    let ni = neighbors[i];
                    let nj = neighbors[j];
                    if adjacency.get(&ni).map(|v| v.contains(&nj)).unwrap_or(false) {
                        triangles += 1;
                    }
                }
            }

            // Clustering local = triângulos / triângulos possíveis
            let possible = k * (k - 1) / 2;
            if possible > 0 {
                total_clustering += triangles as f64 / possible as f64;
                nodes_counted += 1;
            }
        }

        if nodes_counted > 0 {
            total_clustering / nodes_counted as f64
        } else {
            0.0
        }
    }

    /// Atualiza nível de emergência baseado em Φ
    pub fn update_emergence(&mut self) {
        self.emergence_level = EmergenceLevel::from_phi(self.phi);
    }

    /// Verifica se sistema é complexo (threshold)
    pub fn is_complex(&self, threshold: f64) -> bool {
        self.phi >= threshold
    }

    /// Sugere OrgType baseado nas métricas
    pub fn suggested_org_type(&self) -> OrgType {
        match self.emergence_level {
            EmergenceLevel::None => OrgType::Aggregate,
            EmergenceLevel::Weak => OrgType::Flock,
            EmergenceLevel::Moderate => {
                if self.clustering_coefficient > 0.5 {
                    OrgType::Colony
                } else {
                    OrgType::Network
                }
            }
            EmergenceLevel::Strong => {
                if self.mean_degree > 4.0 {
                    OrgType::Ecosystem
                } else {
                    OrgType::Hierarchy
                }
            }
            EmergenceLevel::Radical => {
                if self.clustering_coefficient > 0.7 {
                    OrgType::Superorganism
                } else {
                    OrgType::Holobiont
                }
            }
        }
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_type_from_theta() {
        assert_eq!(OrgType::from_theta(0), OrgType::Aggregate);
        assert_eq!(OrgType::from_theta(4), OrgType::Colony);
        assert_eq!(OrgType::from_theta(14), OrgType::Superorganism);
    }

    #[test]
    fn test_org_type_roundtrip() {
        for theta in (0..16).step_by(2) {
            let org = OrgType::from_theta(theta);
            assert_eq!(org.to_theta(), theta);
        }
    }

    #[test]
    fn test_emergence_level() {
        assert_eq!(OrgType::Aggregate.emergence_level(), 0);
        assert_eq!(OrgType::Colony.emergence_level(), 2);
        assert_eq!(OrgType::Superorganism.emergence_level(), 7);
    }

    #[test]
    fn test_is_emergent() {
        assert!(!OrgType::Aggregate.is_emergent());
        assert!(OrgType::Flock.is_emergent());
        assert!(OrgType::Superorganism.is_emergent());
    }

    #[test]
    fn test_complexity_metrics_empty() {
        let metrics = ComplexityMetrics::from_states(&[], &[]);
        assert_eq!(metrics.agent_count, 0);
        assert_eq!(metrics.phi, 0.0);
    }

    #[test]
    fn test_complexity_metrics_simple() {
        let states = vec![1, 1, 2, 2, 3];
        let connections = vec![(0, 1), (1, 2), (2, 3), (3, 4)];

        let metrics = ComplexityMetrics::from_states(&states, &connections);

        assert_eq!(metrics.agent_count, 5);
        assert_eq!(metrics.connection_count, 4);
        assert!(metrics.shannon_entropy > 0.0);
        assert!(metrics.phi > 0.0);
    }

    #[test]
    fn test_shannon_entropy() {
        // Estados uniformes = baixa entropia
        let uniform = vec![1, 1, 1, 1];
        let metrics1 = ComplexityMetrics::from_states(&uniform, &[]);
        assert_eq!(metrics1.shannon_entropy, 0.0);

        // Estados variados = alta entropia
        let varied = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let metrics2 = ComplexityMetrics::from_states(&varied, &[]);
        assert!(metrics2.shannon_entropy > 2.0);
    }

    #[test]
    fn test_emergence_level_from_phi() {
        assert_eq!(EmergenceLevel::from_phi(0.0), EmergenceLevel::None);
        assert_eq!(EmergenceLevel::from_phi(0.2), EmergenceLevel::Weak);
        assert_eq!(EmergenceLevel::from_phi(0.4), EmergenceLevel::Moderate);
        assert_eq!(EmergenceLevel::from_phi(0.6), EmergenceLevel::Strong);
        assert_eq!(EmergenceLevel::from_phi(0.8), EmergenceLevel::Radical);
    }

    #[test]
    fn test_clustering_complete_graph() {
        // Grafo completo de 4 nós
        let connections = vec![
            (0, 1), (0, 2), (0, 3),
            (1, 2), (1, 3),
            (2, 3),
        ];

        let metrics = ComplexityMetrics::from_states(&[0, 0, 0, 0], &connections);
        assert!((metrics.clustering_coefficient - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_suggested_org_type() {
        let mut metrics = ComplexityMetrics::new();

        metrics.phi = 0.05;
        metrics.update_emergence();
        assert_eq!(metrics.suggested_org_type(), OrgType::Aggregate);

        metrics.phi = 0.6;
        metrics.mean_degree = 5.0;
        metrics.update_emergence();
        assert_eq!(metrics.suggested_org_type(), OrgType::Ecosystem);
    }
}
