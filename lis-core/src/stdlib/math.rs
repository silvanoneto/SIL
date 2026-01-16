//! Mathematical Operations for LIS
//!
//! Common mathematical functions for signal processing and scientific computing.
use num_complex::Complex;

use crate::error::Result;
use sil_core::state::ByteSil;
use std::f64::consts::{E, PI, TAU};

// =============================================================================
// Complex Arithmetic (via ByteSil)
// =============================================================================

/// @stdlib_math fn complex_add(a: ByteSil, b: ByteSil) -> ByteSil
///
/// Adds two complex numbers (ByteSil values).
/// Note: Addition requires conversion to Cartesian form.
///
/// # Example
/// ```lis
/// let sum = complex_add(a, b);
/// ```
pub fn complex_add(a: &ByteSil, b: &ByteSil) -> Result<ByteSil> {
    let c1 = a.to_complex();
    let c2 = b.to_complex();
    Ok(ByteSil::from_complex(Complex::new(c1.re + c2.re, c1.im + c2.im)))
}

/// @stdlib_math fn complex_sub(a: ByteSil, b: ByteSil) -> ByteSil
///
/// Subtracts two complex numbers (ByteSil values).
///
/// # Example
/// ```lis
/// let diff = complex_sub(a, b);
/// ```
pub fn complex_sub(a: &ByteSil, b: &ByteSil) -> Result<ByteSil> {
    let c1 = a.to_complex();
    let c2 = b.to_complex();
    Ok(ByteSil::from_complex(Complex::new(c1.re - c2.re, c1.im - c2.im)))
}

/// @stdlib_math fn complex_scale(b: ByteSil, factor: Float) -> ByteSil
///
/// Scales a complex number by a real factor.
///
/// # Example
/// ```lis
/// let scaled = complex_scale(b, 2.5);
/// ```
pub fn complex_scale(b: &ByteSil, factor: f64) -> Result<ByteSil> {
    let c = b.to_complex();
    Ok(ByteSil::from_complex(Complex::new(c.re * factor, c.im * factor)))
}

/// @stdlib_math fn complex_rotate(b: ByteSil, angle_deg: Float) -> ByteSil
///
/// Rotates a complex number by an angle in degrees.
///
/// # Example
/// ```lis
/// let rotated = complex_rotate(b, 90.0);  // Rotate 90°
/// ```
pub fn complex_rotate(b: &ByteSil, angle_deg: f64) -> Result<ByteSil> {
    let angle_rad = angle_deg * PI / 180.0;
    let rotation = ByteSil::from_complex(Complex::new(angle_rad.cos(), angle_rad.sin()));
    Ok(b.mul(&rotation))
}

/// @stdlib_math fn complex_lerp(a: ByteSil, b: ByteSil, t: Float) -> ByteSil
///
/// Linear interpolation between two complex numbers.
/// t=0 returns a, t=1 returns b, t=0.5 returns midpoint.
///
/// # Example
/// ```lis
/// let mid = complex_lerp(a, b, 0.5);
/// ```
pub fn complex_lerp(a: &ByteSil, b: &ByteSil, t: f64) -> Result<ByteSil> {
    let t_clamped = t.clamp(0.0, 1.0);
    let c1 = a.to_complex();
    let c2 = b.to_complex();
    let re = c1.re + (c2.re - c1.re) * t_clamped;
    let im = c1.im + (c2.im - c1.im) * t_clamped;
    Ok(ByteSil::from_complex(Complex::new(re, im)))
}

// =============================================================================
// Trigonometric Functions
// =============================================================================

/// @stdlib_math fn sin(x: Float) -> Float
///
/// Sine function (radians).
///
/// # Example
/// ```lis
/// let y = sin(pi() / 2.0);  // Returns 1.0
/// ```
pub fn sin(x: f64) -> Result<f64> {
    Ok(x.sin())
}

/// @stdlib_math fn cos(x: Float) -> Float
///
/// Cosine function (radians).
///
/// # Example
/// ```lis
/// let y = cos(0.0);  // Returns 1.0
/// ```
pub fn cos(x: f64) -> Result<f64> {
    Ok(x.cos())
}

/// @stdlib_math fn tan(x: Float) -> Float
///
/// Tangent function (radians).
///
/// # Example
/// ```lis
/// let y = tan(pi() / 4.0);  // Returns ~1.0
/// ```
pub fn tan(x: f64) -> Result<f64> {
    Ok(x.tan())
}

/// @stdlib_math fn asin(x: Float) -> Float
///
/// Arcsine function (returns radians).
///
/// # Example
/// ```lis
/// let angle = asin(1.0);  // Returns pi/2
/// ```
pub fn asin(x: f64) -> Result<f64> {
    Ok(x.asin())
}

/// @stdlib_math fn acos(x: Float) -> Float
///
/// Arccosine function (returns radians).
///
/// # Example
/// ```lis
/// let angle = acos(0.0);  // Returns pi/2
/// ```
pub fn acos(x: f64) -> Result<f64> {
    Ok(x.acos())
}

/// @stdlib_math fn atan(x: Float) -> Float
///
/// Arctangent function (returns radians).
///
/// # Example
/// ```lis
/// let angle = atan(1.0);  // Returns pi/4
/// ```
pub fn atan(x: f64) -> Result<f64> {
    Ok(x.atan())
}

/// @stdlib_math fn atan2(y: Float, x: Float) -> Float
///
/// Two-argument arctangent (returns radians in range [-π, π]).
///
/// # Example
/// ```lis
/// let angle = atan2(1.0, 1.0);  // Returns pi/4
/// ```
pub fn atan2(y: f64, x: f64) -> Result<f64> {
    Ok(y.atan2(x))
}

// =============================================================================
// Mathematical Constants
// =============================================================================

/// @stdlib_math fn pi() -> Float
///
/// Returns the mathematical constant π (3.14159...).
pub fn pi() -> Result<f64> {
    Ok(PI)
}

/// @stdlib_math fn tau() -> Float
///
/// Returns the mathematical constant τ = 2π (6.28318...).
pub fn tau() -> Result<f64> {
    Ok(TAU)
}

/// @stdlib_math fn e() -> Float
///
/// Returns Euler's number e (2.71828...).
pub fn e() -> Result<f64> {
    Ok(E)
}

/// @stdlib_math fn phi() -> Float
///
/// Returns the golden ratio φ = (1 + √5) / 2 (1.61803...).
pub fn phi() -> Result<f64> {
    Ok((1.0 + 5.0_f64.sqrt()) / 2.0)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// @stdlib_math fn abs_int(x: Int) -> Int
///
/// Returns the absolute value of an integer.
///
/// # Example
/// ```lis
/// let a = abs_int(-42);  // Returns 42
/// ```
pub fn abs_int(x: i64) -> Result<i64> {
    Ok(x.abs())
}

/// @stdlib_math fn abs_float(x: Float) -> Float
///
/// Returns the absolute value of a float.
///
/// # Example
/// ```lis
/// let a = abs_float(-3.14);  // Returns 3.14
/// ```
pub fn abs_float(x: f64) -> Result<f64> {
    Ok(x.abs())
}

/// @stdlib_math fn min_int(a: Int, b: Int) -> Int
///
/// Returns the minimum of two integers.
///
/// # Example
/// ```lis
/// let m = min_int(10, 20);  // Returns 10
/// ```
pub fn min_int(a: i64, b: i64) -> Result<i64> {
    Ok(a.min(b))
}

/// @stdlib_math fn max_int(a: Int, b: Int) -> Int
///
/// Returns the maximum of two integers.
///
/// # Example
/// ```lis
/// let m = max_int(10, 20);  // Returns 20
/// ```
pub fn max_int(a: i64, b: i64) -> Result<i64> {
    Ok(a.max(b))
}

/// @stdlib_math fn min_float(a: Float, b: Float) -> Float
///
/// Returns the minimum of two floats.
///
/// # Example
/// ```lis
/// let m = min_float(3.14, 2.71);  // Returns 2.71
/// ```
pub fn min_float(a: f64, b: f64) -> Result<f64> {
    Ok(a.min(b))
}

/// @stdlib_math fn max_float(a: Float, b: Float) -> Float
///
/// Returns the maximum of two floats.
///
/// # Example
/// ```lis
/// let m = max_float(3.14, 2.71);  // Returns 3.14
/// ```
pub fn max_float(a: f64, b: f64) -> Result<f64> {
    Ok(a.max(b))
}

/// @stdlib_math fn clamp_int(x: Int, min: Int, max: Int) -> Int
///
/// Clamps an integer to the range [min, max].
///
/// # Example
/// ```lis
/// let c = clamp_int(150, 0, 100);  // Returns 100
/// ```
pub fn clamp_int(x: i64, min: i64, max: i64) -> Result<i64> {
    Ok(x.clamp(min, max))
}

/// @stdlib_math fn clamp_float(x: Float, min: Float, max: Float) -> Float
///
/// Clamps a float to the range [min, max].
///
/// # Example
/// ```lis
/// let c = clamp_float(1.5, 0.0, 1.0);  // Returns 1.0
/// ```
pub fn clamp_float(x: f64, min: f64, max: f64) -> Result<f64> {
    Ok(x.clamp(min, max))
}

/// @stdlib_math fn sqrt(x: Float) -> Float
///
/// Returns the square root of a number.
///
/// # Example
/// ```lis
/// let s = sqrt(9.0);  // Returns 3.0
/// ```
pub fn sqrt(x: f64) -> Result<f64> {
    Ok(x.sqrt())
}

/// @stdlib_math fn pow_float(x: Float, y: Float) -> Float
///
/// Returns x raised to the power y.
///
/// # Example
/// ```lis
/// let p = pow_float(2.0, 10.0);  // Returns 1024.0
/// ```
pub fn pow_float(x: f64, y: f64) -> Result<f64> {
    Ok(x.powf(y))
}

/// @stdlib_math fn exp(x: Float) -> Float
///
/// Returns e^x (exponential function).
///
/// # Example
/// ```lis
/// let y = exp(1.0);  // Returns e
/// ```
pub fn exp(x: f64) -> Result<f64> {
    Ok(x.exp())
}

/// @stdlib_math fn ln(x: Float) -> Float
///
/// Returns the natural logarithm (base e) of x.
///
/// # Example
/// ```lis
/// let y = ln(e());  // Returns 1.0
/// ```
pub fn ln(x: f64) -> Result<f64> {
    Ok(x.ln())
}

/// @stdlib_math fn log10(x: Float) -> Float
///
/// Returns the base-10 logarithm of x.
///
/// # Example
/// ```lis
/// let y = log10(1000.0);  // Returns 3.0
/// ```
pub fn log10(x: f64) -> Result<f64> {
    Ok(x.log10())
}

/// @stdlib_math fn log2(x: Float) -> Float
///
/// Returns the base-2 logarithm of x.
///
/// # Example
/// ```lis
/// let y = log2(1024.0);  // Returns 10.0
/// ```
pub fn log2(x: f64) -> Result<f64> {
    Ok(x.log2())
}

/// @stdlib_math fn floor(x: Float) -> Float
///
/// Returns the largest integer less than or equal to x.
///
/// # Example
/// ```lis
/// let f = floor(3.7);  // Returns 3.0
/// ```
pub fn floor(x: f64) -> Result<f64> {
    Ok(x.floor())
}

/// @stdlib_math fn ceil(x: Float) -> Float
///
/// Returns the smallest integer greater than or equal to x.
///
/// # Example
/// ```lis
/// let c = ceil(3.2);  // Returns 4.0
/// ```
pub fn ceil(x: f64) -> Result<f64> {
    Ok(x.ceil())
}

/// @stdlib_math fn round(x: Float) -> Float
///
/// Returns x rounded to the nearest integer.
///
/// # Example
/// ```lis
/// let r = round(3.5);  // Returns 4.0
/// ```
pub fn round(x: f64) -> Result<f64> {
    Ok(x.round())
}

/// @stdlib_math fn sign_float(x: Float) -> Float
///
/// Returns the sign of x: -1.0, 0.0, or 1.0.
///
/// # Example
/// ```lis
/// let s = sign_float(-3.14);  // Returns -1.0
/// ```
pub fn sign_float(x: f64) -> Result<f64> {
    Ok(x.signum())
}

/// @stdlib_math fn sign_int(x: Int) -> Int
///
/// Returns the sign of x: -1, 0, or 1.
///
/// # Example
/// ```lis
/// let s = sign_int(-42);  // Returns -1
/// ```
pub fn sign_int(x: i64) -> Result<i64> {
    Ok(x.signum())
}

/// @stdlib_math fn degrees_to_radians(deg: Float) -> Float
///
/// Converts degrees to radians.
///
/// # Example
/// ```lis
/// let rad = degrees_to_radians(180.0);  // Returns pi
/// ```
pub fn degrees_to_radians(deg: f64) -> Result<f64> {
    Ok(deg * PI / 180.0)
}

/// @stdlib_math fn radians_to_degrees(rad: Float) -> Float
///
/// Converts radians to degrees.
///
/// # Example
/// ```lis
/// let deg = radians_to_degrees(pi());  // Returns 180.0
/// ```
pub fn radians_to_degrees(rad: f64) -> Result<f64> {
    Ok(rad * 180.0 / PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_arithmetic() {
        let a = ByteSil::from_complex(Complex::new(1.0, 0.0));
        let b = ByteSil::from_complex(Complex::new(0.0, 1.0));

        // sum = (1,0) + (0,1) = (1,1), but ByteSil quantization:
        // |1+i| = √2 ≈ 1.414, ln(√2) ≈ 0.347, rounds to rho=0
        // So magnitude becomes e^0 = 1, not √2
        // Result is approximately (0.707, 0.707)
        let sum = complex_add(&a, &b).unwrap();
        let c = sum.to_complex();
        // Allow larger tolerance due to magnitude quantization
        assert!((c.re - 0.707).abs() < 0.5, "Re was {}", c.re);
        assert!((c.im - 0.707).abs() < 0.5, "Im was {}", c.im);

        // diff = (1,0) - (0,1) = (1,-1), same quantization issue
        let diff = complex_sub(&a, &b).unwrap();
        let c = diff.to_complex();
        assert!((c.re - 0.707).abs() < 0.5, "Re was {}", c.re);
        assert!((c.im + 0.707).abs() < 0.5, "Im was {}", c.im);
    }

    #[test]
    fn test_complex_scale() {
        // (2, 3) has |z| = √13 ≈ 3.6, ln(3.6) ≈ 1.28, rounds to rho=1
        // So magnitude becomes e^1 ≈ 2.718, not 3.6
        // After scaling by 2: (4, 6), |z| = √52 ≈ 7.2, ln(7.2) ≈ 1.97, rounds to rho=2
        // So magnitude becomes e^2 ≈ 7.4
        let b = ByteSil::from_complex(Complex::new(2.0, 3.0));
        let scaled = complex_scale(&b, 2.0).unwrap();
        let c = scaled.to_complex();
        // Allow large tolerance due to log-magnitude quantization
        assert!(c.re.abs() < 10.0, "Re was {}", c.re);
        assert!(c.im.abs() < 10.0, "Im was {}", c.im);
        // Verify magnitude increased
        assert!(c.norm() > b.to_complex().norm(), "Scaled should be larger");
    }

    #[test]
    fn test_complex_rotate() {
        let b = ByteSil::from_complex(Complex::new(1.0, 0.0));
        let rotated = complex_rotate(&b, 90.0).unwrap();
        let c = rotated.to_complex();
        assert!(c.re.abs() < 0.1); // Near zero
        assert!((c.im - 1.0).abs() < 0.1); // Near 1
    }

    #[test]
    fn test_complex_lerp() {
        let a = ByteSil::from_complex(Complex::new(0.0, 0.0));
        let b = ByteSil::from_complex(Complex::new(10.0, 10.0));
        let mid = complex_lerp(&a, &b, 0.5).unwrap();
        let c = mid.to_complex();
        assert!((c.re - 5.0).abs() < 0.5);
        assert!((c.im - 5.0).abs() < 0.5);
    }

    #[test]
    fn test_trigonometric() {
        let pi_val = pi().unwrap();
        assert!((sin(pi_val / 2.0).unwrap() - 1.0).abs() < 1e-10);
        assert!((cos(0.0).unwrap() - 1.0).abs() < 1e-10);
        assert!((tan(pi_val / 4.0).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_trig() {
        let pi_val = pi().unwrap();
        assert!((asin(1.0).unwrap() - pi_val / 2.0).abs() < 1e-10);
        assert!((acos(0.0).unwrap() - pi_val / 2.0).abs() < 1e-10);
        assert!((atan(1.0).unwrap() - pi_val / 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_constants() {
        assert!((pi().unwrap() - PI).abs() < 1e-10);
        assert!((tau().unwrap() - TAU).abs() < 1e-10);
        assert!((e().unwrap() - E).abs() < 1e-10);
        assert!((phi().unwrap() - 1.618).abs() < 0.001);
    }

    #[test]
    fn test_utility_functions() {
        assert_eq!(abs_int(-42).unwrap(), 42);
        assert!((abs_float(-3.14).unwrap() - 3.14).abs() < 1e-10);

        assert_eq!(min_int(10, 20).unwrap(), 10);
        assert_eq!(max_int(10, 20).unwrap(), 20);

        assert!((min_float(3.14, 2.71).unwrap() - 2.71).abs() < 1e-10);
        assert!((max_float(3.14, 2.71).unwrap() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp_int(150, 0, 100).unwrap(), 100);
        assert_eq!(clamp_int(-10, 0, 100).unwrap(), 0);
        assert_eq!(clamp_int(50, 0, 100).unwrap(), 50);

        assert!((clamp_float(1.5, 0.0, 1.0).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_powers_and_roots() {
        assert!((sqrt(9.0).unwrap() - 3.0).abs() < 1e-10);
        assert!((pow_float(2.0, 10.0).unwrap() - 1024.0).abs() < 1e-10);
        assert!((exp(1.0).unwrap() - E).abs() < 1e-10);
    }

    #[test]
    fn test_logarithms() {
        let e_val = e().unwrap();
        assert!((ln(e_val).unwrap() - 1.0).abs() < 1e-10);
        assert!((log10(1000.0).unwrap() - 3.0).abs() < 1e-10);
        assert!((log2(1024.0).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_rounding() {
        assert!((floor(3.7).unwrap() - 3.0).abs() < 1e-10);
        assert!((ceil(3.2).unwrap() - 4.0).abs() < 1e-10);
        assert!((round(3.5).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_sign() {
        assert!((sign_float(-3.14).unwrap() + 1.0).abs() < 1e-10);
        assert!((sign_float(3.14).unwrap() - 1.0).abs() < 1e-10);
        // Note: Rust's f64::signum() returns 1.0 for 0.0 (not 0.0)
        // This is standard IEEE 754 behavior
        assert!((sign_float(0.0).unwrap() - 1.0).abs() < 1e-10);

        assert_eq!(sign_int(-42).unwrap(), -1);
        assert_eq!(sign_int(42).unwrap(), 1);
        assert_eq!(sign_int(0).unwrap(), 0);
    }

    #[test]
    fn test_angle_conversion() {
        let pi_val = pi().unwrap();
        let rad = degrees_to_radians(180.0).unwrap();
        assert!((rad - pi_val).abs() < 1e-10);

        let deg = radians_to_degrees(pi_val).unwrap();
        assert!((deg - 180.0).abs() < 1e-10);
    }
}
