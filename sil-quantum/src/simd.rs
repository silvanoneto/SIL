//! SIMD Optimizations for Quantum Superposition
//!
//! Vectorizes superposition operations across Signal Intermediate Language (SIL) layers
//! using SIMD instructions (AVX2/NEON).
//!
//! ## Performance
//!
//! - Without SIMD: O(S × 16) with 16 sequential operations per state
//! - With SIMD: O(S × 2-4) using 4-8 wide SIMD vectors
//! - Expected speedup: 4-8× for large state counts (S > 100)
//!
//! ## Architecture Support
//!
//! - x86_64 with AVX2: 8-wide f32 vectors
//! - aarch64 with NEON: 4-wide f32 vectors
//! - Fallback: Scalar implementation

use sil_core::prelude::*;

/// SIMD-optimized superposition for x86_64 with AVX2
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn superpose_simd(states: &[SilState], weights: &[f32]) -> SilState {
    use std::arch::x86_64::*;

    assert_eq!(states.len(), weights.len());

    if states.is_empty() {
        return SilState::neutral();
    }

    let mut result_layers = [ByteSil::NULL; 16];

    unsafe {
        // Process 8 layers at a time with AVX2
        for layer_group in 0..2 {
            let layer_start = layer_group * 8;

            // Accumulate weighted magnitudes (rho)
            let mut rho_acc = _mm256_setzero_ps();
            let mut theta_acc = _mm256_setzero_ps();

            for (state, &weight) in states.iter().zip(weights.iter()) {
                // Load 8 rho values
                let mut rho_vals = [0.0f32; 8];
                let mut theta_vals = [0.0f32; 8];

                for i in 0..8 {
                    let layer_idx = layer_start + i;
                    if layer_idx < 16 {
                        let bytesil = state.get(layer_idx);
                        rho_vals[i] = bytesil.rho as f32;
                        theta_vals[i] = bytesil.theta as f32;
                    }
                }

                let rho_vec = _mm256_loadu_ps(rho_vals.as_ptr());
                let theta_vec = _mm256_loadu_ps(theta_vals.as_ptr());
                let weight_vec = _mm256_set1_ps(weight);

                // Weighted sum
                rho_acc = _mm256_fmadd_ps(rho_vec, weight_vec, rho_acc);
                theta_acc = _mm256_fmadd_ps(theta_vec, weight_vec, theta_acc);
            }

            // Store results
            let mut rho_result = [0.0f32; 8];
            let mut theta_result = [0.0f32; 8];
            _mm256_storeu_ps(rho_result.as_mut_ptr(), rho_acc);
            _mm256_storeu_ps(theta_result.as_mut_ptr(), theta_acc);

            for i in 0..8 {
                let layer_idx = layer_start + i;
                if layer_idx < 16 {
                    let rho = rho_result[i].clamp(-8.0, 7.0) as i8;
                    let theta = (theta_result[i] as u8) % 16;
                    result_layers[layer_idx] = ByteSil { rho, theta };
                }
            }
        }
    }

    SilState { layers: result_layers }
}

/// SIMD-optimized superposition for aarch64 with NEON
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub fn superpose_simd(states: &[SilState], weights: &[f32]) -> SilState {
    use std::arch::aarch64::*;

    assert_eq!(states.len(), weights.len());

    if states.is_empty() {
        return SilState::neutral();
    }

    let mut result_layers = [ByteSil::NULL; 16];

    unsafe {
        // Process 4 layers at a time with NEON
        for layer_group in 0..4 {
            let layer_start = layer_group * 4;

            // Accumulate weighted magnitudes
            let mut rho_acc = vdupq_n_f32(0.0);
            let mut theta_acc = vdupq_n_f32(0.0);

            for (state, &weight) in states.iter().zip(weights.iter()) {
                let mut rho_vals = [0.0f32; 4];
                let mut theta_vals = [0.0f32; 4];

                for i in 0..4 {
                    let layer_idx = layer_start + i;
                    if layer_idx < 16 {
                        let bytesil = state.get(layer_idx);
                        rho_vals[i] = bytesil.rho as f32;
                        theta_vals[i] = bytesil.theta as f32;
                    }
                }

                let rho_vec = vld1q_f32(rho_vals.as_ptr());
                let theta_vec = vld1q_f32(theta_vals.as_ptr());
                let weight_vec = vdupq_n_f32(weight);

                // Weighted sum
                rho_acc = vmlaq_f32(rho_acc, rho_vec, weight_vec);
                theta_acc = vmlaq_f32(theta_acc, theta_vec, weight_vec);
            }

            // Store results
            let mut rho_result = [0.0f32; 4];
            let mut theta_result = [0.0f32; 4];
            vst1q_f32(rho_result.as_mut_ptr(), rho_acc);
            vst1q_f32(theta_result.as_mut_ptr(), theta_acc);

            for i in 0..4 {
                let layer_idx = layer_start + i;
                if layer_idx < 16 {
                    let rho = rho_result[i].clamp(-8.0, 7.0) as i8;
                    let theta = (theta_result[i] as u8) % 16;
                    result_layers[layer_idx] = ByteSil { rho, theta };
                }
            }
        }
    }

    SilState { layers: result_layers }
}

/// Scalar fallback for superposition (no SIMD)
#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "avx2"),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
pub fn superpose_simd(states: &[SilState], weights: &[f32]) -> SilState {
    superpose_scalar(states, weights)
}

/// Scalar implementation (always available as fallback)
pub fn superpose_scalar(states: &[SilState], weights: &[f32]) -> SilState {
    assert_eq!(states.len(), weights.len());

    if states.is_empty() {
        return SilState::neutral();
    }

    let mut result_layers = [ByteSil::NULL; 16];

    for layer_idx in 0..16 {
        let mut weighted_rho = 0.0f32;
        let mut weighted_theta = 0.0f32;

        for (state, &weight) in states.iter().zip(weights.iter()) {
            let bytesil = state.get(layer_idx);
            weighted_rho += bytesil.rho as f32 * weight;
            weighted_theta += bytesil.theta as f32 * weight;
        }

        let rho = weighted_rho.clamp(-8.0, 7.0) as i8;
        let theta = (weighted_theta as u8) % 16;
        result_layers[layer_idx] = ByteSil { rho, theta };
    }

    SilState { layers: result_layers }
}

/// Auto-select best implementation based on architecture and state count
pub fn superpose_auto(states: &[SilState], weights: &[f32]) -> SilState {
    const SIMD_THRESHOLD: usize = 10; // Use SIMD for S >= 10 states

    if states.len() >= SIMD_THRESHOLD {
        superpose_simd(states, weights)
    } else {
        superpose_scalar(states, weights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_states(count: usize) -> Vec<SilState> {
        (0..count)
            .map(|i| {
                SilState::neutral()
                    .with_layer(0, ByteSil { rho: (i % 8) as i8, theta: (i % 16) as u8 })
                    .with_layer(5, ByteSil { rho: ((i + 1) % 8) as i8, theta: ((i + 5) % 16) as u8 })
            })
            .collect()
    }

    fn create_weights(count: usize) -> Vec<f32> {
        let sum: f32 = (1..=count).map(|i| i as f32).sum();
        (1..=count).map(|i| i as f32 / sum).collect()
    }

    #[test]
    fn test_superpose_scalar() {
        let states = create_test_states(5);
        let weights = create_weights(5);

        let result = superpose_scalar(&states, &weights);

        // Result should not be null
        assert!(!result.get(0).is_null());
    }

    #[test]
    fn test_superpose_simd() {
        let states = create_test_states(5);
        let weights = create_weights(5);

        let result = superpose_simd(&states, &weights);

        // Result should not be null
        assert!(!result.get(0).is_null());
    }

    #[test]
    fn test_simd_matches_scalar() {
        let states = create_test_states(10);
        let weights = create_weights(10);

        let result_scalar = superpose_scalar(&states, &weights);
        let result_simd = superpose_simd(&states, &weights);

        // Results should be similar (allowing for small floating point differences)
        for i in 0..16 {
            let scalar_val = result_scalar.get(i);
            let simd_val = result_simd.get(i);

            // Allow ±1 difference due to rounding
            assert!(
                (scalar_val.rho - simd_val.rho).abs() <= 1,
                "Layer {}: rho mismatch: scalar={}, simd={}",
                i, scalar_val.rho, simd_val.rho
            );
            assert!(
                (scalar_val.theta as i16 - simd_val.theta as i16).abs() <= 1,
                "Layer {}: theta mismatch: scalar={}, simd={}",
                i, scalar_val.theta, simd_val.theta
            );
        }
    }

    #[test]
    fn test_superpose_auto() {
        // Small state count - should use scalar
        let small_states = create_test_states(5);
        let small_weights = create_weights(5);
        let result_small = superpose_auto(&small_states, &small_weights);
        assert!(!result_small.get(0).is_null());

        // Large state count - should use SIMD
        let large_states = create_test_states(100);
        let large_weights = create_weights(100);
        let result_large = superpose_auto(&large_states, &large_weights);
        assert!(!result_large.get(0).is_null());
    }

    #[test]
    fn test_empty_states() {
        let result = superpose_scalar(&[], &[]);
        // Should return neutral state (all NULL ByteSil)
        let neutral = SilState::neutral();
        for i in 0..16 {
            assert_eq!(result.get(i), neutral.get(i));
        }
    }

    #[test]
    fn test_single_state() {
        let states = create_test_states(1);
        let weights = vec![1.0];

        let result = superpose_scalar(&states, &weights);

        // With weight 1.0, should be similar to original
        assert_eq!(result.get(0).rho, states[0].get(0).rho);
        assert_eq!(result.get(0).theta, states[0].get(0).theta);
    }
}
