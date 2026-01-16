//! # Kolmogorov-Arnold Networks (KAN)
//!
//! KANs use learnable activation functions (B-splines) instead of fixed activations.
//! Key advantage: 10x fewer parameters for smooth functions.
//!
//! ## Whitepaper Reference
//! - §C.18: KANs
//!
//! ## Implementation
//!
//! Uses precomputed B-spline lookup tables for fast inference.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};

/// B-spline basis function evaluation
///
/// Evaluates cubic B-spline at position t ∈ [0, 1].
pub fn bspline_basis(t: f64, control_points: &[f64; 4]) -> f64 {
    let t = t.clamp(0.0, 1.0);

    // Cubic B-spline basis functions
    let t2 = t * t;
    let t3 = t2 * t;

    let b0 = (1.0 - t).powi(3) / 6.0;
    let b1 = (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0;
    let b2 = (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0;
    let b3 = t3 / 6.0;

    b0 * control_points[0] + b1 * control_points[1] + b2 * control_points[2] + b3 * control_points[3]
}

/// KAN layer forward pass
///
/// Each connection has its own learnable spline function.
/// Input: 4 elements, Output: 4 elements (using 4x4 = 16 spline functions).
///
/// # Arguments
/// * `input` - Input State (first 4 elements used)
/// * `spline_params` - 16 splines × 4 control points = 64 params (stored in 4 States)
///
/// # Returns
/// Output State (first 4 elements)
pub fn kan_layer(input: &SilState, spline_params: &[SilState; 4]) -> SilState {
    let mut result = SilState::vacuum();

    for j in 0..4 {
        let mut sum = 0.0;

        for i in 0..4 {
            // Get input value and normalize to [0, 1]
            let x = magnitude(&input.get(i));
            let t = (x.tanh() + 1.0) / 2.0; // Map to [0, 1]

            // Get control points for this connection
            let spline_idx = i * 4 + j;
            let param_state_idx = spline_idx / 4;
            let param_offset = (spline_idx % 4) * 4;

            let control_points = [
                magnitude(&spline_params[param_state_idx].get(param_offset)),
                magnitude(&spline_params[param_state_idx].get((param_offset + 1).min(15))),
                magnitude(&spline_params[param_state_idx].get((param_offset + 2).min(15))),
                magnitude(&spline_params[param_state_idx].get((param_offset + 3).min(15))),
            ];

            // Evaluate spline
            sum += bspline_basis(t, &control_points);
        }

        result = result.with_layer(j, from_mag_phase(sum, 0.0));
    }

    result
}

/// Simplified KAN layer with single State of parameters
///
/// Uses 16 parameters (one per connection) with linear interpolation.
pub fn kan_layer_simple(input: &SilState, weights: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for j in 0..4 {
        let mut sum = 0.0;

        for i in 0..4 {
            let x = magnitude(&input.get(i));
            let w_idx = i * 4 + j;
            let w = magnitude(&weights.get(w_idx));

            // Simple learnable activation: w * x + (1-|w|) * tanh(x)
            let linear = w * x;
            let nonlinear = (1.0 - w.abs()) * x.tanh();
            sum += linear + nonlinear;
        }

        result = result.with_layer(j, from_mag_phase(sum, 0.0));
    }

    result
}

/// Initialize KAN spline parameters
pub fn kan_init(_num_inputs: usize, _num_outputs: usize) -> SilState {
    let mut result = SilState::vacuum();

    // Initialize with small random-like values
    for i in 0..NUM_LAYERS {
        let val = ((i * 17 + 31) % 100) as f64 / 1000.0 - 0.05;
        result = result.with_layer(i, from_mag_phase(val, 0.0));
    }

    result
}

/// TinyKAN: Extremely compact KAN for MCU deployment
///
/// Uses lookup table instead of computing splines.
pub struct TinyKAN {
    /// Lookup table: 16 entries per spline
    lut: [[f64; 16]; 16],
}

impl TinyKAN {
    /// Create new TinyKAN with initialized LUT
    pub fn new() -> Self {
        let mut lut = [[0.0; 16]; 16];

        // Initialize with identity-like mapping
        for i in 0..16 {
            for j in 0..16 {
                let t = j as f64 / 15.0;
                lut[i][j] = t; // Linear by default
            }
        }

        Self { lut }
    }

    /// Forward pass using LUT
    pub fn forward(&self, input: &SilState) -> SilState {
        let mut result = SilState::vacuum();

        for j in 0..4 {
            let mut sum = 0.0;

            for i in 0..4 {
                let x = magnitude(&input.get(i));
                let t = ((x.tanh() + 1.0) / 2.0 * 15.0).round() as usize;
                let t = t.min(15);

                let lut_idx = i * 4 + j;
                sum += self.lut[lut_idx][t];
            }

            result = result.with_layer(j, from_mag_phase(sum, 0.0));
        }

        result
    }
}

impl Default for TinyKAN {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_bspline_basis() {
        let control_points = [0.0, 1.0, 1.0, 0.0];

        // At t=0.5, should be near 1.0 (peak)
        let val = bspline_basis(0.5, &control_points);
        assert!(val > 0.5);
    }

    #[test]
    fn test_kan_layer_simple() {
        let input = SilState::neutral();
        let weights = kan_init(4, 4);

        let output = kan_layer_simple(&input, &weights);

        // Should produce non-zero output
        let sum: f64 = (0..4)
            .map(|i| magnitude(&output.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_tiny_kan() {
        let kan = TinyKAN::new();
        let input = SilState::neutral();

        let output = kan.forward(&input);

        let sum: f64 = (0..4)
            .map(|i| magnitude(&output.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }
}
