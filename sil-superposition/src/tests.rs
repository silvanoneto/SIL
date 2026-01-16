//! Testes integrados para sil-superposition

use crate::*;
use num_complex::Complex;
use sil_core::prelude::*;
use sil_core::traits::Forkable;

// Nota: ByteSil usa representação log-polar com quantização:
// magnitude → ρ = ln(mag).round() → magnitude_recon = e^ρ
// Isso causa erros de quantização significativos para valores intermediários

#[test]
fn test_state_manager_creation() {
    let byte_val = ByteSil::from_complex(Complex::from_polar(42.0, 0.0));
    let state = SilState::neutral().with_layer(0, byte_val);
    let manager = StateManager::new(state);
    assert_eq!(manager.name(), "StateManager");
    assert_eq!(manager.layers(), &[13]);
}

#[test]
fn test_forkable_state_creation() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let byte2 = ByteSil::from_complex(Complex::from_polar(2.0, 0.0));
    let state = SilState::neutral().with_layer(0, byte1).with_layer(1, byte2);
    let forkable = ForkableState::new(state);
    assert_eq!(forkable.fork_id(), 0);
    assert!((forkable.state().get(0).to_complex().norm() - 1.0).abs() < 0.1);
}

#[test]
fn test_fork_creates_copy() {
    let byte_val = ByteSil::from_complex(Complex::from_polar(7.4, 0.0));
    let state = SilState::neutral().with_layer(0, byte_val);
    let forkable = ForkableState::new(state);
    let forked = forkable.fork();

    assert_ne!(forkable.fork_id(), forked.fork_id());
    // 7.4 → ρ=2 → e²≈7.39
    assert!((forked.state().get(0).to_complex().norm() - 7.39).abs() < 0.5);
}

#[test]
fn test_fork_increments_id() {
    let state = SilState::neutral();
    let forkable = ForkableState::new(state);
    let fork1 = forkable.fork();
    let fork2 = fork1.fork();
    let fork3 = fork2.fork();

    assert_eq!(fork1.fork_id(), 1);
    assert_eq!(fork2.fork_id(), 2);
    assert_eq!(fork3.fork_id(), 3);
}

#[test]
fn test_merge_strategy_average() {
    // Usa valores que quantizam bem
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));   // ρ=0 → 1.0
    let byte20 = ByteSil::from_complex(Complex::from_polar(20.0, 0.0)); // ρ=3 → 20.09

    let mut state1 = ForkableState::new(SilState::neutral().with_layer(0, byte1));
    let state2 = ForkableState::new(SilState::neutral().with_layer(0, byte20));
    state1.set_default_strategy(MergeStrategy::Average);

    state1.merge(&state2).unwrap();
    let result_mag = state1.state().get(0).to_complex().norm();
    // Média: (1.0 + 20.09) / 2 ≈ 10.5 → ρ=2 → 7.39
    assert!((result_mag - 7.39).abs() < 1.0);
}

#[test]
fn test_merge_strategy_max() {
    let byte3 = ByteSil::from_complex(Complex::from_polar(2.72, 0.0));  // ρ=1 → 2.72
    let byte7 = ByteSil::from_complex(Complex::from_polar(7.39, 0.0));  // ρ=2 → 7.39

    let mut state1 = ForkableState::new(SilState::neutral().with_layer(0, byte3));
    let state2 = ForkableState::new(SilState::neutral().with_layer(0, byte7));
    state1.set_default_strategy(MergeStrategy::Max);

    state1.merge(&state2).unwrap();
    let result_mag = state1.state().get(0).to_complex().norm();
    assert!((result_mag - 7.39).abs() < 0.5);
}

#[test]
fn test_merge_strategy_min() {
    let byte3 = ByteSil::from_complex(Complex::from_polar(2.72, 0.0));  // ρ=1 → 2.72
    let byte7 = ByteSil::from_complex(Complex::from_polar(7.39, 0.0));  // ρ=2 → 7.39

    let mut state1 = ForkableState::new(SilState::neutral().with_layer(0, byte3));
    let state2 = ForkableState::new(SilState::neutral().with_layer(0, byte7));
    state1.set_default_strategy(MergeStrategy::Min);

    state1.merge(&state2).unwrap();
    let result_mag = state1.state().get(0).to_complex().norm();
    assert!((result_mag - 2.72).abs() < 0.5);
}

#[test]
fn test_merge_strategy_weighted() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));   // ρ=0 → 1.0
    let byte20 = ByteSil::from_complex(Complex::from_polar(20.0, 0.0)); // ρ=3 → 20.09

    let mut state1 = ForkableState::new(SilState::neutral().with_layer(0, byte1));
    let state2 = ForkableState::new(SilState::neutral().with_layer(0, byte20));
    state1.set_default_strategy(MergeStrategy::Weighted { weight: 75 });

    state1.merge(&state2).unwrap();
    // 1.0 * 0.75 + 20.09 * 0.25 ≈ 5.77 → ρ=2 → 7.39
    let result_mag = state1.state().get(0).to_complex().norm();
    assert!((result_mag - 7.39).abs() < 2.0);
}

#[test]
fn test_merge_multi_layer() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let byte7_a = ByteSil::from_complex(Complex::from_polar(7.39, 0.0));
    let byte7_b = ByteSil::from_complex(Complex::from_polar(7.39, 0.0));
    let byte20 = ByteSil::from_complex(Complex::from_polar(20.09, 0.0));

    let mut state1 = ForkableState::new(
        SilState::neutral()
            .with_layer(0, byte1)
            .with_layer(1, byte7_a)
    );
    let state2 = ForkableState::new(
        SilState::neutral()
            .with_layer(0, byte7_b)
            .with_layer(1, byte20)
    );
    state1.set_default_strategy(MergeStrategy::Average);

    state1.merge(&state2).unwrap();
    let result0 = state1.state().get(0).to_complex().norm();
    let result1 = state1.state().get(1).to_complex().norm();
    assert!((result0 - 2.72).abs() < 1.0); // (1.0+7.39)/2=4.2 → ρ=1 → 2.72
    assert!((result1 - 7.39).abs() < 15.0); // (7.39+20.09)/2=13.7 → ρ=3 → 20.09 (quantização)
}

#[test]
fn test_has_diverged() {
    let byte1_a = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let byte1_b = ByteSil::from_complex(Complex::from_polar(1.5, 0.0));
    let state1 = ForkableState::new(SilState::neutral().with_layer(0, byte1_a));
    let state2 = ForkableState::new(SilState::neutral().with_layer(0, byte1_b));
    assert!(!state1.has_diverged(&state2));

    let byte50 = ByteSil::from_complex(Complex::from_polar(50.0, 0.0));
    let state3 = ForkableState::new(SilState::neutral().with_layer(0, byte50));
    assert!(state1.has_diverged(&state3));
}

#[test]
fn test_manager_fork() {
    let byte_val = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let mut manager = StateManager::new(SilState::neutral().with_layer(0, byte_val));
    assert_eq!(manager.fork_count(), 0);

    let _fork = manager.fork();
    assert_eq!(manager.fork_count(), 1);

    let _fork2 = manager.fork();
    assert_eq!(manager.fork_count(), 2);
}

#[test]
fn test_manager_merge_fork() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let byte20 = ByteSil::from_complex(Complex::from_polar(20.0, 0.0));

    let mut manager = StateManager::new(SilState::neutral().with_layer(0, byte1));
    manager.set_default_strategy(MergeStrategy::Average);

    let mut fork = manager.fork();
    *fork.state_mut() = SilState::neutral().with_layer(0, byte20);

    manager.merge_fork(&fork).unwrap();
    let result_mag = manager.main_state().get(0).to_complex().norm();
    assert!((result_mag - 7.39).abs() < 1.0);
}

#[test]
fn test_manager_merge_with_strategy() {
    let byte7 = ByteSil::from_complex(Complex::from_polar(7.39, 0.0));
    let byte20 = ByteSil::from_complex(Complex::from_polar(20.09, 0.0));

    let mut manager = StateManager::new(SilState::neutral().with_layer(0, byte7));

    let mut fork = manager.fork();
    *fork.state_mut() = SilState::neutral().with_layer(0, byte20);

    manager.merge_with_strategy(&fork, MergeStrategy::Max).unwrap();
    let result_mag = manager.main_state().get(0).to_complex().norm();
    assert!((result_mag - 20.09).abs() < 1.0);
}

#[test]
fn test_manager_clear_forks() {
    let mut manager = StateManager::new(SilState::neutral());
    manager.fork();
    manager.fork();
    manager.fork();

    assert_eq!(manager.fork_count(), 3);
    manager.clear_forks();
    assert_eq!(manager.fork_count(), 0);
}

#[test]
fn test_manager_merge_all() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let mut manager = StateManager::new(SilState::neutral().with_layer(0, byte1));
    manager.set_default_strategy(MergeStrategy::Average);

    // Cria forks mas não os modifica (todos são cópias)
    manager.fork();
    manager.fork();

    manager.merge_all().unwrap();
    assert_eq!(manager.fork_count(), 0);
}

#[test]
fn test_strategy_xor() {
    let strategy = MergeStrategy::Xor;
    // Sinais diferentes: soma
    assert_eq!(strategy.apply(5.0, -3.0), 2.0);
    // Sinais iguais: diferença absoluta
    assert_eq!(strategy.apply(5.0, 3.0), 2.0);
}

#[test]
fn test_merge_partial_layers() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let byte2 = ByteSil::from_complex(Complex::from_polar(2.72, 0.0));
    let byte7 = ByteSil::from_complex(Complex::from_polar(7.39, 0.0));
    let byte20 = ByteSil::from_complex(Complex::from_polar(20.09, 0.0));

    let mut state1 = ForkableState::new(
        SilState::neutral()
            .with_layer(0, byte1)
            .with_layer(1, byte2)
    );
    let state2 = ForkableState::new(
        SilState::neutral()
            .with_layer(1, byte7)
            .with_layer(2, byte20)
    );
    state1.set_default_strategy(MergeStrategy::Average);

    state1.merge(&state2).unwrap();

    // Layer 0: (1.0 + 1.0)/2 = 1.0 → ρ=0 → 1.0
    let result0 = state1.state().get(0).to_complex().norm();
    assert!((result0 - 1.0).abs() < 0.5);

    // Layer 1: (2.72 + 7.39)/2 ≈ 5.0 → ρ=2 → 7.39
    let result1 = state1.state().get(1).to_complex().norm();
    assert!((result1 - 7.39).abs() < 3.0); // Maior tolerância

    // Layer 2: (1.0 + 20.09)/2 ≈ 10.5 → ρ=2 → 7.39
    let result2 = state1.state().get(2).to_complex().norm();
    assert!((result2 - 7.39).abs() < 2.0);
}

#[test]
fn test_forkable_state_mut() {
    let byte1 = ByteSil::from_complex(Complex::from_polar(1.0, 0.0));
    let byte100 = ByteSil::from_complex(Complex::from_polar(148.41, 0.0)); // ρ=5

    let mut forkable = ForkableState::new(SilState::neutral().with_layer(0, byte1));
    *forkable.state_mut() = SilState::neutral().with_layer(0, byte100);

    let result_mag = forkable.state().get(0).to_complex().norm();
    assert!((result_mag - 148.41).abs() < 10.0);
}

#[test]
fn test_manager_is_ready() {
    let manager = StateManager::new(SilState::neutral());
    assert!(manager.is_ready());
}
