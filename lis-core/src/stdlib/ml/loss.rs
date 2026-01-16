//! # Loss Functions
//!
//! Loss functions for training neural networks.
//!
//! ## Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `mse` | Mean Squared Error |
//! | `mae` | Mean Absolute Error |
//! | `cross_entropy` | Cross-entropy loss |
//! | `binary_cross_entropy` | Binary cross-entropy |
//! | `kl_divergence` | Kullback-Leibler divergence |
//! | `huber_loss` | Huber loss (smooth L1) |
//! | `cosine_loss` | Cosine similarity loss |
//!
//! ## Implementation Notes
//!
//! All loss functions operate on SilState (16 layers).
//! Losses use ByteSil magnitude for computation.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase};

/// Mean Squared Error: (1/n) Σ (y - ŷ)²
///
/// # Arguments
/// * `pred` - Predicted SilState
/// * `target` - Target SilState
///
/// # Returns
/// MSE loss as f64
pub fn mse(pred: &SilState, target: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i));
        let t = magnitude(&target.get(i));
        let diff = p - t;
        sum += diff * diff;
    }
    sum / NUM_LAYERS as f64
}

/// Mean Squared Error on magnitude and phase separately
///
/// Returns (magnitude_mse, phase_mse)
pub fn mse_complex(pred: &SilState, target: &SilState) -> (f64, f64) {
    let mut mag_sum = 0.0;
    let mut phase_sum = 0.0;

    for i in 0..NUM_LAYERS {
        let p = pred.get(i);
        let t = target.get(i);

        let mag_diff = magnitude(&p) - magnitude(&t);
        mag_sum += mag_diff * mag_diff;

        // Phase difference (handle wrap-around)
        let phase_diff = (phase(&p) - phase(&t)).abs();
        let phase_diff = if phase_diff > std::f64::consts::PI {
            2.0 * std::f64::consts::PI - phase_diff
        } else {
            phase_diff
        };
        phase_sum += phase_diff * phase_diff;
    }

    (mag_sum / NUM_LAYERS as f64, phase_sum / NUM_LAYERS as f64)
}

/// Mean Absolute Error: (1/n) Σ |y - ŷ|
///
/// # Arguments
/// * `pred` - Predicted SilState
/// * `target` - Target SilState
///
/// # Returns
/// MAE loss as f64
pub fn mae(pred: &SilState, target: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i));
        let t = magnitude(&target.get(i));
        sum += (p - t).abs();
    }
    sum / NUM_LAYERS as f64
}

/// Cross-Entropy Loss: -Σ target * log(pred)
///
/// Assumes pred is output of softmax (probabilities).
/// Small epsilon added for numerical stability.
///
/// # Arguments
/// * `pred` - Predicted probabilities (after softmax)
/// * `target` - One-hot encoded target
///
/// # Returns
/// Cross-entropy loss as f64
pub fn cross_entropy(pred: &SilState, target: &SilState) -> f64 {
    const EPSILON: f64 = 1e-10;

    let mut loss = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i)).max(EPSILON);
        let t = magnitude(&target.get(i));
        loss -= t * p.ln();
    }
    loss
}

/// Binary Cross-Entropy: -[y*log(p) + (1-y)*log(1-p)]
///
/// For binary classification with sigmoid output.
///
/// # Arguments
/// * `pred` - Predicted probability (single layer or aggregated)
/// * `target` - Binary target (0 or 1)
///
/// # Returns
/// Binary cross-entropy loss as f64
pub fn binary_cross_entropy(pred: f64, target: f64) -> f64 {
    const EPSILON: f64 = 1e-10;

    let p = pred.clamp(EPSILON, 1.0 - EPSILON);
    -(target * p.ln() + (1.0 - target) * (1.0 - p).ln())
}

/// Binary Cross-Entropy on State (averages over layers)
pub fn binary_cross_entropy_state(pred: &SilState, target: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i));
        let t = magnitude(&target.get(i));
        sum += binary_cross_entropy(p, t);
    }
    sum / NUM_LAYERS as f64
}

/// Kullback-Leibler Divergence: Σ P * log(P/Q)
///
/// Measures how P differs from Q.
///
/// # Arguments
/// * `p` - True distribution
/// * `q` - Approximating distribution
///
/// # Returns
/// KL divergence as f64
pub fn kl_divergence(p: &SilState, q: &SilState) -> f64 {
    const EPSILON: f64 = 1e-10;

    let mut kl = 0.0;
    for i in 0..NUM_LAYERS {
        let p_val = magnitude(&p.get(i)).max(EPSILON);
        let q_val = magnitude(&q.get(i)).max(EPSILON);
        kl += p_val * (p_val / q_val).ln();
    }
    kl
}

/// Huber Loss (Smooth L1): δ²/2 * (√(1 + (x/δ)²) - 1)
///
/// Combines MSE for small errors and MAE for large errors.
/// Less sensitive to outliers than MSE.
///
/// # Arguments
/// * `pred` - Predicted SilState
/// * `target` - Target SilState
/// * `delta` - Threshold for switching between L1/L2 (default 1.0)
///
/// # Returns
/// Huber loss as f64
pub fn huber_loss(pred: &SilState, target: &SilState, delta: f64) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i));
        let t = magnitude(&target.get(i));
        let diff = (p - t).abs();

        let loss = if diff <= delta {
            0.5 * diff * diff
        } else {
            delta * (diff - 0.5 * delta)
        };
        sum += loss;
    }
    sum / NUM_LAYERS as f64
}

/// Smooth L1 Loss (same as Huber with delta=1)
pub fn smooth_l1_loss(pred: &SilState, target: &SilState) -> f64 {
    huber_loss(pred, target, 1.0)
}

/// Cosine Similarity Loss: 1 - cos_sim(pred, target)
///
/// Measures angular distance between vectors.
///
/// # Arguments
/// * `pred` - Predicted SilState
/// * `target` - Target SilState
///
/// # Returns
/// Cosine loss (0 = identical direction, 2 = opposite)
pub fn cosine_loss(pred: &SilState, target: &SilState) -> f64 {
    let mut dot = 0.0;
    let mut pred_norm = 0.0;
    let mut target_norm = 0.0;

    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i));
        let t = magnitude(&target.get(i));

        dot += p * t;
        pred_norm += p * p;
        target_norm += t * t;
    }

    let norm_product = (pred_norm * target_norm).sqrt();
    if norm_product < 1e-10 {
        return 1.0; // Undefined, return max loss
    }

    let cos_sim = dot / norm_product;
    1.0 - cos_sim
}

/// Hinge Loss: max(0, 1 - y * ŷ)
///
/// Used for SVM-style margin-based classification.
///
/// # Arguments
/// * `pred` - Predicted score
/// * `target` - Target (-1 or +1)
///
/// # Returns
/// Hinge loss as f64
pub fn hinge_loss(pred: f64, target: f64) -> f64 {
    (1.0 - target * pred).max(0.0)
}

/// Hinge Loss on State
pub fn hinge_loss_state(pred: &SilState, target: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&pred.get(i));
        let t = magnitude(&target.get(i));
        sum += hinge_loss(p, t);
    }
    sum / NUM_LAYERS as f64
}

/// Focal Loss: -α * (1 - p)^γ * log(p)
///
/// Focuses training on hard examples.
/// Used in object detection (RetinaNet).
///
/// # Arguments
/// * `pred` - Predicted probability
/// * `target` - Binary target
/// * `gamma` - Focusing parameter (default 2.0)
/// * `alpha` - Class weight (default 0.25)
///
/// # Returns
/// Focal loss as f64
pub fn focal_loss(pred: f64, target: f64, gamma: f64, alpha: f64) -> f64 {
    const EPSILON: f64 = 1e-10;

    let p = pred.clamp(EPSILON, 1.0 - EPSILON);

    if target > 0.5 {
        -alpha * (1.0 - p).powf(gamma) * p.ln()
    } else {
        -(1.0 - alpha) * p.powf(gamma) * (1.0 - p).ln()
    }
}

/// Triplet Loss: max(0, d(a,p) - d(a,n) + margin)
///
/// For metric learning / embeddings.
///
/// # Arguments
/// * `anchor` - Anchor embedding
/// * `positive` - Positive example (same class)
/// * `negative` - Negative example (different class)
/// * `margin` - Margin between positive and negative (default 1.0)
///
/// # Returns
/// Triplet loss as f64
pub fn triplet_loss(
    anchor: &SilState,
    positive: &SilState,
    negative: &SilState,
    margin: f64,
) -> f64 {
    let d_pos = mse(anchor, positive).sqrt();
    let d_neg = mse(anchor, negative).sqrt();
    (d_pos - d_neg + margin).max(0.0)
}

/// Combined loss with weights
///
/// # Arguments
/// * `pred` - Predicted SilState
/// * `target` - Target SilState
/// * `mse_weight` - Weight for MSE component
/// * `cosine_weight` - Weight for cosine component
pub fn combined_loss(
    pred: &SilState,
    target: &SilState,
    mse_weight: f64,
    cosine_weight: f64,
) -> f64 {
    let mse = mse(pred, target);
    let cosine = cosine_loss(pred, target);
    mse_weight * mse + cosine_weight * cosine
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_mse_identical() {
        let state = SilState::neutral();
        let mse = mse(&state, &state);
        assert!(mse.abs() < 1e-10);
    }

    #[test]
    fn test_mae_identical() {
        let state = SilState::neutral();
        let mae = mae(&state, &state);
        assert!(mae.abs() < 1e-10);
    }

    #[test]
    fn test_cross_entropy_perfect() {
        // One-hot target
        let mut target = SilState::vacuum();
        target = target.with_layer(0, from_mag_phase(1.0, 0.0));

        // Perfect prediction
        let mut pred = SilState::vacuum();
        pred = pred.with_layer(0, from_mag_phase(1.0, 0.0));

        let ce = cross_entropy(&pred, &target);
        // Cross entropy: -target * log(pred) = -1.0 * log(1.0) = 0
        // But ByteSil quantization means pred mag may not be exactly 1.0
        assert!(ce.abs() < 1.0, "Cross entropy was {}", ce);
    }

    #[test]
    fn test_huber_loss() {
        let state = SilState::neutral();
        let huber = huber_loss(&state, &state, 1.0);
        assert!(huber.abs() < 1e-10);
    }

    #[test]
    fn test_cosine_loss_identical() {
        let state = SilState::neutral();
        let cosine = cosine_loss(&state, &state);
        assert!(cosine.abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_identical() {
        // Uniform distribution
        let mut state = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            state = state.with_layer(i, from_mag_phase(1.0 / NUM_LAYERS as f64, 0.0));
        }

        let kl = kl_divergence(&state, &state);
        assert!(kl.abs() < 1e-10);
    }

    #[test]
    fn test_binary_cross_entropy() {
        let bce_correct = binary_cross_entropy(0.9, 1.0);
        let bce_wrong = binary_cross_entropy(0.1, 1.0);

        assert!(bce_correct < bce_wrong);
    }
}
