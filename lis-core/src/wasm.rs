//! # WebAssembly Bindings (wasm-bindgen)
//!
//! JavaScript/TypeScript interface for LIS compiler and runtime.
//!
//! ## Usage (JavaScript/TypeScript)
//!
//! ```typescript
//! import { Compiler, compile, compileToJsil } from 'lis-core';
//!
//! // Quick compile
//! const source = `
//!   fn main() {
//!     let x = 42;
//!     print(x);
//!   }
//! `;
//! const assembly = compile(source);
//! console.log(assembly);
//!
//! // Using compiler instance
//! const compiler = new Compiler();
//! const ast = compiler.parse(source);
//! console.log(ast);
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    Compiler as RustCompiler, Lexer, Parser, Program,
    Error,
};

#[cfg(feature = "jsil")]
use crate::JsilStats;

// Set panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// Error Handling
// ============================================================================

fn to_js_error(e: Error) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// ============================================================================
// Compiler Bindings
// ============================================================================

/// LIS Compiler for WebAssembly
#[wasm_bindgen]
pub struct Compiler {
    inner: RustCompiler,
}

#[wasm_bindgen]
impl Compiler {
    /// Create a new LIS compiler
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RustCompiler::new(),
        }
    }

    /// Compile LIS source to SIL assembly
    ///
    /// # Arguments
    /// * `source` - LIS source code string
    ///
    /// # Returns
    /// SIL assembly string
    #[wasm_bindgen]
    pub fn compile(&mut self, source: &str) -> std::result::Result<String, JsValue> {
        let tokens = Lexer::new(source)
            .tokenize()
            .map_err(to_js_error)?;

        let ast = Parser::new(tokens)
            .parse()
            .map_err(to_js_error)?;

        self.inner
            .compile(&ast)
            .map_err(to_js_error)
    }

    /// Parse LIS source to AST
    ///
    /// # Arguments
    /// * `source` - LIS source code string
    ///
    /// # Returns
    /// Program AST as JSON
    #[wasm_bindgen]
    pub fn parse(&self, source: &str) -> std::result::Result<JsValue, JsValue> {
        let tokens = Lexer::new(source)
            .tokenize()
            .map_err(to_js_error)?;

        let ast = Parser::new(tokens)
            .parse()
            .map_err(to_js_error)?;

        let serialized = WasmProgram::from(ast);
        serde_wasm_bindgen::to_value(&serialized).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Tokenize LIS source code
    ///
    /// # Arguments
    /// * `source` - LIS source code string
    ///
    /// # Returns
    /// Array of token strings
    #[wasm_bindgen]
    pub fn tokenize(&self, source: &str) -> std::result::Result<Vec<String>, JsValue> {
        let tokens = Lexer::new(source)
            .tokenize()
            .map_err(to_js_error)?;

        Ok(tokens.iter().map(|t| format!("{:?}", t)).collect())
    }
}

// ============================================================================
// AST Bindings
// ============================================================================

/// WebAssembly-serializable Program AST
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmProgram {
    items: Vec<String>,
}

impl From<Program> for WasmProgram {
    fn from(p: Program) -> Self {
        Self {
            items: p.items.iter().map(|item| format!("{:?}", item)).collect(),
        }
    }
}

#[wasm_bindgen]
impl WasmProgram {
    /// Get number of items in the program
    #[wasm_bindgen(getter)]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get all items as JSON array
    #[wasm_bindgen(getter)]
    pub fn items(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.items).unwrap()
    }
}

// ============================================================================
// JSIL Stats Bindings
// ============================================================================

#[cfg(feature = "jsil")]
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmJsilStats {
    uncompressed_size: usize,
    compressed_size: usize,
    record_count: usize,
    compression_ratio: f64,
}

#[cfg(feature = "jsil")]
impl From<JsilStats> for WasmJsilStats {
    fn from(s: JsilStats) -> Self {
        Self {
            uncompressed_size: s.uncompressed_size,
            compressed_size: s.compressed_size,
            record_count: s.record_count,
            compression_ratio: s.compression_ratio,
        }
    }
}

#[cfg(feature = "jsil")]
#[wasm_bindgen]
impl WasmJsilStats {
    #[wasm_bindgen(getter)]
    pub fn uncompressed_size(&self) -> usize {
        self.uncompressed_size
    }

    #[wasm_bindgen(getter)]
    pub fn compressed_size(&self) -> usize {
        self.compressed_size
    }

    #[wasm_bindgen(getter)]
    pub fn record_count(&self) -> usize {
        self.record_count
    }

    #[wasm_bindgen(getter)]
    pub fn compression_ratio(&self) -> f64 {
        self.compression_ratio
    }

    /// Get formatted report
    #[wasm_bindgen]
    pub fn report(&self) -> String {
        format!(
            "JSIL Compilation Report:\n\
             - Records: {}\n\
             - Uncompressed: {} bytes\n\
             - Compressed: {} bytes\n\
             - Ratio: {:.1}%",
            self.record_count,
            self.uncompressed_size,
            self.compressed_size,
            self.compression_ratio * 100.0
        )
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Quick compile LIS to SIL assembly
///
/// # Arguments
/// * `source` - LIS source code string
///
/// # Returns
/// SIL assembly string
#[wasm_bindgen]
pub fn compile(source: &str) -> std::result::Result<String, JsValue> {
    crate::compile(source).map_err(to_js_error)
}

/// Quick parse LIS source to AST
///
/// # Arguments
/// * `source` - LIS source code string
///
/// # Returns
/// Program AST as JSON
#[wasm_bindgen]
pub fn parse(source: &str) -> std::result::Result<JsValue, JsValue> {
    let ast = crate::parse(source).map_err(to_js_error)?;
    let serialized = WasmProgram::from(ast);
    serde_wasm_bindgen::to_value(&serialized).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Tokenize LIS source code
///
/// # Arguments
/// * `source` - LIS source code string
///
/// # Returns
/// Array of token strings
#[wasm_bindgen]
pub fn tokenize(source: &str) -> std::result::Result<Vec<String>, JsValue> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(to_js_error)?;

    Ok(tokens.iter().map(|t| format!("{:?}", t)).collect())
}

// ============================================================================
// Version Information
// ============================================================================

/// Get LIS compiler version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================================================================
// Runtime Stdlib Bindings
// ============================================================================

use sil_core::state::ByteSil;

/// ByteSil wrapper for WASM
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmByteSil {
    inner: ByteSil,
}

#[wasm_bindgen]
impl WasmByteSil {
    /// Get the rho (log-magnitude) component
    #[wasm_bindgen(getter)]
    pub fn rho(&self) -> i8 {
        self.inner.rho
    }

    /// Get the theta (phase) component
    #[wasm_bindgen(getter)]
    pub fn theta(&self) -> u8 {
        self.inner.theta
    }

    /// Get the raw byte value (serialization)
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> u8 {
        self.inner.to_u8()
    }

    /// Check if this is null (zero)
    #[wasm_bindgen(getter)]
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// Convert to string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        format!("ByteSil(ρ={}, θ={})", self.inner.rho, self.inner.theta)
    }

    /// Convert to complex number (returns [real, imag])
    #[wasm_bindgen(js_name = toComplex)]
    pub fn to_complex_js(&self) -> Vec<f64> {
        let c = self.inner.to_complex();
        vec![c.re, c.im]
    }

    /// Get polar coordinates (returns [rho, theta_degrees])
    #[wasm_bindgen(js_name = toPolar)]
    pub fn to_polar_js(&self) -> Vec<f64> {
        let (rho, theta_deg) = self.inner.to_polar();
        vec![rho as f64, theta_deg as f64]
    }
}

// ============================================================================
// ByteSil Operations
// ============================================================================

/// Create a new ByteSil from rho and theta components
#[wasm_bindgen]
pub fn bytesil_new(rho: i8, theta: u8) -> WasmByteSil {
    WasmByteSil {
        inner: ByteSil::new(rho, theta),
    }
}

/// Create ByteSil from a raw byte value
#[wasm_bindgen]
pub fn bytesil_from_u8(byte: u8) -> WasmByteSil {
    WasmByteSil {
        inner: ByteSil::from_u8(byte),
    }
}

/// Create ByteSil from complex number (real, imaginary)
#[wasm_bindgen]
pub fn bytesil_from_complex(re: f64, im: f64) -> WasmByteSil {
    use num_complex::Complex;
    WasmByteSil {
        inner: ByteSil::from_complex(Complex::new(re, im)),
    }
}

/// Get the NULL ByteSil (origin, ~0)
#[wasm_bindgen]
pub fn bytesil_null() -> WasmByteSil {
    WasmByteSil { inner: ByteSil::NULL }
}

/// Get the ONE ByteSil (1 + 0i)
#[wasm_bindgen]
pub fn bytesil_one() -> WasmByteSil {
    WasmByteSil { inner: ByteSil::ONE }
}

/// Get the I ByteSil (0 + 1i)
#[wasm_bindgen]
pub fn bytesil_i() -> WasmByteSil {
    WasmByteSil { inner: ByteSil::I }
}

/// Get the NEG_ONE ByteSil (-1 + 0i)
#[wasm_bindgen]
pub fn bytesil_neg_one() -> WasmByteSil {
    WasmByteSil { inner: ByteSil::NEG_ONE }
}

/// Get the NEG_I ByteSil (0 - 1i)
#[wasm_bindgen]
pub fn bytesil_neg_i() -> WasmByteSil {
    WasmByteSil { inner: ByteSil::NEG_I }
}

/// Get the MAX ByteSil (e^7 + 0i)
#[wasm_bindgen]
pub fn bytesil_max() -> WasmByteSil {
    WasmByteSil { inner: ByteSil::MAX }
}

/// Multiply two ByteSil values (O(1) in log-polar)
#[wasm_bindgen]
pub fn bytesil_mul(a: &WasmByteSil, b: &WasmByteSil) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.mul(&b.inner),
    }
}

/// Divide two ByteSil values (O(1) in log-polar)
#[wasm_bindgen]
pub fn bytesil_div(a: &WasmByteSil, b: &WasmByteSil) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.div(&b.inner),
    }
}

/// Raise ByteSil to integer power (O(1) in log-polar)
#[wasm_bindgen]
pub fn bytesil_pow(a: &WasmByteSil, n: i32) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.pow(n),
    }
}

/// Take nth root of ByteSil (O(1) in log-polar)
#[wasm_bindgen]
pub fn bytesil_root(a: &WasmByteSil, n: i32) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.root(n),
    }
}

/// Invert ByteSil (1/z)
#[wasm_bindgen]
pub fn bytesil_inv(a: &WasmByteSil) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.inv(),
    }
}

/// Conjugate a ByteSil value (flip phase)
#[wasm_bindgen]
pub fn bytesil_conj(a: &WasmByteSil) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.conj(),
    }
}

/// XOR two ByteSil values (entropy-preserving)
#[wasm_bindgen]
pub fn bytesil_xor(a: &WasmByteSil, b: &WasmByteSil) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.xor(&b.inner),
    }
}

/// Mix two ByteSil values (average in log-polar space)
#[wasm_bindgen]
pub fn bytesil_mix(a: &WasmByteSil, b: &WasmByteSil) -> WasmByteSil {
    WasmByteSil {
        inner: a.inner.mix(&b.inner),
    }
}

/// Get norm (collapsed magnitude) of ByteSil (0-15)
#[wasm_bindgen]
pub fn bytesil_norm(a: &WasmByteSil) -> u8 {
    a.inner.norm()
}

/// Get phase index of ByteSil (0-15)
#[wasm_bindgen]
pub fn bytesil_phase(a: &WasmByteSil) -> u8 {
    a.inner.phase()
}

/// Get magnitude as f64 (e^rho)
#[wasm_bindgen]
pub fn bytesil_magnitude(a: &WasmByteSil) -> f64 {
    (a.inner.rho as f64).exp()
}

/// Get phase angle in radians
#[wasm_bindgen]
pub fn bytesil_phase_radians(a: &WasmByteSil) -> f64 {
    (a.inner.theta as f64) * std::f64::consts::PI / 8.0
}

/// Check if two ByteSil values are equal
#[wasm_bindgen]
pub fn bytesil_eq(a: &WasmByteSil, b: &WasmByteSil) -> bool {
    a.inner == b.inner
}

// ============================================================================
// Math Functions
// ============================================================================

/// Absolute value
#[wasm_bindgen]
pub fn math_abs(x: f64) -> f64 {
    x.abs()
}

/// Square root
#[wasm_bindgen]
pub fn math_sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Power
#[wasm_bindgen]
pub fn math_pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Natural logarithm
#[wasm_bindgen]
pub fn math_ln(x: f64) -> f64 {
    x.ln()
}

/// Logarithm base 10
#[wasm_bindgen]
pub fn math_log10(x: f64) -> f64 {
    x.log10()
}

/// Exponential (e^x)
#[wasm_bindgen]
pub fn math_exp(x: f64) -> f64 {
    x.exp()
}

/// Sine
#[wasm_bindgen]
pub fn math_sin(x: f64) -> f64 {
    x.sin()
}

/// Cosine
#[wasm_bindgen]
pub fn math_cos(x: f64) -> f64 {
    x.cos()
}

/// Tangent
#[wasm_bindgen]
pub fn math_tan(x: f64) -> f64 {
    x.tan()
}

/// Arc sine
#[wasm_bindgen]
pub fn math_asin(x: f64) -> f64 {
    x.asin()
}

/// Arc cosine
#[wasm_bindgen]
pub fn math_acos(x: f64) -> f64 {
    x.acos()
}

/// Arc tangent
#[wasm_bindgen]
pub fn math_atan(x: f64) -> f64 {
    x.atan()
}

/// Arc tangent of y/x
#[wasm_bindgen]
pub fn math_atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// Floor
#[wasm_bindgen]
pub fn math_floor(x: f64) -> f64 {
    x.floor()
}

/// Ceiling
#[wasm_bindgen]
pub fn math_ceil(x: f64) -> f64 {
    x.ceil()
}

/// Round
#[wasm_bindgen]
pub fn math_round(x: f64) -> f64 {
    x.round()
}

/// Minimum of two values
#[wasm_bindgen]
pub fn math_min(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Maximum of two values
#[wasm_bindgen]
pub fn math_max(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Clamp value between min and max
#[wasm_bindgen]
pub fn math_clamp(x: f64, min: f64, max: f64) -> f64 {
    x.clamp(min, max)
}

/// PI constant
#[wasm_bindgen]
pub fn math_pi() -> f64 {
    std::f64::consts::PI
}

/// E constant (Euler's number)
#[wasm_bindgen]
pub fn math_e() -> f64 {
    std::f64::consts::E
}
