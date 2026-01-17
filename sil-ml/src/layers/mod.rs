//! # Neural Network Layers
//!
//! Basic neural network layer implementations.

// Module stubs for future implementation
// mod dense;
// mod conv;
// mod norm;
// mod embedding;
// mod dropout;

// Re-export common layer functions
use sil_core::state::{SilState, NUM_LAYERS};
use crate::core::activations::activation as apply_activation;
use crate::core::linalg::{matmul_4x4, add, scale_state};
use crate::core::stats::{mean, std};
use crate::core::tensor::{magnitude, phase, from_mag_phase};

/// Dense (fully connected) layer forward pass
///
/// y = activation(W × x + b)
pub fn dense_forward(
    input: &SilState,
    weights: &SilState,
    bias: &SilState,
    activation: &str,
) -> SilState {
    let wx = matmul_4x4(weights, input);
    let wx_b = add(&wx, bias);
    apply_activation(&wx_b, activation)
}

/// Dropout: Randomly zeroes elements during training
pub fn dropout(input: &SilState, p: f64, training: bool, seed: u64) -> SilState {
    if !training || p <= 0.0 {
        return *input;
    }

    let p = p.clamp(0.0, 1.0);
    let scale = 1.0 / (1.0 - p);

    let mut result = SilState::vacuum();
    let mut rng_state = seed;

    for i in 0..NUM_LAYERS {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let random = (rng_state >> 33) as f64 / (1u64 << 31) as f64;

        let val = input.get(i);
        if random < p {
            result = result.with_layer(i, from_mag_phase(0.0, phase(&val)));
        } else {
            let scaled = magnitude(&val) * scale;
            result = result.with_layer(i, from_mag_phase(scaled, phase(&val)));
        }
    }

    result
}

/// Batch Normalization (per-sample)
pub fn batch_norm(
    input: &SilState,
    gamma: Option<&SilState>,
    beta: Option<&SilState>,
    eps: f64,
) -> SilState {
    let mean = mean(input);
    let std = std(input);
    let std_eps = (std * std + eps).sqrt();

    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let val = input.get(i);
        let normalized = (magnitude(&val) - mean) / std_eps;

        let gamma_val = gamma.map_or(1.0, |g| magnitude(&g.get(i)));
        let beta_val = beta.map_or(0.0, |b| magnitude(&b.get(i)));

        let scaled = normalized * gamma_val + beta_val;
        result = result.with_layer(i, from_mag_phase(scaled, phase(&val)));
    }

    result
}

/// Layer Normalization
pub fn layer_norm(
    input: &SilState,
    gamma: Option<&SilState>,
    beta: Option<&SilState>,
    eps: f64,
) -> SilState {
    batch_norm(input, gamma, beta, eps)
}

/// RMS Normalization (LLaMA style)
pub fn rms_norm(input: &SilState, gamma: Option<&SilState>, eps: f64) -> SilState {
    let mut sum_sq = 0.0;
    for i in 0..NUM_LAYERS {
        let mag = magnitude(&input.get(i));
        sum_sq += mag * mag;
    }
    let rms = (sum_sq / NUM_LAYERS as f64 + eps).sqrt();

    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let val = input.get(i);
        let normalized = magnitude(&val) / rms;

        let gamma_val = gamma.map_or(1.0, |g| magnitude(&g.get(i)));
        let scaled = normalized * gamma_val;

        result = result.with_layer(i, from_mag_phase(scaled, phase(&val)));
    }

    result
}

/// Residual connection
pub fn residual(input: &SilState, transformed: &SilState) -> SilState {
    add(input, transformed)
}

/// Scaled residual connection
pub fn residual_scaled(input: &SilState, transformed: &SilState, scale: f64) -> SilState {
    let scaled = scale_state(transformed, scale);
    add(input, &scaled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_forward() {
        let input = SilState::neutral();
        let weights = SilState::neutral();
        let bias = SilState::vacuum();

        let output = dense_forward(&input, &weights, &bias, "relu");

        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&output.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_dropout_inference() {
        let input = SilState::neutral();
        let output = dropout(&input, 0.5, false, 12345);

        for i in 0..NUM_LAYERS {
            let in_val = magnitude(&input.get(i));
            let out_val = magnitude(&output.get(i));
            assert!((in_val - out_val).abs() < 1e-10);
        }
    }
}
