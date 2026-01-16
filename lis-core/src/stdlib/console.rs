//! Console I/O Operations for LIS
//!
//! Functions for printing to stdout and reading from stdin.

use crate::error::Result;
use sil_core::state::{ByteSil, SilState};
use std::io::{self, Write};

/// @stdlib_io fn print_int(x: Int)
///
/// Prints an integer to stdout (without newline).
pub fn print_int(x: i64) -> Result<()> {
    print!("{}", x);
    io::stdout().flush().ok();
    Ok(())
}

/// @stdlib_io fn print_float(x: Float)
///
/// Prints a float to stdout (without newline).
pub fn print_float(x: f64) -> Result<()> {
    print!("{}", x);
    io::stdout().flush().ok();
    Ok(())
}

/// @stdlib_io fn print_string(s: String)
///
/// Prints a string to stdout (without newline).
pub fn print_string(s: &str) -> Result<()> {
    print!("{}", s);
    io::stdout().flush().ok();
    Ok(())
}

/// @stdlib_io fn print_bool(b: Bool)
///
/// Prints a boolean to stdout (without newline).
pub fn print_bool(b: bool) -> Result<()> {
    print!("{}", b);
    io::stdout().flush().ok();
    Ok(())
}

/// @stdlib_io fn print_bytesil(b: ByteSil)
///
/// Prints a ByteSil value to stdout in human-readable format.
pub fn print_bytesil(b: &ByteSil) -> Result<()> {
    let c = b.to_complex(); let (re, im) = (c.re, c.im);
    print!("ByteSil({:.3} + {:.3}i)", re, im);
    io::stdout().flush().ok();
    Ok(())
}

/// @stdlib_io fn print_state(s: State)
///
/// Prints a State to stdout showing all 16 layers.
pub fn print_state(s: &SilState) -> Result<()> {
    println!("State:");
    for i in 0..16 {
        let layer = s.layer(i);
        let c = layer.to_complex(); let (re, im) = (c.re, c.im);
        println!("  L{:X}: ({:.3} + {:.3}i)", i, re, im);
    }
    Ok(())
}

/// @stdlib_io fn println(s: String)
///
/// Prints a string followed by a newline.
pub fn println(s: &str) -> Result<()> {
    println!("{}", s);
    Ok(())
}

/// @stdlib_io fn read_line() -> String
///
/// Reads a line from stdin.
pub fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| crate::error::Error::IoError {
            message: format!("Failed to read line: {}", e),
        })?;
    Ok(input.trim().to_string())
}

/// @stdlib_io fn read_int() -> Int
///
/// Reads an integer from stdin.
pub fn read_int() -> Result<i64> {
    let input = read_line()?;
    input.parse().map_err(|e| crate::error::Error::IoError {
        message: format!("Failed to parse integer: {}", e),
    })
}

/// @stdlib_io fn read_float() -> Float
///
/// Reads a float from stdin.
pub fn read_float() -> Result<f64> {
    let input = read_line()?;
    input.parse().map_err(|e| crate::error::Error::IoError {
        message: format!("Failed to parse float: {}", e),
    })
}
