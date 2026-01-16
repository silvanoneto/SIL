//! # Transformer Components
//!
//! Encoder and decoder layers for transformer architecture.

use sil_core::state::SilState;
use super::attention::self_attention_4x4;
use super::layers::{layer_norm, residual};
use super::linalg::matmul_4x4;

/// Transformer encoder layer
///
/// 1. Multi-head self-attention
/// 2. Add & Norm
/// 3. Feed-forward network
/// 4. Add & Norm
pub fn encoder_layer(
    input: &SilState,
    w_q: &SilState,
    w_k: &SilState,
    w_v: &SilState,
    w_ff1: &SilState,
    w_ff2: &SilState,
    eps: f64,
) -> SilState {
    // Self-attention
    let attn_out = self_attention_4x4(input, w_q, w_k, w_v);

    // Add & Norm
    let norm1 = layer_norm(&residual(input, &attn_out), None, None, eps);

    // Feed-forward: FFN(x) = W2 × GELU(W1 × x)
    let ff1 = matmul_4x4(w_ff1, &norm1);
    let ff1_act = super::activations::gelu_state(&ff1);
    let ff2 = matmul_4x4(w_ff2, &ff1_act);

    // Add & Norm
    layer_norm(&residual(&norm1, &ff2), None, None, eps)
}

/// Feed-forward network (MLP block)
pub fn feedforward(
    input: &SilState,
    w1: &SilState,
    w2: &SilState,
    activation: &str,
) -> SilState {
    let hidden = matmul_4x4(w1, input);
    let activated = super::activations::activation(&hidden, activation);
    matmul_4x4(w2, &activated)
}

/// SwiGLU activation (used in LLaMA)
///
/// SwiGLU(x, W, V) = Swish(xW) ⊙ xV
pub fn swiglu(input: &SilState, w: &SilState, v: &SilState) -> SilState {
    let xw = matmul_4x4(w, input);
    let xv = matmul_4x4(v, input);
    let swish_xw = super::activations::swish_state(&xw);
    super::linalg::hadamard(&swish_xw, &xv)
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_feedforward() {
        let input = SilState::neutral();
        let w1 = SilState::neutral();
        let w2 = SilState::neutral();

        let output = feedforward(&input, &w1, &w2, "gelu");

        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&output.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }
}
