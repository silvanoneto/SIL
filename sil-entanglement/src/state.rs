//! Estado emaranhado

use sil_core::prelude::*;
use sil_core::traits::{Entangled, EntanglementError as CoreEntanglementError};
use crate::registry::EntanglementRegistry;
use std::sync::atomic::{AtomicU64, Ordering};

// Contador global para IDs únicos
static GLOBAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Estado emaranhado com pares
#[derive(Debug)]
pub struct EntangledState {
    /// Estado SIL interno
    state: SilState,
    /// Registro de pares
    registry: EntanglementRegistry,
    /// ID único deste estado
    id: u64,
    /// Força de correlação padrão
    default_correlation: f32,
}

impl Clone for EntangledState {
    fn clone(&self) -> Self {
        // Gera novo ID único usando o contador global
        let id = GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Self {
            state: self.state,
            registry: self.registry.clone(),
            id,
            default_correlation: self.default_correlation,
        }
    }
}

impl EntangledState {
    /// Cria novo estado emaranhado
    pub fn new(state: SilState) -> Self {
        // Usa contador global para garantir IDs únicos
        let id = GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Self {
            state,
            registry: EntanglementRegistry::new(),
            id,
            default_correlation: 1.0,
        }
    }

    /// Estado interno
    pub fn state(&self) -> &SilState {
        &self.state
    }

    /// Estado interno mutável
    pub fn state_mut(&mut self) -> &mut SilState {
        &mut self.state
    }

    /// ID deste estado
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Define correlação padrão
    pub fn set_default_correlation(&mut self, correlation: f32) {
        self.default_correlation = correlation.clamp(0.0, 1.0);
    }

    /// Número de pares
    pub fn pair_count(&self) -> usize {
        self.registry.pair_count()
    }

    /// Sincroniza estado com par (média ponderada pela correlação)
    fn sync_with_pair(&mut self, pair_id: u64) -> Result<(), CoreEntanglementError> {
        let pair_info = self.registry.get_pair(pair_id)
            .ok_or_else(|| CoreEntanglementError::PairNotFound(pair_id.to_string()))?;

        let correlation = pair_info.correlation;
        if correlation < 0.1 {
            return Err(CoreEntanglementError::Broken);
        }

        let other_state = pair_info.correlated_state;

        // Sincroniza camadas baseado na correlação
        let mut new_state = self.state;

        // Sincroniza todas as camadas
        for layer_idx in 0..NUM_LAYERS {
            let local = self.state.get(layer_idx);
            let remote = other_state.get(layer_idx);

            // Média ponderada pela correlação nos componentes log-polares
            let local_rho = local.rho as f32;
            let remote_rho = remote.rho as f32;
            let synced_rho = local_rho * (1.0 - correlation) + remote_rho * correlation;

            let local_theta = local.theta as f32;
            let remote_theta = remote.theta as f32;
            let synced_theta = local_theta * (1.0 - correlation) + remote_theta * correlation;

            // Cria novo ByteSil com valores interpolados
            let synced_bytesil = ByteSil {
                rho: synced_rho.clamp(-8.0, 7.0).round() as i8,
                theta: (synced_theta.round() as u8) % 16,
            };
            new_state.set_layer(layer_idx, synced_bytesil);
        }

        self.state = new_state;

        // Reduz correlação levemente a cada sync (entropia)
        self.registry.reduce_correlation(pair_id, 0.01);

        Ok(())
    }

    /// Atualiza estado do par
    pub fn update_pair_state(&mut self, pair_id: u64, state: SilState) -> Result<(), CoreEntanglementError> {
        let pair_info = self.registry.get_pair_mut(pair_id)
            .ok_or_else(|| CoreEntanglementError::PairNotFound(pair_id.to_string()))?;

        pair_info.correlated_state = state;
        Ok(())
    }
}

impl Entangled for EntangledState {
    type PairId = u64;

    fn entangle(&mut self, other: &mut Self) -> Result<Self::PairId, CoreEntanglementError> {
        // Verifica se já está emaranhado com o mesmo estado
        if self.id == other.id {
            return Err(CoreEntanglementError::AlreadyEntangled);
        }

        // Cria par em ambos os registros
        let pair_id = self.registry.add_pair(other.state, self.default_correlation);
        let _other_pair_id = other.registry.add_pair(self.state, other.default_correlation);

        // IDs devem ser consistentes (simplificação)
        Ok(pair_id)
    }

    fn is_entangled_with(&self, pair_id: &Self::PairId) -> bool {
        self.registry.has_pair(*pair_id)
    }

    fn sync(&mut self, pair_id: &Self::PairId) -> Result<(), CoreEntanglementError> {
        self.sync_with_pair(*pair_id)
    }

    fn disentangle(&mut self, pair_id: &Self::PairId) -> Result<(), CoreEntanglementError> {
        if self.registry.remove_pair(*pair_id) {
            Ok(())
        } else {
            Err(CoreEntanglementError::PairNotFound(pair_id.to_string()))
        }
    }

    fn entangled_pairs(&self) -> Vec<Self::PairId> {
        self.registry.all_pairs()
    }
}

impl SilComponent for EntangledState {
    fn name(&self) -> &str {
        "EntangledState"
    }

    fn layers(&self) -> &[LayerId] {
        &[14] // LE - Entanglement
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
    use sil_core::traits::Entangled;

    #[test]
    fn test_create_entangled_state() {
        let state = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 0, theta: 0 }));
        assert_eq!(state.pair_count(), 0);
    }

    #[test]
    fn test_entangle_states() {
        let mut state1 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
        let mut state2 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 2, theta: 0 }));

        let pair_id = state1.entangle(&mut state2).unwrap();
        assert!(state1.is_entangled_with(&pair_id));
    }

    #[test]
    fn test_entangle_self() {
        let mut state1 = EntangledState::new(SilState::neutral());
        let result = state1.entangle(&mut state1.clone());
        // Deve permitir porque clone tem ID diferente
        assert!(result.is_ok());
    }
}
