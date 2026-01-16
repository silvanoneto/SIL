//! # Sparsity and Quantization
//!
//! Model compression for edge deployment.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};

/// Magnitude-based pruning
pub fn prune_magnitude(state: &SilState, threshold: f64) -> SilState {
    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        if magnitude(&val).abs() >= threshold {
            result = result.with_layer(i, val);
        }
    }
    result
}

/// Top-k pruning (keep k largest)
pub fn prune_topk(state: &SilState, k: usize) -> SilState {
    let top_indices = super::stats::topk(state, k);
    let mut result = SilState::vacuum();
    for idx in top_indices {
        result = result.with_layer(idx, state.get(idx));
    }
    result
}

/// Quantize to INT8 range [-128, 127]
pub fn quantize_int8(state: &SilState) -> (SilState, f64) {
    let max_val = super::stats::max(state).abs();
    let min_val = super::stats::min(state).abs();
    let scale = max_val.max(min_val).max(1e-10);

    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = magnitude(&state.get(i));
        let quantized = (val / scale * 127.0).round().clamp(-128.0, 127.0);
        result = result.with_layer(i, from_mag_phase(quantized, 0.0));
    }

    (result, scale)
}

/// Dequantize from INT8
pub fn dequantize_int8(state: &SilState, scale: f64) -> SilState {
    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = magnitude(&state.get(i)) * scale / 127.0;
        result = result.with_layer(i, from_mag_phase(val, 0.0));
    }
    result
}

/// Compute sparsity ratio
pub fn sparsity_ratio(state: &SilState, threshold: f64) -> f64 {
    let zeros = (0..NUM_LAYERS)
        .filter(|&i| magnitude(&state.get(i)).abs() < threshold)
        .count();
    zeros as f64 / NUM_LAYERS as f64
}
