//! # Liquid Neural Networks (LNN)
//!
//! Continuous-time recurrent networks with adaptive time constants.
//! Superior generalization with 19K params vs 10M Transformer.
//!
//! ## Whitepaper Reference
//! - §C.20: LNNs
//!
//! ## Key Formula
//!
//! τ(x) · dh/dt = -h + f(x, h)
//!
//! where τ is input-dependent (adaptive time constant).

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};
use super::linalg::{matmul_4x4, add};
use super::activations::tanh_state;

/// Liquid cell forward step
///
/// Implements: τ(x) · dh/dt = -h + f(x, h)
/// Using Euler integration: h_new = h + dt/τ * (-h + f(x, h))
///
/// # Arguments
/// * `x` - Input
/// * `h_prev` - Previous hidden state
/// * `w_x` - Input weights
/// * `w_h` - Hidden weights
/// * `w_tau` - Time constant weights
/// * `dt` - Time step
///
/// # Returns
/// New hidden state
pub fn lnn_cell(
    x: &SilState,
    h_prev: &SilState,
    w_x: &SilState,
    w_h: &SilState,
    w_tau: &SilState,
    dt: f64,
) -> SilState {
    // Compute f(x, h) = tanh(W_x × x + W_h × h)
    let wx = matmul_4x4(w_x, x);
    let wh = matmul_4x4(w_h, h_prev);
    let f = tanh_state(&add(&wx, &wh));

    // Compute adaptive time constant τ(x)
    let tau = lnn_tau(x, w_tau);

    // Euler step: h_new = h + (dt/τ) * (-h + f)
    let mut h_new = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let h_val = magnitude(&h_prev.get(i));
        let f_val = magnitude(&f.get(i));
        let tau_val = magnitude(&tau.get(i)).max(0.01); // Minimum τ

        let derivative = -h_val + f_val;
        let new_val = h_val + (dt / tau_val) * derivative;

        h_new = h_new.with_layer(i, from_mag_phase(new_val, 0.0));
    }

    h_new
}

/// Compute adaptive time constant τ(x)
///
/// τ(x) = softplus(W_τ × x + b_τ)
pub fn lnn_tau(x: &SilState, w_tau: &SilState) -> SilState {
    let tau_raw = matmul_4x4(w_tau, x);

    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = magnitude(&tau_raw.get(i));
        // Softplus with minimum value
        let tau = if val > 20.0 {
            val
        } else {
            (1.0 + val.exp()).ln()
        }
        .max(0.1);

        result = result.with_layer(i, from_mag_phase(tau, 0.0));
    }

    result
}

/// ODE Euler integration step
///
/// x_new = x + dt * f(x)
pub fn ode_euler(x: &SilState, derivative: &SilState, dt: f64) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let x_val = magnitude(&x.get(i));
        let d_val = magnitude(&derivative.get(i));
        let new_val = x_val + dt * d_val;

        result = result.with_layer(i, from_mag_phase(new_val, 0.0));
    }

    result
}

/// RK4 integration step (more accurate than Euler)
pub fn ode_rk4(
    x: &SilState,
    h: &SilState,
    w_x: &SilState,
    w_h: &SilState,
    w_tau: &SilState,
    dt: f64,
) -> SilState {
    // k1 = f(h)
    let k1 = lnn_derivative(x, h, w_x, w_h, w_tau);

    // k2 = f(h + dt/2 * k1)
    let h2 = ode_euler(h, &k1, dt / 2.0);
    let k2 = lnn_derivative(x, &h2, w_x, w_h, w_tau);

    // k3 = f(h + dt/2 * k2)
    let h3 = ode_euler(h, &k2, dt / 2.0);
    let k3 = lnn_derivative(x, &h3, w_x, w_h, w_tau);

    // k4 = f(h + dt * k3)
    let h4 = ode_euler(h, &k3, dt);
    let k4 = lnn_derivative(x, &h4, w_x, w_h, w_tau);

    // h_new = h + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
    let mut h_new = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let h_val = magnitude(&h.get(i));
        let k1_val = magnitude(&k1.get(i));
        let k2_val = magnitude(&k2.get(i));
        let k3_val = magnitude(&k3.get(i));
        let k4_val = magnitude(&k4.get(i));

        let new_val = h_val + (dt / 6.0) * (k1_val + 2.0 * k2_val + 2.0 * k3_val + k4_val);
        h_new = h_new.with_layer(i, from_mag_phase(new_val, 0.0));
    }

    h_new
}

/// Compute LNN derivative: dh/dt = (-h + f(x, h)) / τ(x)
fn lnn_derivative(
    x: &SilState,
    h: &SilState,
    w_x: &SilState,
    w_h: &SilState,
    w_tau: &SilState,
) -> SilState {
    let wx = matmul_4x4(w_x, x);
    let wh = matmul_4x4(w_h, h);
    let f = tanh_state(&add(&wx, &wh));
    let tau = lnn_tau(x, w_tau);

    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let h_val = magnitude(&h.get(i));
        let f_val = magnitude(&f.get(i));
        let tau_val = magnitude(&tau.get(i)).max(0.01);

        let deriv = (-h_val + f_val) / tau_val;
        result = result.with_layer(i, from_mag_phase(deriv, 0.0));
    }

    result
}

/// Closed-form continuous (CfC) neural network
///
/// h(t) = σ(τ) * f(x) + (1 - σ(τ)) * h_prev
pub fn cfc_cell(
    x: &SilState,
    h_prev: &SilState,
    w_x: &SilState,
    _w_h: &SilState,
    w_tau: &SilState,
) -> SilState {
    // Compute f(x)
    let wx = matmul_4x4(w_x, x);
    let f = tanh_state(&wx);

    // Compute interpolation factor from τ
    let tau = lnn_tau(x, w_tau);
    let sigma = super::activations::sigmoid_state(&tau);

    // Interpolate: h = σ * f + (1 - σ) * h_prev
    let mut h_new = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let f_val = magnitude(&f.get(i));
        let h_val = magnitude(&h_prev.get(i));
        let s_val = magnitude(&sigma.get(i));

        let new_val = s_val * f_val + (1.0 - s_val) * h_val;
        h_new = h_new.with_layer(i, from_mag_phase(new_val, 0.0));
    }

    h_new
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_lnn_cell() {
        let x = SilState::neutral();
        let h = SilState::vacuum();
        let w_x = SilState::neutral();
        let w_h = SilState::neutral();
        let w_tau = SilState::neutral();

        let h_new = lnn_cell(&x, &h, &w_x, &w_h, &w_tau, 0.1);

        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&h_new.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_lnn_tau() {
        let x = SilState::neutral();
        let w_tau = SilState::neutral();

        let tau = lnn_tau(&x, &w_tau);

        // τ should be positive
        for i in 0..NUM_LAYERS {
            assert!(magnitude(&tau.get(i)) > 0.0);
        }
    }

    #[test]
    fn test_cfc_cell() {
        let x = SilState::neutral();
        let h = SilState::vacuum();
        let w_x = SilState::neutral();
        let w_h = SilState::neutral();
        let w_tau = SilState::neutral();

        let h_new = cfc_cell(&x, &h, &w_x, &w_h, &w_tau);

        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&h_new.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }
}
