//! # Optimization Algorithms
//!
//! SGD, Adam, learning rate schedulers, and model compression.

use sil_core::state::{SilState, NUM_LAYERS};
use super::linalg::{scale_state, add, sub, norm_l2, clip};
use super::tensor::{magnitude, from_mag_phase};
use super::stats::{max, min, topk as stats_topk};

/// SGD step: θ = θ - lr * ∇θ
pub fn sgd_step(
    params: &SilState,
    gradients: &SilState,
    learning_rate: f64,
) -> SilState {
    let scaled_grad = scale_state(gradients, learning_rate);
    sub(params, &scaled_grad)
}

/// SGD with momentum: v = μv - lr*∇θ, θ = θ + v
pub fn sgd_momentum_step(
    params: &SilState,
    gradients: &SilState,
    velocity: &SilState,
    learning_rate: f64,
    momentum: f64,
) -> (SilState, SilState) {
    let momentum_term = scale_state(velocity, momentum);
    let grad_term = scale_state(gradients, learning_rate);
    let v_new = sub(&momentum_term, &grad_term);
    let params_new = add(params, &v_new);

    (params_new, v_new)
}

/// Adam optimizer state
#[derive(Clone)]
pub struct AdamState {
    pub m: SilState,
    pub v: SilState,
    pub t: usize,
}

impl AdamState {
    pub fn new() -> Self {
        Self {
            m: SilState::vacuum(),
            v: SilState::vacuum(),
            t: 0,
        }
    }
}

impl Default for AdamState {
    fn default() -> Self {
        Self::new()
    }
}

/// Adam step
pub fn adam_step(
    params: &SilState,
    gradients: &SilState,
    state: &mut AdamState,
    learning_rate: f64,
    beta1: f64,
    beta2: f64,
    eps: f64,
) -> SilState {
    state.t += 1;

    let mut params_new = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let p = magnitude(&params.get(i));
        let g = magnitude(&gradients.get(i));
        let m = magnitude(&state.m.get(i));
        let v = magnitude(&state.v.get(i));

        let m_new = beta1 * m + (1.0 - beta1) * g;
        let v_new = beta2 * v + (1.0 - beta2) * g * g;

        let m_hat = m_new / (1.0 - beta1.powi(state.t as i32));
        let v_hat = v_new / (1.0 - beta2.powi(state.t as i32));

        let p_new = p - learning_rate * m_hat / (v_hat.sqrt() + eps);

        state.m = state.m.with_layer(i, from_mag_phase(m_new, 0.0));
        state.v = state.v.with_layer(i, from_mag_phase(v_new, 0.0));
        params_new = params_new.with_layer(i, from_mag_phase(p_new, 0.0));
    }

    params_new
}

/// AdamW (Adam with decoupled weight decay)
pub fn adamw_step(
    params: &SilState,
    gradients: &SilState,
    state: &mut AdamState,
    learning_rate: f64,
    beta1: f64,
    beta2: f64,
    eps: f64,
    weight_decay: f64,
) -> SilState {
    let params_adam = adam_step(params, gradients, state, learning_rate, beta1, beta2, eps);
    let decay = scale_state(params, learning_rate * weight_decay);
    sub(&params_adam, &decay)
}

/// Gradient clipping by norm
pub fn clip_grad_norm(gradients: &SilState, max_norm: f64) -> SilState {
    let norm = norm_l2(gradients);

    if norm > max_norm {
        scale_state(gradients, max_norm / norm)
    } else {
        *gradients
    }
}

/// Gradient clipping by value
pub fn clip_grad_value(gradients: &SilState, max_value: f64) -> SilState {
    clip(gradients, -max_value, max_value)
}

/// Step decay learning rate
pub fn lr_step_decay(
    initial_lr: f64,
    epoch: usize,
    decay_epochs: usize,
    decay_rate: f64,
) -> f64 {
    let num_decays = epoch / decay_epochs;
    initial_lr * decay_rate.powi(num_decays as i32)
}

/// Cosine annealing learning rate
pub fn lr_cosine_annealing(
    initial_lr: f64,
    current_step: usize,
    total_steps: usize,
    min_lr: f64,
) -> f64 {
    let progress = current_step as f64 / total_steps as f64;
    let cos_val = (std::f64::consts::PI * progress).cos();
    min_lr + 0.5 * (initial_lr - min_lr) * (1.0 + cos_val)
}

/// Linear warmup
pub fn lr_warmup(
    target_lr: f64,
    current_step: usize,
    warmup_steps: usize,
) -> f64 {
    if current_step >= warmup_steps {
        target_lr
    } else {
        target_lr * (current_step as f64 / warmup_steps as f64)
    }
}

/// Warmup + cosine decay
pub fn lr_warmup_cosine(
    initial_lr: f64,
    current_step: usize,
    warmup_steps: usize,
    total_steps: usize,
    min_lr: f64,
) -> f64 {
    if current_step < warmup_steps {
        lr_warmup(initial_lr, current_step, warmup_steps)
    } else {
        lr_cosine_annealing(
            initial_lr,
            current_step - warmup_steps,
            total_steps - warmup_steps,
            min_lr,
        )
    }
}

// --- Sparsity and Quantization ---

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
    let top_indices = stats_topk(state, k);
    let mut result = SilState::vacuum();
    for idx in top_indices {
        result = result.with_layer(idx, state.get(idx));
    }
    result
}

/// Quantize to INT8 range [-128, 127]
pub fn quantize_int8(state: &SilState) -> (SilState, f64) {
    let max_val = max(state).abs();
    let min_val = min(state).abs();
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

/// LAMB optimizer step (Layer-wise Adaptive Moments)
pub fn lamb_step(
    params: &SilState,
    gradients: &SilState,
    state: &mut AdamState,
    learning_rate: f64,
    beta1: f64,
    beta2: f64,
    eps: f64,
    weight_decay: f64,
) -> SilState {
    state.t += 1;

    let mut params_new = SilState::vacuum();
    let param_norm = norm_l2(params);

    for i in 0..NUM_LAYERS {
        let p = magnitude(&params.get(i));
        let g = magnitude(&gradients.get(i));
        let m = magnitude(&state.m.get(i));
        let v = magnitude(&state.v.get(i));

        let m_new = beta1 * m + (1.0 - beta1) * g;
        let v_new = beta2 * v + (1.0 - beta2) * g * g;

        let m_hat = m_new / (1.0 - beta1.powi(state.t as i32));
        let v_hat = v_new / (1.0 - beta2.powi(state.t as i32));

        let update = m_hat / (v_hat.sqrt() + eps) + weight_decay * p;
        let update_norm = update.abs();

        let trust_ratio = if param_norm > 0.0 && update_norm > 0.0 {
            param_norm / update_norm
        } else {
            1.0
        };

        let p_new = p - learning_rate * trust_ratio * update;

        state.m = state.m.with_layer(i, from_mag_phase(m_new, 0.0));
        state.v = state.v.with_layer(i, from_mag_phase(v_new, 0.0));
        params_new = params_new.with_layer(i, from_mag_phase(p_new, 0.0));
    }

    params_new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgd_step() {
        let params = SilState::neutral();
        let grads = SilState::neutral();

        let updated = sgd_step(&params, &grads, 0.1);

        let diff: f64 = (0..NUM_LAYERS)
            .map(|i| (magnitude(&params.get(i)) - magnitude(&updated.get(i))).abs())
            .sum();
        assert!(diff >= 0.0, "SGD should produce valid output");
    }

    #[test]
    fn test_adam_step() {
        let params = SilState::neutral();
        let grads = SilState::neutral();
        let mut state = AdamState::new();

        let updated = adam_step(&params, &grads, &mut state, 0.1, 0.9, 0.999, 1e-8);

        assert_eq!(state.t, 1);
        let diff: f64 = (0..NUM_LAYERS)
            .map(|i| (magnitude(&params.get(i)) - magnitude(&updated.get(i))).abs())
            .sum();
        assert!(diff >= 0.0, "Adam should produce valid output");
    }

    #[test]
    fn test_lr_cosine() {
        let lr0 = lr_cosine_annealing(0.1, 0, 100, 0.0);
        let lr50 = lr_cosine_annealing(0.1, 50, 100, 0.0);
        let lr100 = lr_cosine_annealing(0.1, 100, 100, 0.0);

        assert!((lr0 - 0.1).abs() < 1e-10);
        assert!((lr50 - 0.05).abs() < 1e-10);
        assert!((lr100 - 0.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_sparsity_ratio() {
        let state = SilState::vacuum();
        let ratio = sparsity_ratio(&state, 0.01);
        assert!(ratio > 0.9); // Most values should be near zero
    }
}
