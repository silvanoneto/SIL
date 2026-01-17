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
use super::tensor::{magnitude, phase};

/// Mean Squared Error: (1/n) Σ (y - ŷ)²
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

/// Huber Loss (Smooth L1)
///
/// Combines MSE for small errors and MAE for large errors.
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
        return 1.0;
    }

    let cos_sim = dot / norm_product;
    1.0 - cos_sim
}

/// Hinge Loss: max(0, 1 - y * ŷ)
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
pub fn combined_loss(
    pred: &SilState,
    target: &SilState,
    mse_weight: f64,
    cosine_weight: f64,
) -> f64 {
    let mse_loss = mse(pred, target);
    let cosine = cosine_loss(pred, target);
    mse_weight * mse_loss + cosine_weight * cosine
}

/// Contrastive loss for self-supervised learning
pub fn contrastive_loss(anchor: &SilState, positive: &SilState, negatives: &[SilState], temperature: f64) -> f64 {
    const EPSILON: f64 = 1e-10;
    
    let pos_sim = super::linalg::dot(anchor, positive) / temperature;
    
    let mut neg_sum = 0.0;
    for neg in negatives {
        let neg_sim = super::linalg::dot(anchor, neg) / temperature;
        neg_sum += neg_sim.exp();
    }
    
    -pos_sim + (pos_sim.exp() + neg_sum).ln().max(EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tensor::from_mag_phase;

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
        let mut target = SilState::vacuum();
        target = target.with_layer(0, from_mag_phase(1.0, 0.0));

        let mut pred = SilState::vacuum();
        pred = pred.with_layer(0, from_mag_phase(1.0, 0.0));

        let ce = cross_entropy(&pred, &target);
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
}
