//! # Dynamic Offloading
//!
//! Runtime decisions for edge/cloud computation offloading.
//!
//! ## Whitepaper Reference
//! - §C.33: ρ_Sil metric for offload decisions
//! - __edge/10: Dynamic offloading strategies
//!
//! ## Decision Factors
//!
//! 1. **Computational complexity** (ρ_Sil)
//! 2. **Network conditions** (bandwidth, latency)
//! 3. **Device state** (battery, temperature, load)
//! 4. **Task constraints** (deadline, privacy)

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};

/// Offload target options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffloadTarget {
    /// Process locally on device
    Local,
    /// Offload to nearby edge node
    Edge,
    /// Offload to fog/gateway
    Fog,
    /// Offload to cloud
    Cloud,
    /// Split computation (partial offload)
    Split { local_layers: usize },
    /// Compress and defer
    Defer,
}

/// Device state for offload decisions
#[derive(Debug, Clone)]
pub struct DeviceState {
    pub cpu_load: f64,          // 0-1
    pub memory_load: f64,       // 0-1
    pub battery_fraction: f64,  // 0-1
    pub temperature_celsius: f64,
    pub tflops: f64,            // Local compute capacity
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            cpu_load: 0.5,
            memory_load: 0.5,
            battery_fraction: 1.0,
            temperature_celsius: 40.0,
            tflops: 0.1,
        }
    }
}

/// Network state for offload decisions
#[derive(Debug, Clone)]
pub struct NetworkState {
    pub bandwidth_mbps: f64,
    pub latency_ms: f64,
    pub packet_loss: f64,       // 0-1
    pub edge_available: bool,
    pub fog_available: bool,
    pub cloud_available: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            bandwidth_mbps: 10.0,
            latency_ms: 50.0,
            packet_loss: 0.01,
            edge_available: true,
            fog_available: true,
            cloud_available: true,
        }
    }
}

/// Task constraints
#[derive(Debug, Clone)]
pub struct TaskConstraints {
    pub deadline_ms: f64,       // Max acceptable latency
    pub privacy_sensitive: bool, // Can't offload
    pub real_time: bool,        // Prefer local
    pub energy_budget_mj: f64,  // Energy constraint
}

impl Default for TaskConstraints {
    fn default() -> Self {
        Self {
            deadline_ms: 1000.0,
            privacy_sensitive: false,
            real_time: false,
            energy_budget_mj: 100.0,
        }
    }
}

/// Offload decision result
#[derive(Debug, Clone)]
pub struct OffloadDecision {
    pub target: OffloadTarget,
    pub confidence: f64,
    pub estimated_latency_ms: f64,
    pub estimated_energy_mj: f64,
    pub reason: &'static str,
}

/// Make offload decision based on all factors
///
/// # Arguments
/// * `rho_sil` - Computational complexity (0-1)
/// * `device` - Current device state
/// * `network` - Current network state
/// * `constraints` - Task constraints
///
/// # Returns
/// OffloadDecision with target and estimates
pub fn offload_decision(
    rho_sil: f64,
    device: &DeviceState,
    network: &NetworkState,
    constraints: &TaskConstraints,
) -> OffloadDecision {
    // Privacy check - must stay local
    if constraints.privacy_sensitive {
        return OffloadDecision {
            target: OffloadTarget::Local,
            confidence: 1.0,
            estimated_latency_ms: estimate_local_latency(rho_sil, device),
            estimated_energy_mj: estimate_local_energy(rho_sil, device),
            reason: "Privacy constraint - local only",
        };
    }

    // Calculate critical threshold (from rho_sil.rs formula)
    let rho_critical = calculate_rho_critical(device);

    // Estimate latencies for each option
    let local_latency = estimate_local_latency(rho_sil, device);
    let edge_latency = estimate_edge_latency(rho_sil, network);
    let fog_latency = estimate_fog_latency(rho_sil, network);
    let cloud_latency = estimate_cloud_latency(rho_sil, network);

    // Energy estimates
    let local_energy = estimate_local_energy(rho_sil, device);
    let offload_energy = estimate_offload_energy(rho_sil, network);

    // Real-time prefers local if feasible
    if constraints.real_time && local_latency < constraints.deadline_ms {
        return OffloadDecision {
            target: OffloadTarget::Local,
            confidence: 0.9,
            estimated_latency_ms: local_latency,
            estimated_energy_mj: local_energy,
            reason: "Real-time constraint - local feasible",
        };
    }

    // Low battery - prefer offload if deadline allows
    if device.battery_fraction < 0.2 && network.edge_available {
        if edge_latency < constraints.deadline_ms {
            return OffloadDecision {
                target: OffloadTarget::Edge,
                confidence: 0.85,
                estimated_latency_ms: edge_latency,
                estimated_energy_mj: offload_energy,
                reason: "Low battery - offload to edge",
            };
        }
    }

    // High temperature - throttle by offloading
    if device.temperature_celsius > 70.0 {
        if network.fog_available && fog_latency < constraints.deadline_ms {
            return OffloadDecision {
                target: OffloadTarget::Fog,
                confidence: 0.8,
                estimated_latency_ms: fog_latency,
                estimated_energy_mj: offload_energy,
                reason: "Thermal throttle - offload to fog",
            };
        }
    }

    // ρ_Sil decision
    if rho_sil <= rho_critical {
        // Process locally
        OffloadDecision {
            target: OffloadTarget::Local,
            confidence: 1.0 - rho_sil / rho_critical,
            estimated_latency_ms: local_latency,
            estimated_energy_mj: local_energy,
            reason: "ρ_Sil below threshold - local",
        }
    } else if rho_sil < 0.5 && network.edge_available {
        // Edge offload
        OffloadDecision {
            target: OffloadTarget::Edge,
            confidence: 0.8,
            estimated_latency_ms: edge_latency,
            estimated_energy_mj: offload_energy,
            reason: "ρ_Sil moderate - edge offload",
        }
    } else if rho_sil < 0.7 && network.fog_available {
        // Fog offload
        OffloadDecision {
            target: OffloadTarget::Fog,
            confidence: 0.75,
            estimated_latency_ms: fog_latency,
            estimated_energy_mj: offload_energy,
            reason: "ρ_Sil high - fog offload",
        }
    } else if network.cloud_available && cloud_latency < constraints.deadline_ms {
        // Cloud offload
        OffloadDecision {
            target: OffloadTarget::Cloud,
            confidence: 0.7,
            estimated_latency_ms: cloud_latency,
            estimated_energy_mj: offload_energy,
            reason: "ρ_Sil very high - cloud offload",
        }
    } else {
        // Split computing as fallback
        let split_layers = calculate_split_point(rho_sil, device, network);
        OffloadDecision {
            target: OffloadTarget::Split { local_layers: split_layers },
            confidence: 0.6,
            estimated_latency_ms: (local_latency + edge_latency) / 2.0,
            estimated_energy_mj: (local_energy + offload_energy) / 2.0,
            reason: "Split computing - partial offload",
        }
    }
}

/// Calculate ρ_critical threshold based on device state
fn calculate_rho_critical(device: &DeviceState) -> f64 {
    let rho_base = 0.3; // Base threshold

    // Factors that lower threshold (more willing to offload)
    let phi_cpu = 1.0 - device.cpu_load * 0.5;
    let phi_mem = 1.0 - device.memory_load * 0.3;
    let phi_bat = device.battery_fraction.clamp(0.1, 1.0);
    let phi_temp = if device.temperature_celsius > 60.0 {
        0.8 - (device.temperature_celsius - 60.0) * 0.02
    } else {
        1.0
    };

    (rho_base * phi_cpu * phi_mem * phi_bat * phi_temp).clamp(0.1, 0.8)
}

/// Estimate local processing latency
fn estimate_local_latency(rho_sil: f64, device: &DeviceState) -> f64 {
    // Base latency proportional to complexity
    let base = rho_sil * 100.0; // 0-100ms base

    // Adjust for device capacity and load
    let capacity_factor = 1.0 / device.tflops.max(0.01);
    let load_factor = 1.0 + device.cpu_load;

    base * capacity_factor * load_factor
}

/// Estimate edge processing latency
fn estimate_edge_latency(rho_sil: f64, network: &NetworkState) -> f64 {
    if !network.edge_available {
        return f64::INFINITY;
    }

    // Network RTT + transfer time + remote compute
    let rtt = network.latency_ms * 2.0;
    let transfer = 16.0 * 4.0 * 8.0 / network.bandwidth_mbps; // State size in bits / bandwidth
    let compute = rho_sil * 10.0; // Edge is ~10x faster

    rtt + transfer + compute
}

/// Estimate fog processing latency
fn estimate_fog_latency(rho_sil: f64, network: &NetworkState) -> f64 {
    if !network.fog_available {
        return f64::INFINITY;
    }

    let rtt = network.latency_ms * 3.0; // Fog is farther
    let transfer = 16.0 * 4.0 * 8.0 / network.bandwidth_mbps;
    let compute = rho_sil * 5.0; // Fog is ~20x faster

    rtt + transfer + compute
}

/// Estimate cloud processing latency
fn estimate_cloud_latency(rho_sil: f64, network: &NetworkState) -> f64 {
    if !network.cloud_available {
        return f64::INFINITY;
    }

    let rtt = network.latency_ms * 5.0; // Cloud is distant
    let transfer = 16.0 * 4.0 * 8.0 / network.bandwidth_mbps;
    let compute = rho_sil * 2.0; // Cloud is ~50x faster

    rtt + transfer + compute
}

/// Estimate local energy consumption
fn estimate_local_energy(rho_sil: f64, device: &DeviceState) -> f64 {
    // Base energy proportional to complexity
    let base = rho_sil * 10.0; // 0-10 mJ base

    // Efficiency factor based on device
    let efficiency = device.tflops * 10.0; // More TFLOPS = more efficient per op

    base / efficiency.max(0.1)
}

/// Estimate offload energy (radio + wait)
fn estimate_offload_energy(_rho_sil: f64, network: &NetworkState) -> f64 {
    // Radio energy for transmission
    let tx_energy = 16.0 * 4.0 / network.bandwidth_mbps * 0.5; // ~0.5 mJ/MB

    // Wait energy (idle listening)
    let wait_energy = network.latency_ms * 0.001; // ~1 µJ/ms idle

    tx_energy + wait_energy
}

/// Calculate optimal split point for split computing
fn calculate_split_point(rho_sil: f64, device: &DeviceState, network: &NetworkState) -> usize {
    // Simple heuristic: process rho_critical fraction locally
    let rho_critical = calculate_rho_critical(device);
    let local_fraction = (rho_critical / rho_sil).clamp(0.0, 1.0);

    // Consider network quality
    let network_factor = if network.latency_ms > 100.0 {
        1.2 // Poor network = do more locally
    } else {
        1.0
    };

    ((local_fraction * network_factor * NUM_LAYERS as f64) as usize).min(NUM_LAYERS)
}

/// Batch offload decisions for multiple tasks
///
/// # Arguments
/// * `tasks` - Vector of (rho_sil, constraints) pairs
/// * `device` - Device state
/// * `network` - Network state
///
/// # Returns
/// Vector of offload decisions
pub fn batch_offload_decisions(
    tasks: &[(f64, TaskConstraints)],
    device: &DeviceState,
    network: &NetworkState,
) -> Vec<OffloadDecision> {
    tasks
        .iter()
        .map(|(rho, constraints)| offload_decision(*rho, device, network, constraints))
        .collect()
}

/// Check if offload is beneficial given current conditions
///
/// Quick check without full decision computation.
pub fn should_offload_quick(rho_sil: f64, device: &DeviceState) -> bool {
    let rho_critical = calculate_rho_critical(device);
    rho_sil > rho_critical
}

/// Compress state for offloading (lossy)
///
/// Reduces precision to minimize transfer size.
///
/// # Arguments
/// * `state` - State to compress
/// * `quality` - Quality factor (0-1, higher = better quality)
///
/// # Returns
/// Compressed state
pub fn compress_for_offload(state: &SilState, quality: f64) -> SilState {
    let quality = quality.clamp(0.1, 1.0);

    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let val = state.get(i);

        // Quantize magnitude
        let quant_levels = (quality * 256.0) as i32;
        let quantized_mag = (magnitude(&val) * quant_levels as f64).round() / quant_levels as f64;

        // Quantize phase (less precision needed for offload)
        let phase_levels = (quality * 64.0) as i32;
        let quantized_phase = (phase(&val) * phase_levels as f64).round() / phase_levels as f64;

        result = result.with_layer(i, from_mag_phase(quantized_mag, quantized_phase));
    }

    result
}

/// Calculate effective bandwidth considering packet loss
pub fn effective_bandwidth(bandwidth_mbps: f64, packet_loss: f64) -> f64 {
    // With packet loss, effective throughput decreases
    // Using simplified model: effective = nominal * (1 - loss)²
    let loss_factor = (1.0 - packet_loss.clamp(0.0, 1.0)).powi(2);
    bandwidth_mbps * loss_factor
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_offload_decision_local() {
        let device = DeviceState {
            cpu_load: 0.1,
            battery_fraction: 1.0,
            ..Default::default()
        };
        let network = NetworkState::default();
        let constraints = TaskConstraints::default();

        let decision = offload_decision(0.1, &device, &network, &constraints);

        assert_eq!(decision.target, OffloadTarget::Local);
    }

    #[test]
    fn test_offload_decision_privacy() {
        let device = DeviceState::default();
        let network = NetworkState::default();
        let constraints = TaskConstraints {
            privacy_sensitive: true,
            ..Default::default()
        };

        let decision = offload_decision(0.9, &device, &network, &constraints);

        assert_eq!(decision.target, OffloadTarget::Local);
        assert_eq!(decision.reason, "Privacy constraint - local only");
    }

    #[test]
    fn test_offload_decision_high_complexity() {
        let device = DeviceState::default();
        let network = NetworkState::default();
        let constraints = TaskConstraints::default();

        let decision = offload_decision(0.8, &device, &network, &constraints);

        // Should offload to fog or cloud
        assert!(matches!(
            decision.target,
            OffloadTarget::Fog | OffloadTarget::Cloud
        ));
    }

    #[test]
    fn test_should_offload_quick() {
        let device = DeviceState {
            cpu_load: 0.9,
            memory_load: 0.8,
            battery_fraction: 0.2,
            ..Default::default()
        };

        // High device load = lower threshold = more likely to offload
        assert!(should_offload_quick(0.3, &device));
    }

    #[test]
    fn test_compress_for_offload() {
        let state = SilState::neutral();
        let compressed = compress_for_offload(&state, 0.5);

        // Compressed should still be valid state
        for i in 0..NUM_LAYERS {
            assert!(magnitude(&compressed.get(i)).is_finite());
        }
    }

    #[test]
    fn test_effective_bandwidth() {
        assert!((effective_bandwidth(100.0, 0.0) - 100.0).abs() < 1e-10);
        assert!(effective_bandwidth(100.0, 0.1) < 100.0);
        assert!(effective_bandwidth(100.0, 0.5) < 50.0);
    }
}
