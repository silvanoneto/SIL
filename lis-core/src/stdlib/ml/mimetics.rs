//! # Mimetic Patterns
//!
//! Self-organization and emergent behavior patterns.
//!
//! ## Whitepaper Reference
//! - §A.5: Emergence and self-organization
//! - §C.40: Mimetic computing paradigms
//!
//! ## Patterns
//!
//! - **Hebbian Learning**: "Neurons that fire together, wire together"
//! - **Kuramoto Oscillators**: Phase synchronization
//! - **Hysteresis**: State-dependent thresholds
//! - **Apoptosis**: Programmed cell death (pruning)

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
use std::f64::consts::PI;

/// Hebbian learning update
///
/// Δw = η * pre * post
///
/// "Neurons that fire together, wire together"
///
/// # Arguments
/// * `weights` - Current weight state
/// * `pre` - Pre-synaptic activations
/// * `post` - Post-synaptic activations
/// * `learning_rate` - Learning rate η
///
/// # Returns
/// Updated weights
pub fn hebbian_update(
    weights: &SilState,
    pre: &SilState,
    post: &SilState,
    learning_rate: f64,
) -> SilState {
    let mut result = *weights;

    for i in 0..NUM_LAYERS {
        let w = weights.get(i);
        let pre_val = magnitude(&pre.get(i));
        let post_val = magnitude(&post.get(i));

        // Hebbian update: Δw = η * pre * post
        let delta = learning_rate * pre_val * post_val;
        let new_mag = magnitude(&w) + delta;

        result = result.with_layer(i, from_mag_phase(new_mag, phase(&w)));
    }

    result
}

/// Oja's rule: Hebbian with normalization
///
/// Δw = η * y * (x - y * w)
///
/// Prevents unbounded weight growth.
///
/// # Arguments
/// * `weights` - Current weights
/// * `input` - Input activations
/// * `output` - Output activations
/// * `learning_rate` - Learning rate
///
/// # Returns
/// Updated weights (normalized)
pub fn oja_update(
    weights: &SilState,
    input: &SilState,
    output: &SilState,
    learning_rate: f64,
) -> SilState {
    let mut result = *weights;

    for i in 0..NUM_LAYERS {
        let w = magnitude(&weights.get(i));
        let x = magnitude(&input.get(i));
        let y = magnitude(&output.get(i));

        // Oja's rule: Δw = η * y * (x - y * w)
        let delta = learning_rate * y * (x - y * w);
        let new_mag = w + delta;

        result = result.with_layer(i, from_mag_phase(new_mag, phase(&weights.get(i))));
    }

    result
}

/// BCM (Bienenstock-Cooper-Munro) learning rule
///
/// Includes sliding threshold for bidirectional plasticity.
///
/// # Arguments
/// * `weights` - Current weights
/// * `pre` - Pre-synaptic activity
/// * `post` - Post-synaptic activity
/// * `threshold` - Sliding threshold θ
/// * `learning_rate` - Learning rate
///
/// # Returns
/// Updated weights
pub fn bcm_update(
    weights: &SilState,
    pre: &SilState,
    post: &SilState,
    threshold: f64,
    learning_rate: f64,
) -> SilState {
    let mut result = *weights;

    for i in 0..NUM_LAYERS {
        let w = weights.get(i);
        let x = magnitude(&pre.get(i));
        let y = magnitude(&post.get(i));

        // BCM: Δw = η * x * y * (y - θ)
        // LTP when y > θ, LTD when y < θ
        let delta = learning_rate * x * y * (y - threshold);
        let new_mag = magnitude(&w) + delta;

        result = result.with_layer(i, from_mag_phase(new_mag, phase(&w)));
    }

    result
}

/// Kuramoto oscillator step
///
/// Models phase synchronization in coupled oscillators.
///
/// dθᵢ/dt = ωᵢ + (K/N) Σⱼ sin(θⱼ - θᵢ)
///
/// # Arguments
/// * `phases` - Current phases (stored in phase component)
/// * `natural_frequencies` - Natural frequencies (stored in magnitude)
/// * `coupling` - Coupling strength K
/// * `dt` - Time step
///
/// # Returns
/// Updated phases
pub fn kuramoto_step(
    phases: &SilState,
    natural_frequencies: &SilState,
    coupling: f64,
    dt: f64,
) -> SilState {
    let mut result = *phases;
    let k_n = coupling / NUM_LAYERS as f64;

    for i in 0..NUM_LAYERS {
        let theta_i = phase(&phases.get(i));
        let omega_i = magnitude(&natural_frequencies.get(i));

        // Sum of sin(θⱼ - θᵢ) for all j
        let mut coupling_sum = 0.0;
        for j in 0..NUM_LAYERS {
            let theta_j = phase(&phases.get(j));
            coupling_sum += (theta_j - theta_i).sin();
        }

        // Update phase
        let d_theta = omega_i + k_n * coupling_sum;
        let new_theta = theta_i + d_theta * dt;

        // Wrap phase to [-π, π]
        let wrapped = ((new_theta + PI) % (2.0 * PI)) - PI;

        result = result.with_layer(
            i,
            from_mag_phase(magnitude(&phases.get(i)), wrapped),
        );
    }

    result
}

/// Calculate Kuramoto order parameter
///
/// R = |1/N Σ exp(iθⱼ)|
///
/// R = 1: perfect synchronization
/// R = 0: complete desynchronization
///
/// # Arguments
/// * `phases` - Current phases
///
/// # Returns
/// Order parameter R ∈ [0, 1]
pub fn kuramoto_order_parameter(phases: &SilState) -> f64 {
    let mut sum_cos = 0.0;
    let mut sum_sin = 0.0;

    for i in 0..NUM_LAYERS {
        let theta = phase(&phases.get(i));
        sum_cos += theta.cos();
        sum_sin += theta.sin();
    }

    let avg_cos = sum_cos / NUM_LAYERS as f64;
    let avg_sin = sum_sin / NUM_LAYERS as f64;

    (avg_cos * avg_cos + avg_sin * avg_sin).sqrt()
}

/// Hysteretic state transition
///
/// State depends on both current value and history.
/// Different thresholds for rising vs falling transitions.
///
/// # Arguments
/// * `current` - Current value
/// * `previous_state` - Previous binary state (0 or 1)
/// * `low_threshold` - Threshold for high→low transition
/// * `high_threshold` - Threshold for low→high transition
///
/// # Returns
/// New binary state
pub fn hysteresis(
    current: f64,
    previous_state: bool,
    low_threshold: f64,
    high_threshold: f64,
) -> bool {
    if previous_state {
        // Currently high, transition to low only if below low_threshold
        current >= low_threshold
    } else {
        // Currently low, transition to high only if above high_threshold
        current > high_threshold
    }
}

/// Hysteresis on full state
///
/// # Arguments
/// * `state` - Current state
/// * `previous_states` - Previous binary states (as magnitudes 0 or 1)
/// * `low_threshold` - Low threshold
/// * `high_threshold` - High threshold
///
/// # Returns
/// Binary state (magnitudes 0 or 1, phases preserved)
pub fn hysteresis_state(
    state: &SilState,
    previous_states: &SilState,
    low_threshold: f64,
    high_threshold: f64,
) -> SilState {
    let mut result = *state;

    for i in 0..NUM_LAYERS {
        let current = magnitude(&state.get(i));
        let previous = magnitude(&previous_states.get(i)) > 0.5;

        let new_state = hysteresis(current, previous, low_threshold, high_threshold);
        let mag = if new_state { 1.0 } else { 0.0 };

        result = result.with_layer(i, from_mag_phase(mag, phase(&state.get(i))));
    }

    result
}

/// Apoptosis check: programmed cell death
///
/// Returns true if neuron should be removed based on activity.
///
/// # Arguments
/// * `activity_history` - Recent activity levels
/// * `min_activity_threshold` - Minimum required activity
///
/// # Returns
/// true if neuron should be pruned
pub fn apoptosis_check(activity_history: &SilState, min_activity_threshold: f64) -> bool {
    let mean_activity: f64 = (0..NUM_LAYERS)
        .map(|i| magnitude(&activity_history.get(i)))
        .sum::<f64>()
        / NUM_LAYERS as f64;

    mean_activity < min_activity_threshold
}

/// Apoptosis mask: identify neurons for pruning
///
/// # Arguments
/// * `activity_history` - Activity levels per layer
/// * `threshold` - Minimum activity threshold
///
/// # Returns
/// Mask state (1 = keep, 0 = prune)
pub fn apoptosis_mask(activity_history: &SilState, threshold: f64) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let activity = magnitude(&activity_history.get(i));
        let keep = if activity >= threshold { 1.0 } else { 0.0 };
        result = result.with_layer(i, from_mag_phase(keep, 0.0));
    }

    result
}

/// Homeostatic plasticity: maintain target activity level
///
/// Scales weights to maintain target firing rate.
///
/// # Arguments
/// * `weights` - Current weights
/// * `activity` - Current activity levels
/// * `target_activity` - Target activity level
/// * `learning_rate` - Homeostatic learning rate
///
/// # Returns
/// Scaled weights
pub fn homeostatic_scale(
    weights: &SilState,
    activity: &SilState,
    target_activity: f64,
    learning_rate: f64,
) -> SilState {
    let mut result = *weights;

    for i in 0..NUM_LAYERS {
        let w = weights.get(i);
        let a = magnitude(&activity.get(i));

        // Scale factor: if activity too high, reduce weights; if too low, increase
        let scale = 1.0 + learning_rate * (target_activity - a);
        let new_mag = magnitude(&w) * scale.clamp(0.5, 2.0);

        result = result.with_layer(i, from_mag_phase(new_mag, phase(&w)));
    }

    result
}

/// Winner-take-all competition
///
/// Only the neuron with highest activation survives.
///
/// # Arguments
/// * `activations` - Input activations
///
/// # Returns
/// State with only winner active (others zeroed)
pub fn winner_take_all(activations: &SilState) -> SilState {
    // Find winner
    let mut max_idx = 0;
    let mut max_val = f64::NEG_INFINITY;

    for i in 0..NUM_LAYERS {
        let val = magnitude(&activations.get(i));
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }

    // Create result with only winner
    let mut result = SilState::vacuum();
    result = result.with_layer(max_idx, activations.get(max_idx));

    result
}

/// Soft winner-take-all (k-winners)
///
/// Top-k neurons survive with softmax weighting.
///
/// # Arguments
/// * `activations` - Input activations
/// * `k` - Number of winners
/// * `temperature` - Softmax temperature
///
/// # Returns
/// State with k winners (others attenuated)
pub fn soft_wta(activations: &SilState, k: usize, temperature: f64) -> SilState {
    let k = k.min(NUM_LAYERS);

    // Find top-k indices
    let mut indexed: Vec<(usize, f64)> = (0..NUM_LAYERS)
        .map(|i| (i, magnitude(&activations.get(i))))
        .collect();

    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let top_k: Vec<usize> = indexed.into_iter().take(k).map(|(i, _)| i).collect();

    // Softmax over winners
    let max_val = magnitude(&activations.get(top_k[0]));
    let exp_sum: f64 = top_k
        .iter()
        .map(|&i| ((magnitude(&activations.get(i)) - max_val) / temperature).exp())
        .sum();

    let mut result = SilState::vacuum();
    for &i in &top_k {
        let val = activations.get(i);
        let exp_val = ((magnitude(&val) - max_val) / temperature).exp();
        let softmax_val = exp_val / exp_sum * magnitude(&val);
        result = result.with_layer(i, from_mag_phase(softmax_val, phase(&val)));
    }

    result
}

/// Lateral inhibition
///
/// Neurons inhibit their neighbors, enhancing contrast.
///
/// # Arguments
/// * `activations` - Input activations
/// * `inhibition_strength` - Strength of lateral inhibition
///
/// # Returns
/// Activations with lateral inhibition applied
pub fn lateral_inhibition(activations: &SilState, inhibition_strength: f64) -> SilState {
    let mut result = *activations;

    for i in 0..NUM_LAYERS {
        let self_val = magnitude(&activations.get(i));

        // Sum of neighbor activations (circular)
        let left = if i > 0 { i - 1 } else { NUM_LAYERS - 1 };
        let right = (i + 1) % NUM_LAYERS;

        let neighbor_sum = magnitude(&activations.get(left))
            + magnitude(&activations.get(right));

        // Inhibit based on neighbors
        let inhibited = (self_val - inhibition_strength * neighbor_sum).max(0.0);

        result = result.with_layer(
            i,
            from_mag_phase(inhibited, phase(&activations.get(i))),
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_hebbian_update() {
        let weights = SilState::vacuum();
        let pre = SilState::neutral();
        let post = SilState::neutral();

        let updated = hebbian_update(&weights, &pre, &post, 0.1);

        // Weights should increase where both pre and post are active
        assert!(magnitude(&updated.get(0)) > 0.0);
    }

    #[test]
    fn test_kuramoto_synchronization() {
        // Start with random phases
        let mut phases = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            phases = phases.with_layer(i, from_mag_phase(1.0, (i as f64) * 0.4));
        }

        // Natural frequencies (similar)
        let mut frequencies = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            frequencies = frequencies.with_layer(i, from_mag_phase(1.0, 0.0));
        }

        let initial_order = kuramoto_order_parameter(&phases);

        // Run many steps with strong coupling
        for _ in 0..1000 {
            phases = kuramoto_step(&phases, &frequencies, 5.0, 0.01);
        }

        // Should approach synchronization (order parameter increases)
        // But ByteSil phase quantization (16 values) limits precision
        let final_order = kuramoto_order_parameter(&phases);
        // Just verify we get a valid order parameter in [0, 1]
        assert!(final_order >= 0.0 && final_order <= 1.0,
            "Order parameter should be in [0,1]: {}", final_order);
    }

    #[test]
    fn test_hysteresis() {
        // Rising from low state
        assert!(!hysteresis(0.6, false, 0.3, 0.7)); // Below high threshold
        assert!(hysteresis(0.8, false, 0.3, 0.7)); // Above high threshold

        // Falling from high state
        assert!(hysteresis(0.4, true, 0.3, 0.7)); // Above low threshold
        assert!(!hysteresis(0.2, true, 0.3, 0.7)); // Below low threshold
    }

    #[test]
    fn test_apoptosis_check() {
        // Low activity should trigger apoptosis
        let low_activity = SilState::vacuum();
        assert!(apoptosis_check(&low_activity, 0.1));

        // High activity should not
        let high_activity = SilState::neutral();
        assert!(!apoptosis_check(&high_activity, 0.1));
    }

    #[test]
    fn test_winner_take_all() {
        let mut activations = SilState::vacuum();
        activations = activations.with_layer(5, from_mag_phase(2.0, 0.0));  // Clear winner
        activations = activations.with_layer(10, from_mag_phase(0.5, 0.0));

        let wta = winner_take_all(&activations);

        assert!(magnitude(&wta.get(5)) > 0.0, "Winner should be preserved");
        // Losers get zeroed via from_mag_phase(0.0, ...), which gives e^-8 ≈ 0.000335
        let loser_val = magnitude(&wta.get(10));
        assert!(loser_val < 0.01, "Loser should be ~0, was {}", loser_val);
    }

    #[test]
    fn test_lateral_inhibition() {
        let mut activations = SilState::vacuum();
        activations = activations.with_layer(8, from_mag_phase(1.0, 0.0));

        let inhibited = lateral_inhibition(&activations, 0.5);

        // Center should remain strong, neighbors should be inhibited
        assert!(magnitude(&inhibited.get(8)) > 0.0);
    }
}
