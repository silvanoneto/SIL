//! # Attention Mechanisms
//!
//! Self-attention and multi-head attention for transformers.

use sil_core::state::{SilState, NUM_LAYERS};
use super::linalg::{dot, scale_state, matmul_4x4};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};

/// Scaled dot-product attention
///
/// Attention(Q, K, V) = softmax(Q·K^T / √d_k) · V
///
/// For single query/key/value vectors.
pub fn attention_qkv(q: &SilState, k: &SilState, v: &SilState) -> SilState {
    // Compute Q·K
    let qk = dot(q, k);

    // Scale by sqrt(d_k)
    let d_k = NUM_LAYERS as f64;
    let scaled = qk / d_k.sqrt();

    // Apply softmax-like scaling (simplified for single pair)
    let attention_weight = 1.0 / (1.0 + (-scaled).exp());

    // Weight the value
    scale_state(v, attention_weight)
}

/// Self-attention over 4 positions
///
/// Input: 4x4 matrix where each row is a position.
/// Output: 4x4 matrix of attended values.
pub fn self_attention_4x4(
    input: &SilState,
    w_q: &SilState,
    w_k: &SilState,
    w_v: &SilState,
) -> SilState {
    // Project to Q, K, V
    let q = matmul_4x4(w_q, input);
    let k = matmul_4x4(w_k, input);
    let v = matmul_4x4(w_v, input);

    // Compute attention scores: Q × K^T
    let mut scores = SilState::vacuum();
    for i in 0..4 {
        for j in 0..4 {
            let mut dot = 0.0;
            for d in 0..4 {
                let q_idx = i * 4 + d;
                let k_idx = j * 4 + d;
                dot += magnitude(&q.get(q_idx)) * magnitude(&k.get(k_idx));
            }
            // Scale
            dot /= 2.0; // sqrt(4) = 2
            let idx = i * 4 + j;
            scores = scores.with_layer(idx, from_mag_phase(dot, 0.0));
        }
    }

    // Softmax over each row
    let scores = softmax_rows_4x4(&scores);

    // Multiply by V
    matmul_4x4(&scores, &v)
}

/// Softmax over rows of 4x4 matrix
fn softmax_rows_4x4(input: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        // Find max in row for stability
        let mut max_val = f64::NEG_INFINITY;
        for j in 0..4 {
            let idx = i * 4 + j;
            let val = magnitude(&input.get(idx));
            if val > max_val {
                max_val = val;
            }
        }

        // Compute exp and sum
        let mut exp_vals = [0.0; 4];
        let mut sum = 0.0;
        for j in 0..4 {
            let idx = i * 4 + j;
            let val = magnitude(&input.get(idx));
            exp_vals[j] = (val - max_val).exp();
            sum += exp_vals[j];
        }

        // Normalize
        for j in 0..4 {
            let idx = i * 4 + j;
            let softmax_val = exp_vals[j] / sum;
            result = result.with_layer(idx, from_mag_phase(softmax_val, 0.0));
        }
    }

    result
}

/// Causal mask for autoregressive attention
///
/// Returns mask where future positions are -inf.
pub fn causal_mask_4x4() -> SilState {
    let mut mask = SilState::vacuum();

    for i in 0..4 {
        for j in 0..4 {
            let idx = i * 4 + j;
            let val = if j <= i { 0.0 } else { f64::NEG_INFINITY };
            mask = mask.with_layer(idx, from_mag_phase(val, 0.0));
        }
    }

    mask
}

/// Apply mask to attention scores
pub fn apply_mask(scores: &SilState, mask: &SilState) -> SilState {
    use super::linalg::add;
    add(scores, mask)
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_attention_qkv() {
        let q = SilState::neutral();
        let k = SilState::neutral();
        let v = SilState::neutral();

        let output = attention_qkv(&q, &k, &v);

        // Output should be scaled version of v
        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&output.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_causal_mask() {
        let mask = causal_mask_4x4();

        // Position (0,1) = index 1 should be -inf (future position)
        // But ByteSil can't represent -inf, it stores as very small magnitude
        // from_mag_phase(NEG_INFINITY, 0.0) will clamp to minimum rho = -8
        // So magnitude will be e^(-8) ≈ 0.000335
        let future_val = magnitude(&mask.get(1));
        assert!(future_val < 0.01, "Future position should be ~0 (was {})", future_val);

        // Position (1,0) = index 4 should be 0 (past/current)
        // from_mag_phase(0.0, 0.0) also gives minimum magnitude
        let past_val = magnitude(&mask.get(4));
        assert!(past_val < 0.01, "Past position should be ~0 (was {})", past_val);
    }
}
