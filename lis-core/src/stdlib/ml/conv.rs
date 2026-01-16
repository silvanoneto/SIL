//! # Convolutional Layers
//!
//! 1D and 2D convolution operations for signal and image processing.
//!
//! ## Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `conv1d` | 1D convolution |
//! | `conv2d` | 2D convolution (4x4) |
//! | `separable_conv` | Depthwise separable convolution |
//! | `max_pool` | Max pooling |
//! | `avg_pool` | Average pooling |

use sil_core::state::{ByteSil, SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};

/// 1D Convolution
///
/// Treats State as 16-element signal, convolves with kernel.
///
/// # Arguments
/// * `input` - Input signal (16 elements)
/// * `kernel` - Convolution kernel (uses first `kernel_size` elements)
/// * `kernel_size` - Size of kernel (1-16)
/// * `stride` - Stride for convolution
///
/// # Returns
/// Convolved output
pub fn conv1d(
    input: &SilState,
    kernel: &SilState,
    kernel_size: usize,
    stride: usize,
) -> SilState {
    let kernel_size = kernel_size.min(NUM_LAYERS);
    let stride = stride.max(1);

    let mut result = SilState::vacuum();
    let mut out_idx = 0;

    let mut i = 0;
    while i + kernel_size <= NUM_LAYERS && out_idx < NUM_LAYERS {
        let mut sum = 0.0;
        let mut phase_sum = 0.0;

        for k in 0..kernel_size {
            let in_val = input.get(i + k);
            let k_val = kernel.get(k);
            sum += magnitude(&in_val) * magnitude(&k_val);
            phase_sum += phase(&in_val) + phase(&k_val);
        }

        let avg_phase = phase_sum / kernel_size as f64;
        result = result.with_layer(out_idx, from_mag_phase(sum, avg_phase));

        out_idx += 1;
        i += stride;
    }

    result
}

/// 2D Convolution (4x4 input, smaller kernel)
///
/// Interprets State as 4x4 matrix.
///
/// # Arguments
/// * `input` - Input matrix (4x4 = 16 elements, row-major)
/// * `kernel` - Convolution kernel (e.g., 3x3 or 2x2)
/// * `kernel_size` - Size of square kernel
///
/// # Returns
/// Convolved output
pub fn conv2d(input: &SilState, kernel: &SilState, kernel_size: usize) -> SilState {
    let kernel_size = kernel_size.min(4);
    let output_size = 4 - kernel_size + 1;

    let mut result = SilState::vacuum();

    for out_i in 0..output_size {
        for out_j in 0..output_size {
            let mut sum = 0.0;

            for ki in 0..kernel_size {
                for kj in 0..kernel_size {
                    let in_i = out_i + ki;
                    let in_j = out_j + kj;
                    let in_idx = in_i * 4 + in_j;
                    let k_idx = ki * kernel_size + kj;

                    let in_val = magnitude(&input.get(in_idx));
                    let k_val = if k_idx < NUM_LAYERS {
                        magnitude(&kernel.get(k_idx))
                    } else {
                        0.0
                    };

                    sum += in_val * k_val;
                }
            }

            let out_idx = out_i * 4 + out_j;
            if out_idx < NUM_LAYERS {
                result = result.with_layer(out_idx, from_mag_phase(sum, 0.0));
            }
        }
    }

    result
}

/// Max pooling 2D (4x4 → 2x2)
///
/// Takes maximum in each 2x2 region.
pub fn max_pool_2x2(input: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for pool_i in 0..2 {
        for pool_j in 0..2 {
            let mut max_val = f64::NEG_INFINITY;
            let mut max_phase = 0.0;

            for di in 0..2 {
                for dj in 0..2 {
                    let in_i = pool_i * 2 + di;
                    let in_j = pool_j * 2 + dj;
                    let in_idx = in_i * 4 + in_j;

                    let val = input.get(in_idx);
                    let val_mag = magnitude(&val);
                    if val_mag > max_val {
                        max_val = val_mag;
                        max_phase = phase(&val);
                    }
                }
            }

            let out_idx = pool_i * 2 + pool_j;
            result = result.with_layer(out_idx, from_mag_phase(max_val, max_phase));
        }
    }

    result
}

/// Average pooling 2D (4x4 → 2x2)
pub fn avg_pool_2x2(input: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for pool_i in 0..2 {
        for pool_j in 0..2 {
            let mut sum = 0.0;
            let mut phase_sum = 0.0;

            for di in 0..2 {
                for dj in 0..2 {
                    let in_i = pool_i * 2 + di;
                    let in_j = pool_j * 2 + dj;
                    let in_idx = in_i * 4 + in_j;

                    let val = input.get(in_idx);
                    sum += magnitude(&val);
                    phase_sum += phase(&val);
                }
            }

            let out_idx = pool_i * 2 + pool_j;
            let avg = sum / 4.0;
            let avg_phase = phase_sum / 4.0;
            result = result.with_layer(out_idx, from_mag_phase(avg, avg_phase));
        }
    }

    result
}

/// Global average pooling
///
/// Reduces all 16 elements to single value.
pub fn global_avg_pool(input: &SilState) -> ByteSil {
    let mut sum = 0.0;
    let mut phase_sum = 0.0;

    for i in 0..NUM_LAYERS {
        let val = input.get(i);
        sum += magnitude(&val);
        phase_sum += phase(&val);
    }

    from_mag_phase(sum / NUM_LAYERS as f64, phase_sum / NUM_LAYERS as f64)
}

/// Global max pooling
pub fn global_max_pool(input: &SilState) -> ByteSil {
    let mut max_val = f64::NEG_INFINITY;
    let mut max_phase = 0.0;

    for i in 0..NUM_LAYERS {
        let val = input.get(i);
        let val_mag = magnitude(&val);
        if val_mag > max_val {
            max_val = val_mag;
            max_phase = phase(&val);
        }
    }

    from_mag_phase(max_val, max_phase)
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_conv1d() {
        let input = SilState::neutral();
        let mut kernel = SilState::vacuum();
        kernel = kernel.with_layer(0, from_mag_phase(1.0, 0.0));
        kernel = kernel.with_layer(1, from_mag_phase(1.0, 0.0));
        kernel = kernel.with_layer(2, from_mag_phase(1.0, 0.0));

        let output = conv1d(&input, &kernel, 3, 1);

        // Should produce valid output
        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&output.get(i)).abs())
            .sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_max_pool() {
        let input = SilState::neutral();
        let output = max_pool_2x2(&input);

        // Output should have values in first 4 positions
        let has_values = (0..4)
            .any(|i| magnitude(&output.get(i)).abs() > 1e-10);
        assert!(has_values);
    }

    #[test]
    fn test_global_avg_pool() {
        let input = SilState::neutral();
        let output = global_avg_pool(&input);

        // Should be a single averaged value
        assert!(magnitude(&output).is_finite());
    }
}
