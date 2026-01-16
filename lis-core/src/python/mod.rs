//! # Python Bindings (PyO3)
//!
//! Python interface for LIS compiler and runtime.
//!
//! ## Usage (Python)
//!
//! ```python
//! import lis_core
//!
//! # Compile LIS to SIL assembly
//! source = """
//! fn main() {
//!     let x = 42;
//!     print(x);
//! }
//! """
//! assembly = lis_core.compile(source)
//! print(assembly)
//!
//! # Compile to JSIL
//! stats = lis_core.compile_to_jsil(source, "output.jsil")
//! print(f"Compressed: {stats.compression_ratio:.1%}")
//! ```

use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyIOError, PyTypeError};
use pyo3::types::PyDict;

use crate::{
    Compiler, Lexer, Parser, Program, Expr, Stmt, Item, Literal,
    Type, TypeKind, TypeChecker,
    Error, Result,
};

#[cfg(feature = "jsil")]
use crate::JsilStats;

// ============================================================================
// Compiler Bindings
// ============================================================================

/// Python wrapper for LIS Compiler
#[pyclass(name = "Compiler")]
pub struct PyCompiler {
    inner: Compiler,
}

#[pymethods]
impl PyCompiler {
    /// Create a new LIS compiler
    #[new]
    fn new() -> Self {
        Self {
            inner: Compiler::new(),
        }
    }

    /// Compile LIS source to SIL assembly
    ///
    /// Args:
    ///     source: LIS source code string
    ///
    /// Returns:
    ///     SIL assembly string
    fn compile(&mut self, source: &str) -> PyResult<String> {
        let tokens = Lexer::new(source)
            .tokenize()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let ast = Parser::new(tokens)
            .parse()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        self.inner
            .compile(&ast)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Compile LIS source to JSIL (compressed bytecode)
    ///
    /// Args:
    ///     source: LIS source code string
    ///     output_path: Path to output JSIL file
    ///     compression: Compression mode ("xor_rotate", "xor", "rotate", or "none")
    ///
    /// Returns:
    ///     JsilStats with compression information
    #[cfg(feature = "jsil")]
    #[pyo3(signature = (source, output_path, compression=None))]
    fn compile_to_jsil(
        &mut self,
        source: &str,
        output_path: &str,
        compression: Option<&str>,
    ) -> PyResult<PyJsilStats> {
        use sil_core::io::jsil::CompressionMode;

        let mode = match compression {
            Some("xor_rotate") | None => CompressionMode::XorRotate,
            Some("xor") => CompressionMode::Xor,
            Some("rotate") => CompressionMode::Rotate,
            Some("none") => CompressionMode::None,
            Some(other) => {
                return Err(PyValueError::new_err(format!(
                    "Unknown compression mode: {}. Valid: xor_rotate, xor, rotate, none",
                    other
                )))
            }
        };

        let stats = crate::compile_to_jsil(source, output_path, Some(mode))
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyJsilStats::from(stats))
    }

    /// Parse LIS source to AST (for inspection)
    ///
    /// Returns:
    ///     PyProgram AST object
    fn parse(&self, source: &str) -> PyResult<PyProgram> {
        let tokens = Lexer::new(source)
            .tokenize()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let ast = Parser::new(tokens)
            .parse()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyProgram::from(ast))
    }

    fn __repr__(&self) -> String {
        "Compiler()".to_string()
    }
}

// ============================================================================
// AST Bindings
// ============================================================================

/// Python wrapper for LIS Program AST
#[pyclass(name = "Program")]
#[derive(Clone)]
pub struct PyProgram {
    #[pyo3(get)]
    items: Vec<String>,
}

impl From<Program> for PyProgram {
    fn from(p: Program) -> Self {
        Self {
            items: p.items.iter().map(|item| format!("{:?}", item)).collect(),
        }
    }
}

#[pymethods]
impl PyProgram {
    fn __repr__(&self) -> String {
        format!("Program(items={})", self.items.len())
    }

    fn __str__(&self) -> String {
        format!("Program with {} items:\n{}", self.items.len(), self.items.join("\n"))
    }
}

// ============================================================================
// Type System Bindings
// ============================================================================

/// Python wrapper for Type
#[pyclass(name = "Type")]
#[derive(Clone)]
pub struct PyType {
    #[pyo3(get)]
    name: String,
}

#[pymethods]
impl PyType {
    fn __repr__(&self) -> String {
        format!("Type({})", self.name)
    }

    fn __str__(&self) -> String {
        self.name.clone()
    }
}

// ============================================================================
// JSIL Stats Bindings
// ============================================================================

/// Statistics from JSIL compilation
#[cfg(feature = "jsil")]
#[pyclass(name = "JsilStats")]
#[derive(Clone)]
pub struct PyJsilStats {
    #[pyo3(get)]
    uncompressed_size: usize,
    #[pyo3(get)]
    compressed_size: usize,
    #[pyo3(get)]
    record_count: usize,
    #[pyo3(get)]
    compression_ratio: f64,
}

#[cfg(feature = "jsil")]
impl From<JsilStats> for PyJsilStats {
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
#[pymethods]
impl PyJsilStats {
    fn report(&self) -> String {
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

    fn __repr__(&self) -> String {
        format!(
            "JsilStats(records={}, ratio={:.1}%)",
            self.record_count,
            self.compression_ratio * 100.0
        )
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Quick compile LIS to SIL assembly
///
/// Args:
///     source: LIS source code string
///
/// Returns:
///     SIL assembly string
#[pyfunction]
fn compile(source: &str) -> PyResult<String> {
    crate::compile(source).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Quick parse LIS source to AST
///
/// Args:
///     source: LIS source code string
///
/// Returns:
///     PyProgram AST object
#[pyfunction]
fn parse(source: &str) -> PyResult<PyProgram> {
    let ast = crate::parse(source).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyProgram::from(ast))
}

/// Compile LIS source to JSIL file
///
/// Args:
///     source: LIS source code string
///     output_path: Path to output JSIL file
///     compression: Optional compression mode
///
/// Returns:
///     JsilStats with compression information
#[cfg(feature = "jsil")]
#[pyfunction]
#[pyo3(signature = (source, output_path, compression=None))]
fn compile_to_jsil(
    source: &str,
    output_path: &str,
    compression: Option<&str>,
) -> PyResult<PyJsilStats> {
    use sil_core::io::jsil::CompressionMode;

    let mode = match compression {
        Some("xor_rotate") | None => CompressionMode::XorRotate,
        Some("xor") => CompressionMode::Xor,
        Some("rotate") => CompressionMode::Rotate,
        Some("none") => CompressionMode::None,
        Some(other) => {
            return Err(PyValueError::new_err(format!(
                "Unknown compression mode: {}. Valid: xor_rotate, xor, rotate, none",
                other
            )))
        }
    };

    let stats = crate::compile_to_jsil(source, output_path, Some(mode))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(PyJsilStats::from(stats))
}

/// Tokenize LIS source code
///
/// Args:
///     source: LIS source code string
///
/// Returns:
///     List of token strings
#[pyfunction]
fn tokenize(source: &str) -> PyResult<Vec<String>> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(tokens.iter().map(|t| format!("{:?}", t)).collect())
}

// ============================================================================
// Module Definition
// ============================================================================

/// lis-core Python module
#[pymodule]
fn lis_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Classes
    m.add_class::<PyCompiler>()?;
    m.add_class::<PyProgram>()?;
    m.add_class::<PyType>()?;

    #[cfg(feature = "jsil")]
    m.add_class::<PyJsilStats>()?;

    // Functions
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;

    #[cfg(feature = "jsil")]
    m.add_function(wrap_pyfunction!(compile_to_jsil, m)?)?;

    Ok(())
}
