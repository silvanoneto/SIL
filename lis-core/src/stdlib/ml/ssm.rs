//! # State Space Models (SSM) / Mamba
//!
//! Linear-complexity sequence modeling with O(n) vs O(n²) attention.
//!
//! ## Whitepaper Reference
//! - §C.19: SSMs/Mamba
//!
//! ## Implementation
//!
//! Implements basic SSM: h_t = A·h_{t-1} + B·x_t, y_t = C·h_t + D·x_t
//! And Mamba-style selective mechanism.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
use super::linalg::{matmul_4x4, add};
use super::activations::sigmoid_state;

/// SSM step: h_t = A·h_{t-1} + B·x_t
///
/// # Arguments
/// * `h_prev` - Previous hidden state
/// * `x` - Current input
/// * `a` - State transition matrix (4x4)
/// * `b` - Input projection matrix (4x4)
///
/// # Returns
/// New hidden state
pub fn ssm_step(
    h_prev: &SilState,
    x: &SilState,
    a: &SilState,
    b: &SilState,
) -> SilState {
    let ah = matmul_4x4(a, h_prev);
    let bx = matmul_4x4(b, x);
    add(&ah, &bx)
}

/// SSM output: y = C·h + D·x
///
/// # Arguments
/// * `h` - Current hidden state
/// * `x` - Current input
/// * `c` - Output projection matrix (4x4)
/// * `d` - Skip connection matrix (4x4)
///
/// # Returns
/// Output state
pub fn ssm_output(
    h: &SilState,
    x: &SilState,
    c: &SilState,
    d: &SilState,
) -> SilState {
    let ch = matmul_4x4(c, h);
    let dx = matmul_4x4(d, x);
    add(&ch, &dx)
}

/// Mamba selective mechanism
///
/// Makes A and B input-dependent for content-aware filtering.
///
/// # Arguments
/// * `x` - Input
/// * `h_prev` - Previous hidden state
/// * `w_a` - Weights for computing Δ (discretization)
/// * `w_b` - Weights for computing B
///
/// # Returns
/// New hidden state with selective filtering
pub fn mamba_selective(
    x: &SilState,
    h_prev: &SilState,
    w_a: &SilState,
    w_b: &SilState,
) -> SilState {
    // Compute input-dependent parameters
    // Δ = softplus(Linear(x))
    let delta_raw = matmul_4x4(w_a, x);
    let delta = softplus_state(&delta_raw);

    // B = Linear(x)
    let b = matmul_4x4(w_b, x);

    // Discretize: A_bar = exp(Δ·A), B_bar = Δ·B
    // Simplified: use delta as scaling factor
    let mut h_new = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let h_val = magnitude(&h_prev.get(i));
        let x_val = magnitude(&x.get(i));
        let d_val = magnitude(&delta.get(i));
        let b_val = magnitude(&b.get(i));

        // Exponential decay with input-dependent rate
        let decay = (-d_val).exp();
        let input_scale = (1.0 - decay) * b_val;

        let new_val = decay * h_val + input_scale * x_val;
        h_new = h_new.with_layer(i, from_mag_phase(new_val, 0.0));
    }

    h_new
}

/// Softplus activation: log(1 + e^x)
fn softplus(x: f64) -> f64 {
    if x > 20.0 {
        x // Avoid overflow
    } else {
        (1.0 + x.exp()).ln()
    }
}

fn softplus_state(state: &SilState) -> SilState {
    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        result = result.with_layer(i, from_mag_phase(softplus(magnitude(&val)), phase(&val)));
    }
    result
}

/// Full Mamba block (simplified)
///
/// Combines: projection → conv → SSM → output projection
pub fn mamba_block(
    x: &SilState,
    h_prev: &SilState,
    w_in: &SilState,
    w_conv: &SilState,
    w_ssm_a: &SilState,
    w_ssm_b: &SilState,
    w_out: &SilState,
) -> (SilState, SilState) {
    // Input projection
    let x_proj = matmul_4x4(w_in, x);

    // 1D convolution (simplified as matmul)
    let x_conv = matmul_4x4(w_conv, &x_proj);

    // SSM with selective mechanism
    let h_new = mamba_selective(&x_conv, h_prev, w_ssm_a, w_ssm_b);

    // Output projection with gating
    let gate = sigmoid_state(&x_proj);
    let gated = super::linalg::hadamard(&h_new, &gate);
    let output = matmul_4x4(w_out, &gated);

    (output, h_new)
}

/// Process sequence with Mamba
pub fn mamba_sequence(
    sequence: &[SilState],
    w_in: &SilState,
    w_conv: &SilState,
    w_ssm_a: &SilState,
    w_ssm_b: &SilState,
    w_out: &SilState,
) -> Vec<SilState> {
    let mut outputs = Vec::with_capacity(sequence.len());
    let mut h = SilState::vacuum();

    for x in sequence {
        let (y, h_new) = mamba_block(x, &h, w_in, w_conv, w_ssm_a, w_ssm_b, w_out);
        outputs.push(y);
        h = h_new;
    }

    outputs
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_ssm_step() {
        let h = SilState::vacuum();
        let x = SilState::neutral();
        let a = SilState::neutral();
        let b = SilState::neutral();

        let h_new = ssm_step(&h, &x, &a, &b);

        // Should produce non-zero output with non-zero input
        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&h_new.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_mamba_selective() {
        let x = SilState::neutral();
        let h = SilState::vacuum();
        let w_a = SilState::neutral();
        let w_b = SilState::neutral();

        let h_new = mamba_selective(&x, &h, &w_a, &w_b);

        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&h_new.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }
}
