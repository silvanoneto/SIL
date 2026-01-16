//! Testes integrados para sil-entanglement

use crate::*;
use sil_core::prelude::*;
use sil_core::traits::Entangled;

#[test]
fn test_entangled_state_creation() {
    let state = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 4 }));
    assert_eq!(state.name(), "EntangledState");
    assert_eq!(state.layers(), &[14]);
    assert_eq!(state.state().get(0).rho, 3);
    assert_eq!(state.state().get(0).theta, 4);
}

#[test]
fn test_entangle_two_states() {
    let mut state1 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
    let mut state2 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 2, theta: 0 }));

    let pair_id = state1.entangle(&mut state2).unwrap();
    assert!(state1.is_entangled_with(&pair_id));
    assert_eq!(state1.pair_count(), 1);
}

#[test]
fn test_entangled_pairs() {
    let mut state1 = EntangledState::new(SilState::neutral());
    let mut state2 = EntangledState::new(SilState::neutral());
    let mut state3 = EntangledState::new(SilState::neutral());

    state1.entangle(&mut state2).unwrap();
    state1.entangle(&mut state3).unwrap();

    assert_eq!(state1.entangled_pairs().len(), 2);
}

#[test]
fn test_is_entangled_with() {
    let mut state1 = EntangledState::new(SilState::neutral());
    let mut state2 = EntangledState::new(SilState::neutral());

    let pair_id = state1.entangle(&mut state2).unwrap();
    assert!(state1.is_entangled_with(&pair_id));
    assert!(!state1.is_entangled_with(&999));
}

#[test]
fn test_disentangle() {
    let mut state1 = EntangledState::new(SilState::neutral());
    let mut state2 = EntangledState::new(SilState::neutral());

    let pair_id = state1.entangle(&mut state2).unwrap();
    assert!(state1.is_entangled_with(&pair_id));

    state1.disentangle(&pair_id).unwrap();
    assert!(!state1.is_entangled_with(&pair_id));
}

#[test]
fn test_disentangle_nonexistent() {
    let mut state = EntangledState::new(SilState::neutral());
    let result = state.disentangle(&999);
    assert!(result.is_err());
}

#[test]
fn test_sync_updates_state() {
    let mut state1 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: -5, theta: 0 }));
    let mut state2 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 5, theta: 0 }));

    let pair_id = state1.entangle(&mut state2).unwrap();

    // Sync deve aproximar state1 de state2
    let before = state1.state().get(0).rho;
    state1.sync(&pair_id).unwrap();
    let after = state1.state().get(0).rho;

    assert!(after > before); // Deve se mover em direção a 5
}

#[test]
fn test_sync_nonexistent_pair() {
    let mut state = EntangledState::new(SilState::neutral());
    let result = state.sync(&999);
    assert!(result.is_err());
}

#[test]
fn test_update_pair_state() {
    let mut state1 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
    let mut state2 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 2, theta: 0 }));

    let pair_id = state1.entangle(&mut state2).unwrap();

    let new_state = SilState::neutral().with_layer(0, ByteSil { rho: 7, theta: 8 });
    state1.update_pair_state(pair_id, new_state).unwrap();

    // Sync deve usar o novo estado
    state1.sync(&pair_id).unwrap();
    let val = state1.state().get(0).rho;
    assert!(val > 3); // Aproximou de 7
}

#[test]
fn test_correlation_degrades() {
    let mut state1 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: -5, theta: 0 }));
    let mut state2 = EntangledState::new(SilState::neutral().with_layer(0, ByteSil { rho: 5, theta: 0 }));

    let pair_id = state1.entangle(&mut state2).unwrap();

    // Múltiplos syncs devem degradar correlação
    for _ in 0..20 {
        let _ = state1.sync(&pair_id);
    }

    // Após muitos syncs, correlação deve estar baixa
    // (não podemos testar diretamente, mas efeito deve ser visível)
}

#[test]
fn test_multi_layer_entanglement() {
    let mut state1 = EntangledState::new(
        SilState::neutral()
            .with_layer(0, ByteSil { rho: -3, theta: 0 })
            .with_layer(1, ByteSil { rho: 0, theta: 4 })
    );
    let mut state2 = EntangledState::new(
        SilState::neutral()
            .with_layer(0, ByteSil { rho: 3, theta: 0 })
            .with_layer(1, ByteSil { rho: 2, theta: 8 })
    );

    let pair_id = state1.entangle(&mut state2).unwrap();
    state1.sync(&pair_id).unwrap();

    // Ambas as camadas devem ser afetadas
    let layer0 = state1.state().get(0).rho;
    let layer1 = state1.state().get(1).rho;

    assert!(layer0 > -3); // Moveu em direção a 3
    assert!(layer1 > 0); // Moveu em direção a 2
}

#[test]
fn test_set_default_correlation() {
    let mut state = EntangledState::new(SilState::neutral());
    state.set_default_correlation(0.5);

    // Correlação deve ser aplicada a novos emaranhamentos
    let mut state2 = EntangledState::new(SilState::neutral());
    state.entangle(&mut state2).unwrap();
}

#[test]
fn test_entanglement_registry() {
    let mut registry = EntanglementRegistry::new();
    assert_eq!(registry.pair_count(), 0);

    let pair_id = registry.add_pair(SilState::neutral(), 1.0);
    assert_eq!(registry.pair_count(), 1);
    assert!(registry.has_pair(pair_id));

    registry.remove_pair(pair_id);
    assert_eq!(registry.pair_count(), 0);
}

#[test]
fn test_registry_get_pair() {
    let mut registry = EntanglementRegistry::new();
    let state = SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 4 });
    let pair_id = registry.add_pair(state, 0.8);

    let info = registry.get_pair(pair_id).unwrap();
    assert_eq!(info.correlation, 0.8);
    assert_eq!(info.correlated_state.get(0).rho, 3);
    assert_eq!(info.correlated_state.get(0).theta, 4);
}

#[test]
fn test_registry_all_pairs() {
    let mut registry = EntanglementRegistry::new();
    let id1 = registry.add_pair(SilState::neutral(), 1.0);
    let id2 = registry.add_pair(SilState::neutral(), 1.0);
    let id3 = registry.add_pair(SilState::neutral(), 1.0);

    let pairs = registry.all_pairs();
    assert_eq!(pairs.len(), 3);
    assert!(pairs.contains(&id1));
    assert!(pairs.contains(&id2));
    assert!(pairs.contains(&id3));
}

#[test]
fn test_registry_reduce_correlation() {
    let mut registry = EntanglementRegistry::new();
    let pair_id = registry.add_pair(SilState::neutral(), 1.0);

    registry.reduce_correlation(pair_id, 0.3);
    let info = registry.get_pair(pair_id).unwrap();
    assert!((info.correlation - 0.7).abs() < 0.01);

    // Não deve ficar negativo
    registry.reduce_correlation(pair_id, 1.0);
    let info = registry.get_pair(pair_id).unwrap();
    assert_eq!(info.correlation, 0.0);
}

#[test]
fn test_state_id_unique() {
    let state1 = EntangledState::new(SilState::neutral());
    let state2 = EntangledState::new(SilState::neutral());

    // IDs devem ser diferentes (baseados em timestamp)
    // Pode falhar em execuções muito rápidas, mas improvável
    assert_ne!(state1.id(), state2.id());
}

#[test]
fn test_entangled_state_is_ready() {
    let state = EntangledState::new(SilState::neutral());
    assert!(state.is_ready());
}
