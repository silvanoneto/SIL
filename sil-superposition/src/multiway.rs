//! # Multiway — Grafo de Estados Superpostos
//!
//! Implementa grafo multiway para representar todas as branches de estado
//! simultaneamente, similar ao multiway system de Wolfram.
//!
//! ## Operações
//!
//! - **Fork**: Cria novo branch a partir do estado atual
//! - **Merge**: Combina branches usando estratégia
//! - **Prune**: Remove branches que não satisfazem predicado
//! - **Select**: Escolhe branch baseado em critério
//! - **Clone**: Duplica branch inteiro

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use sil_core::SilState;

/// Operações sobre superposições
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuperOp {
    /// Cria novo branch
    Fork,
    /// Combina branches
    Merge,
    /// Remove branches inválidos
    Prune,
    /// Seleciona branch específico
    Select,
    /// Duplica branch
    Clone,
}

impl fmt::Display for SuperOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fork => write!(f, "Fork"),
            Self::Merge => write!(f, "Merge"),
            Self::Prune => write!(f, "Prune"),
            Self::Select => write!(f, "Select"),
            Self::Clone => write!(f, "Clone"),
        }
    }
}

/// Estratégia de superposição (interpretação theta de LD)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum SuperStrategy {
    /// Mantém único estado (colapso)
    Single = 0,
    /// Fork binário (dois branches)
    Binary = 2,
    /// Superposição uniforme
    #[default]
    Uniform = 4,
    /// Superposição ponderada
    Weighted = 6,
    /// Todos os branches possíveis
    Exhaustive = 8,
    /// Branches selecionados por heurística
    Heuristic = 10,
    /// Branches probabilísticos
    Probabilistic = 12,
    /// Grafo multiway completo
    Multiway = 14,
}

impl SuperStrategy {
    /// Cria SuperStrategy a partir de theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0b1110 {
            0 => Self::Single,
            2 => Self::Binary,
            4 => Self::Uniform,
            6 => Self::Weighted,
            8 => Self::Exhaustive,
            10 => Self::Heuristic,
            12 => Self::Probabilistic,
            14 => Self::Multiway,
            _ => Self::Uniform,
        }
    }

    /// Converte para theta
    pub fn to_theta(self) -> u8 {
        self as u8
    }

    /// Número típico de branches
    pub fn typical_branch_count(&self) -> usize {
        match self {
            Self::Single => 1,
            Self::Binary => 2,
            Self::Uniform | Self::Weighted => 4,
            Self::Heuristic => 8,
            Self::Probabilistic => 16,
            Self::Exhaustive | Self::Multiway => 32,
        }
    }

    /// Nome descritivo
    pub fn name(&self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::Binary => "Binary",
            Self::Uniform => "Uniform",
            Self::Weighted => "Weighted",
            Self::Exhaustive => "Exhaustive",
            Self::Heuristic => "Heuristic",
            Self::Probabilistic => "Probabilistic",
            Self::Multiway => "Multiway",
        }
    }
}

impl fmt::Display for SuperStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// ID de um branch no grafo
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub u64);

impl BranchId {
    /// Branch raiz
    pub const ROOT: Self = Self(0);

    /// Cria novo ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for BranchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "branch-{}", self.0)
    }
}

/// Um branch no grafo multiway
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Branch {
    /// ID do branch
    pub id: BranchId,
    /// Estado neste branch
    pub state: SilState,
    /// Branch pai (None para root)
    pub parent: Option<BranchId>,
    /// Branches filhos
    pub children: Vec<BranchId>,
    /// Peso/probabilidade deste branch
    pub weight: f64,
    /// Profundidade na árvore
    pub depth: usize,
    /// Timestamp de criação
    pub created_at: u64,
    /// Marcado para poda?
    pub marked_for_prune: bool,
}

impl Branch {
    /// Cria novo branch
    pub fn new(id: BranchId, state: SilState, parent: Option<BranchId>, depth: usize) -> Self {
        Self {
            id,
            state,
            parent,
            children: Vec::new(),
            weight: 1.0,
            depth,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            marked_for_prune: false,
        }
    }

    /// Cria branch raiz
    pub fn root(state: SilState) -> Self {
        Self::new(BranchId::ROOT, state, None, 0)
    }

    /// Define peso
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Verifica se é folha
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Verifica se é raiz
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

/// Grafo multiway de estados
#[derive(Clone, Debug)]
pub struct MultiwayGraph {
    /// Todos os branches
    branches: HashMap<BranchId, Branch>,
    /// Próximo ID disponível
    next_id: u64,
    /// Estratégia ativa
    pub strategy: SuperStrategy,
    /// Limite máximo de branches
    pub max_branches: usize,
}

impl MultiwayGraph {
    /// Cria novo grafo com estado raiz
    pub fn new(initial_state: SilState) -> Self {
        let root = Branch::root(initial_state);
        let mut branches = HashMap::new();
        branches.insert(BranchId::ROOT, root);

        Self {
            branches,
            next_id: 1,
            strategy: SuperStrategy::default(),
            max_branches: 1000,
        }
    }

    /// Define estratégia
    pub fn with_strategy(mut self, strategy: SuperStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Define limite de branches
    pub fn with_max_branches(mut self, max: usize) -> Self {
        self.max_branches = max;
        self
    }

    /// Retorna branch raiz
    pub fn root(&self) -> Option<&Branch> {
        self.branches.get(&BranchId::ROOT)
    }

    /// Retorna branch por ID
    pub fn get(&self, id: &BranchId) -> Option<&Branch> {
        self.branches.get(id)
    }

    /// Retorna branch mutável
    pub fn get_mut(&mut self, id: &BranchId) -> Option<&mut Branch> {
        self.branches.get_mut(id)
    }

    /// Número de branches
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Número de folhas
    pub fn leaf_count(&self) -> usize {
        self.branches.values().filter(|b| b.is_leaf()).count()
    }

    /// Profundidade máxima
    pub fn max_depth(&self) -> usize {
        self.branches.values().map(|b| b.depth).max().unwrap_or(0)
    }

    /// Cria fork de um branch
    pub fn fork(&mut self, parent_id: &BranchId) -> Option<BranchId> {
        if self.branches.len() >= self.max_branches {
            return None;
        }

        let parent = self.branches.get(parent_id)?;
        let new_id = BranchId(self.next_id);
        self.next_id += 1;

        let new_branch = Branch::new(
            new_id,
            parent.state.clone(),
            Some(*parent_id),
            parent.depth + 1,
        );

        // Adiciona filho ao pai
        self.branches.get_mut(parent_id)?.children.push(new_id);

        self.branches.insert(new_id, new_branch);
        Some(new_id)
    }

    /// Cria múltiplos forks
    pub fn fork_n(&mut self, parent_id: &BranchId, n: usize) -> Vec<BranchId> {
        (0..n)
            .filter_map(|_| self.fork(parent_id))
            .collect()
    }

    /// Atualiza estado de um branch
    pub fn update_state(&mut self, id: &BranchId, state: SilState) -> bool {
        if let Some(branch) = self.branches.get_mut(id) {
            branch.state = state;
            true
        } else {
            false
        }
    }

    /// Marca branch para poda
    pub fn mark_for_prune(&mut self, id: &BranchId) -> bool {
        if let Some(branch) = self.branches.get_mut(id) {
            branch.marked_for_prune = true;
            true
        } else {
            false
        }
    }

    /// Remove branches marcados
    pub fn prune(&mut self) -> usize {
        let to_remove: Vec<_> = self.branches
            .iter()
            .filter(|(_, b)| b.marked_for_prune)
            .map(|(id, _)| *id)
            .collect();

        let count = to_remove.len();

        for id in to_remove {
            self.remove_branch(&id);
        }

        count
    }

    /// Remove um branch e seus descendentes
    fn remove_branch(&mut self, id: &BranchId) {
        if let Some(branch) = self.branches.remove(id) {
            // Remove referência do pai
            if let Some(parent_id) = branch.parent {
                if let Some(parent) = self.branches.get_mut(&parent_id) {
                    parent.children.retain(|c| c != id);
                }
            }

            // Remove filhos recursivamente
            for child_id in branch.children {
                self.remove_branch(&child_id);
            }
        }
    }

    /// Retorna todas as folhas
    pub fn leaves(&self) -> Vec<&Branch> {
        self.branches.values().filter(|b| b.is_leaf()).collect()
    }

    /// Retorna IDs de todas as folhas
    pub fn leaf_ids(&self) -> Vec<BranchId> {
        self.branches
            .values()
            .filter(|b| b.is_leaf())
            .map(|b| b.id)
            .collect()
    }

    /// Seleciona branch por peso máximo
    pub fn select_max_weight(&self) -> Option<&Branch> {
        self.branches
            .values()
            .filter(|b| b.is_leaf())
            .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Calcula peso total das folhas
    pub fn total_leaf_weight(&self) -> f64 {
        self.branches
            .values()
            .filter(|b| b.is_leaf())
            .map(|b| b.weight)
            .sum()
    }

    /// Normaliza pesos das folhas para somar 1
    pub fn normalize_weights(&mut self) {
        let total = self.total_leaf_weight();
        if total > 0.0 {
            for branch in self.branches.values_mut() {
                if branch.is_leaf() {
                    branch.weight /= total;
                }
            }
        }
    }

    /// Aplica operação SuperOp
    pub fn apply_op(&mut self, op: SuperOp, target: &BranchId) -> Option<BranchId> {
        match op {
            SuperOp::Fork => self.fork(target),
            SuperOp::Clone => self.fork(target), // Clone é igual a fork
            SuperOp::Prune => {
                self.mark_for_prune(target);
                Some(*target)
            }
            SuperOp::Select => {
                // Marca todos exceto target para poda
                let to_mark: Vec<_> = self.branches
                    .keys()
                    .filter(|id| *id != target && *id != &BranchId::ROOT)
                    .copied()
                    .collect();
                for id in to_mark {
                    self.mark_for_prune(&id);
                }
                Some(*target)
            }
            SuperOp::Merge => {
                // Merge precisa de implementação específica
                None
            }
        }
    }
}

impl Default for MultiwayGraph {
    fn default() -> Self {
        Self::new(SilState::neutral())
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_super_strategy_from_theta() {
        assert_eq!(SuperStrategy::from_theta(0), SuperStrategy::Single);
        assert_eq!(SuperStrategy::from_theta(4), SuperStrategy::Uniform);
        assert_eq!(SuperStrategy::from_theta(14), SuperStrategy::Multiway);
    }

    #[test]
    fn test_super_strategy_roundtrip() {
        for theta in (0..16).step_by(2) {
            let strategy = SuperStrategy::from_theta(theta);
            assert_eq!(strategy.to_theta(), theta);
        }
    }

    #[test]
    fn test_branch_creation() {
        let state = SilState::neutral();
        let branch = Branch::root(state);

        assert_eq!(branch.id, BranchId::ROOT);
        assert!(branch.is_root());
        assert!(branch.is_leaf());
        assert_eq!(branch.depth, 0);
    }

    #[test]
    fn test_multiway_graph_creation() {
        let graph = MultiwayGraph::new(SilState::neutral());

        assert_eq!(graph.branch_count(), 1);
        assert_eq!(graph.leaf_count(), 1);
        assert!(graph.root().is_some());
    }

    #[test]
    fn test_fork() {
        let mut graph = MultiwayGraph::new(SilState::neutral());

        let fork_id = graph.fork(&BranchId::ROOT);
        assert!(fork_id.is_some());

        assert_eq!(graph.branch_count(), 2);
        assert_eq!(graph.leaf_count(), 1); // Só o fork é folha agora

        let root = graph.root().unwrap();
        assert!(!root.is_leaf());
        assert_eq!(root.children.len(), 1);
    }

    #[test]
    fn test_fork_n() {
        let mut graph = MultiwayGraph::new(SilState::neutral());

        let forks = graph.fork_n(&BranchId::ROOT, 3);
        assert_eq!(forks.len(), 3);
        assert_eq!(graph.branch_count(), 4);
        assert_eq!(graph.leaf_count(), 3);
    }

    #[test]
    fn test_prune() {
        let mut graph = MultiwayGraph::new(SilState::neutral());
        let forks = graph.fork_n(&BranchId::ROOT, 3);

        graph.mark_for_prune(&forks[0]);
        graph.mark_for_prune(&forks[1]);

        let pruned = graph.prune();
        assert_eq!(pruned, 2);
        assert_eq!(graph.branch_count(), 2); // root + 1 fork restante
    }

    #[test]
    fn test_select_max_weight() {
        let mut graph = MultiwayGraph::new(SilState::neutral());
        let forks = graph.fork_n(&BranchId::ROOT, 3);

        graph.get_mut(&forks[1]).unwrap().weight = 10.0;

        let selected = graph.select_max_weight();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, forks[1]);
    }

    #[test]
    fn test_normalize_weights() {
        let mut graph = MultiwayGraph::new(SilState::neutral());
        let forks = graph.fork_n(&BranchId::ROOT, 4);

        for fork in &forks {
            graph.get_mut(fork).unwrap().weight = 2.0;
        }

        graph.normalize_weights();

        let total = graph.total_leaf_weight();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_max_depth() {
        let mut graph = MultiwayGraph::new(SilState::neutral());

        let fork1 = graph.fork(&BranchId::ROOT).unwrap();
        let fork2 = graph.fork(&fork1).unwrap();
        let _fork3 = graph.fork(&fork2).unwrap();

        assert_eq!(graph.max_depth(), 3);
    }

    #[test]
    fn test_super_op_fork() {
        let mut graph = MultiwayGraph::new(SilState::neutral());

        let result = graph.apply_op(SuperOp::Fork, &BranchId::ROOT);
        assert!(result.is_some());
        assert_eq!(graph.branch_count(), 2);
    }
}
