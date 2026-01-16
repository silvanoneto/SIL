//! # Python Bindings (PyO3)
//!
//! Python interface for SIL-Core runtime.
//!
//! ## Usage (Python)
//!
//! ```python
//! import sil_core
//!
//! # Create ByteSil
//! b = sil_core.ByteSil(0, 4)  # Unit imaginary (i)
//! print(f"ByteSil: rho={b.rho}, theta={b.theta}")
//!
//! # Create SIL state
//! state = sil_core.SilState.neutral()
//! print(f"State: {state}")
//!
//! # Access layers
//! layer0 = state.get_layer(0)
//! print(f"Layer 0: {layer0}")
//! ```

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use crate::state::{ByteSil as RustByteSil, SilState as RustSilState, NUM_LAYERS};

// ============================================================================
// ByteSil Bindings
// ============================================================================

/// Python wrapper for ByteSil (complex number in log-polar form)
#[pyclass(name = "ByteSil")]
#[derive(Clone, Copy)]
pub struct PyByteSil {
    inner: RustByteSil,
}

#[pymethods]
impl PyByteSil {
    /// Create a new ByteSil
    ///
    /// Args:
    ///     rho: Log-magnitude [-8, 7]
    ///     theta: Phase index [0, 15]
    #[new]
    fn new(rho: i8, theta: u8) -> PyResult<Self> {
        if rho < -8 || rho > 7 {
            return Err(PyValueError::new_err("rho must be in range [-8, 7]"));
        }
        if theta > 15 {
            return Err(PyValueError::new_err("theta must be in range [0, 15]"));
        }
        Ok(Self {
            inner: RustByteSil::new(rho, theta),
        })
    }

    /// Create null ByteSil (zero state)
    #[staticmethod]
    fn null() -> Self {
        Self {
            inner: RustByteSil::null(),
        }
    }

    /// Create unit ByteSil (1 + 0i)
    #[staticmethod]
    fn one() -> Self {
        Self {
            inner: RustByteSil::ONE,
        }
    }

    /// Create imaginary unit (0 + 1i)
    #[staticmethod]
    fn i() -> Self {
        Self {
            inner: RustByteSil::I,
        }
    }

    /// Create negative one (-1 + 0i)
    #[staticmethod]
    fn neg_one() -> Self {
        Self {
            inner: RustByteSil::NEG_ONE,
        }
    }

    /// Create ByteSil from complex coordinates
    #[staticmethod]
    fn from_complex(real: f64, imag: f64) -> Self {
        use num_complex::Complex;
        Self {
            inner: RustByteSil::from_complex(Complex::new(real, imag)),
        }
    }

    /// Create from single byte (packed format)
    #[staticmethod]
    fn from_u8(byte: u8) -> Self {
        Self {
            inner: RustByteSil::from_u8(byte),
        }
    }

    /// Get log-magnitude (rho)
    #[getter]
    fn rho(&self) -> i8 {
        self.inner.rho
    }

    /// Get phase index (theta)
    #[getter]
    fn theta(&self) -> u8 {
        self.inner.theta
    }

    /// Get magnitude in linear scale (e^rho)
    fn magnitude(&self) -> f64 {
        (self.inner.rho as f64).exp()
    }

    /// Get phase in radians
    fn phase_radians(&self) -> f64 {
        (self.inner.theta as f64) * std::f64::consts::PI / 8.0
    }

    /// Convert to complex number (real, imag)
    fn to_complex(&self) -> (f64, f64) {
        let c = self.inner.to_complex();
        (c.re, c.im)
    }

    /// Convert to packed byte
    fn to_u8(&self) -> u8 {
        self.inner.to_u8()
    }

    /// Check if null
    fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// Multiply two ByteSil values
    fn mul(&self, other: &PyByteSil) -> PyByteSil {
        PyByteSil {
            inner: self.inner.mul(&other.inner),
        }
    }

    fn __mul__(&self, other: &PyByteSil) -> PyByteSil {
        self.mul(other)
    }

    /// Divide two ByteSil values
    fn div(&self, other: &PyByteSil) -> PyByteSil {
        PyByteSil {
            inner: self.inner.div(&other.inner),
        }
    }

    fn __truediv__(&self, other: &PyByteSil) -> PyByteSil {
        self.div(other)
    }

    /// Power operation
    fn pow(&self, n: i32) -> PyByteSil {
        PyByteSil {
            inner: self.inner.pow(n),
        }
    }

    /// XOR two ByteSil values
    fn xor(&self, other: &PyByteSil) -> PyByteSil {
        PyByteSil {
            inner: self.inner.xor(&other.inner),
        }
    }

    fn __xor__(&self, other: &PyByteSil) -> PyByteSil {
        self.xor(other)
    }

    /// Conjugate (invert phase)
    fn conj(&self) -> PyByteSil {
        PyByteSil {
            inner: self.inner.conj(),
        }
    }

    /// Inverse (1/z)
    fn inv(&self) -> PyByteSil {
        PyByteSil {
            inner: self.inner.inv(),
        }
    }

    /// Mix two ByteSil values
    fn mix(&self, other: &PyByteSil) -> PyByteSil {
        PyByteSil {
            inner: self.inner.mix(&other.inner),
        }
    }

    fn __repr__(&self) -> String {
        format!("ByteSil(rho={}, theta={})", self.inner.rho, self.inner.theta)
    }

    fn __str__(&self) -> String {
        format!("(ρ={}, θ={})", self.inner.rho, self.inner.theta)
    }
}

// ============================================================================
// SilState Bindings
// ============================================================================

/// Python wrapper for SilState (16-layer complex vector)
#[pyclass(name = "SilState")]
#[derive(Clone)]
pub struct PySilState {
    inner: RustSilState,
}

#[pymethods]
impl PySilState {
    /// Create neutral SilState (all layers = 1 + 0i)
    #[staticmethod]
    fn neutral() -> Self {
        Self {
            inner: RustSilState::neutral(),
        }
    }

    /// Create vacuum SilState (all layers = null)
    #[staticmethod]
    fn vacuum() -> Self {
        Self {
            inner: RustSilState::vacuum(),
        }
    }

    /// Get a specific layer
    fn get_layer(&self, index: usize) -> PyResult<PyByteSil> {
        if index >= NUM_LAYERS {
            return Err(PyValueError::new_err(format!(
                "Layer index {} out of range [0, {})",
                index, NUM_LAYERS
            )));
        }
        Ok(PyByteSil {
            inner: self.inner.layers[index],
        })
    }

    /// Set a specific layer (mutates in place)
    fn set_layer(&mut self, index: usize, value: &PyByteSil) -> PyResult<()> {
        if index >= NUM_LAYERS {
            return Err(PyValueError::new_err(format!(
                "Layer index {} out of range [0, {})",
                index, NUM_LAYERS
            )));
        }
        self.inner.layers[index] = value.inner;
        Ok(())
    }

    /// Set a specific layer (returns new state)
    fn with_layer(&self, index: usize, value: &PyByteSil) -> PyResult<Self> {
        if index >= NUM_LAYERS {
            return Err(PyValueError::new_err(format!(
                "Layer index {} out of range [0, {})",
                index, NUM_LAYERS
            )));
        }
        Ok(Self {
            inner: self.inner.with_layer(index, value.inner),
        })
    }

    /// Get all layers as list
    fn get_all_layers(&self) -> Vec<PyByteSil> {
        self.inner
            .layers
            .iter()
            .map(|&bs| PyByteSil { inner: bs })
            .collect()
    }

    /// Get hash of state (lower 64 bits)
    fn hash(&self) -> u64 {
        self.inner.hash() as u64
    }

    fn __repr__(&self) -> String {
        format!("SilState(hash={})", self.inner.hash())
    }

    fn __str__(&self) -> String {
        let layers: Vec<String> = self
            .inner
            .layers
            .iter()
            .enumerate()
            .map(|(i, bs)| format!("L{:X}: (ρ={}, θ={})", i, bs.rho, bs.theta))
            .collect();
        format!("SilState[\n  {}\n]", layers.join(",\n  "))
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Get the number of layers in a SilState
#[pyfunction]
fn get_num_layers() -> usize {
    NUM_LAYERS
}

/// Create a neutral state
#[pyfunction]
fn create_state() -> PySilState {
    PySilState::neutral()
}

/// Create a ByteSil from rho and theta
#[pyfunction]
fn create_bytesil(rho: i8, theta: u8) -> PyResult<PyByteSil> {
    PyByteSil::new(rho, theta)
}

// ============================================================================
// Module Definition
// ============================================================================

/// sil_core Python module
#[pymodule]
#[pyo3(name = "_sil_core")]
fn _sil_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("NUM_LAYERS", NUM_LAYERS)?;

    // Classes
    m.add_class::<PyByteSil>()?;
    m.add_class::<PySilState>()?;

    // Functions
    m.add_function(wrap_pyfunction!(get_num_layers, m)?)?;
    m.add_function(wrap_pyfunction!(create_state, m)?)?;
    m.add_function(wrap_pyfunction!(create_bytesil, m)?)?;

    Ok(())
}
