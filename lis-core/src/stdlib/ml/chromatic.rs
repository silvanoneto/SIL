//! # Chromatic Computing
//!
//! Edge/fog/cloud routing based on ρ_Sil complexity metric.
//!
//! ## Whitepaper Reference
//! - §C.33: ρ_Sil metric
//! - __edge/10: Edge AI routing decisions
//!
//! ## Chromatic Model
//!
//! Maps computational complexity to a "color" spectrum:
//! - Red (0°): Ultra-low complexity → Local MCU
//! - Orange (30°): Low complexity → Edge device
//! - Yellow (60°): Medium complexity → Fog node
//! - Green (120°): High complexity → Cloud
//! - Blue (240°): Very high complexity → HPC cluster

use sil_core::state::{SilState, NUM_LAYERS};
use std::f64::consts::PI;
use crate::stdlib::ml::utils::magnitude;

/// Chromatic zones for routing decisions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChromaticZone {
    /// Local MCU processing (ρ < 0.1)
    UltraLocal,
    /// Edge device (0.1 ≤ ρ < 0.3)
    Edge,
    /// Fog node (0.3 ≤ ρ < 0.5)
    Fog,
    /// Cloud datacenter (0.5 ≤ ρ < 0.8)
    Cloud,
    /// HPC cluster (ρ ≥ 0.8)
    HPC,
}

/// Routing decision with confidence
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub zone: ChromaticZone,
    pub hue: f64,           // 0-2π chromatic angle
    pub confidence: f64,    // 0-1 decision confidence
    pub rho_sil: f64,       // Original ρ_Sil value
    pub latency_estimate: f64, // Estimated latency in ms
}

/// Map ρ_Sil to chromatic hue (0-2π)
///
/// The chromatic spectrum represents computational intensity:
/// - 0° (Red): Minimal computation
/// - 120° (Green): Moderate computation
/// - 240° (Blue): Heavy computation
///
/// # Arguments
/// * `rho` - ρ_Sil complexity metric (0-1)
///
/// # Returns
/// Chromatic hue in radians (0-2π)
pub fn chromatic_hue(rho: f64) -> f64 {
    let rho = rho.clamp(0.0, 1.0);
    // Map [0, 1] to [0, 4π/3] (red to blue)
    rho * 4.0 * PI / 3.0
}

/// Convert chromatic hue to RGB for visualization
///
/// # Arguments
/// * `hue` - Hue in radians (0-2π)
///
/// # Returns
/// (R, G, B) each in 0-1 range
pub fn hue_to_rgb(hue: f64) -> (f64, f64, f64) {
    let h = (hue * 3.0 / PI) % 6.0; // Convert to 0-6 range
    let x = 1.0 - (h % 2.0 - 1.0).abs();

    match h as usize {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    }
}

/// Route computation based on ρ_Sil value
///
/// # Arguments
/// * `rho` - ρ_Sil complexity metric
///
/// # Returns
/// ChromaticZone indicating where to process
pub fn chromatic_route(rho: f64) -> ChromaticZone {
    if rho < 0.1 {
        ChromaticZone::UltraLocal
    } else if rho < 0.3 {
        ChromaticZone::Edge
    } else if rho < 0.5 {
        ChromaticZone::Fog
    } else if rho < 0.8 {
        ChromaticZone::Cloud
    } else {
        ChromaticZone::HPC
    }
}

/// Full routing decision with confidence and estimates
///
/// # Arguments
/// * `rho` - ρ_Sil complexity metric
/// * `bandwidth_mbps` - Available network bandwidth
/// * `local_tflops` - Local device TFLOPS
///
/// # Returns
/// RoutingDecision with zone, confidence, and estimates
pub fn route_decision(rho: f64, bandwidth_mbps: f64, local_tflops: f64) -> RoutingDecision {
    let zone = chromatic_route(rho);
    let hue = chromatic_hue(rho);

    // Confidence based on how far from zone boundaries
    let confidence = match zone {
        ChromaticZone::UltraLocal => 1.0 - (rho / 0.1),
        ChromaticZone::Edge => 1.0 - ((rho - 0.2).abs() / 0.1),
        ChromaticZone::Fog => 1.0 - ((rho - 0.4).abs() / 0.1),
        ChromaticZone::Cloud => 1.0 - ((rho - 0.65).abs() / 0.15),
        ChromaticZone::HPC => (rho - 0.8) / 0.2,
    }
    .clamp(0.0, 1.0);

    // Latency estimate (simplified model)
    let latency_estimate = match zone {
        ChromaticZone::UltraLocal => rho * 10.0 / local_tflops.max(0.001), // Local compute
        ChromaticZone::Edge => 5.0 + rho * 50.0,                           // ~5-55ms
        ChromaticZone::Fog => 20.0 + rho * 100.0,                          // ~20-120ms
        ChromaticZone::Cloud => 50.0 + (16.0 * 4.0) / bandwidth_mbps * 1000.0, // Network + compute
        ChromaticZone::HPC => 100.0 + (16.0 * 4.0) / bandwidth_mbps * 1000.0,
    };

    RoutingDecision {
        zone,
        hue,
        confidence,
        rho_sil: rho,
        latency_estimate,
    }
}

/// Calculate optimal split point for split computing
///
/// In split computing, part of the model runs locally, part remotely.
/// This finds the optimal layer to split based on bandwidth and compute.
///
/// # Arguments
/// * `layer_complexities` - ρ_Sil for each of 16 layers
/// * `bandwidth_mbps` - Network bandwidth
/// * `local_tflops` - Local compute capacity
///
/// # Returns
/// Optimal split layer (0 = all remote, 16 = all local)
pub fn split_point(layer_complexities: &SilState, bandwidth_mbps: f64, local_tflops: f64) -> usize {
    let mut best_split = 0;
    let mut best_cost = f64::INFINITY;

    for split in 0..=NUM_LAYERS {
        // Local compute cost
        let mut local_cost = 0.0;
        for i in 0..split {
            local_cost += magnitude(&layer_complexities.get(i)) / local_tflops.max(0.001);
        }

        // Network transfer cost (intermediate activations)
        let transfer_cost = if split < NUM_LAYERS {
            16.0 * 4.0 / bandwidth_mbps * 1000.0 // 16 floats * 4 bytes
        } else {
            0.0
        };

        // Remote compute cost (assumed 10x faster)
        let mut remote_cost = 0.0;
        for i in split..NUM_LAYERS {
            remote_cost += magnitude(&layer_complexities.get(i)) / (local_tflops * 10.0).max(0.001);
        }

        let total_cost = local_cost + transfer_cost + remote_cost;

        if total_cost < best_cost {
            best_cost = total_cost;
            best_split = split;
        }
    }

    best_split
}

/// Adaptive batch size based on complexity
///
/// Higher complexity → smaller batches to fit in memory
///
/// # Arguments
/// * `rho` - ρ_Sil complexity
/// * `available_memory_mb` - Available memory
///
/// # Returns
/// Recommended batch size
pub fn adaptive_batch_size(rho: f64, available_memory_mb: f64) -> usize {
    // Estimate memory per sample: ~16 * sizeof(f64) * (1 + rho * 10) layers of activations
    let bytes_per_sample = 16.0 * 8.0 * (1.0 + rho * 10.0);
    let max_batch = (available_memory_mb * 1024.0 * 1024.0 / bytes_per_sample) as usize;
    max_batch.max(1).min(1024) // Clamp to [1, 1024]
}

/// Priority queue weight for task scheduling
///
/// Maps zone to scheduling priority (higher = more urgent)
///
/// # Arguments
/// * `zone` - Chromatic routing zone
/// * `deadline_ms` - Task deadline in milliseconds
///
/// # Returns
/// Priority weight (higher = schedule sooner)
pub fn task_priority(zone: ChromaticZone, deadline_ms: f64) -> f64 {
    let zone_weight = match zone {
        ChromaticZone::UltraLocal => 1.0,
        ChromaticZone::Edge => 1.5,
        ChromaticZone::Fog => 2.0,
        ChromaticZone::Cloud => 3.0,
        ChromaticZone::HPC => 5.0,
    };

    // Urgency increases as deadline approaches
    let urgency = 1000.0 / deadline_ms.max(1.0);

    zone_weight * urgency
}

/// Calculate load balancing weights for multiple nodes
///
/// # Arguments
/// * `node_capacities` - TFLOPS capacity of each node
/// * `current_loads` - Current load (0-1) of each node
///
/// # Returns
/// Weight for each node (sum to 1)
pub fn load_balance_weights(node_capacities: &[f64], current_loads: &[f64]) -> Vec<f64> {
    if node_capacities.is_empty() {
        return vec![];
    }

    // Available capacity = total capacity * (1 - current load)
    let available: Vec<f64> = node_capacities
        .iter()
        .zip(current_loads.iter())
        .map(|(cap, load)| cap * (1.0 - load.clamp(0.0, 1.0)))
        .collect();

    let total: f64 = available.iter().sum();

    if total < 1e-10 {
        // All nodes overloaded, distribute equally
        vec![1.0 / node_capacities.len() as f64; node_capacities.len()]
    } else {
        available.iter().map(|a| a / total).collect()
    }
}

/// Estimate energy cost for routing decision
///
/// # Arguments
/// * `zone` - Target chromatic zone
/// * `rho` - ρ_Sil complexity
/// * `battery_fraction` - Remaining battery (0-1)
///
/// # Returns
/// Estimated energy cost in mJ
pub fn energy_estimate(zone: ChromaticZone, rho: f64, battery_fraction: f64) -> f64 {
    let base_cost = match zone {
        ChromaticZone::UltraLocal => 0.1, // 0.1 mJ base
        ChromaticZone::Edge => 1.0,       // 1 mJ (radio on)
        ChromaticZone::Fog => 5.0,        // 5 mJ (longer tx)
        ChromaticZone::Cloud => 10.0,     // 10 mJ (full tx)
        ChromaticZone::HPC => 20.0,       // 20 mJ
    };

    let compute_cost = rho * 10.0; // Compute scales with complexity

    // Low battery penalty (prefer local to save power)
    let battery_factor = if battery_fraction < 0.2 {
        match zone {
            ChromaticZone::UltraLocal => 1.0,
            _ => 2.0, // Penalize network use when low battery
        }
    } else {
        1.0
    };

    (base_cost + compute_cost) * battery_factor
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_chromatic_hue() {
        assert!((chromatic_hue(0.0) - 0.0).abs() < 1e-10);
        assert!(chromatic_hue(0.5) > 0.0);
        assert!(chromatic_hue(1.0) <= 4.0 * PI / 3.0 + 0.01);
    }

    #[test]
    fn test_chromatic_route() {
        assert_eq!(chromatic_route(0.05), ChromaticZone::UltraLocal);
        assert_eq!(chromatic_route(0.2), ChromaticZone::Edge);
        assert_eq!(chromatic_route(0.4), ChromaticZone::Fog);
        assert_eq!(chromatic_route(0.6), ChromaticZone::Cloud);
        assert_eq!(chromatic_route(0.9), ChromaticZone::HPC);
    }

    #[test]
    fn test_route_decision() {
        let decision = route_decision(0.3, 100.0, 1.0);
        assert_eq!(decision.zone, ChromaticZone::Fog);
        assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
        assert!(decision.latency_estimate > 0.0);
    }

    #[test]
    fn test_split_point() {
        let mut state = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            // Increasing complexity
            state = state.with_layer(i, from_mag_phase(i as f64 / 16.0, 0.0));
        }

        let split = split_point(&state, 100.0, 1.0);
        assert!(split <= NUM_LAYERS);
    }

    #[test]
    fn test_load_balance() {
        let capacities = vec![10.0, 20.0, 10.0];
        let loads = vec![0.0, 0.0, 0.0];

        let weights = load_balance_weights(&capacities, &loads);

        assert_eq!(weights.len(), 3);
        assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-10);
        assert!(weights[1] > weights[0]); // Higher capacity = higher weight
    }

    #[test]
    fn test_energy_estimate() {
        let local_energy = energy_estimate(ChromaticZone::UltraLocal, 0.1, 1.0);
        let cloud_energy = energy_estimate(ChromaticZone::Cloud, 0.1, 1.0);

        assert!(local_energy < cloud_energy); // Local should be cheaper
    }
}
