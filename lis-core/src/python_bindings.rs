// Python bindings for LIS using PyO3

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};

use crate::compile as compile_lis;
use sil_core::state::{ByteSil as ByteSilCore, SilState as SilStateCore};

/// Python wrapper for ByteSil
#[pyclass]
#[derive(Clone)]
pub struct ByteSil {
    inner: ByteSilCore,
}

#[pymethods]
impl ByteSil {
    #[new]
    pub fn new(rho: i8, theta: u8) -> PyResult<Self> {
        if rho < -8 || rho > 7 {
            return Err(PyValueError::new_err("rho must be in range -8 to 7"));
        }
        if theta > 15 {
            return Err(PyValueError::new_err("theta must be in range 0-15"));
        }
        Ok(Self {
            inner: ByteSilCore::new(rho, theta),
        })
    }

    #[getter]
    pub fn rho(&self) -> i8 {
        self.inner.rho
    }

    #[getter]
    pub fn theta(&self) -> u8 {
        self.inner.theta
    }

    pub fn __repr__(&self) -> String {
        format!("ByteSil(rho={}, theta={})", self.inner.rho, self.inner.theta)
    }

    pub fn __str__(&self) -> String {
        format!("(ρ={}, θ={})", self.inner.rho, self.inner.theta)
    }

    /// Get magnitude in linear scale (e^rho)
    pub fn magnitude(&self) -> f64 {
        (self.inner.rho as f64).exp()
    }

    /// Get phase in radians
    pub fn phase(&self) -> f64 {
        (self.inner.theta as f64) * std::f64::consts::PI / 8.0
    }

    /// Multiply two ByteSil values (log-polar arithmetic)
    pub fn __mul__(&self, other: &ByteSil) -> Self {
        Self {
            inner: self.inner.mul(&other.inner),
        }
    }

    /// Divide two ByteSil values
    pub fn __truediv__(&self, other: &ByteSil) -> Self {
        Self {
            inner: self.inner.div(&other.inner),
        }
    }

    /// XOR two ByteSil values
    pub fn __xor__(&self, other: &ByteSil) -> Self {
        Self {
            inner: self.inner.xor(&other.inner),
        }
    }

    /// Conjugate (invert phase)
    pub fn conjugate(&self) -> Self {
        Self {
            inner: self.inner.conj(),
        }
    }

    /// Inverse (1/z)
    pub fn inverse(&self) -> Self {
        Self {
            inner: self.inner.inv(),
        }
    }

    /// Power (z^n)
    pub fn power(&self, n: i32) -> Self {
        Self {
            inner: self.inner.pow(n),
        }
    }

    /// Add phase (useful for rotations)
    pub fn rotate(&self, phase_delta: i32) -> Self {
        let new_theta = ((self.inner.theta as i32 + phase_delta).rem_euclid(16)) as u8;
        Self {
            inner: ByteSilCore::new(self.inner.rho, new_theta),
        }
    }

    /// Check if this is the null value
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// Convert to a single byte
    pub fn to_u8(&self) -> u8 {
        self.inner.to_u8()
    }

    /// Create from a single byte
    #[staticmethod]
    pub fn from_u8(byte: u8) -> Self {
        Self {
            inner: ByteSilCore::from_u8(byte),
        }
    }

    /// Create null ByteSil (zero state)
    #[staticmethod]
    pub fn null() -> Self {
        Self {
            inner: ByteSilCore::null(),
        }
    }

    /// Create unit ByteSil (1 + 0i)
    #[staticmethod]
    pub fn one() -> Self {
        Self {
            inner: ByteSilCore::ONE,
        }
    }

    /// Create imaginary unit (0 + 1i)
    #[staticmethod]
    pub fn i() -> Self {
        Self {
            inner: ByteSilCore::I,
        }
    }
}

/// Python wrapper for SilState
#[pyclass]
#[derive(Clone)]
pub struct SilState {
    inner: SilStateCore,
}

#[pymethods]
impl SilState {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: SilStateCore::neutral(),
        }
    }

    #[staticmethod]
    pub fn neutral() -> Self {
        Self {
            inner: SilStateCore::neutral(),
        }
    }

    #[staticmethod]
    pub fn vacuum() -> Self {
        Self {
            inner: SilStateCore::vacuum(),
        }
    }

    /// Get ByteSil from a specific layer
    pub fn get_layer(&self, layer: u8) -> PyResult<ByteSil> {
        if layer > 15 {
            return Err(PyValueError::new_err("layer must be in range 0-15"));
        }
        Ok(ByteSil {
            inner: self.inner.layers[layer as usize],
        })
    }

    /// Set ByteSil in a specific layer
    pub fn set_layer(&mut self, layer: u8, value: &ByteSil) -> PyResult<()> {
        if layer > 15 {
            return Err(PyValueError::new_err("layer must be in range 0-15"));
        }
        self.inner.layers[layer as usize] = value.inner;
        Ok(())
    }

    /// Get all 16 layers as a list
    pub fn get_all_layers(&self) -> Vec<ByteSil> {
        self.inner
            .layers
            .iter()
            .map(|&bs| ByteSil { inner: bs })
            .collect()
    }

    /// Get hash of the state (returns lower 64 bits)
    pub fn hash(&self) -> u64 {
        self.inner.hash() as u64
    }

    pub fn __repr__(&self) -> String {
        format!("SilState(hash={})", self.inner.hash())
    }

    pub fn __str__(&self) -> String {
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

/// Compilation result
#[pyclass]
pub struct CompileResult {
    #[pyo3(get)]
    pub assembly: String,
    #[pyo3(get)]
    pub bytecode: Vec<u8>,
    #[pyo3(get)]
    pub success: bool,
}

#[pymethods]
impl CompileResult {
    pub fn __repr__(&self) -> String {
        format!(
            "CompileResult(success={}, bytecode_size={})",
            self.success,
            self.bytecode.len()
        )
    }
}

/// LIS Compiler
#[pyclass]
pub struct Compiler {}

#[pymethods]
impl Compiler {
    #[new]
    pub fn new() -> Self {
        Self {}
    }

    /// Compile LIS source code to assembly
    pub fn compile(&self, source: String) -> PyResult<CompileResult> {
        match compile_lis(&source) {
            Ok(assembly) => {
                // In a real implementation, we'd assemble to bytecode here
                Ok(CompileResult {
                    assembly: assembly.clone(),
                    bytecode: assembly.as_bytes().to_vec(),
                    success: true,
                })
            }
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Compilation failed: {}",
                e
            ))),
        }
    }
}

/// LIS Runtime
#[pyclass]
pub struct Runtime {}

#[pymethods]
impl Runtime {
    #[new]
    pub fn new() -> Self {
        Self {}
    }

    /// Execute bytecode and return final state
    pub fn execute(&self, _bytecode: Vec<u8>) -> PyResult<SilState> {
        // Placeholder: In a real implementation, we'd execute the VSP bytecode
        Ok(SilState::neutral())
    }
}

/// Convenience function to compile LIS code
#[pyfunction]
fn compile_lis_code(source: &str) -> PyResult<String> {
    compile_lis(source).map_err(|e| PyRuntimeError::new_err(format!("Compilation failed: {}", e)))
}

/// Create a ByteSil from rho and theta
#[pyfunction]
fn create_bytesil(rho: i8, theta: u8) -> PyResult<ByteSil> {
    ByteSil::new(rho, theta)
}

/// Create a neutral SilState
#[pyfunction]
fn create_state() -> SilState {
    SilState::neutral()
}

/// Python module initialization
#[pymodule]
#[pyo3(name = "_lis_core")]
fn _lis_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<ByteSil>()?;
    m.add_class::<SilState>()?;

    // Compiler
    m.add_class::<Compiler>()?;
    m.add_class::<CompileResult>()?;

    // Runtime
    m.add_class::<Runtime>()?;

    // Convenience functions
    m.add_function(wrap_pyfunction!(compile_lis_code, m)?)?;
    m.add_function(wrap_pyfunction!(create_bytesil, m)?)?;
    m.add_function(wrap_pyfunction!(create_state, m)?)?;

    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
