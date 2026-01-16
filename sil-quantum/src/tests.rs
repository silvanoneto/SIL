//! Testes integrados para sil-quantum

use crate::*;
use num_complex::Complex;
use sil_core::prelude::*;
use sil_core::traits::QuantumState;

#[test]
fn test_quantum_processor_creation() {
    let qp = QuantumProcessor::new();
    assert_eq!(qp.name(), "QuantumProcessor");
    assert_eq!(qp.layers(), &[12]);
    assert_eq!(qp.coherence(), 1.0);
}

#[test]
fn test_quantum_superpose_single_state() {
    let qp = QuantumProcessor::new();
    let states = vec![SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(42.0, 0.0)))];
    let weights = vec![1.0];

    let result = qp.superpose(&states, &weights);
    let val = result.get(0).to_complex().norm() as f32;
    assert!(val > 10.0);
}

#[test]
fn test_quantum_superpose_equal_weights() {
    let qp = QuantumProcessor::new();
    let states = vec![
        SilState::neutral().with_layer(0, ByteSil::NULL),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
    ];
    let weights = vec![0.5, 0.5];

    let result = qp.superpose(&states, &weights);
    let val = result.get(0).to_complex().norm() as f32;
    assert!(val > 0.0);
}

#[test]
fn test_quantum_superpose_unequal_weights() {
    let qp = QuantumProcessor::new();
    let states = vec![
        SilState::neutral().with_layer(0, ByteSil::NULL),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
    ];
    let weights = vec![0.2, 0.8];

    let result = qp.superpose(&states, &weights);
    let val = result.get(0).to_complex().norm() as f32;
    assert!(val > 5.0);
}

#[test]
fn test_quantum_superpose_multi_layer() {
    let qp = QuantumProcessor::new();
    let states = vec![
        SilState::neutral()
            .with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0)))
            .with_layer(1, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
        SilState::neutral()
            .with_layer(0, ByteSil::from_complex(Complex::from_polar(9.0, 0.0)))
            .with_layer(1, ByteSil::from_complex(Complex::from_polar(20.0, 0.0))),
    ];
    let weights = vec![0.5, 0.5];

    let result = qp.superpose(&states, &weights);
    assert!(!result.get(0).is_null());
    assert!(!result.get(1).is_null());
}

#[test]
fn test_quantum_auto_normalize() {
    let qp = QuantumProcessor::new();
    let states = vec![
        SilState::neutral().with_layer(0, ByteSil::NULL),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
    ];
    // Pesos não normalizados
    let weights = vec![1.0, 1.0];

    let result = qp.superpose(&states, &weights);
    // Deve normalizar para 0.5, 0.5
    let val = result.get(0).to_complex().norm() as f32;
    assert!(val > 0.0);
}

#[test]
fn test_quantum_coherence() {
    let qp = QuantumProcessor::new();
    assert_eq!(qp.coherence(), 1.0);
    assert!(qp.is_coherent());
}

#[test]
fn test_quantum_decoherence() {
    let mut qp = QuantumProcessor::new();
    let initial = qp.coherence();

    qp.apply_decoherence();
    assert!(qp.coherence() < initial);

    // Aplica múltiplas vezes
    for _ in 0..20 {
        qp.apply_decoherence();
    }

    assert!(qp.coherence() < 0.8);
}

#[test]
fn test_quantum_collapse_deterministic() {
    let mut qp = QuantumProcessor::with_config(QuantumConfig::default());

    let result1 = qp.collapse(12345);
    let result2 = qp.collapse(12345);

    // Mesma seed deve dar mesmo resultado
    assert_eq!(result1.get(0).rho, result2.get(0).rho);
}

#[test]
fn test_quantum_is_superposed() {
    let qp = QuantumProcessor::new();
    assert!(!qp.is_superposed());
}

#[test]
fn test_quantum_state_count() {
    let qp = QuantumProcessor::new();
    assert_eq!(qp.state_count(), 0);
}

#[test]
fn test_quantum_clear() {
    let mut qp = QuantumProcessor::new();

    qp.clear();
    assert_eq!(qp.state_count(), 0);
}

#[test]
fn test_quantum_config_custom() {
    let config = QuantumConfig {
        decoherence_rate: 0.05,
        min_coherence: 0.2,
        auto_normalize: false,
        allow_negative_weights: true,
    };

    let qp = QuantumProcessor::with_config(config);
    assert!(qp.is_coherent());
}

#[test]
fn test_quantum_superpose_three_states() {
    let qp = QuantumProcessor::new();
    let states = vec![
        SilState::neutral().with_layer(0, ByteSil::NULL),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(5.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
    ];
    let weights = vec![0.2, 0.3, 0.5];

    let result = qp.superpose(&states, &weights);
    let val = result.get(0).to_complex().norm() as f32;
    assert!(val > 3.0);
}

#[test]
fn test_quantum_is_ready() {
    let mut qp = QuantumProcessor::new();
    assert!(qp.is_ready()); // Coerência inicial é 1.0

    // Reduz coerência abaixo do mínimo
    for _ in 0..100 {
        qp.apply_decoherence();
    }
    assert!(!qp.is_ready());
}
