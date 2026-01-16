//! # Embeddings and Positional Encoding
//!
//! Token embeddings and position encoding for sequence models.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};

/// Sinusoidal positional encoding (Vaswani et al., 2017)
///
/// PE(pos, 2i) = sin(pos / 10000^(2i/d))
/// PE(pos, 2i+1) = cos(pos / 10000^(2i/d))
pub fn sinusoidal_pe(position: usize, dim: usize) -> SilState {
    let dim = dim.min(NUM_LAYERS);
    let mut result = SilState::vacuum();

    for i in 0..(dim / 2) {
        let angle = position as f64 / 10000_f64.powf(2.0 * i as f64 / dim as f64);

        let sin_val = angle.sin();
        let cos_val = angle.cos();

        if 2 * i < NUM_LAYERS {
            result = result.with_layer(2 * i, from_mag_phase(sin_val, 0.0));
        }
        if 2 * i + 1 < NUM_LAYERS {
            result = result.with_layer(2 * i + 1, from_mag_phase(cos_val, 0.0));
        }
    }

    result
}

/// Rotary Position Embedding (RoPE) - used in LLaMA
///
/// Rotates query/key vectors by position-dependent angles.
pub fn rope(x: &SilState, position: usize, base: f64) -> SilState {
    let mut result = *x;

    for i in 0..(NUM_LAYERS / 2) {
        let theta = position as f64 / base.powf(2.0 * i as f64 / NUM_LAYERS as f64);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        let x0 = magnitude(&x.get(2 * i));
        let x1 = magnitude(&x.get(2 * i + 1));

        let rot0 = x0 * cos_t - x1 * sin_t;
        let rot1 = x0 * sin_t + x1 * cos_t;

        result = result.with_layer(2 * i, from_mag_phase(rot0, 0.0));
        result = result.with_layer(2 * i + 1, from_mag_phase(rot1, 0.0));
    }

    result
}

/// ALiBi position bias (Attention with Linear Biases)
///
/// Returns bias to add to attention scores.
pub fn alibi_bias(query_pos: usize, key_pos: usize, head_slope: f64) -> f64 {
    let distance = (query_pos as i64 - key_pos as i64).abs() as f64;
    -head_slope * distance
}

/// Simple token embedding lookup
///
/// Retrieves embedding from table (simulated with position).
pub fn embedding_lookup(token_id: usize, embed_dim: usize) -> SilState {
    // Generate deterministic embedding based on token_id
    let mut result = SilState::vacuum();
    let embed_dim = embed_dim.min(NUM_LAYERS);

    for i in 0..embed_dim {
        // Pseudo-random but deterministic value
        let val = ((token_id * 17 + i * 31) % 1000) as f64 / 1000.0 - 0.5;
        result = result.with_layer(i, from_mag_phase(val, 0.0));
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_sinusoidal_pe() {
        let pe0 = sinusoidal_pe(0, 16);
        let pe1 = sinusoidal_pe(1, 16);

        // Position 0 and 1 should have different encodings
        let diff: f64 = (0..NUM_LAYERS)
            .map(|i| (magnitude(&pe0.get(i)) - magnitude(&pe1.get(i))).abs())
            .sum();
        assert!(diff > 0.0);
    }

    #[test]
    fn test_rope() {
        let x = SilState::neutral();
        let rotated = rope(&x, 10, 10000.0);

        // Rotated should be different from original
        let diff: f64 = (0..NUM_LAYERS)
            .map(|i| (magnitude(&x.get(i)) - magnitude(&rotated.get(i))).abs())
            .sum();
        assert!(diff > 0.0);
    }
}
