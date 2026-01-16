//! # ρ_Sil - Informational Complexity Metric
//!
//! Measures informational density for edge/cloud routing decisions.
//!
//! ## Formula
//!
//! ρ_Sil = α·K̂(x) + β·R_TT(x) + γ·H_ε(x)
//!
//! ## Whitepaper Reference
//! - §C.33: ρ_Sil metric
//! - __edge/10: Edge AI decisions

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::magnitude;

/// Device class based on ρ_Sil threshold
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceClass {
    Nano,      // ρ < 0.1
    Micro,     // 0.1 ≤ ρ < 0.2
    Mini,      // 0.2 ≤ ρ < 0.4
    Standard,  // 0.4 ≤ ρ < 0.6
    Edge,      // 0.6 ≤ ρ < 0.8
    Cloud,     // ρ ≥ 0.8
}

/// Calculate ρ_Sil complexity metric
///
/// Uses simplified approximation based on:
/// - Transition count (proxy for Kolmogorov complexity)
/// - Entropy
/// - Variance
pub fn rho_sil(state: &SilState) -> f64 {
    let alpha = 0.4; // Weight for transitions
    let beta = 0.3;  // Weight for entropy
    let gamma = 0.3; // Weight for variance

    // Transition count (changes between adjacent layers)
    let transitions = transition_count(state);

    // Normalized entropy
    let entropy = super::stats::entropy(state) / (NUM_LAYERS as f64).ln();

    // Normalized variance
    let variance = super::stats::variance(state);
    let max_variance = 1.0; // Assume max variance of 1.0
    let norm_variance = (variance / max_variance).min(1.0);

    // Combine
    alpha * transitions + beta * entropy + gamma * norm_variance
}

/// Fast ρ_Sil approximation (O(n) instead of computing full complexity)
pub fn rho_sil_fast(state: &SilState) -> f64 {
    transition_count(state)
}

/// Count transitions (sign changes, large jumps)
fn transition_count(state: &SilState) -> f64 {
    let mut count = 0.0;
    let threshold = 0.1;

    for i in 1..NUM_LAYERS {
        let prev = magnitude(&state.get(i - 1));
        let curr = magnitude(&state.get(i));
        let diff = (curr - prev).abs();

        if diff > threshold {
            count += 1.0;
        }
    }

    // Normalize to [0, 1]
    count / (NUM_LAYERS - 1) as f64
}

/// Calculate dynamic critical threshold
///
/// ρ_crítico = ρ_base × φ_cpu × φ_mem × φ_bat
pub fn rho_critical(rho_base: f64, cpu_load: f64, mem_load: f64, battery: f64) -> f64 {
    // Higher load = lower threshold (more willing to offload)
    let phi_cpu = 1.0 - cpu_load.clamp(0.0, 1.0) * 0.5;
    let phi_mem = 1.0 - mem_load.clamp(0.0, 1.0) * 0.3;
    let phi_bat = battery.clamp(0.1, 1.0); // Low battery = lower threshold

    rho_base * phi_cpu * phi_mem * phi_bat
}

/// Determine if should offload based on ρ_Sil
pub fn should_offload(rho: f64, rho_critical: f64) -> bool {
    rho > rho_critical
}

/// Get device class from ρ_Sil value
pub fn device_class(rho: f64) -> DeviceClass {
    if rho < 0.1 {
        DeviceClass::Nano
    } else if rho < 0.2 {
        DeviceClass::Micro
    } else if rho < 0.4 {
        DeviceClass::Mini
    } else if rho < 0.6 {
        DeviceClass::Standard
    } else if rho < 0.8 {
        DeviceClass::Edge
    } else {
        DeviceClass::Cloud
    }
}

/// β_Sil - Microscopic quantum of perception (wavelet-based)
pub fn beta_sil(state: &SilState) -> f64 {
    // Simplified: use high-frequency energy
    let mut high_freq_energy = 0.0;

    for i in 1..NUM_LAYERS {
        let prev = magnitude(&state.get(i - 1));
        let curr = magnitude(&state.get(i));
        let diff = curr - prev;
        high_freq_energy += diff * diff;
    }

    high_freq_energy.sqrt() / NUM_LAYERS as f64
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_rho_sil() {
        let state = SilState::neutral();
        let rho = rho_sil(&state);

        // Should be in [0, 1] range
        assert!(rho >= 0.0 && rho <= 1.0);
    }

    #[test]
    fn test_rho_critical() {
        let base = 0.5;

        // Low load, full battery = high threshold
        let high = rho_critical(base, 0.1, 0.1, 1.0);

        // High load, low battery = low threshold
        let low = rho_critical(base, 0.9, 0.9, 0.2);

        assert!(high > low);
    }

    #[test]
    fn test_device_class() {
        assert_eq!(device_class(0.05), DeviceClass::Nano);
        assert_eq!(device_class(0.5), DeviceClass::Standard);
        assert_eq!(device_class(0.9), DeviceClass::Cloud);
    }
}
