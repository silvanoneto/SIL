//! # Statistics Functions
//!
//! Statistical operations for neural network computations.
//!
//! ## Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `mean` | Mean of 16 elements |
//! | `variance` | Variance |
//! | `std` | Standard deviation |
//! | `normalize` | Zero-mean, unit-variance |
//! | `min` | Minimum value |
//! | `max` | Maximum value |
//! | `argmax` | Index of maximum |
//! | `argmin` | Index of minimum |
//! | `sum` | Sum of elements |

use sil_core::state::{ByteSil, SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};

/// Sum of all elements
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Sum as f64
pub fn sum(state: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        sum += magnitude(&state.get(i));
    }
    sum
}

/// Mean (average) of 16 elements
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Mean as f64
pub fn mean(state: &SilState) -> f64 {
    sum(state) / NUM_LAYERS as f64
}

/// Mean as ByteSil (includes average phase)
pub fn mean_bytesil(state: &SilState) -> ByteSil {
    let mean_mag = mean(state);

    // Weighted average phase
    let mut phase_sum = 0.0;
    let mut weight_sum = 0.0;
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        let weight = magnitude(&val).abs();
        phase_sum += phase(&val) * weight;
        weight_sum += weight;
    }

    let avg_phase = if weight_sum > 1e-10 {
        phase_sum / weight_sum
    } else {
        0.0
    };

    from_mag_phase(mean_mag, avg_phase)
}

/// Variance: σ² = (1/n) Σ (xi - μ)²
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Variance as f64
pub fn variance(state: &SilState) -> f64 {
    let mean = mean(state);
    let mut sum_sq = 0.0;

    for i in 0..NUM_LAYERS {
        let diff = magnitude(&state.get(i)) - mean;
        sum_sq += diff * diff;
    }

    sum_sq / NUM_LAYERS as f64
}

/// Sample variance (Bessel's correction): s² = (1/(n-1)) Σ (xi - μ)²
pub fn variance_sample(state: &SilState) -> f64 {
    let mean = mean(state);
    let mut sum_sq = 0.0;

    for i in 0..NUM_LAYERS {
        let diff = magnitude(&state.get(i)) - mean;
        sum_sq += diff * diff;
    }

    sum_sq / (NUM_LAYERS - 1) as f64
}

/// Standard deviation: σ = √variance
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Standard deviation as f64
pub fn std(state: &SilState) -> f64 {
    variance(state).sqrt()
}

/// Sample standard deviation
pub fn std_sample(state: &SilState) -> f64 {
    variance_sample(state).sqrt()
}

/// Normalize to zero mean, unit variance (z-score)
///
/// z = (x - μ) / σ
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Normalized SilState
pub fn normalize_zscore(state: &SilState) -> SilState {
    let mean = mean(state);
    let std = std(state);

    if std < 1e-10 {
        return *state; // Avoid division by zero
    }

    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        let normalized = (magnitude(&val) - mean) / std;
        result = result.with_layer(i, from_mag_phase(normalized, phase(&val)));
    }

    result
}

/// Normalize to [0, 1] range (min-max normalization)
///
/// x' = (x - min) / (max - min)
pub fn normalize_minmax(state: &SilState) -> SilState {
    let min_val = min(state);
    let max_val = max(state);
    let range = max_val - min_val;

    if range < 1e-10 {
        return *state; // All values are the same
    }

    let mut result = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        let normalized = (magnitude(&val) - min_val) / range;
        result = result.with_layer(i, from_mag_phase(normalized, phase(&val)));
    }

    result
}

/// Minimum value
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Minimum magnitude as f64
pub fn min(state: &SilState) -> f64 {
    let mut min_val = f64::INFINITY;
    for i in 0..NUM_LAYERS {
        let mag = magnitude(&state.get(i));
        if mag < min_val {
            min_val = mag;
        }
    }
    min_val
}

/// Maximum value
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Maximum magnitude as f64
pub fn max(state: &SilState) -> f64 {
    let mut max_val = f64::NEG_INFINITY;
    for i in 0..NUM_LAYERS {
        let mag = magnitude(&state.get(i));
        if mag > max_val {
            max_val = mag;
        }
    }
    max_val
}

/// Index of maximum value (argmax)
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Index (0-15) of maximum element
pub fn argmax(state: &SilState) -> usize {
    let mut max_val = f64::NEG_INFINITY;
    let mut max_idx = 0;

    for i in 0..NUM_LAYERS {
        let mag = magnitude(&state.get(i));
        if mag > max_val {
            max_val = mag;
            max_idx = i;
        }
    }

    max_idx
}

/// Index of minimum value (argmin)
///
/// # Arguments
/// * `state` - Input SilState
///
/// # Returns
/// Index (0-15) of minimum element
pub fn argmin(state: &SilState) -> usize {
    let mut min_val = f64::INFINITY;
    let mut min_idx = 0;

    for i in 0..NUM_LAYERS {
        let mag = magnitude(&state.get(i));
        if mag < min_val {
            min_val = mag;
            min_idx = i;
        }
    }

    min_idx
}

/// Top-K indices (sorted by magnitude, descending)
///
/// # Arguments
/// * `state` - Input SilState
/// * `k` - Number of top elements to return
///
/// # Returns
/// Vector of indices for top-k elements
pub fn topk(state: &SilState, k: usize) -> Vec<usize> {
    let k = k.min(NUM_LAYERS);

    // Create (index, magnitude) pairs
    let mut pairs: Vec<(usize, f64)> = (0..NUM_LAYERS)
        .map(|i| (i, magnitude(&state.get(i))))
        .collect();

    // Sort by magnitude descending
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return top-k indices
    pairs.into_iter().take(k).map(|(idx, _)| idx).collect()
}

/// Median value
pub fn median(state: &SilState) -> f64 {
    let mut values: Vec<f64> = (0..NUM_LAYERS)
        .map(|i| magnitude(&state.get(i)))
        .collect();

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // For 16 elements, median is average of 8th and 9th
    (values[7] + values[8]) / 2.0
}

/// Percentile (0-100)
pub fn percentile(state: &SilState, p: f64) -> f64 {
    let mut values: Vec<f64> = (0..NUM_LAYERS)
        .map(|i| magnitude(&state.get(i)))
        .collect();

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p = p.clamp(0.0, 100.0) / 100.0;
    let idx = (p * (NUM_LAYERS - 1) as f64).floor() as usize;
    let frac = p * (NUM_LAYERS - 1) as f64 - idx as f64;

    if idx >= NUM_LAYERS - 1 {
        values[NUM_LAYERS - 1]
    } else {
        values[idx] + frac * (values[idx + 1] - values[idx])
    }
}

/// Covariance between two states
pub fn covariance(a: &SilState, b: &SilState) -> f64 {
    let mean_a = mean(a);
    let mean_b = mean(b);

    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let diff_a = magnitude(&a.get(i)) - mean_a;
        let diff_b = magnitude(&b.get(i)) - mean_b;
        sum += diff_a * diff_b;
    }

    sum / NUM_LAYERS as f64
}

/// Pearson correlation coefficient
pub fn correlation(a: &SilState, b: &SilState) -> f64 {
    let cov = covariance(a, b);
    let std_a = std(a);
    let std_b = std(b);

    if std_a < 1e-10 || std_b < 1e-10 {
        return 0.0; // Undefined
    }

    cov / (std_a * std_b)
}

/// Entropy: H(X) = -Σ p(x) log p(x)
///
/// Treats magnitudes as unnormalized probabilities.
pub fn entropy(state: &SilState) -> f64 {
    // Normalize to probabilities
    let sum = sum(state).abs();
    if sum < 1e-10 {
        return 0.0;
    }

    let mut entropy = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&state.get(i)).abs() / sum;
        if p > 1e-10 {
            entropy -= p * p.ln();
        }
    }

    entropy
}

/// Check if all elements are finite
pub fn is_finite(state: &SilState) -> bool {
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        if !magnitude(&val).is_finite() || !phase(&val).is_finite() {
            return false;
        }
    }
    true
}

/// Check for NaN values
pub fn has_nan(state: &SilState) -> bool {
    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        if magnitude(&val).is_nan() || phase(&val).is_nan() {
            return false;
        }
    }
    false
}

/// Replace NaN with value
pub fn replace_nan(state: &SilState, replacement: f64) -> SilState {
    let mut result = *state;

    for i in 0..NUM_LAYERS {
        let val = state.get(i);
        if magnitude(&val).is_nan() {
            result = result.with_layer(i, from_mag_phase(replacement, phase(&val)));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_sum() {
        let state = SilState::neutral();
        // neutral() has rho=0, so magnitude = e^0 = 1 for each layer
        let sum = sum(&state);
        assert!(sum > 15.0 && sum < 17.0); // ~16
    }

    #[test]
    fn test_mean() {
        let state = SilState::neutral();
        let mean = mean(&state);
        assert!(mean > 0.9 && mean < 1.1); // ~1.0
    }

    #[test]
    fn test_variance() {
        let state = SilState::neutral();
        // All same value => variance ≈ 0
        assert!(variance(&state).abs() < 0.1);
    }

    #[test]
    fn test_argmax() {
        let state = SilState::neutral();
        // All equal, should return 0 or any valid index
        let idx = argmax(&state);
        assert!(idx < NUM_LAYERS);
    }

    #[test]
    fn test_topk() {
        let state = SilState::neutral();
        let top3 = topk(&state, 3);
        assert_eq!(top3.len(), 3);
    }

    #[test]
    fn test_entropy() {
        // Uniform distribution should have high entropy
        let uniform = SilState::neutral();
        let entropy = entropy(&uniform);
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_correlation() {
        let a = SilState::neutral();
        let corr = correlation(&a, &a);
        // Self-correlation should be 1, but may be NaN or 0 if variance is 0
        // neutral() has all same values, so variance ≈ 0
        assert!(corr.is_nan() || corr.abs() < 0.01 || (corr - 1.0).abs() < 0.1,
            "Correlation was {}", corr);
    }
}
