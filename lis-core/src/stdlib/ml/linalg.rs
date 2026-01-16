//! # Linear Algebra Operations
//!
//! Matrix and vector operations for neural network computations.
//!
//! ## State as Matrix
//!
//! SilState can be interpreted as:
//! - **Vector**: 16-element vector (activations, embeddings)
//! - **Matrix**: 4x4 matrix (weights for 4→4 transformations)
//!
//! ## Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `matmul_4x4` | 4x4 matrix multiplication |
//! | `dot` | Dot product (16 elements) |
//! | `outer` | Outer product |
//! | `transpose_4x4` | Transpose 4x4 matrix |
//! | `hadamard` | Element-wise multiply |
//! | `scale_state` | Scalar multiplication |
//!
//! ## Implementation Notes
//!
//! Operations use ByteSil magnitude for real-valued computations.
//! Complex operations preserve phase relationships.

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};

/// Matrix multiplication: C = A × B (4x4 matrices)
///
/// Interprets two States as 4x4 matrices (row-major).
///
/// # Arguments
/// * `a` - First matrix (4x4)
/// * `b` - Second matrix (4x4)
///
/// # Returns
/// Product matrix as SilState
pub fn matmul_4x4(a: &SilState, b: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    // Matrix multiply: C[i,j] = Σ A[i,k] * B[k,j]
    for i in 0..4 {
        for j in 0..4 {
            let mut sum_mag = 0.0;
            let mut sum_phase = 0.0;

            for k in 0..4 {
                let a_idx = i * 4 + k;
                let b_idx = k * 4 + j;

                let a_val = a.get(a_idx);
                let b_val = b.get(b_idx);

                let a_mag = magnitude(&a_val);
                let b_mag = magnitude(&b_val);

                // Magnitude multiplication
                sum_mag += a_mag * b_mag;

                // Phase accumulation (weighted by magnitude product)
                let weight = a_mag.abs() * b_mag.abs();
                sum_phase += (phase(&a_val) + phase(&b_val)) * weight;
            }

            let c_idx = i * 4 + j;
            // Normalize phase by total weight (avoid division by zero)
            let result_phase = if sum_mag.abs() > 1e-10 {
                sum_phase / sum_mag.abs()
            } else {
                0.0
            };
            result = result.with_layer(c_idx, from_mag_phase(sum_mag, result_phase));
        }
    }

    result
}

/// Matrix-vector multiplication: y = W × x
///
/// W is 4x4 matrix, x is first 4 elements of state.
///
/// # Arguments
/// * `w` - Weight matrix (4x4)
/// * `x` - Input vector (first 4 elements used)
///
/// # Returns
/// Output vector in first 4 elements, rest zeroed
pub fn matvec_4x4(w: &SilState, x: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        let mut sum = 0.0;
        for j in 0..4 {
            let w_idx = i * 4 + j;
            let w_val = magnitude(&w.get(w_idx));
            let x_val = magnitude(&x.get(j));
            sum += w_val * x_val;
        }
        result = result.with_layer(i, from_mag_phase(sum, 0.0));
    }

    result
}

/// Dot product: a · b = Σ ai × bi
///
/// # Arguments
/// * `a` - First State (16 elements)
/// * `b` - Second State (16 elements)
///
/// # Returns
/// Scalar dot product as f64
pub fn dot(a: &SilState, b: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let a_val = magnitude(&a.get(i));
        let b_val = magnitude(&b.get(i));
        sum += a_val * b_val;
    }
    sum
}

/// Dot product as ByteSil (preserves phase information)
pub fn dot_bytesil(a: &SilState, b: &SilState) -> sil_core::state::ByteSil {
    let dot = dot(a, b);

    // Compute average phase
    let mut phase_sum = 0.0;
    let mut weight_sum = 0.0;
    for i in 0..NUM_LAYERS {
        let a_val = a.get(i);
        let b_val = b.get(i);
        let a_mag = magnitude(&a_val);
        let b_mag = magnitude(&b_val);
        let weight = a_mag.abs() * b_mag.abs();
        phase_sum += (phase(&a_val) + phase(&b_val)) * weight;
        weight_sum += weight;
    }

    let result_phase = if weight_sum > 1e-10 {
        phase_sum / weight_sum
    } else {
        0.0
    };

    from_mag_phase(dot, result_phase)
}

/// Outer product: C = a ⊗ b
///
/// Creates 4x4 matrix from two 4-element vectors.
///
/// # Arguments
/// * `a` - First vector (first 4 elements)
/// * `b` - Second vector (first 4 elements)
///
/// # Returns
/// Outer product matrix (4x4) as SilState
pub fn outer_4x4(a: &SilState, b: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        for j in 0..4 {
            let a_val = a.get(i);
            let b_val = b.get(j);

            let mag = magnitude(&a_val) * magnitude(&b_val);
            let result_phase = phase(&a_val) + phase(&b_val);

            let idx = i * 4 + j;
            result = result.with_layer(idx, from_mag_phase(mag, result_phase));
        }
    }

    result
}

/// Transpose: A^T
///
/// Transposes a 4x4 matrix.
///
/// # Arguments
/// * `a` - Matrix to transpose (4x4)
///
/// # Returns
/// Transposed matrix as SilState
pub fn transpose_4x4(a: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        for j in 0..4 {
            let src_idx = i * 4 + j;
            let dst_idx = j * 4 + i;
            result = result.with_layer(dst_idx, a.get(src_idx));
        }
    }

    result
}

/// Hadamard (element-wise) product: C = A ⊙ B
///
/// # Arguments
/// * `a` - First State
/// * `b` - Second State
///
/// # Returns
/// Element-wise product as SilState
pub fn hadamard(a: &SilState, b: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let a_val = a.get(i);
        let b_val = b.get(i);

        let mag = magnitude(&a_val) * magnitude(&b_val);
        let result_phase = phase(&a_val) + phase(&b_val);

        result = result.with_layer(i, from_mag_phase(mag, result_phase));
    }

    result
}

/// Element-wise addition: C = A + B
pub fn add(a: &SilState, b: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let a_val = a.get(i);
        let b_val = b.get(i);

        let a_mag = magnitude(&a_val);
        let b_mag = magnitude(&b_val);
        let a_phase = phase(&a_val);
        let b_phase = phase(&b_val);

        // Add as complex numbers
        let a_re = a_mag * a_phase.cos();
        let a_im = a_mag * a_phase.sin();
        let b_re = b_mag * b_phase.cos();
        let b_im = b_mag * b_phase.sin();

        let c_re = a_re + b_re;
        let c_im = a_im + b_im;

        let mag = (c_re * c_re + c_im * c_im).sqrt();
        let result_phase = c_im.atan2(c_re);

        result = result.with_layer(i, from_mag_phase(mag, result_phase));
    }

    result
}

/// Element-wise subtraction: C = A - B
pub fn sub(a: &SilState, b: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let a_val = a.get(i);
        let b_val = b.get(i);

        let a_mag = magnitude(&a_val);
        let b_mag = magnitude(&b_val);
        let a_phase = phase(&a_val);
        let b_phase = phase(&b_val);

        let a_re = a_mag * a_phase.cos();
        let a_im = a_mag * a_phase.sin();
        let b_re = b_mag * b_phase.cos();
        let b_im = b_mag * b_phase.sin();

        let c_re = a_re - b_re;
        let c_im = a_im - b_im;

        let mag = (c_re * c_re + c_im * c_im).sqrt();
        let result_phase = c_im.atan2(c_re);

        result = result.with_layer(i, from_mag_phase(mag, result_phase));
    }

    result
}

/// Scalar multiplication: C = α × A
///
/// # Arguments
/// * `a` - State to scale
/// * `scalar` - Scalar multiplier
///
/// # Returns
/// Scaled State
pub fn scale_state(a: &SilState, scalar: f64) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let val = a.get(i);
        let new_mag = magnitude(&val) * scalar;
        result = result.with_layer(i, from_mag_phase(new_mag, phase(&val)));
    }

    result
}

/// L2 Norm: ||x||₂ = √(Σ xi²)
///
/// # Arguments
/// * `a` - State vector
///
/// # Returns
/// L2 norm as f64
pub fn norm_l2(a: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        let mag = magnitude(&a.get(i));
        sum += mag * mag;
    }
    sum.sqrt()
}

/// L1 Norm: ||x||₁ = Σ |xi|
pub fn norm_l1(a: &SilState) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_LAYERS {
        sum += magnitude(&a.get(i)).abs();
    }
    sum
}

/// Frobenius Norm (for matrices): ||A||_F = √(Σ aij²)
/// Same as L2 norm for flattened matrix.
pub fn frobenius_norm(a: &SilState) -> f64 {
    norm_l2(a)
}

/// Normalize vector to unit length
pub fn normalize_l2(a: &SilState) -> SilState {
    let norm = norm_l2(a);
    if norm < 1e-10 {
        return *a; // Avoid division by zero
    }
    scale_state(a, 1.0 / norm)
}

/// Identity matrix (4x4)
pub fn identity_4x4() -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        let idx = i * 4 + i; // Diagonal
        result = result.with_layer(idx, from_mag_phase(1.0, 0.0));
    }

    result
}

/// Trace of 4x4 matrix: tr(A) = Σ aii
pub fn trace_4x4(a: &SilState) -> f64 {
    let mut trace = 0.0;
    for i in 0..4 {
        let idx = i * 4 + i;
        trace += magnitude(&a.get(idx));
    }
    trace
}

/// Diagonal of 4x4 matrix as vector (first 4 elements)
pub fn diag_4x4(a: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        let idx = i * 4 + i;
        result = result.with_layer(i, a.get(idx));
    }

    result
}

/// Create diagonal matrix from vector (first 4 elements)
pub fn diag_from_vec(v: &SilState) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..4 {
        let idx = i * 4 + i;
        result = result.with_layer(idx, v.get(i));
    }

    result
}

/// Clip values to range
pub fn clip(a: &SilState, min_val: f64, max_val: f64) -> SilState {
    let mut result = SilState::vacuum();

    for i in 0..NUM_LAYERS {
        let val = a.get(i);
        let clipped = magnitude(&val).clamp(min_val, max_val);
        result = result.with_layer(i, from_mag_phase(clipped, phase(&val)));
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_matmul_identity() {
        let identity = identity_4x4();
        let a = SilState::neutral();

        let result = matmul_4x4(&identity, &a);

        // I × A = A (approximately, due to phase handling)
        for i in 0..NUM_LAYERS {
            let orig = magnitude(&a.get(i));
            let res = magnitude(&result.get(i));
            assert!((orig - res).abs() < 1e-5, "Layer {}: {} vs {}", i, orig, res);
        }
    }

    #[test]
    fn test_dot_product() {
        let a = SilState::neutral();
        let b = SilState::neutral();

        let dot = dot(&a, &b);
        assert!(dot > 0.0); // Non-zero for neutral state
    }

    #[test]
    fn test_transpose() {
        let mut a = SilState::vacuum();
        // Set element at (0,1) = index 1
        a = a.with_layer(1, from_mag_phase(5.0, 0.0));

        let t = transpose_4x4(&a);

        // Element (0,1) should move to (1,0) = index 4
        // ByteSil quantization: ln(5) ≈ 1.6, stored as i8, recovered as e^1 or e^2
        let val_at_4 = magnitude(&t.get(4));
        assert!((val_at_4 - 5.0).abs() < 3.0, "Expected ~5.0, got {}", val_at_4);

        // Position 1 should now be from vacuum (very small)
        let val_at_1 = magnitude(&t.get(1));
        assert!(val_at_1 < 0.01, "Expected ~0, got {}", val_at_1);
    }

    #[test]
    fn test_hadamard() {
        let a = SilState::neutral();
        let result = hadamard(&a, &a);

        // Element-wise square
        for i in 0..NUM_LAYERS {
            let orig = magnitude(&a.get(i));
            let res = magnitude(&result.get(i));
            assert!((res - orig * orig).abs() < 1e-10);
        }
    }

    #[test]
    fn test_scale() {
        let a = SilState::neutral();
        let scaled = scale_state(&a, 2.0);

        // neutral() has rho=0, so magnitude = e^0 = 1
        // Scaling by 2 gives magnitude 2, which becomes rho = ln(2) ≈ 0.693
        // This rounds to rho=1 (i8), so recovered magnitude = e^1 ≈ 2.718
        // Or rho=0, recovered = 1.0
        for i in 0..NUM_LAYERS {
            let orig = magnitude(&a.get(i));
            let res = magnitude(&scaled.get(i));
            // Due to extreme quantization (i8 for log-magnitude), allow large tolerance
            assert!(res >= orig * 0.5 && res <= orig * 4.0,
                "Layer {}: expected ~{}, got {}", i, orig * 2.0, res);
        }
    }

    #[test]
    fn test_norm_l2() {
        let a = identity_4x4(); // 4 ones on diagonal
        let norm = norm_l2(&a);
        // √4 = 2, but ByteSil quantization affects precision
        // magnitude(1.0) = e^(ln(1)) = e^0 = 1, but quantized to i8
        assert!((norm - 2.0).abs() < 0.5, "L2 norm was {}", norm);
    }

    #[test]
    fn test_normalize() {
        let a = SilState::neutral();
        let normalized = normalize_l2(&a);
        let norm = norm_l2(&normalized);
        // ByteSil quantization limits precision significantly
        assert!((norm - 1.0).abs() < 0.5, "Normalized L2 norm was {}", norm);
    }

    #[test]
    fn test_trace() {
        let identity = identity_4x4();
        let trace = trace_4x4(&identity);
        assert!((trace - 4.0).abs() < 1e-10);
    }
}
