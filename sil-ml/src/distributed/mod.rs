//! # Distributed ML Module
//!
//! Federated learning, Byzantine detection, and multicomputer inference.

pub mod byzantine;
pub mod checkpoint;
pub mod cluster;
pub mod compression;
pub mod federated;
pub mod mesh;
pub mod privacy;

pub use byzantine::*;
pub use checkpoint::*;
pub use cluster::*;
pub use compression::*;
pub use federated::*;
pub use mesh::*;
pub use privacy::*;

use sil_core::state::{SilState, NUM_LAYERS};
use crate::core::tensor::{magnitude, from_mag_phase};
use crate::core::linalg::{add, scale_state};

/// FedAvg aggregation
pub fn fedavg_aggregate(models: &[SilState], weights: Option<&[f64]>) -> SilState {
    if models.is_empty() {
        return SilState::vacuum();
    }
    
    let n = models.len() as f64;
    let uniform_weight = 1.0 / n;
    
    let mut result = SilState::vacuum();
    
    for (idx, model) in models.iter().enumerate() {
        let w = weights.map_or(uniform_weight, |ws| ws.get(idx).copied().unwrap_or(uniform_weight));
        let weighted = scale_state(model, w);
        result = add(&result, &weighted);
    }
    
    result
}

/// FedProx aggregation with proximal term
pub fn fedprox_aggregate(
    models: &[SilState],
    global_model: &SilState,
    mu: f64,
) -> SilState {
    let avg = fedavg_aggregate(models, None);
    
    // Proximal regularization: avg + mu * (global - avg)
    let mut result = SilState::vacuum();
    
    for i in 0..NUM_LAYERS {
        let a = magnitude(&avg.get(i));
        let g = magnitude(&global_model.get(i));
        let prox = a + mu * (g - a);
        result = result.with_layer(i, from_mag_phase(prox, 0.0));
    }
    
    result
}

/// Detect Byzantine (malicious/faulty) updates using median
pub fn byzantine_robust_aggregate(models: &[SilState]) -> SilState {
    if models.is_empty() {
        return SilState::vacuum();
    }
    
    let mut result = SilState::vacuum();
    
    for i in 0..NUM_LAYERS {
        let mut values: Vec<f64> = models.iter()
            .map(|m| magnitude(&m.get(i)))
            .collect();
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        // Use median
        let median = if values.len() % 2 == 0 {
            (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
        } else {
            values[values.len() / 2]
        };
        
        result = result.with_layer(i, from_mag_phase(median, 0.0));
    }
    
    result
}

/// Trimmed mean (remove top/bottom k before averaging)
pub fn trimmed_mean_aggregate(models: &[SilState], trim_ratio: f64) -> SilState {
    if models.is_empty() {
        return SilState::vacuum();
    }
    
    let trim_count = ((models.len() as f64 * trim_ratio) as usize).min(models.len() / 2 - 1);
    
    let mut result = SilState::vacuum();
    
    for i in 0..NUM_LAYERS {
        let mut values: Vec<f64> = models.iter()
            .map(|m| magnitude(&m.get(i)))
            .collect();
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        // Trim top and bottom
        let trimmed: Vec<f64> = values[trim_count..values.len() - trim_count].to_vec();
        let mean = trimmed.iter().sum::<f64>() / trimmed.len() as f64;
        
        result = result.with_layer(i, from_mag_phase(mean, 0.0));
    }
    
    result
}

/// Add Gaussian noise for differential privacy
pub fn add_dp_noise(state: &SilState, epsilon: f64, delta: f64, sensitivity: f64) -> SilState {
    // Gaussian mechanism: σ = sensitivity * sqrt(2 * ln(1.25/delta)) / epsilon
    let sigma = sensitivity * (2.0 * (1.25 / delta).ln()).sqrt() / epsilon;
    
    let mut result = SilState::vacuum();
    let mut seed: u64 = 0x12345678_9ABCDEF0;
    
    for i in 0..NUM_LAYERS {
        // Simple Box-Muller for Gaussian
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u1 = (seed >> 33) as f64 / (1u64 << 31) as f64;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u2 = (seed >> 33) as f64 / (1u64 << 31) as f64;
        
        let z = (-2.0 * u1.max(1e-10).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let noise = z * sigma;
        
        let val = magnitude(&state.get(i)) + noise;
        result = result.with_layer(i, from_mag_phase(val, 0.0));
    }
    
    result
}

/// Gradient compression (top-k sparsification)
pub fn compress_gradients(gradients: &SilState, k: usize) -> (SilState, Vec<usize>) {
    let top_indices = crate::core::stats::topk(gradients, k);
    let mut compressed = SilState::vacuum();
    
    for &idx in &top_indices {
        compressed = compressed.with_layer(idx, gradients.get(idx));
    }
    
    (compressed, top_indices)
}

/// Decompress gradients
pub fn decompress_gradients(compressed: &SilState, indices: &[usize], shape: usize) -> SilState {
    let mut result = SilState::vacuum();
    
    for &idx in indices {
        if idx < shape && idx < NUM_LAYERS {
            result = result.with_layer(idx, compressed.get(idx));
        }
    }
    
    result
}

/// Partition strategy for distributed inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionStrategy {
    LayerParallel,    // Each node processes different layers
    DataParallel,     // Each node processes different batches
    PipelineParallel, // Pipeline stages across nodes
    Hybrid,           // Adaptive combination
}

/// Node in distributed network
#[derive(Debug, Clone)]
pub struct DistributedNode {
    pub id: u64,
    pub address: String,
    pub capacity: f64,
}

/// Distributed inference coordinator
pub struct DistributedCoordinator {
    pub nodes: Vec<DistributedNode>,
    pub strategy: PartitionStrategy,
}

impl DistributedCoordinator {
    pub fn new(strategy: PartitionStrategy) -> Self {
        Self {
            nodes: Vec::new(),
            strategy,
        }
    }
    
    pub fn add_node(&mut self, node: DistributedNode) {
        self.nodes.push(node);
    }
    
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn total_capacity(&self) -> f64 {
        self.nodes.iter().map(|n| n.capacity).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fedavg() {
        let models = vec![
            SilState::neutral(),
            SilState::neutral(),
        ];
        
        let avg = fedavg_aggregate(&models, None);
        
        // Average of identical models should be same
        for i in 0..NUM_LAYERS {
            let orig = magnitude(&models[0].get(i));
            let result = magnitude(&avg.get(i));
            assert!((orig - result).abs() < 0.1);
        }
    }

    #[test]
    fn test_byzantine_robust() {
        let models = vec![
            SilState::neutral(),
            SilState::neutral(),
            SilState::neutral(),
        ];
        
        let robust = byzantine_robust_aggregate(&models);
        
        // Should handle identical inputs
        for i in 0..NUM_LAYERS {
            assert!(magnitude(&robust.get(i)).is_finite());
        }
    }
}
