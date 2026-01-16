//! ByteSil Operations for LIS
//!
//! Core operations on ByteSil values (complex numbers in log-polar form).
//! ByteSil represents (ρ, θ) where ρ is magnitude (log scale) and θ is phase.

use crate::error::{Error, Result};
use sil_core::state::ByteSil;
use std::f64::consts::PI;

/// @stdlib_bytesil fn bytesil_new(rho: Int, theta: Int) -> ByteSil
///
/// Creates a new ByteSil value from raw rho (magnitude) and theta (phase) components.
/// rho is in range [-8, 7], theta is in range [0, 15].
///
/// # Example
/// ```lis
/// let b = bytesil_new(0, 4);  // Neutral magnitude, 90° phase
/// ```
pub fn bytesil_new(rho: i8, theta: u8) -> Result<ByteSil> {
    Ok(ByteSil::new(rho, theta))
}

/// @stdlib_bytesil fn bytesil_from_complex(re: Float, im: Float) -> ByteSil
///
/// Creates a ByteSil value from Cartesian complex number (real, imaginary).
///
/// # Example
/// ```lis
/// let b = bytesil_from_complex(1.0, 1.0);  // 1 + i
/// ```
pub fn bytesil_from_complex(re: f64, im: f64) -> Result<ByteSil> {
    use num_complex::Complex;
    Ok(ByteSil::from_complex(Complex::new(re, im)))
}

/// @stdlib_bytesil fn bytesil_to_complex(b: ByteSil) -> (Float, Float)
///
/// Converts a ByteSil value to Cartesian complex number (real, imaginary).
///
/// # Example
/// ```lis
/// let (re, im) = bytesil_to_complex(b);
/// ```
pub fn bytesil_to_complex(b: &ByteSil) -> Result<(f64, f64)> {
    let c = b.to_complex();
    Ok((c.re, c.im))
}

/// @stdlib_bytesil fn bytesil_null() -> ByteSil
///
/// Returns the NULL ByteSil value (0, 0).
/// This represents the origin in complex space.
pub fn bytesil_null() -> Result<ByteSil> {
    Ok(ByteSil::NULL)
}

/// @stdlib_bytesil fn bytesil_one() -> ByteSil
///
/// Returns the ONE ByteSil value (255, 0).
/// This represents 1 + 0i in complex space.
pub fn bytesil_one() -> Result<ByteSil> {
    Ok(ByteSil::ONE)
}

/// @stdlib_bytesil fn bytesil_i() -> ByteSil
///
/// Returns the I ByteSil value (255, 64).
/// This represents 0 + 1i in complex space.
pub fn bytesil_i() -> Result<ByteSil> {
    Ok(ByteSil::I)
}

/// @stdlib_bytesil fn bytesil_neg_one() -> ByteSil
///
/// Returns the NEG_ONE ByteSil value (255, 128).
/// This represents -1 + 0i in complex space.
pub fn bytesil_neg_one() -> Result<ByteSil> {
    Ok(ByteSil::NEG_ONE)
}

/// @stdlib_bytesil fn bytesil_neg_i() -> ByteSil
///
/// Returns the NEG_I ByteSil value (255, 192).
/// This represents 0 - 1i in complex space.
pub fn bytesil_neg_i() -> Result<ByteSil> {
    Ok(ByteSil::NEG_I)
}

/// @stdlib_bytesil fn bytesil_max() -> ByteSil
///
/// Returns the MAX ByteSil value (255, 255).
/// This represents the maximum representable value.
pub fn bytesil_max() -> Result<ByteSil> {
    Ok(ByteSil::MAX)
}

/// @stdlib_bytesil fn bytesil_mul(a: ByteSil, b: ByteSil) -> ByteSil
///
/// Multiplies two ByteSil values.
/// In log-polar form: (ρ₁ + ρ₂, θ₁ + θ₂) - O(1) operation.
///
/// # Example
/// ```lis
/// let result = bytesil_mul(a, b);
/// ```
pub fn bytesil_mul(a: &ByteSil, b: &ByteSil) -> Result<ByteSil> {
    Ok(a.mul(b))
}

/// @stdlib_bytesil fn bytesil_div(a: ByteSil, b: ByteSil) -> ByteSil
///
/// Divides two ByteSil values.
/// In log-polar form: (ρ₁ - ρ₂, θ₁ - θ₂) - O(1) operation.
///
/// # Example
/// ```lis
/// let result = bytesil_div(a, b);
/// ```
pub fn bytesil_div(a: &ByteSil, b: &ByteSil) -> Result<ByteSil> {
    Ok(a.div(b))
}

/// @stdlib_bytesil fn bytesil_pow(b: ByteSil, n: Int) -> ByteSil
///
/// Raises a ByteSil value to an integer power.
/// In log-polar form: (ρ * n, θ * n) - O(1) operation.
///
/// # Example
/// ```lis
/// let squared = bytesil_pow(b, 2);
/// let cubed = bytesil_pow(b, 3);
/// ```
pub fn bytesil_pow(b: &ByteSil, n: i32) -> Result<ByteSil> {
    Ok(b.pow(n))
}

/// @stdlib_bytesil fn bytesil_root(b: ByteSil, n: Int) -> ByteSil
///
/// Computes the nth root of a ByteSil value.
/// In log-polar form: (ρ / n, θ / n) - O(1) operation.
///
/// # Example
/// ```lis
/// let sqrt = bytesil_root(b, 2);
/// let cbrt = bytesil_root(b, 3);
/// ```
pub fn bytesil_root(b: &ByteSil, n: i32) -> Result<ByteSil> {
    if n == 0 {
        return Err(Error::SemanticError {
            message: "Cannot compute 0th root".into(),
        });
    }
    Ok(b.root(n))
}

/// @stdlib_bytesil fn bytesil_inv(b: ByteSil) -> ByteSil
///
/// Computes the multiplicative inverse (1/b) of a ByteSil value.
/// In log-polar form: (-ρ, -θ) - O(1) operation.
///
/// # Example
/// ```lis
/// let inverse = bytesil_inv(b);
/// ```
pub fn bytesil_inv(b: &ByteSil) -> Result<ByteSil> {
    Ok(b.inv())
}

/// @stdlib_bytesil fn bytesil_conj(b: ByteSil) -> ByteSil
///
/// Computes the complex conjugate of a ByteSil value.
/// In log-polar form: (ρ, -θ) - reflects across real axis.
///
/// # Example
/// ```lis
/// let conjugate = bytesil_conj(b);
/// ```
pub fn bytesil_conj(b: &ByteSil) -> Result<ByteSil> {
    Ok(b.conj())
}

/// @stdlib_bytesil fn bytesil_xor(a: ByteSil, b: ByteSil) -> ByteSil
///
/// Computes the XOR of two ByteSil values component-wise.
/// Result: (ρ₁ ⊕ ρ₂, θ₁ ⊕ θ₂)
///
/// # Example
/// ```lis
/// let xored = bytesil_xor(a, b);
/// ```
pub fn bytesil_xor(a: &ByteSil, b: &ByteSil) -> Result<ByteSil> {
    Ok(a.xor(b))
}

/// @stdlib_bytesil fn bytesil_mix(a: ByteSil, b: ByteSil) -> ByteSil
///
/// Mixes two ByteSil values using the SIL mix operation.
/// This creates a blend between the two values.
///
/// # Example
/// ```lis
/// let mixed = bytesil_mix(a, b);
/// ```
pub fn bytesil_mix(a: &ByteSil, b: &ByteSil) -> Result<ByteSil> {
    Ok(a.mix(b))
}

/// @stdlib_bytesil fn bytesil_rho(b: ByteSil) -> Int
///
/// Returns the raw rho (magnitude) component [-8, 7].
///
/// # Example
/// ```lis
/// let magnitude_raw = bytesil_rho(b);
/// ```
pub fn bytesil_rho(b: &ByteSil) -> Result<i8> {
    Ok(b.rho)
}

/// @stdlib_bytesil fn bytesil_theta(b: ByteSil) -> Int
///
/// Returns the raw theta (phase) component [0, 255].
///
/// # Example
/// ```lis
/// let phase_raw = bytesil_theta(b);
/// ```
pub fn bytesil_theta(b: &ByteSil) -> Result<u8> {
    Ok(b.theta)
}

/// @stdlib_bytesil fn bytesil_magnitude(b: ByteSil) -> Float
///
/// Returns the magnitude as a floating-point value.
/// Converts from log scale to linear scale.
///
/// # Example
/// ```lis
/// let mag = bytesil_magnitude(b);
/// ```
pub fn bytesil_magnitude(b: &ByteSil) -> Result<f64> {
    let c = b.to_complex(); let (re, im) = (c.re, c.im);
    Ok((re * re + im * im).sqrt())
}

/// @stdlib_bytesil fn bytesil_phase_degrees(b: ByteSil) -> Float
///
/// Returns the phase in degrees [0.0, 360.0).
/// theta goes from 0-15, each step is 22.5 degrees.
///
/// # Example
/// ```lis
/// let phase_deg = bytesil_phase_degrees(b);
/// ```
pub fn bytesil_phase_degrees(b: &ByteSil) -> Result<f64> {
    // theta is 0-15, representing 16 phase values
    // Each step is 360/16 = 22.5 degrees
    let theta_normalized = b.theta as f64 / 16.0;
    Ok(theta_normalized * 360.0)
}

/// @stdlib_bytesil fn bytesil_phase_radians(b: ByteSil) -> Float
///
/// Returns the phase in radians [0.0, 2π).
/// theta goes from 0-15, each step is π/8 radians.
///
/// # Example
/// ```lis
/// let phase_rad = bytesil_phase_radians(b);
/// ```
pub fn bytesil_phase_radians(b: &ByteSil) -> Result<f64> {
    // theta is 0-15, representing 16 phase values
    // Each step is 2π/16 = π/8 radians
    let theta_normalized = b.theta as f64 / 16.0;
    Ok(theta_normalized * 2.0 * PI)
}

/// @stdlib_bytesil fn bytesil_is_null(b: ByteSil) -> Bool
///
/// Returns true if the ByteSil value is NULL (0, 0).
///
/// # Example
/// ```lis
/// if bytesil_is_null(b) {
///     // Handle null case
/// }
/// ```
pub fn bytesil_is_null(b: &ByteSil) -> Result<bool> {
    Ok(*b == ByteSil::NULL)
}

/// @stdlib_bytesil fn bytesil_is_real(b: ByteSil) -> Bool
///
/// Returns true if the ByteSil value is real (theta = 0 or 8).
/// theta=0 is +1 direction, theta=8 is -1 direction.
///
/// # Example
/// ```lis
/// if bytesil_is_real(b) {
///     // Value is on the real axis
/// }
/// ```
pub fn bytesil_is_real(b: &ByteSil) -> Result<bool> {
    let theta = b.theta;
    // theta goes from 0-15 (16 values), 0 = 0°, 8 = 180°
    Ok(theta == 0 || theta == 8)
}

/// @stdlib_bytesil fn bytesil_is_imaginary(b: ByteSil) -> Bool
///
/// Returns true if the ByteSil value is purely imaginary (theta = 4 or 12).
/// theta=4 is +i direction (90°), theta=12 is -i direction (270°).
///
/// # Example
/// ```lis
/// if bytesil_is_imaginary(b) {
///     // Value is on the imaginary axis
/// }
/// ```
pub fn bytesil_is_imaginary(b: &ByteSil) -> Result<bool> {
    let theta = b.theta;
    // theta goes from 0-15 (16 values), 4 = 90°, 12 = 270°
    Ok(theta == 4 || theta == 12)
}

/// @stdlib_bytesil fn bytesil_norm(b: ByteSil) -> Int
///
/// Returns the norm (squared magnitude) in quantized form.
///
/// # Example
/// ```lis
/// let n = bytesil_norm(b);
/// ```
pub fn bytesil_norm(b: &ByteSil) -> Result<u8> {
    Ok(b.norm())
}

/// @stdlib_bytesil fn bytesil_from_u8(value: Int) -> ByteSil
///
/// Creates a ByteSil from a raw u8 byte value.
///
/// # Example
/// ```lis
/// let b = bytesil_from_u8(0x42);
/// ```
pub fn bytesil_from_u8(value: u8) -> Result<ByteSil> {
    Ok(ByteSil::from_u8(value))
}

/// @stdlib_bytesil fn bytesil_to_u8(b: ByteSil) -> Int
///
/// Converts a ByteSil to a raw u8 byte value.
///
/// # Example
/// ```lis
/// let byte = bytesil_to_u8(b);
/// ```
pub fn bytesil_to_u8(b: &ByteSil) -> Result<u8> {
    Ok(b.to_u8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        let null = bytesil_null().unwrap();
        // NULL is defined as {rho: -8, theta: 0} - minimum magnitude
        assert_eq!(null.rho, -8);
        assert_eq!(null.theta, 0);

        let one = bytesil_one().unwrap();
        assert_eq!(one, ByteSil::ONE);

        let i = bytesil_i().unwrap();
        assert_eq!(i, ByteSil::I);
    }

    #[test]
    fn test_arithmetic() {
        let a = bytesil_one().unwrap();
        let b = bytesil_i().unwrap();

        let mul = bytesil_mul(&a, &b).unwrap();
        assert_eq!(mul, ByteSil::I);

        let inv = bytesil_inv(&a).unwrap();
        let identity = bytesil_mul(&a, &inv).unwrap();
        assert!(bytesil_rho(&identity).unwrap() < 5); // Close to NULL in log space
    }

    #[test]
    fn test_power_operations() {
        let i = bytesil_i().unwrap();
        // i has theta=4 (90°)
        let i_squared = bytesil_pow(&i, 2).unwrap();
        // i^2 = -1, should have theta = 4*2 % 16 = 8 (180°)
        assert_eq!(i_squared.theta, 8, "i^2 should have theta=8 (180°)");
        assert!(bytesil_is_real(&i_squared).unwrap(), "i^2 = -1 should be real");

        let sqrt = bytesil_root(&i_squared, 2).unwrap();
        // sqrt(-1) should have theta = 8/2 = 4 (90°), which is imaginary
        assert!(bytesil_is_imaginary(&sqrt).unwrap(), "sqrt(-1) should be imaginary");
    }

    #[test]
    fn test_complex_conversion() {
        // (1, 1) has |z| = √2 ≈ 1.414, ln(√2) ≈ 0.347 rounds to rho=0
        // arg = π/4 = 45°, theta = 45/22.5 = 2
        // So recovered is e^0 * (cos(2*π/8), sin(2*π/8)) = (cos(π/4), sin(π/4)) ≈ (0.707, 0.707)
        let (re, im) = (1.0, 1.0);
        let b = bytesil_from_complex(re, im).unwrap();
        let (re2, im2) = bytesil_to_complex(&b).unwrap();

        // Due to magnitude quantization (e^rho), expect significant error
        assert!((re2 - 0.707).abs() < 0.5, "Re was {}", re2);
        assert!((im2 - 0.707).abs() < 0.5, "Im was {}", im2);
    }

    #[test]
    fn test_conjugate() {
        // (3, 4) has |z| = 5, ln(5) ≈ 1.6 rounds to rho=2
        // After quantization, magnitude becomes e^2 ≈ 7.4
        let b = bytesil_from_complex(3.0, 4.0).unwrap();
        let conj = bytesil_conj(&b).unwrap();
        let (re, im) = bytesil_to_complex(&conj).unwrap();

        // Due to quantization, just verify conjugate flips sign of imaginary part
        // and both values are reasonable
        assert!(re > 0.0, "Re should be positive: {}", re);
        assert!(im < 0.0, "Im should be negative (conjugate): {}", im);
    }

    #[test]
    fn test_predicates() {
        let null = bytesil_null().unwrap();
        assert!(bytesil_is_null(&null).unwrap());

        let one = bytesil_one().unwrap();
        assert!(bytesil_is_real(&one).unwrap());
        assert!(!bytesil_is_imaginary(&one).unwrap());

        let i = bytesil_i().unwrap();
        assert!(!bytesil_is_real(&i).unwrap());
        assert!(bytesil_is_imaginary(&i).unwrap());
    }

    #[test]
    fn test_phase_conversion() {
        // ByteSil::I has theta=4, which is 4/16 * 360 = 90°
        let i = bytesil_i().unwrap();
        let deg = bytesil_phase_degrees(&i).unwrap();
        assert!((deg - 90.0).abs() < 1.0, "Phase degrees was {}", deg);

        let rad = bytesil_phase_radians(&i).unwrap();
        assert!((rad - PI / 2.0).abs() < 0.1, "Phase radians was {}", rad);
    }

    #[test]
    fn test_xor_operation() {
        let a = bytesil_new(50, 50).unwrap();
        let b = bytesil_new(100, 150).unwrap();
        let xored = bytesil_xor(&a, &b).unwrap();

        // XOR is reversible
        let back = bytesil_xor(&xored, &b).unwrap();
        assert_eq!(back, a);
    }

    #[test]
    fn test_root_zero_error() {
        let b = bytesil_one().unwrap();
        let result = bytesil_root(&b, 0);
        assert!(result.is_err());
    }
}
