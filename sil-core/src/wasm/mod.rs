//! # WebAssembly Bindings (wasm-bindgen)
//!
//! JavaScript/TypeScript interface for SIL-Core runtime and VSP.
//!
//! ## Usage (JavaScript/TypeScript)
//!
//! ```typescript
//! import { SilState, ByteSil, Vsp } from 'sil-core';
//!
//! // Create SIL state
//! const state = SilState.neutral();
//! console.log(state);
//!
//! // Create VSP and execute bytecode
//! const vsp = new Vsp();
//! vsp.load(bytecode);
//! const result = vsp.run();
//! console.log(result);
//! ```

use wasm_bindgen::prelude::*;

use crate::{
    ByteSil as RustByteSil, SilState as RustSilState, NUM_LAYERS,
    Vsp as RustVsp, VspConfig,
    vsp::SilcFile,
};

// Set panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// Error Handling
// ============================================================================

fn to_js_error<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// ============================================================================
// ByteSil Bindings
// ============================================================================

/// ByteSil - Complex number in log-polar form (WebAssembly)
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct ByteSil {
    inner: RustByteSil,
}

#[wasm_bindgen]
impl ByteSil {
    /// Create a new ByteSil
    ///
    /// # Arguments
    /// * `rho` - Log-magnitude ∈ [-8, 7]
    /// * `theta` - Phase index ∈ [0, 15] (maps to [0, 2π))
    #[wasm_bindgen(constructor)]
    pub fn new(rho: i8, theta: u8) -> Self {
        Self {
            inner: RustByteSil::new(rho, theta),
        }
    }

    /// Create null ByteSil (ρ=-8, θ=0 ≈ 0+0i)
    #[wasm_bindgen]
    pub fn null() -> Self {
        Self {
            inner: RustByteSil::null(),
        }
    }

    /// Create one ByteSil (ρ=0, θ=0 = 1+0i)
    #[wasm_bindgen]
    pub fn one() -> Self {
        Self {
            inner: RustByteSil::ONE,
        }
    }

    /// Create ByteSil from Cartesian coordinates
    ///
    /// # Arguments
    /// * `real` - Real part
    /// * `imag` - Imaginary part
    #[wasm_bindgen(js_name = fromCartesian)]
    pub fn from_cartesian(real: f64, imag: f64) -> Self {
        Self {
            inner: RustByteSil::from_complex(num_complex::Complex::new(real, imag)),
        }
    }

    /// Get log-magnitude (rho) ∈ [-8, 7]
    #[wasm_bindgen(getter)]
    pub fn rho(&self) -> i8 {
        self.inner.rho
    }

    /// Get phase index (theta) ∈ [0, 15]
    #[wasm_bindgen(getter)]
    pub fn theta(&self) -> u8 {
        self.inner.theta
    }

    /// Convert to complex number [real, imag]
    #[wasm_bindgen(js_name = toComplex)]
    pub fn to_complex(&self) -> Vec<f64> {
        let c = self.inner.to_complex();
        vec![c.re, c.im]
    }

    /// Multiply two ByteSil values (O(1) log-polar multiply)
    #[wasm_bindgen]
    pub fn mul(&self, other: &ByteSil) -> ByteSil {
        ByteSil {
            inner: self.inner.mul(&other.inner),
        }
    }

    /// Divide two ByteSil values (O(1) log-polar divide)
    #[wasm_bindgen]
    pub fn div(&self, other: &ByteSil) -> ByteSil {
        ByteSil {
            inner: self.inner.div(&other.inner),
        }
    }

    /// Power operation (O(1) log-polar power)
    #[wasm_bindgen]
    pub fn pow(&self, exponent: i32) -> ByteSil {
        ByteSil {
            inner: self.inner.pow(exponent),
        }
    }

    /// String representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        let c = self.inner.to_complex();
        format!("{:.3} + {:.3}i", c.re, c.im)
    }
}

// ============================================================================
// SilState Bindings
// ============================================================================

/// SilState - 16-layer complex vector (WebAssembly)
#[wasm_bindgen]
#[derive(Clone)]
pub struct SilState {
    inner: RustSilState,
}

#[wasm_bindgen]
impl SilState {
    /// Create a new SilState from array of ByteSil values
    ///
    /// # Arguments
    /// * `layers` - Array of 16 ByteSil objects
    #[wasm_bindgen(constructor)]
    pub fn new(layers: Vec<ByteSil>) -> std::result::Result<SilState, JsValue> {
        if layers.len() != NUM_LAYERS {
            return Err(JsValue::from_str(&format!(
                "Expected {} layers, got {}",
                NUM_LAYERS,
                layers.len()
            )));
        }

        let mut rust_layers = [RustByteSil::NULL; NUM_LAYERS];
        for (i, layer) in layers.iter().enumerate() {
            rust_layers[i] = layer.inner;
        }

        Ok(Self {
            inner: RustSilState::from_layers(rust_layers),
        })
    }

    /// Create neutral SilState (all layers = 1 + 0i)
    #[wasm_bindgen]
    pub fn neutral() -> Self {
        Self {
            inner: RustSilState::neutral(),
        }
    }

    /// Create vacuum SilState (all layers = null ≈ 0)
    #[wasm_bindgen]
    pub fn vacuum() -> Self {
        Self {
            inner: RustSilState::vacuum(),
        }
    }

    /// Get a specific layer
    ///
    /// # Arguments
    /// * `index` - Layer index [0, 15]
    #[wasm_bindgen(js_name = getLayer)]
    pub fn get_layer(&self, index: usize) -> std::result::Result<ByteSil, JsValue> {
        if index >= NUM_LAYERS {
            return Err(JsValue::from_str(&format!(
                "Layer index {} out of range [0, {})",
                index, NUM_LAYERS
            )));
        }
        Ok(ByteSil {
            inner: self.inner.get(index),
        })
    }

    /// Set a specific layer (returns new state)
    ///
    /// # Arguments
    /// * `index` - Layer index [0, 15]
    /// * `value` - ByteSil value
    #[wasm_bindgen(js_name = withLayer)]
    pub fn with_layer(&self, index: usize, value: ByteSil) -> std::result::Result<SilState, JsValue> {
        if index >= NUM_LAYERS {
            return Err(JsValue::from_str(&format!(
                "Layer index {} out of range [0, {})",
                index, NUM_LAYERS
            )));
        }
        Ok(Self {
            inner: self.inner.with_layer(index, value.inner),
        })
    }

    /// Get all layers as array
    #[wasm_bindgen]
    pub fn layers(&self) -> Vec<ByteSil> {
        (0..NUM_LAYERS)
            .map(|i| ByteSil {
                inner: self.inner.get(i),
            })
            .collect()
    }

    /// Convert to JSON-serializable format
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> std::result::Result<JsValue, JsValue> {
        let layers: Vec<[i8; 2]> = (0..NUM_LAYERS)
            .map(|i| {
                let layer = self.inner.get(i);
                [layer.rho, layer.theta as i8]
            })
            .collect();

        serde_wasm_bindgen::to_value(&layers).map_err(to_js_error)
    }

    /// String representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        let layers: Vec<String> = (0..NUM_LAYERS)
            .map(|i| {
                let layer = self.inner.get(i);
                let c = layer.to_complex();
                format!("L{:X}: {:.2}+{:.2}i", i, c.re, c.im)
            })
            .collect();
        format!("SilState[\n  {}\n]", layers.join("\n  "))
    }
}

// ============================================================================
// VSP Bindings
// ============================================================================

/// Virtual SIL Processor (WebAssembly)
#[wasm_bindgen]
pub struct Vsp {
    inner: RustVsp,
}

#[wasm_bindgen]
impl Vsp {
    /// Create a new VSP with default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> std::result::Result<Vsp, JsValue> {
        let config = VspConfig::default();
        let inner = RustVsp::new(config).map_err(to_js_error)?;
        Ok(Self { inner })
    }

    /// Load bytecode (.silc format)
    ///
    /// # Arguments
    /// * `bytecode` - Uint8Array of .silc bytecode
    #[wasm_bindgen]
    pub fn load(&mut self, bytecode: &[u8]) -> std::result::Result<(), JsValue> {
        self.inner.load(bytecode).map_err(to_js_error)
    }

    /// Run program to completion
    ///
    /// # Returns
    /// Final SilState after execution
    #[wasm_bindgen]
    pub fn run(&mut self) -> std::result::Result<SilState, JsValue> {
        let result = self.inner.run().map_err(to_js_error)?;
        Ok(SilState { inner: result })
    }

    /// Execute a single step
    ///
    /// # Returns
    /// True if execution can continue, false if halted
    #[wasm_bindgen]
    pub fn step(&mut self) -> std::result::Result<bool, JsValue> {
        self.inner.step().map_err(to_js_error)
    }

    /// Reset VSP to initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get cycle count
    #[wasm_bindgen]
    pub fn cycles(&self) -> u32 {
        self.inner.cycles() as u32
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Assemble SIL assembly to bytecode (.silc format)
///
/// # Arguments
/// * `assembly` - SIL assembly string
///
/// # Returns
/// Bytecode as Uint8Array
#[wasm_bindgen]
pub fn assemble(assembly: &str) -> std::result::Result<Vec<u8>, JsValue> {
    let silc = crate::assemble(assembly).map_err(to_js_error)?;
    Ok(silc.to_bytes())
}

/// Disassemble bytecode to SIL assembly
///
/// # Arguments
/// * `bytecode` - Bytecode as Uint8Array
///
/// # Returns
/// SIL assembly string
#[wasm_bindgen]
pub fn disassemble(bytecode: &[u8]) -> std::result::Result<String, JsValue> {
    let silc = SilcFile::from_bytes(bytecode).map_err(to_js_error)?;
    Ok(crate::disassemble(&silc.code))
}

/// Get number of layers in SilState
#[wasm_bindgen(js_name = getNumLayers)]
pub fn get_num_layers() -> usize {
    NUM_LAYERS
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
