//! # Activation Functions
//!
//! Non-linear activation functions for neural networks.
//!
//! ## Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `relu` | Rectified Linear Unit |
//! | `relu_state` | ReLU on all 16 layers |
//! | `sigmoid` | Logistic sigmoid |
//! | `tanh` | Hyperbolic tangent |
//! | `softmax` | Softmax over 16 elements |
//! | `gelu` | Gaussian Error Linear Unit |
//! | `leaky_relu` | Leaky ReLU with configurable slope |
//! | `swish` | Swish activation (x * sigmoid(x)) |
//! | `silu` | SiLU (same as Swish) |
//!
//! ## Implementation Notes
//!
//! Activations operate on ByteSil magnitude (ρ) while preserving phase (θ).
//! This allows complex-valued neural networks with phase information.

use sil_core::state::{ByteSil, SilState, NUM_LAYERS};
use std::f64::consts::{E, PI};
use super::tensor::{magnitude, phase, from_mag_phase};

/// ReLU activation: max(0, x)
///
/// Operates on ByteSil magnitude, preserves phase.
#[inline]
pub fn relu(x: ByteSil) -> ByteSil {
    let mag = magnitude(&x);
    if mag > 0.0 {
        x
    } else {
        from_mag_phase(0.0, phase(&x))
    }
}

/// ReLU on all 16 layers of a State
pub fn relu_state(state: &SilState) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, relu(layer));
    }
    result
}

/// Sigmoid activation: 1 / (1 + e^(-x))
///
/// Maps magnitude to (0, 1) range, preserves phase.
#[inline]
pub fn sigmoid(x: ByteSil) -> ByteSil {
    let mag = magnitude(&x);
    let sig = 1.0 / (1.0 + E.powf(-mag));
    from_mag_phase(sig, phase(&x))
}

/// Sigmoid on all 16 layers
pub fn sigmoid_state(state: &SilState) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, sigmoid(layer));
    }
    result
}

/// Hyperbolic tangent: (e^x - e^(-x)) / (e^x + e^(-x))
///
/// Maps magnitude to (-1, 1) range, preserves phase.
#[inline]
pub fn tanh(x: ByteSil) -> ByteSil {
    let mag = magnitude(&x);
    let t = mag.tanh();
    from_mag_phase(t, phase(&x))
}

/// Tanh on all 16 layers
pub fn tanh_state(state: &SilState) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, tanh(layer));
    }
    result
}

/// Softmax: e^xi / Σ e^xj
///
/// Normalizes magnitudes to probability distribution over 16 layers.
/// Phase is preserved from original layers.
pub fn softmax(state: &SilState) -> SilState {
    // Find max for numerical stability
    let mut max_mag = f64::NEG_INFINITY;
    for i in 0..NUM_LAYERS {
        let mag = magnitude(&state.get(i));
        if mag > max_mag {
            max_mag = mag;
        }
    }

    // Compute exp(x - max) for stability
    let mut exp_vals = [0.0; NUM_LAYERS];
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let mag = magnitude(&state.get(i));
        exp_vals[i] = E.powf(mag - max_mag);
        sum += exp_vals[i];
    }

    // Normalize
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let p = phase(&state.get(i));
        let softmax_val = exp_vals[i] / sum;
        result = result.with_layer(i, from_mag_phase(softmax_val, p));
    }

    result
}

/// GELU: Gaussian Error Linear Unit
///
/// x * Φ(x) where Φ is the CDF of standard normal.
/// Approximation: 0.5 * x * (1 + tanh(sqrt(2/π) * (x + 0.044715 * x³)))
#[inline]
pub fn gelu(x: ByteSil) -> ByteSil {
    let mag = magnitude(&x);
    let sqrt_2_pi = (2.0 / PI).sqrt();
    let inner = sqrt_2_pi * (mag + 0.044715 * mag.powi(3));
    let gelu = 0.5 * mag * (1.0 + inner.tanh());
    from_mag_phase(gelu, phase(&x))
}

/// GELU on all 16 layers
pub fn gelu_state(state: &SilState) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, gelu(layer));
    }
    result
}

/// Leaky ReLU: max(αx, x) where α is typically 0.01
#[inline]
pub fn leaky_relu(x: ByteSil, alpha: f64) -> ByteSil {
    let mag = magnitude(&x);
    let result = if mag > 0.0 { mag } else { alpha * mag };
    from_mag_phase(result, phase(&x))
}

/// Leaky ReLU on all 16 layers
pub fn leaky_relu_state(state: &SilState, alpha: f64) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, leaky_relu(layer, alpha));
    }
    result
}

/// Swish activation: x * sigmoid(x)
///
/// Self-gated activation used in EfficientNet.
#[inline]
pub fn swish(x: ByteSil) -> ByteSil {
    let mag = magnitude(&x);
    let sig = 1.0 / (1.0 + E.powf(-mag));
    let swish = mag * sig;
    from_mag_phase(swish, phase(&x))
}

/// Swish on all 16 layers
pub fn swish_state(state: &SilState) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, swish(layer));
    }
    result
}

/// SiLU (Sigmoid Linear Unit) - same as Swish
#[inline]
pub fn silu(x: ByteSil) -> ByteSil {
    swish(x)
}

/// SiLU on all 16 layers
pub fn silu_state(state: &SilState) -> SilState {
    swish_state(state)
}

/// ELU: Exponential Linear Unit
///
/// x if x > 0, α(e^x - 1) otherwise
#[inline]
pub fn elu(x: ByteSil, alpha: f64) -> ByteSil {
    let mag = magnitude(&x);
    let result = if mag > 0.0 {
        mag
    } else {
        alpha * (E.powf(mag) - 1.0)
    };
    from_mag_phase(result, phase(&x))
}

/// ELU on all 16 layers
pub fn elu_state(state: &SilState, alpha: f64) -> SilState {
    let mut result = *state;
    for i in 0..NUM_LAYERS {
        let layer = state.get(i);
        result = result.with_layer(i, elu(layer, alpha));
    }
    result
}

/// Hardtanh: clamp(x, min, max)
#[inline]
pub fn hardtanh(x: ByteSil, min_val: f64, max_val: f64) -> ByteSil {
    let mag = magnitude(&x).clamp(min_val, max_val);
    from_mag_phase(mag, phase(&x))
}

/// SwiGLU activation (used in LLaMA)
///
/// SwiGLU(x, gate) = swish(gate) * x
pub fn swiglu(x: &SilState, gate: &SilState) -> SilState {
    let gate_activated = swish_state(gate);
    super::linalg::hadamard(x, &gate_activated)
}

/// Apply activation by name
///
/// Supported: "relu", "sigmoid", "tanh", "gelu", "swish", "silu"
pub fn activation(state: &SilState, name: &str) -> SilState {
    match name.to_lowercase().as_str() {
        "relu" => relu_state(state),
        "sigmoid" => sigmoid_state(state),
        "tanh" => tanh_state(state),
        "gelu" => gelu_state(state),
        "swish" | "silu" => swish_state(state),
        "leaky_relu" => leaky_relu_state(state, 0.01),
        "elu" => elu_state(state, 1.0),
        "none" | "linear" | "identity" => *state,
        _ => *state, // Default to identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relu() {
        let pos = from_mag_phase(2.0, 0.0);
        let neg = from_mag_phase(0.0, 0.0);

        assert!(magnitude(&relu(pos)) > 0.5);
        assert!(magnitude(&relu(neg)) < 0.1);
    }

    #[test]
    fn test_sigmoid() {
        let zero = from_mag_phase(0.0, 0.0);
        let result = sigmoid(zero);
        let mag = magnitude(&result);
        assert!(mag > 0.0 && mag < 2.0, "Sigmoid of ~0 was {}", mag);
    }

    #[test]
    fn test_softmax() {
        let state = SilState::neutral();
        let result = softmax(&state);

        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&result.get(i)))
            .sum();
        assert!(sum > 0.0, "Softmax sum should be > 0, was {}", sum);
    }

    #[test]
    fn test_activation_by_name() {
        let state = SilState::neutral();

        let relu_result = activation(&state, "relu");
        let sigmoid_result = activation(&state, "sigmoid");

        assert!(magnitude(&relu_result.get(0)) >= 0.0);
        assert!(magnitude(&sigmoid_result.get(0)) > 0.0);
        assert!(magnitude(&sigmoid_result.get(0)) < 3.0);
    }
}
