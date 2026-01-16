//! Gerenciador de fork/merge de estados

use num_complex::Complex;
use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{Forkable, MergeError};
use crate::error::{SuperpositionError, SuperpositionResult};
use crate::strategy::MergeStrategy;

/// Estado que pode ser forkado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkableState {
    /// Estado SIL interno
    state: SilState,
    /// ID do fork original (0 = original)
    fork_id: u64,
    /// Timestamp do fork
    fork_timestamp: u64,
    /// Estratégia de merge padrão
    default_strategy: MergeStrategy,
}

impl ForkableState {
    /// Cria novo estado forkable
    pub fn new(state: SilState) -> Self {
        Self {
            state,
            fork_id: 0,
            fork_timestamp: 0,
            default_strategy: MergeStrategy::default(),
        }
    }

    /// Retorna referência ao estado interno
    pub fn state(&self) -> &SilState {
        &self.state
    }

    /// Retorna estado interno mutável
    pub fn state_mut(&mut self) -> &mut SilState {
        &mut self.state
    }

    /// Retorna ID do fork
    pub fn fork_id(&self) -> u64 {
        self.fork_id
    }

    /// Define estratégia de merge padrão
    pub fn set_default_strategy(&mut self, strategy: MergeStrategy) {
        self.default_strategy = strategy;
    }

    /// Verifica divergência baseada em diferença de valores
    fn calculate_divergence(&self, other: &Self) -> f32 {
        let mut total_diff = 0.0;
        let count = 16; // NUM_LAYERS

        // Itera sobre todas as 16 camadas
        for layer_id in 0..16 {
            let byte_a = self.state.get(layer_id);
            let byte_b = other.state.get(layer_id);

            // Converte para complexo e calcula diferença de magnitude
            let complex_a = byte_a.to_complex();
            let complex_b = byte_b.to_complex();
            total_diff += (complex_a.norm() - complex_b.norm()).abs() as f32;
        }

        total_diff / count as f32
    }
}

impl Forkable for ForkableState {
    fn fork(&self) -> Self {
        let mut forked = self.clone();
        // Gera ID único baseado em timestamp simulado
        forked.fork_id = self.fork_id + 1;
        forked.fork_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        forked
    }

    fn merge(&mut self, other: &Self) -> Result<(), MergeError> {
        // Usa estratégia padrão para merge
        let strategy = self.default_strategy;

        // Merge camada por camada (todas as 16 camadas)
        let mut new_state = SilState::neutral();
        for layer_id in 0..16 {
            let byte_a = self.state.get(layer_id);
            let byte_b = other.state.get(layer_id);

            // Converte para complexo, aplica estratégia na magnitude, mantém fase do primeiro
            let complex_a = byte_a.to_complex();
            let complex_b = byte_b.to_complex();

            let mag_a = complex_a.norm();
            let mag_b = complex_b.norm();
            let merged_mag = strategy.apply(mag_a as f32, mag_b as f32) as f64;

            // Mantém a fase do estado A
            let phase_a = complex_a.arg();
            let merged_complex = Complex::from_polar(merged_mag, phase_a);
            let merged_byte = ByteSil::from_complex(merged_complex);

            new_state = new_state.with_layer(layer_id, merged_byte);
        }

        self.state = new_state;
        Ok(())
    }

    fn has_diverged(&self, other: &Self) -> bool {
        let divergence = self.calculate_divergence(other);
        divergence > 1.0 // Threshold arbitrário
    }
}

/// Gerenciador de estados com fork/merge
#[derive(Debug, Clone)]
pub struct StateManager {
    /// Estado principal
    main: ForkableState,
    /// Forks ativos
    forks: Vec<ForkableState>,
    /// Estratégia de merge padrão
    default_strategy: MergeStrategy,
}

impl StateManager {
    /// Cria novo gerenciador
    pub fn new(initial_state: SilState) -> Self {
        Self {
            main: ForkableState::new(initial_state),
            forks: Vec::new(),
            default_strategy: MergeStrategy::default(),
        }
    }

    /// Cria fork do estado principal
    pub fn fork(&mut self) -> ForkableState {
        let forked = self.main.fork();
        self.forks.push(forked.clone());
        forked
    }

    /// Merge fork no estado principal
    pub fn merge_fork(&mut self, fork: &ForkableState) -> SuperpositionResult<()> {
        self.main.merge(fork)
            .map_err(SuperpositionError::from)
    }

    /// Merge fork com estratégia específica
    pub fn merge_with_strategy(
        &mut self,
        fork: &ForkableState,
        strategy: MergeStrategy,
    ) -> SuperpositionResult<()> {
        let old_strategy = self.main.default_strategy;
        self.main.set_default_strategy(strategy);
        let result = self.merge_fork(fork);
        self.main.set_default_strategy(old_strategy);
        result
    }

    /// Merge todos os forks ativos
    pub fn merge_all(&mut self) -> SuperpositionResult<()> {
        let forks = self.forks.drain(..).collect::<Vec<_>>();
        for fork in forks {
            self.merge_fork(&fork)?;
        }
        Ok(())
    }

    /// Estado principal
    pub fn main_state(&self) -> &SilState {
        self.main.state()
    }

    /// Número de forks ativos
    pub fn fork_count(&self) -> usize {
        self.forks.len()
    }

    /// Limpa todos os forks
    pub fn clear_forks(&mut self) {
        self.forks.clear();
    }

    /// Define estratégia padrão
    pub fn set_default_strategy(&mut self, strategy: MergeStrategy) {
        self.default_strategy = strategy;
        self.main.set_default_strategy(strategy);
    }
}

impl SilComponent for StateManager {
    fn name(&self) -> &str {
        "StateManager"
    }

    fn layers(&self) -> &[LayerId] {
        &[13] // LD - Superposição
    }

    fn version(&self) -> &str {
        "2026.1.11"
    }

    fn is_ready(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_forkable_state() {
        let byte_val = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
        let state = SilState::neutral().with_layer(0, byte_val);
        let forkable = ForkableState::new(state);
        assert_eq!(forkable.fork_id(), 0);
    }

    #[test]
    fn test_fork_state() {
        let byte_val = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
        let state = SilState::neutral().with_layer(0, byte_val);
        let forkable = ForkableState::new(state);
        let forked = forkable.fork();

        assert_eq!(forked.fork_id(), 1);
        let retrieved = forked.state().get(0);
        assert!((retrieved.to_complex().norm() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_merge_average() {
        let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
        let byte20 = ByteSil::from_complex(Complex::from_polar(20.09, 0.0));

        let mut state1 = ForkableState::new(SilState::neutral().with_layer(0, byte1));
        let state2 = ForkableState::new(SilState::neutral().with_layer(0, byte20));

        state1.merge(&state2).unwrap();
        let result_mag = state1.state().get(0).to_complex().norm();
        // (1.0 + 20.09)/2 ≈ 10.5 → ρ=2 → 7.39
        assert!((result_mag - 7.39).abs() < 1.0);
    }

    #[test]
    fn test_manager_fork() {
        let byte_val = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
        let mut manager = StateManager::new(SilState::neutral().with_layer(0, byte_val));
        let fork = manager.fork();

        assert_eq!(manager.fork_count(), 1);
        let retrieved = fork.state().get(0);
        assert!((retrieved.to_complex().norm() - 1.0).abs() < 0.1);
    }
}
