//! # Recurrent Neural Networks
//!
//! RNN, LSTM, and GRU cell implementations.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
use super::activations::{tanh_state, sigmoid_state};
use super::linalg::{matmul_4x4, add, hadamard};

/// Simple RNN cell
///
/// h_t = tanh(W_h × h_{t-1} + W_x × x_t + b)
pub fn rnn_cell(
    x: &SilState,
    h_prev: &SilState,
    w_h: &SilState,
    w_x: &SilState,
    bias: &SilState,
) -> SilState {
    let h_part = matmul_4x4(w_h, h_prev);
    let x_part = matmul_4x4(w_x, x);
    let sum = add(&h_part, &x_part);
    let sum_bias = add(&sum, bias);
    tanh_state(&sum_bias)
}

/// LSTM cell (simplified)
///
/// Implements forget gate, input gate, cell state, output gate.
pub fn lstm_cell(
    x: &SilState,
    h_prev: &SilState,
    c_prev: &SilState,
    // Weights for input
    w_xi: &SilState,
    w_hi: &SilState,
    // Weights for forget
    w_xf: &SilState,
    w_hf: &SilState,
    // Weights for cell
    w_xc: &SilState,
    w_hc: &SilState,
    // Weights for output
    w_xo: &SilState,
    w_ho: &SilState,
) -> (SilState, SilState) {
    // Input gate: i = σ(W_xi × x + W_hi × h)
    let i_x = matmul_4x4(w_xi, x);
    let i_h = matmul_4x4(w_hi, h_prev);
    let i_gate = sigmoid_state(&add(&i_x, &i_h));

    // Forget gate: f = σ(W_xf × x + W_hf × h)
    let f_x = matmul_4x4(w_xf, x);
    let f_h = matmul_4x4(w_hf, h_prev);
    let f_gate = sigmoid_state(&add(&f_x, &f_h));

    // Cell candidate: c̃ = tanh(W_xc × x + W_hc × h)
    let c_x = matmul_4x4(w_xc, x);
    let c_h = matmul_4x4(w_hc, h_prev);
    let c_candidate = tanh_state(&add(&c_x, &c_h));

    // Cell state: c = f ⊙ c_prev + i ⊙ c̃
    let f_c = hadamard(&f_gate, c_prev);
    let i_c = hadamard(&i_gate, &c_candidate);
    let c_new = add(&f_c, &i_c);

    // Output gate: o = σ(W_xo × x + W_ho × h)
    let o_x = matmul_4x4(w_xo, x);
    let o_h = matmul_4x4(w_ho, h_prev);
    let o_gate = sigmoid_state(&add(&o_x, &o_h));

    // Hidden state: h = o ⊙ tanh(c)
    let c_tanh = tanh_state(&c_new);
    let h_new = hadamard(&o_gate, &c_tanh);

    (h_new, c_new)
}

/// GRU cell (simplified)
///
/// Implements reset gate and update gate.
pub fn gru_cell(
    x: &SilState,
    h_prev: &SilState,
    w_xr: &SilState,
    w_hr: &SilState,
    w_xz: &SilState,
    w_hz: &SilState,
    w_xh: &SilState,
    w_hh: &SilState,
) -> SilState {
    // Reset gate: r = σ(W_xr × x + W_hr × h)
    let r_x = matmul_4x4(w_xr, x);
    let r_h = matmul_4x4(w_hr, h_prev);
    let r_gate = sigmoid_state(&add(&r_x, &r_h));

    // Update gate: z = σ(W_xz × x + W_hz × h)
    let z_x = matmul_4x4(w_xz, x);
    let z_h = matmul_4x4(w_hz, h_prev);
    let z_gate = sigmoid_state(&add(&z_x, &z_h));

    // Candidate hidden: h̃ = tanh(W_xh × x + W_hh × (r ⊙ h))
    let rh = hadamard(&r_gate, h_prev);
    let h_x = matmul_4x4(w_xh, x);
    let h_rh = matmul_4x4(w_hh, &rh);
    let h_candidate = tanh_state(&add(&h_x, &h_rh));

    // New hidden: h = (1 - z) ⊙ h_prev + z ⊙ h̃
    let one_minus_z = subtract_from_one(&z_gate);
    let keep = hadamard(&one_minus_z, h_prev);
    let update = hadamard(&z_gate, &h_candidate);
    add(&keep, &update)
}

/// Helper: 1 - x for each element
fn subtract_from_one(state: &SilState) -> SilState {
    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        result = result.with_layer(i, from_mag_phase(1.0 - magnitude(&val), phase(&val)));
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_rnn_cell() {
        let x = SilState::neutral();
        let h = SilState::vacuum();
        let w_h = SilState::neutral();
        let w_x = SilState::neutral();
        let bias = SilState::vacuum();

        let h_new = rnn_cell(&x, &h, &w_h, &w_x, &bias);

        // Should produce non-zero output
        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&h_new.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }
}
