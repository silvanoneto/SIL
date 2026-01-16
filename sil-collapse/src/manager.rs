//! Gerenciador de collapse e checkpoints

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{Collapsible, CollapseError as CoreCollapseError};
use crate::checkpoint::{CheckpointStorage, CheckpointId};

/// Estado que pode colapsar
#[derive(Debug, Clone)]
pub struct CollapsibleState {
    /// Estado SIL interno
    state: SilState,
    /// Estado inicial
    initial_state: SilState,
    /// Storage de checkpoints
    storage: CheckpointStorage,
    /// Threshold para colapso (baseado em L(F))
    collapse_threshold: f32,
    /// Contador de operações desde último colapso
    operations_count: u64,
}

impl CollapsibleState {
    /// Cria novo estado collapsible
    pub fn new(state: SilState) -> Self {
        Self {
            initial_state: state.clone(),
            state,
            storage: CheckpointStorage::default(),
            collapse_threshold: 100.0,
            operations_count: 0,
        }
    }

    /// Cria com limite customizado de checkpoints
    pub fn with_max_checkpoints(state: SilState, max_checkpoints: usize) -> Self {
        Self {
            initial_state: state.clone(),
            state,
            storage: CheckpointStorage::new(max_checkpoints),
            collapse_threshold: 100.0,
            operations_count: 0,
        }
    }

    /// Estado atual
    pub fn state(&self) -> &SilState {
        &self.state
    }

    /// Estado mutável
    pub fn state_mut(&mut self) -> &mut SilState {
        self.operations_count += 1;
        &mut self.state
    }

    /// Define threshold de colapso
    pub fn set_collapse_threshold(&mut self, threshold: f32) {
        self.collapse_threshold = threshold;
    }

    /// Número de checkpoints
    pub fn checkpoint_count(&self) -> usize {
        self.storage.count()
    }

    /// Calcula valor de L(F) - medida de "finalidade"
    fn calculate_finality(&self) -> f32 {
        // L(F) baseado em:
        // - Número de operações
        // - Divergência do estado inicial
        // - Número de checkpoints

        let operation_factor = self.operations_count as f32;

        let mut divergence = 0.0;
        let count = NUM_LAYERS;

        for layer_idx in 0..NUM_LAYERS {
            let current = self.state.get(layer_idx);
            let initial = self.initial_state.get(layer_idx);

            // Calcula divergência usando diferença de rho (log-magnitude)
            let rho_diff = (current.rho - initial.rho).abs() as f32;
            let theta_diff = ((current.theta as i16 - initial.theta as i16).abs() % 16) as f32;
            divergence += rho_diff + theta_diff * 0.1; // Peso menor para theta
        }

        let divergence_factor = if count > 0 {
            divergence / count as f32
        } else {
            0.0
        };

        let checkpoint_factor = self.storage.count() as f32;

        // Combina fatores
        operation_factor * 0.5 + divergence_factor * 10.0 + checkpoint_factor * 5.0
    }
}

impl SilComponent for CollapsibleState {
    fn name(&self) -> &str {
        "CollapsibleState"
    }

    fn layers(&self) -> &[LayerId] {
        &[15] // LF - Collapse
    }

    fn version(&self) -> &str {
        "2026.1.11"
    }

    fn is_ready(&self) -> bool {
        true
    }
}

impl Collapsible for CollapsibleState {
    type CheckpointId = CheckpointId;

    fn checkpoint(&mut self) -> Result<Self::CheckpointId, CoreCollapseError> {
        let id = self.storage.add(self.state, None);
        Ok(id)
    }

    fn restore(&mut self, id: &Self::CheckpointId) -> Result<(), CoreCollapseError> {
        let checkpoint = self.storage.get(*id)
            .ok_or_else(|| CoreCollapseError::CheckpointNotFound(id.to_string()))?;

        self.state = checkpoint.state;
        self.operations_count = 0;
        Ok(())
    }

    fn collapse(&mut self) -> Result<SilState, CoreCollapseError> {
        if !self.should_collapse() {
            return Err(CoreCollapseError::CannotCollapse(
                "Collapse threshold not reached".into()
            ));
        }

        let collapsed = self.state;

        // Reset para estado inicial
        self.state = self.initial_state;
        self.operations_count = 0;
        self.storage.clear();

        Ok(collapsed)
    }

    fn should_collapse(&self) -> bool {
        self.calculate_finality() >= self.collapse_threshold
    }

    fn checkpoints(&self) -> Vec<Self::CheckpointId> {
        self.storage.list()
    }
}

/// Gerenciador de collapse com configuração
#[derive(Debug, Clone)]
pub struct CollapseManager {
    /// Estado collapsible
    state: CollapsibleState,
    /// Configuração
    config: CollapseConfig,
}

/// Configuração do gerenciador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseConfig {
    /// Máximo de checkpoints
    pub max_checkpoints: usize,
    /// Threshold de colapso
    pub collapse_threshold: f32,
    /// Auto-checkpoint a cada N operações
    pub auto_checkpoint_interval: Option<u64>,
    /// Auto-collapse quando threshold atingido
    pub auto_collapse: bool,
}

impl Default for CollapseConfig {
    fn default() -> Self {
        Self {
            max_checkpoints: 10,
            collapse_threshold: 100.0,
            auto_checkpoint_interval: None,
            auto_collapse: false,
        }
    }
}

impl CollapseManager {
    /// Cria novo gerenciador
    pub fn new(state: SilState) -> Self {
        Self::with_config(state, CollapseConfig::default())
    }

    /// Cria com configuração customizada
    pub fn with_config(state: SilState, config: CollapseConfig) -> Self {
        let mut collapsible = CollapsibleState::with_max_checkpoints(
            state,
            config.max_checkpoints,
        );
        collapsible.set_collapse_threshold(config.collapse_threshold);

        Self {
            state: collapsible,
            config,
        }
    }

    /// Estado atual
    pub fn state(&self) -> &SilState {
        self.state.state()
    }

    /// Estado mutável (incrementa contador)
    pub fn state_mut(&mut self) -> &mut SilState {
        self.state.state_mut()
    }

    /// Cria checkpoint
    pub fn checkpoint(&mut self) -> Result<CheckpointId, CoreCollapseError> {
        self.state.checkpoint()
    }

    /// Restaura checkpoint
    pub fn restore(&mut self, id: &CheckpointId) -> Result<(), CoreCollapseError> {
        self.state.restore(id)
    }

    /// Força colapso
    pub fn collapse(&mut self) -> Result<SilState, CoreCollapseError> {
        self.state.collapse()
    }

    /// Verifica se deve colapsar
    pub fn should_collapse(&self) -> bool {
        self.state.should_collapse()
    }

    /// Lista checkpoints
    pub fn checkpoints(&self) -> Vec<CheckpointId> {
        self.state.checkpoints()
    }

    /// Número de checkpoints
    pub fn checkpoint_count(&self) -> usize {
        self.state.checkpoint_count()
    }

    /// Processa operação com auto-checkpoint
    pub fn process<F>(&mut self, f: F) -> Result<(), CoreCollapseError>
    where
        F: FnOnce(&mut SilState),
    {
        // Auto-checkpoint se configurado
        if let Some(interval) = self.config.auto_checkpoint_interval {
            if self.state.operations_count % interval == 0 {
                self.checkpoint()?;
            }
        }

        // Executa operação
        f(self.state_mut());

        // Auto-collapse se configurado
        if self.config.auto_collapse && self.should_collapse() {
            self.collapse()?;
        }

        Ok(())
    }
}

impl SilComponent for CollapseManager {
    fn name(&self) -> &str {
        "CollapseManager"
    }

    fn layers(&self) -> &[LayerId] {
        &[15] // LF - Colapso
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
    use sil_core::traits::Collapsible;

    #[test]
    fn test_create_collapsible_state() {
        let state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
        assert_eq!(state.checkpoint_count(), 0);
    }

    #[test]
    fn test_checkpoint_and_restore() {
        let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));

        let checkpoint_id = state.checkpoint().unwrap();
        *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 7, theta: 8 });

        state.restore(&checkpoint_id).unwrap();
        assert_eq!(state.state().get(0).rho, 1);
    }

    #[test]
    fn test_collapse() {
        let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
        state.set_collapse_threshold(0.0); // Força colapso

        // Modifica estado
        *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 7, theta: 8 });

        let collapsed = state.collapse().unwrap();
        assert_eq!(collapsed.get(0).rho, 7);

        // Deve ter resetado para inicial
        assert_eq!(state.state().get(0).rho, 1);
    }
}
