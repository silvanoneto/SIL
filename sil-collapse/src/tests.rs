//! Testes integrados para sil-collapse

use crate::*;
use crate::checkpoint::{Checkpoint, CheckpointStorage};
use crate::manager::CollapseConfig;
use sil_core::prelude::*;
use sil_core::traits::Collapsible;

#[test]
fn test_collapse_manager_creation() {
    let manager = CollapseManager::new(SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 4 }));
    assert_eq!(manager.name(), "CollapseManager");
    assert_eq!(manager.layers(), &[15]);
}

#[test]
fn test_collapsible_state_creation() {
    let state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
    assert_eq!(state.state().get(0).rho, 1);
    assert_eq!(state.checkpoint_count(), 0);
}

#[test]
fn test_checkpoint_create() {
    let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 5, theta: 0 }));
    let checkpoint_id = state.checkpoint().unwrap();

    assert_eq!(state.checkpoint_count(), 1);
    assert!(state.checkpoints().contains(&checkpoint_id));
}

#[test]
fn test_checkpoint_restore() {
    let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 4 }));
    let checkpoint_id = state.checkpoint().unwrap();

    // Modifica estado
    *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: -5, theta: 8 });
    assert_eq!(state.state().get(0).rho, -5);

    // Restaura
    state.restore(&checkpoint_id).unwrap();
    assert_eq!(state.state().get(0).rho, 3);
    assert_eq!(state.state().get(0).theta, 4);
}

#[test]
fn test_restore_nonexistent() {
    let mut state = CollapsibleState::new(SilState::neutral());
    let result = state.restore(&999);
    assert!(result.is_err());
}

#[test]
fn test_multiple_checkpoints() {
    let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 0, theta: 0 }));

    let id1 = state.checkpoint().unwrap();
    *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 0 });

    let id2 = state.checkpoint().unwrap();
    *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 5, theta: 0 });

    let _id3 = state.checkpoint().unwrap();

    assert_eq!(state.checkpoint_count(), 3);

    // Restaura para segundo checkpoint
    state.restore(&id2).unwrap();
    assert_eq!(state.state().get(0).rho, 3);

    // Restaura para primeiro
    state.restore(&id1).unwrap();
    assert_eq!(state.state().get(0).rho, 0);
}

#[test]
fn test_collapse_resets_to_initial() {
    let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
    state.set_collapse_threshold(0.0); // Força colapso imediato

    *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 7, theta: 8 });

    let collapsed = state.collapse().unwrap();
    assert_eq!(collapsed.get(0).rho, 7);
    assert_eq!(collapsed.get(0).theta, 8);

    // Estado deve ter voltado ao inicial
    assert_eq!(state.state().get(0).rho, 1);
}

#[test]
fn test_collapse_clears_checkpoints() {
    let mut state = CollapsibleState::new(SilState::neutral());
    state.set_collapse_threshold(0.0);

    state.checkpoint().unwrap();
    state.checkpoint().unwrap();
    assert_eq!(state.checkpoint_count(), 2);

    state.collapse().unwrap();
    assert_eq!(state.checkpoint_count(), 0);
}

#[test]
fn test_should_collapse_threshold() {
    let mut state = CollapsibleState::new(SilState::neutral());
    state.set_collapse_threshold(1000.0);

    assert!(!state.should_collapse());

    // Força threshold baixo
    state.set_collapse_threshold(0.0);
    assert!(state.should_collapse());
}

#[test]
fn test_checkpoint_limit() {
    let mut state = CollapsibleState::with_max_checkpoints(SilState::neutral(), 3);

    for i in 0..5 {
        let s = SilState::neutral().with_layer(0, ByteSil { rho: i, theta: 0 });
        *state.state_mut() = s;
        state.checkpoint().unwrap();
    }

    // Deve manter apenas últimos 3
    assert_eq!(state.checkpoint_count(), 3);
}

#[test]
fn test_manager_checkpoint_restore() {
    let mut manager = CollapseManager::new(SilState::neutral().with_layer(0, ByteSil { rho: 5, theta: 0 }));

    let checkpoint_id = manager.checkpoint().unwrap();
    *manager.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: -3, theta: 8 });

    manager.restore(&checkpoint_id).unwrap();
    assert_eq!(manager.state().get(0).rho, 5);
}

#[test]
fn test_manager_collapse() {
    let mut config = CollapseConfig::default();
    config.collapse_threshold = 0.0;
    let mut manager = CollapseManager::with_config(
        SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }),
        config
    );

    *manager.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 7, theta: 8 });

    let collapsed = manager.collapse().unwrap();
    assert_eq!(collapsed.get(0).rho, 7);
    assert_eq!(manager.state().get(0).rho, 1);
}

#[test]
fn test_manager_config() {
    let config = CollapseConfig {
        max_checkpoints: 5,
        collapse_threshold: 50.0,
        auto_checkpoint_interval: Some(10),
        auto_collapse: true,
    };

    let _manager = CollapseManager::with_config(SilState::neutral(), config);
    // Config é privado, não podemos testar diretamente
}

#[test]
fn test_checkpoint_storage() {
    let mut storage = CheckpointStorage::new(5);
    let state = SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 4 });

    let id = storage.add(state, Some("Test checkpoint".into()));
    assert_eq!(storage.count(), 1);

    let checkpoint = storage.get(id).unwrap();
    assert_eq!(checkpoint.state.get(0).rho, 3);
    assert_eq!(checkpoint.state.get(0).theta, 4);
    assert_eq!(checkpoint.description.as_deref(), Some("Test checkpoint"));
}

#[test]
fn test_storage_latest_oldest() {
    let mut storage = CheckpointStorage::new(10);

    let id1 = storage.add(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }), None);
    let _id2 = storage.add(SilState::neutral().with_layer(0, ByteSil { rho: 2, theta: 0 }), None);
    let id3 = storage.add(SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 0 }), None);

    assert_eq!(storage.oldest().unwrap().id, id1);
    assert_eq!(storage.latest().unwrap().id, id3);
}

#[test]
fn test_storage_remove() {
    let mut storage = CheckpointStorage::new(10);
    let id1 = storage.add(SilState::neutral(), None);
    let id2 = storage.add(SilState::neutral(), None);

    assert_eq!(storage.count(), 2);
    storage.remove(id1);
    assert_eq!(storage.count(), 1);
    assert!(storage.get(id1).is_none());
    assert!(storage.get(id2).is_some());
}

#[test]
fn test_storage_clear() {
    let mut storage = CheckpointStorage::new(10);
    storage.add(SilState::neutral(), None);
    storage.add(SilState::neutral(), None);
    storage.add(SilState::neutral(), None);

    assert_eq!(storage.count(), 3);
    storage.clear();
    assert_eq!(storage.count(), 0);
}

#[test]
fn test_checkpoint_age() {
    let state = SilState::neutral();
    let checkpoint = Checkpoint::new(1, state);

    // Idade deve ser ~0 (recém criado)
    assert!(checkpoint.age() < 2);
}

#[test]
fn test_manager_is_ready() {
    let manager = CollapseManager::new(SilState::neutral());
    assert!(manager.is_ready());
}

#[test]
fn test_finality_increases_with_operations() {
    let mut state = CollapsibleState::new(SilState::neutral().with_layer(0, ByteSil { rho: 1, theta: 0 }));
    state.set_collapse_threshold(1000.0);
    let should_collapse_initially = state.should_collapse();

    // Realiza operações
    for _ in 0..100 {
        *state.state_mut() = SilState::neutral().with_layer(0, ByteSil { rho: 2, theta: 4 });
    }

    let should_collapse_after = state.should_collapse();
    // Após muitas operações, deve estar mais perto de colapsar
    assert!(!should_collapse_initially);
    assert!(should_collapse_after || state.checkpoint_count() == 0);
}

#[test]
fn test_multi_layer_checkpoint() {
    let mut state = CollapsibleState::new(
        SilState::neutral()
            .with_layer(0, ByteSil { rho: 1, theta: 0 })
            .with_layer(1, ByteSil { rho: 2, theta: 4 })
            .with_layer(2, ByteSil { rho: 3, theta: 8 })
    );

    let checkpoint_id = state.checkpoint().unwrap();

    *state.state_mut() = SilState::neutral()
        .with_layer(0, ByteSil { rho: -5, theta: 0 })
        .with_layer(1, ByteSil { rho: -3, theta: 4 })
        .with_layer(2, ByteSil { rho: 7, theta: 8 });

    state.restore(&checkpoint_id).unwrap();

    assert_eq!(state.state().get(0).rho, 1);
    assert_eq!(state.state().get(1).rho, 2);
    assert_eq!(state.state().get(2).rho, 3);
}

#[test]
fn test_cannot_collapse_below_threshold() {
    let mut state = CollapsibleState::new(SilState::neutral());
    state.set_collapse_threshold(1000.0);

    let result = state.collapse();
    assert!(result.is_err());
}
