//! # Statistics Functions
//!
//! Statistical operations for neural network computations.

use sil_core::state::{ByteSil, SilState, NUM_LAYERS};
use super::tensor::{magnitude, phase, from_mag_phase};

/// Sum of all elements
pub fn sum(state: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        sum += magnitude(&state.get(i));
    }
    sum
}

/// Mean (average) of 16 elements
pub fn mean(state: &SilState) -> f64 {
    sum(state) / NUM_LAYERS as f64
}

/// Mean as ByteSil (includes average phase)
pub fn mean_bytesil(state: &SilState) -> ByteSil {
    let mean_mag = mean(state);

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
pub fn variance(state: &SilState) -> f64 {
    let mean = mean(state);
    let mut sum_sq = 0.0;

    for i in 0..NUM_LAYERS {
        let diff = magnitude(&state.get(i)) - mean;
        sum_sq += diff * diff;
    }

    sum_sq / NUM_LAYERS as f64
}

/// Sample variance (Bessel's correction)
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
pub fn std(state: &SilState) -> f64 {
    variance(state).sqrt()
}

/// Sample standard deviation
pub fn std_sample(state: &SilState) -> f64 {
    variance_sample(state).sqrt()
}

/// Normalize to zero mean, unit variance (z-score)
pub fn normalize_zscore(state: &SilState) -> SilState {
    let mean = mean(state);
    let std = std(state);

    if std < 1e-10 {
        return *state;
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
pub fn normalize_minmax(state: &SilState) -> SilState {
    let min_val = min(state);
    let max_val = max(state);
    let range = max_val - min_val;

    if range < 1e-10 {
        return *state;
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
pub fn topk(state: &SilState, k: usize) -> Vec<usize> {
    let k = k.min(NUM_LAYERS);

    let mut pairs: Vec<(usize, f64)> = (0..NUM_LAYERS)
        .map(|i| (i, magnitude(&state.get(i))))
        .collect();

    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    pairs.into_iter().take(k).map(|(idx, _)| idx).collect()
}

/// Median value
pub fn median(state: &SilState) -> f64 {
    let mut values: Vec<f64> = (0..NUM_LAYERS)
        .map(|i| magnitude(&state.get(i)))
        .collect();

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

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
        return 0.0;
    }

    cov / (std_a * std_b)
}

/// Entropy: H(X) = -Σ p(x) log p(x)
pub fn entropy(state: &SilState) -> f64 {
    let total = sum(state).abs();
    if total < 1e-10 {
        return 0.0;
    }

    let mut entropy = 0.0;
    for i in 0..NUM_LAYERS {
        let p = magnitude(&state.get(i)).abs() / total;
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
            return true;
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

/// Exponential moving average
pub fn ema(current: &SilState, previous: &SilState, alpha: f64) -> SilState {
    let mut result = SilState::vacuum();
    
    for i in 0..NUM_LAYERS {
        let curr = magnitude(&current.get(i));
        let prev = magnitude(&previous.get(i));
        let smoothed = alpha * curr + (1.0 - alpha) * prev;
        result = result.with_layer(i, from_mag_phase(smoothed, phase(&current.get(i))));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let state = SilState::neutral();
        let sum = sum(&state);
        assert!(sum > 15.0 && sum < 17.0);
    }

    #[test]
    fn test_mean() {
        let state = SilState::neutral();
        let mean = mean(&state);
        assert!(mean > 0.9 && mean < 1.1);
    }

    #[test]
    fn test_variance() {
        let state = SilState::neutral();
        assert!(variance(&state).abs() < 0.1);
    }

    #[test]
    fn test_argmax() {
        let state = SilState::neutral();
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
        let uniform = SilState::neutral();
        let entropy = entropy(&uniform);
        assert!(entropy > 0.0);
    }
}
