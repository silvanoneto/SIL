//! Example: Federated Learning with Paebiru
//!
//! Demonstrates end-to-end federated training with:
//! - ρ_Sil complexity-based routing
//! - Mesh gossip synchronization
//! - Byzantine-robust aggregation
//! - Delta checkpointing

use sil_core::state::SilState;

fn main() -> sil_ml::Result<()> {
    println!("🌐 Federated Learning Example - Paebiru Pattern");
    println!("================================================\n");

    // 1. Initialize distributed nodes
    println!("1️⃣ Creating mesh network with 4 nodes...");
    let mut nodes = create_mesh_nodes(4);
    println!("   ✓ Mesh ready: {}", format_node_list(&nodes));

    // 2. Initialize model
    println!("\n2️⃣ Initializing model...");
    let mut model = SilState::neutral();
    let mut checkpoint_history = CheckpointHistory::new(5);
    
    let initial_checkpoint = Checkpoint::from_state(
        "model_epoch_0".into(),
        0,
        &model,
    );
    checkpoint_history.add(initial_checkpoint.clone());
    println!("   ✓ Initial model: {} bytes", initial_checkpoint.data.len());

    // 3. Training loop
    println!("\n3️⃣ Federated training loop (5 epochs)...");
    for epoch in 0..5 {
        println!("\n   📍 Epoch {}", epoch);

        // 3a. Compute complexity
        let rho = rho_sil(&model);
        println!("      ρ_sil = {:.3} (complexity)", rho);

        // 3b. Route based on complexity
        let routing = route_decision(rho);
        println!("      Routing: {}", routing);

        // 3c. Simulate local updates on each node
        let mut local_updates = Vec::new();
        for (i, node) in nodes.iter_mut().enumerate() {
            let updated = simulate_local_training(&model, 0.01, i as u32);
            local_updates.push(updated);

            // Checkpoint local model
            let cp_id = format!("node_{}_epoch_{}", i, epoch);
            let _checkpoint = Checkpoint::from_state(cp_id, epoch, &updated);
        }

        // 3d. Broadcast via mesh
        println!("      Broadcasting via mesh...");
        for (i, node) in nodes.iter_mut().enumerate() {
            let update_data = serialize_state(&local_updates[i]);
            node.broadcast(&update_data, epoch)?;
        }

        // 3e. Aggregate updates (Byzantine-robust)
        println!("      Aggregating with Byzantine tolerance...");
        let aggregated = byzantine_robust_aggregate(&local_updates);
        model = aggregated;

        // 3f. Checkpoint aggregated model
        let cp_id = format!("model_epoch_{}", epoch + 1);
        let cp = Checkpoint::from_state(cp_id, epoch + 1, &model);
        checkpoint_history.add(cp.clone());

        // Compute delta from previous
        if checkpoint_history.len() > 1 {
            // Get previous checkpoint
            if let Some(prev) = checkpoint_history.get(&format!("model_epoch_{}", epoch)) {
                if let Ok(delta) = prev.delta_to(&cp) {
                    println!("      Delta compression: {:.1}% of model size", delta.ratio * 100.0);
                }
            }
        }
    }

    // 4. Final statistics
    println!("\n4️⃣ Training Complete!");
    println!("   ✓ Total epochs: 5");
    println!("   ✓ Checkpoints saved: {}", checkpoint_history.len());
    println!("   ✓ Final model ρ_sil: {:.3}", rho_sil(&model));

    // 5. Demonstrate rollback
    println!("\n5️⃣ Demonstrating rollback to epoch 2...");
    // Note: In production, use Arc<Mutex<CheckpointHistory>> for safe sharing
    let target_id = "model_epoch_2".to_string();
    let _ = checkpoint_history.rollback(&target_id);
    println!("   ✓ Rolled back to epoch 2");
    println!("   ✓ Checkpoint history length: {}", checkpoint_history.len());

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_mesh_nodes(count: usize) -> Vec<MeshInference> {
    let mut nodes = Vec::new();
    for i in 0..count {
        let peers: Vec<String> = (0..count)
            .filter(|j| j != &i)
            .map(|j| format!("node_{}", j))
            .collect();

        nodes.push(MeshInference::new(format!("node_{}", i), peers));
    }
    nodes
}

fn format_node_list(nodes: &[MeshInference]) -> String {
    nodes.iter()
        .map(|n| &n.node_id)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
}

fn route_decision(rho: f64) -> String {
    match rho {
        r if r < 0.1 => "🏠 LOCAL (device)",
        r if r < 0.3 => "🌍 EDGE (nearby)",
        r if r < 0.5 => "🔗 MESH (peers)",
        _ => "☁️ CLOUD (offload)",
    }
    .to_string()
}

fn simulate_local_training(model: &SilState, lr: f64, seed: u32) -> SilState {
    // Simulate SGD step: model += -lr * gradient
    let mut result = *model;
    
    for i in 0..16 {
        let layer = model.get(i);
        let mag = magnitude(&layer);
        
        // Simulate gradient: random noise proportional to magnitude
        let noise = ((seed as f64 + i as f64) * 0.001) % 0.01;
        let new_mag = (mag * (1.0 - lr * noise)).max(0.0);
        
        result = result.with_layer(i, from_mag_phase(new_mag, phase(&layer)));
    }
    
    result
}

fn byzantine_robust_aggregate(models: &[SilState]) -> SilState {
    // Simple robust aggregation: median per layer
    if models.is_empty() {
        return SilState::vacuum();
    }

    let mut result = SilState::vacuum();
    
    for layer_idx in 0..16 {
        let mut magnitudes: Vec<f64> = models.iter()
            .map(|m| magnitude(&m.get(layer_idx)))
            .collect();
        
        magnitudes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = magnitudes[magnitudes.len() / 2];
        
        let phase_val = phase(&models[0].get(layer_idx)); // Use first phase
        result = result.with_layer(layer_idx, from_mag_phase(median, phase_val));
    }
    
    result
}

fn serialize_state(state: &SilState) -> Vec<u8> {
    // Simple serialization
    let mut data = Vec::with_capacity(16);
    for i in 0..16 {
        let layer = state.get(i);
        data.push(layer.rho as u8);
        data.push(layer.theta);
    }
    data
}

// Re-export needed types
use sil_ml::distributed::{Checkpoint, CheckpointHistory, MeshInference};
use sil_ml::edge::rho_sil;
use sil_ml::core::tensor::{magnitude, phase, from_mag_phase};
