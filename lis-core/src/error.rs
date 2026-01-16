//! Error types for LIS compiler

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// Collection of multiple errors for error recovery
#[derive(Debug, Clone, Default)]
pub struct Errors {
    errors: Vec<Error>,
}

impl Errors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add an error to the collection
    pub fn push(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Get all errors
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Convert to Result - Ok if no errors, Err with first error otherwise
    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            // Return the first error (most common case)
            Err(self.errors.into_iter().next().unwrap())
        }
    }

    /// Convert to MultiError if there are multiple errors
    pub fn into_multi_result<T>(self, value: T) -> std::result::Result<T, Self> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Found {} error(s):", self.errors.len())?;
        for (i, error) in self.errors.iter().enumerate() {
            writeln!(f, "\n[{}] {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl std::error::Error for Errors {}

impl From<Error> for Errors {
    fn from(error: Error) -> Self {
        let mut errors = Errors::new();
        errors.push(error);
        errors
    }
}

impl IntoIterator for Errors {
    type Item = Error;
    type IntoIter = std::vec::IntoIter<Error>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Lexical error during tokenization
    LexError { message: String, line: usize, col: usize },

    /// Syntax error during parsing
    ParseError { message: String, line: usize, col: usize },

    /// Semantic error (type checking, undefined variables, etc.)
    SemanticError { message: String },

    /// Type error with rich diagnostics
    TypeError {
        kind: TypeErrorKind,
        line: usize,
        col: usize,
        help: Option<String>,
    },

    /// Code generation error
    CodeGenError { message: String },

    /// I/O error
    IoError { message: String },

    /// Manifest (lis.toml) error
    Manifest(String),

    /// Module resolution error
    ModuleError { message: String, path: Option<String> },
}

/// Specific type error kinds for better diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum TypeErrorKind {
    /// Type mismatch: expected one type, found another
    Mismatch {
        expected: String,
        found: String,
        context: String,
    },

    /// Undefined variable or function
    UndefinedVariable { name: String },

    /// Invalid layer access (layer out of bounds L0-LF)
    InvalidLayerAccess { layer: u8 },

    /// Hardware hint conflict
    HardwareConflict {
        required: String,
        found: String,
    },

    /// Infinite type (occurs check in unification)
    InfiniteType { var: usize },

    /// Argument count mismatch
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        function: String,
    },

    /// Cannot apply operator to types
    InvalidOperation {
        op: String,
        left: String,
        right: Option<String>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::LexError { message, line, col } => {
                write!(f, "Lex error at {}:{}: {}", line, col, message)
            }
            Error::ParseError { message, line, col } => {
                write!(f, "Parse error at {}:{}: {}", line, col, message)
            }
            Error::SemanticError { message } => {
                write!(f, "Semantic error: {}", message)
            }
            Error::TypeError { kind, line, col, help } => {
                write!(f, "error[E{}]: ", Self::error_code(kind))?;
                match kind {
                    TypeErrorKind::Mismatch { expected, found, context } => {
                        write!(f, "type mismatch at {}:{}\n", line, col)?;
                        write!(f, "   expected: {}\n", expected)?;
                        write!(f, "      found: {}\n", found)?;
                        if !context.is_empty() {
                            write!(f, "    context: {}\n", context)?;
                        }
                    }
                    TypeErrorKind::UndefinedVariable { name } => {
                        write!(f, "undefined variable '{}' at {}:{}", name, line, col)?;
                    }
                    TypeErrorKind::InvalidLayerAccess { layer } => {
                        write!(f, "invalid layer access L{:X} at {}:{} (valid: L0-LF)", layer, line, col)?;
                    }
                    TypeErrorKind::HardwareConflict { required, found } => {
                        write!(f, "hardware hint conflict at {}:{}\n", line, col)?;
                        write!(f, "   required: {}\n", required)?;
                        write!(f, "      found: {}", found)?;
                    }
                    TypeErrorKind::InfiniteType { var } => {
                        write!(f, "infinite type detected (type variable {})", var)?;
                    }
                    TypeErrorKind::ArgumentCountMismatch { expected, found, function } => {
                        write!(f, "argument count mismatch at {}:{}\n", line, col)?;
                        write!(f, "   function '{}' expects {} arguments, got {}", function, expected, found)?;
                    }
                    TypeErrorKind::InvalidOperation { op, left, right } => {
                        if let Some(right) = right {
                            write!(f, "cannot apply operator '{}' to types {} and {}", op, left, right)?;
                        } else {
                            write!(f, "cannot apply operator '{}' to type {}", op, left)?;
                        }
                    }
                }
                if let Some(help_msg) = help {
                    write!(f, "\n   help: {}", help_msg)?;
                }
                Ok(())
            }
            Error::CodeGenError { message } => {
                write!(f, "Code generation error: {}", message)
            }
            Error::IoError { message } => {
                write!(f, "I/O error: {}", message)
            }
            Error::Manifest(message) => {
                write!(f, "Manifest error: {}", message)
            }
            Error::ModuleError { message, path } => {
                if let Some(p) = path {
                    write!(f, "Module error in '{}': {}", p, message)
                } else {
                    write!(f, "Module error: {}", message)
                }
            }
        }
    }
}

impl Error {
    fn error_code(kind: &TypeErrorKind) -> &'static str {
        match kind {
            TypeErrorKind::Mismatch { .. } => "0308",
            TypeErrorKind::UndefinedVariable { .. } => "0425",
            TypeErrorKind::InvalidLayerAccess { .. } => "0516",
            TypeErrorKind::HardwareConflict { .. } => "0601",
            TypeErrorKind::InfiniteType { .. } => "0308",
            TypeErrorKind::ArgumentCountMismatch { .. } => "0061",
            TypeErrorKind::InvalidOperation { .. } => "0369",
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError {
            message: err.to_string(),
        }
    }
}
