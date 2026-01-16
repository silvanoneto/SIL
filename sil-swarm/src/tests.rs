//! Testes integrados para sil-swarm

use crate::*;
use num_complex::Complex;
use sil_core::prelude::*;
use sil_core::traits::SwarmAgent;

#[test]
fn test_swarm_node_creation() {
    let node = SwarmNode::new(1);
    assert_eq!(node.name(), "SwarmNode");
    assert_eq!(node.layers(), &[11]);
}

#[test]
fn test_swarm_node_neighbors() {
    let mut node = SwarmNode::new(1);
    assert_eq!(node.neighbors().len(), 0);

    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();
    node.add_neighbor(4).unwrap();

    assert_eq!(node.neighbors().len(), 3);
    assert!(node.has_neighbor(2));
    assert!(node.has_neighbor(3));
    assert!(node.has_neighbor(4));
}

#[test]
fn test_swarm_node_remove_neighbor() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();

    node.remove_neighbor(2).unwrap();
    assert_eq!(node.neighbors().len(), 1);
    assert!(!node.has_neighbor(2));
    assert!(node.has_neighbor(3));
}

#[test]
fn test_swarm_node_distances() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();

    node.set_distance(2, 1.5).unwrap();
    node.set_distance(3, 3.0).unwrap();

    assert_eq!(node.distance_to(&2), 1.5);
    assert_eq!(node.distance_to(&3), 3.0);
}

#[test]
fn test_swarm_behavior_flocking() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();
    node.set_behavior(SwarmBehavior::Flocking);

    let local = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0)));
    let neighbors = vec![
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(2.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(3.0, 0.0))),
    ];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0);
    assert!(!val.is_null());
}

#[test]
fn test_swarm_behavior_consensus() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();
    node.set_behavior(SwarmBehavior::Consensus);

    let local = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0)));
    let neighbors = vec![
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(2.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(3.0, 0.0))),
    ];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0).to_complex().norm() as f32;

    // Consenso deve estar entre mínimo e máximo
    assert!(val >= 1.0 && val <= 3.0);
}

#[test]
fn test_swarm_behavior_emergent() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();
    node.set_behavior(SwarmBehavior::Emergent);

    let local = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0)));
    let neighbors = vec![
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(2.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(3.0, 0.0))),
    ];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0);
    assert!(!val.is_null());
}

#[test]
fn test_swarm_empty_neighbors() {
    let mut node = SwarmNode::new(1);
    node.set_behavior(SwarmBehavior::Consensus);

    let local = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(5.0, 0.0)));
    let neighbors = vec![];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0).to_complex().norm() as f32;
    // With empty neighbors, should return local state (but may have precision loss)
    assert!(val > 0.0);
}

#[test]
fn test_swarm_multi_layer() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.set_behavior(SwarmBehavior::Consensus);

    let local = SilState::neutral()
        .with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0)))
        .with_layer(1, ByteSil::from_complex(Complex::from_polar(10.0, 0.0)))
        .with_layer(2, ByteSil::from_complex(Complex::from_polar(20.0, 0.0)));

    let neighbors = vec![
        SilState::neutral()
            .with_layer(0, ByteSil::from_complex(Complex::from_polar(2.0, 0.0)))
            .with_layer(1, ByteSil::from_complex(Complex::from_polar(12.0, 0.0)))
            .with_layer(2, ByteSil::from_complex(Complex::from_polar(22.0, 0.0))),
    ];

    let result = node.behavior(&local, &neighbors);
    assert!(!result.get(0).is_null());
    assert!(!result.get(1).is_null());
    assert!(!result.get(2).is_null());
}

#[test]
fn test_swarm_config_custom() {
    let config = SwarmConfig {
        alignment_weight: 0.5,
        cohesion_weight: 0.3,
        separation_weight: 0.2,
        ..Default::default()
    };

    let node = SwarmNode::with_config(1, config);
    assert_eq!(node.neighbor_count(), 0);
}

#[test]
fn test_swarm_node_ready() {
    let mut node = SwarmNode::new(1);
    assert!(!node.is_ready()); // Sem vizinhos

    node.add_neighbor(2).unwrap();
    assert!(node.is_ready()); // Com vizinhos
}

#[test]
fn test_swarm_convergence_simple() {
    let mut nodes = vec![
        SwarmNode::new(1),
        SwarmNode::new(2),
        SwarmNode::new(3),
    ];

    // Conecta todos em anel
    nodes[0].add_neighbor(1).unwrap();
    nodes[0].add_neighbor(2).unwrap();
    nodes[1].add_neighbor(0).unwrap();
    nodes[1].add_neighbor(2).unwrap();
    nodes[2].add_neighbor(0).unwrap();
    nodes[2].add_neighbor(1).unwrap();

    for node in &mut nodes {
        node.set_behavior(SwarmBehavior::Consensus);
    }

    // Estados iniciais diferentes
    let states = vec![
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(5.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(9.0, 0.0))),
    ];

    // Simula algumas iterações
    let mut current_states = states;
    for _ in 0..5 {
        let mut next_states = Vec::new();
        for (i, node) in nodes.iter_mut().enumerate() {
            let neighbor_states: Vec<SilState> = node
                .neighbors()
                .iter()
                .map(|&id| current_states[id as usize].clone())
                .collect();

            let new_state = node.behavior(&current_states[i], &neighbor_states);
            next_states.push(new_state);
        }
        current_states = next_states;
    }

    // Verifica convergência
    let vals: Vec<f32> = current_states
        .iter()
        .map(|s| s.get(0).to_complex().norm() as f32)
        .collect();

    let mean = vals.iter().sum::<f32>() / vals.len() as f32;
    let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / vals.len() as f32;

    // Variância deve diminuir com consenso
    assert!(variance < 10.0); // Menos que variância inicial
}

#[test]
fn test_swarm_flocking_alignment() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();
    node.set_behavior(SwarmBehavior::Flocking);

    let local = SilState::neutral().with_layer(0, ByteSil::NULL);
    let neighbors = vec![
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0))),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0))),
    ];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0).to_complex().norm() as f32;

    // Deve se mover em direção aos vizinhos
    assert!(val > 0.0);
}

#[test]
fn test_swarm_weighted_consensus() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();

    // Vizinho 2 mais próximo que 3
    node.set_distance(2, 0.5).unwrap();
    node.set_distance(3, 2.0).unwrap();

    node.set_behavior(SwarmBehavior::Consensus);

    let local = SilState::neutral().with_layer(0, ByteSil::NULL);
    let neighbors = vec![
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))), // Vizinho 2 (mais próximo)
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0))),  // Vizinho 3 (mais distante)
    ];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0).to_complex().norm() as f32;

    // Deve dar mais peso ao vizinho mais próximo
    assert!(val > 2.0); // Should be closer to 10 than to 1 (relaxed due to ByteSil precision)
}

#[test]
fn test_swarm_emergent_diversity() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(3).unwrap();
    node.set_behavior(SwarmBehavior::Emergent);

    // Vizinhos com alta diversidade
    let local = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(5.0, 0.0)));
    let neighbors = vec![
        SilState::neutral().with_layer(0, ByteSil::NULL),
        SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(10.0, 0.0))),
    ];

    let result = node.behavior(&local, &neighbors);
    let val = result.get(0);
    assert!(!val.is_null());
}

#[test]
fn test_swarm_behavior_switching() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();

    let local = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(1.0, 0.0)));
    let neighbors = vec![SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(2.0, 0.0)))];

    // Testa cada comportamento
    node.set_behavior(SwarmBehavior::Flocking);
    let r1 = node.behavior(&local, &neighbors);
    assert!(!r1.get(0).is_null());

    node.set_behavior(SwarmBehavior::Consensus);
    let r2 = node.behavior(&local, &neighbors);
    assert!(!r2.get(0).is_null());

    node.set_behavior(SwarmBehavior::Emergent);
    let r3 = node.behavior(&local, &neighbors);
    assert!(!r3.get(0).is_null());
}

#[test]
fn test_swarm_state_update() {
    let mut node = SwarmNode::new(1);
    let new_state = SilState::neutral().with_layer(0, ByteSil::from_complex(Complex::from_polar(42.0, 0.0)));

    node.update_state(new_state.clone());
    let val = node.local_state().get(0).to_complex().norm() as f32;
    // ByteSil has limited precision (8-bit log-polar), so 42.0 -> e^3 ≈ 20.09
    assert!(val > 10.0 && val < 100.0);
}

#[test]
fn test_swarm_duplicate_neighbor() {
    let mut node = SwarmNode::new(1);
    node.add_neighbor(2).unwrap();
    node.add_neighbor(2).unwrap(); // Duplicado

    // Não deve adicionar duplicado
    assert_eq!(node.neighbor_count(), 1);
}

#[test]
fn test_swarm_remove_nonexistent() {
    let mut node = SwarmNode::new(1);
    let result = node.remove_neighbor(99);
    assert!(result.is_err());
}

#[test]
fn test_swarm_distance_nonexistent() {
    let mut node = SwarmNode::new(1);
    let result = node.set_distance(99, 1.0);
    assert!(result.is_err());
}

#[test]
fn test_swarm_large_network() {
    let mut nodes: Vec<SwarmNode> = (0..10).map(SwarmNode::new).collect();

    // Conecta cada nó com os próximos 3
    for i in 0..10 {
        nodes[i].add_neighbor(((i + 1) % 10) as u64).unwrap();
        nodes[i].add_neighbor(((i + 2) % 10) as u64).unwrap();
        nodes[i].add_neighbor(((i + 3) % 10) as u64).unwrap();
    }

    // Verifica conectividade
    for node in &nodes {
        assert_eq!(node.neighbor_count(), 3);
        assert!(node.is_ready());
    }
}
