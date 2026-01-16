//! String Operations for LIS
//!
//! String manipulation and conversion utilities.

use crate::error::Result;
use sil_core::state::{ByteSil, SilState};

/// @stdlib_string fn string_length(s: String) -> Int
pub fn string_length(s: &str) -> Result<usize> {
    Ok(s.len())
}

/// @stdlib_string fn string_concat(a: String, b: String) -> String
pub fn string_concat(a: &str, b: &str) -> Result<String> {
    Ok(format!("{}{}", a, b))
}

/// @stdlib_string fn string_slice(s: String, start: Int, end: Int) -> String
pub fn string_slice(s: &str, start: usize, end: usize) -> Result<String> {
    Ok(s.chars().skip(start).take(end - start).collect())
}

/// @stdlib_string fn string_to_upper(s: String) -> String
pub fn string_to_upper(s: &str) -> Result<String> {
    Ok(s.to_uppercase())
}

/// @stdlib_string fn string_to_lower(s: String) -> String
pub fn string_to_lower(s: &str) -> Result<String> {
    Ok(s.to_lowercase())
}

/// @stdlib_string fn string_contains(s: String, substr: String) -> Bool
pub fn string_contains(s: &str, substr: &str) -> Result<bool> {
    Ok(s.contains(substr))
}

/// @stdlib_string fn string_starts_with(s: String, prefix: String) -> Bool
pub fn string_starts_with(s: &str, prefix: &str) -> Result<bool> {
    Ok(s.starts_with(prefix))
}

/// @stdlib_string fn string_ends_with(s: String, suffix: String) -> Bool
pub fn string_ends_with(s: &str, suffix: &str) -> Result<bool> {
    Ok(s.ends_with(suffix))
}

/// @stdlib_string fn string_equals(a: String, b: String) -> Bool
pub fn string_equals(a: &str, b: &str) -> Result<bool> {
    Ok(a == b)
}

/// @stdlib_string fn int_to_string(x: Int) -> String
pub fn int_to_string(x: i64) -> Result<String> {
    Ok(x.to_string())
}

/// @stdlib_string fn float_to_string(x: Float) -> String
pub fn float_to_string(x: f64) -> Result<String> {
    Ok(x.to_string())
}

/// @stdlib_string fn bool_to_string(b: Bool) -> String
pub fn bool_to_string(b: bool) -> Result<String> {
    Ok(b.to_string())
}

/// @stdlib_string fn bytesil_to_string(b: ByteSil) -> String
pub fn bytesil_to_string(b: &ByteSil) -> Result<String> {
    let c = b.to_complex(); let (re, im) = (c.re, c.im);
    Ok(format!("({:.3} + {:.3}i)", re, im))
}

/// @stdlib_string fn state_to_string(s: State) -> String
pub fn state_to_string(s: &SilState) -> Result<String> {
    let mut result = String::from("State[");
    for i in 0..16 {
        let layer = s.layer(i);
        result.push_str(&format!("{:02X}", layer.to_u8()));
        if i < 15 {
            result.push(' ');
        }
    }
    result.push(']');
    Ok(result)
}

/// @stdlib_string fn string_to_int(s: String) -> Int
pub fn string_to_int(s: &str) -> Result<i64> {
    s.parse().map_err(|e| crate::error::Error::SemanticError {
        message: format!("Failed to parse int: {}", e),
    })
}

/// @stdlib_string fn string_to_float(s: String) -> Float
pub fn string_to_float(s: &str) -> Result<f64> {
    s.parse().map_err(|e| crate::error::Error::SemanticError {
        message: format!("Failed to parse float: {}", e),
    })
}

/// @stdlib_string fn string_trim(s: String) -> String
pub fn string_trim(s: &str) -> Result<String> {
    Ok(s.trim().to_string())
}

/// @stdlib_string fn string_replace(s: String, old: String, new: String) -> String
pub fn string_replace(s: &str, old: &str, new: &str) -> Result<String> {
    Ok(s.replace(old, new))
}

/// @stdlib_string fn string_index_of(s: String, substr: String) -> Int
pub fn string_index_of(s: &str, substr: &str) -> Result<i64> {
    Ok(s.find(substr).map(|i| i as i64).unwrap_or(-1))
}
