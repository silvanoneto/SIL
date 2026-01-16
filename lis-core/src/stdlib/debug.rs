//! Debug and Profiling Utilities for LIS
//!
//! Development and performance analysis tools.

use crate::error::{Error, Result};
use sil_core::state::{ByteSil, SilState};
use std::time::{SystemTime, UNIX_EPOCH};

/// @stdlib_debug fn assert(condition: Bool, message: String)
///
/// Asserts that a condition is true, panics with message if false.
pub fn assert(condition: bool, message: &str) -> Result<()> {
    if !condition {
        return Err(Error::SemanticError {
            message: format!("Assertion failed: {}", message),
        });
    }
    Ok(())
}

/// @stdlib_debug fn assert_eq_int(a: Int, b: Int, message: String)
///
/// Asserts that two integers are equal.
pub fn assert_eq_int(a: i64, b: i64, message: &str) -> Result<()> {
    if a != b {
        return Err(Error::SemanticError {
            message: format!("Assertion failed: {} != {} - {}", a, b, message),
        });
    }
    Ok(())
}

/// @stdlib_debug fn assert_eq_bytesil(a: ByteSil, b: ByteSil, message: String)
///
/// Asserts that two ByteSil values are equal.
pub fn assert_eq_bytesil(a: &ByteSil, b: &ByteSil, message: &str) -> Result<()> {
    if a != b {
        return Err(Error::SemanticError {
            message: format!("Assertion failed: ByteSil values not equal - {}", message),
        });
    }
    Ok(())
}

/// @stdlib_debug fn assert_eq_state(a: State, b: State, message: String)
///
/// Asserts that two states are equal.
pub fn assert_eq_state(a: &SilState, b: &SilState, message: &str) -> Result<()> {
    if a != b {
        return Err(Error::SemanticError {
            message: format!("Assertion failed: States not equal - {}", message),
        });
    }
    Ok(())
}

/// @stdlib_debug fn debug_print(message: String, value: ByteSil)
///
/// Prints a debug message with a ByteSil value.
pub fn debug_print(message: &str, value: &ByteSil) -> Result<()> {
    let c = value.to_complex(); let (re, im) = (c.re, c.im);
    println!("[DEBUG] {}: ({:.3} + {:.3}i)", message, re, im);
    Ok(())
}

/// @stdlib_debug fn trace_state(label: String, s: State)
///
/// Traces a state with a label for debugging.
pub fn trace_state(label: &str, s: &SilState) -> Result<()> {
    println!("[TRACE] {}:", label);
    for i in 0..16 {
        let layer = s.layer(i);
        let c = layer.to_complex(); let (re, im) = (c.re, c.im);
        println!("  L{:X}: ({:.3} + {:.3}i)", i, re, im);
    }
    Ok(())
}

/// @stdlib_debug fn timestamp_millis() -> Int
///
/// Returns the current Unix timestamp in milliseconds.
pub fn timestamp_millis() -> Result<u64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::SemanticError {
            message: format!("Failed to get timestamp: {}", e),
        })?;
    Ok(now.as_millis() as u64)
}

/// @stdlib_debug fn timestamp_micros() -> Int
///
/// Returns the current Unix timestamp in microseconds.
pub fn timestamp_micros() -> Result<u64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::SemanticError {
            message: format!("Failed to get timestamp: {}", e),
        })?;
    Ok(now.as_micros() as u64)
}

/// @stdlib_debug fn sleep_millis(duration: Int)
///
/// Sleeps for the specified number of milliseconds.
pub fn sleep_millis(duration: u64) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(duration));
    Ok(())
}

/// @stdlib_debug fn memory_used() -> Int
///
/// Returns an estimate of memory usage (placeholder).
pub fn memory_used() -> Result<usize> {
    // Placeholder - would need platform-specific implementation
    Ok(0)
}
