//! Type system for LIS
//!
//! This module implements a bidirectional type inference system with support for:
//! - Hardware hints (@gpu, @npu, @cpu, @simd, @photonic)
//! - Complex numbers (log-polar representation)
//! - SIL layers (L0-LF)
//! - State types (16-layer state)

use crate::ast;
use crate::lexer::Span;
use std::fmt;

pub mod checker;
pub mod inference;
pub mod intrinsics;

// Re-export Span for convenience
pub use crate::lexer::Span as TypeSpan;

/// Type kinds in LIS
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    /// Integer type (i64)
    Int,

    /// Floating-point type (f64)
    Float,

    /// Complex number (log-polar: rho, theta)
    Complex,

    /// Single ByteSil value (8-bit complex)
    ByteSil,

    /// Full 16-layer SIL state
    State,

    /// Specific layer (L0-LF)
    Layer(u8),

    /// Function type
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },

    /// Tuple type
    Tuple(Vec<Type>),

    /// Hardware hint annotation
    Hardware(ast::HardwareHint),

    /// Unit type (void)
    Unit,

    /// Boolean type
    Bool,

    /// String type
    String,

    /// Unknown type (for inference)
    Unknown(usize),

    /// Named type (for type aliases)
    Named(std::string::String),

    /// Error type (for error recovery)
    Error,
}

/// Type with source location
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl Type {
    pub fn new(kind: TypeKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn int(span: Span) -> Self {
        Self::new(TypeKind::Int, span)
    }

    pub fn float(span: Span) -> Self {
        Self::new(TypeKind::Float, span)
    }

    pub fn complex(span: Span) -> Self {
        Self::new(TypeKind::Complex, span)
    }

    pub fn bytesil(span: Span) -> Self {
        Self::new(TypeKind::ByteSil, span)
    }

    pub fn state(span: Span) -> Self {
        Self::new(TypeKind::State, span)
    }

    pub fn layer(n: u8, span: Span) -> Self {
        Self::new(TypeKind::Layer(n), span)
    }

    pub fn unit(span: Span) -> Self {
        Self::new(TypeKind::Unit, span)
    }

    pub fn bool(span: Span) -> Self {
        Self::new(TypeKind::Bool, span)
    }

    pub fn string(span: Span) -> Self {
        Self::new(TypeKind::String, span)
    }

    pub fn function(params: Vec<Type>, ret: Type, span: Span) -> Self {
        Self::new(TypeKind::Function {
            params,
            ret: Box::new(ret),
        }, span)
    }

    pub fn hardware(hint: ast::HardwareHint, span: Span) -> Self {
        Self::new(TypeKind::Hardware(hint), span)
    }

    pub fn unknown(id: usize, span: Span) -> Self {
        Self::new(TypeKind::Unknown(id), span)
    }

    pub fn named(name: std::string::String, span: Span) -> Self {
        Self::new(TypeKind::Named(name), span)
    }

    pub fn tuple(elements: Vec<Type>, span: Span) -> Self {
        Self::new(TypeKind::Tuple(elements), span)
    }

    pub fn error(span: Span) -> Self {
        Self::new(TypeKind::Error, span)
    }

    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        use TypeKind::*;

        match (&self.kind, &other.kind) {
            // Error type is compatible with anything (for error recovery)
            (Error, _) | (_, Error) => true,

            // Exact matches
            (Int, Int) | (Float, Float) | (Bool, Bool) | (String, String)
            | (Complex, Complex) | (ByteSil, ByteSil) | (State, State) | (Unit, Unit) => true,

            // Layer compatibility
            (Layer(a), Layer(b)) => a == b,

            // Unknown types are compatible with anything (inference)
            (Unknown(_), _) | (_, Unknown(_)) => true,

            // Hardware hints are compatible if they match
            (Hardware(h1), Hardware(h2)) => h1 == h2,

            // Numeric types can be promoted
            (Int, Float) | (Float, Int) => true,
            (Int, Complex) | (Float, Complex) => true,
            (Complex, ByteSil) | (ByteSil, Complex) => true,

            // Function types
            (Function { params: p1, ret: r1 }, Function { params: p2, ret: r2 }) => {
                p1.len() == p2.len()
                && p1.iter().zip(p2).all(|(a, b)| a.is_compatible_with(b))
                && r1.is_compatible_with(r2)
            }

            // Tuple types
            (Tuple(t1), Tuple(t2)) => {
                t1.len() == t2.len()
                && t1.iter().zip(t2).all(|(a, b)| a.is_compatible_with(b))
            }

            // Named types need resolution (for now, assume compatible)
            (Named(_), _) | (_, Named(_)) => true,

            _ => false,
        }
    }

    /// Find the common supertype of two types
    pub fn common_supertype(&self, other: &Type) -> Option<Type> {
        use TypeKind::*;

        if self.is_compatible_with(other) {
            match (&self.kind, &other.kind) {
                // If one is unknown, use the other
                (Unknown(_), _) => Some(other.clone()),
                (_, Unknown(_)) => Some(self.clone()),

                // Numeric promotion
                (Int, Float) | (Float, Int) => Some(Type::float(self.span)),
                (Int, Complex) | (Complex, Int) | (Float, Complex) | (Complex, Float) => {
                    Some(Type::complex(self.span))
                }
                (Complex, ByteSil) | (ByteSil, Complex) => Some(Type::bytesil(self.span)),

                // Otherwise return self if compatible
                _ => Some(self.clone()),
            }
        } else {
            None
        }
    }

    /// Apply a hardware hint to this type
    pub fn apply_hardware_hint(&self, _hint: ast::HardwareHint) -> Type {
        // Hardware hints don't change the underlying type, just add metadata
        // For now, we keep the type as-is (could be extended for type specialization)
        self.clone()
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self.kind, TypeKind::Int | TypeKind::Float | TypeKind::Complex)
    }

    /// Check if this is a layer type
    pub fn is_layer(&self) -> bool {
        matches!(self.kind, TypeKind::Layer(_))
    }

    /// Check if this is an unknown type
    pub fn is_unknown(&self) -> bool {
        matches!(self.kind, TypeKind::Unknown(_))
    }
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Int => write!(f, "Int"),
            TypeKind::Float => write!(f, "Float"),
            TypeKind::Complex => write!(f, "Complex"),
            TypeKind::ByteSil => write!(f, "ByteSil"),
            TypeKind::State => write!(f, "State"),
            TypeKind::Layer(n) => write!(f, "L{:X}", n),
            TypeKind::Function { params, ret } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            TypeKind::Tuple(elements) => {
                write!(f, "(")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            TypeKind::Hardware(hint) => write!(f, "@{:?}", hint),
            TypeKind::Unit => write!(f, "Unit"),
            TypeKind::Bool => write!(f, "Bool"),
            TypeKind::String => write!(f, "String"),
            TypeKind::Unknown(id) => write!(f, "?{}", id),
            TypeKind::Named(name) => write!(f, "{}", name),
            TypeKind::Error => write!(f, "<error>"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// Convert AST type to type system type
impl From<ast::Type> for Type {
    fn from(ast_ty: ast::Type) -> Self {
        let span = Span::dummy(); // AST doesn't have spans yet
        match ast_ty {
            ast::Type::ByteSil => Type::bytesil(span),
            ast::Type::State => Type::state(span),
            ast::Type::Layer(n) => Type::layer(n, span),
            ast::Type::Hardware(hint) => Type::hardware(hint, span),
            ast::Type::Function { params, ret } => {
                let param_types = params.into_iter().map(Type::from).collect();
                Type::function(param_types, Type::from(*ret), span)
            }
            ast::Type::Tuple(elements) => {
                let element_types = elements.into_iter().map(Type::from).collect();
                Type::tuple(element_types, span)
            }
            ast::Type::Named(name) => {
                // Convert known type names to native types
                match name.as_str() {
                    "Int" => Type::int(span),
                    "Float" => Type::float(span),
                    "Bool" => Type::bool(span),
                    "String" => Type::string(span),
                    "Complex" => Type::complex(span),
                    "ByteSil" => Type::bytesil(span),
                    "State" => Type::state(span),
                    "Unit" => Type::unit(span),
                    _ => Type::named(name, span),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility() {
        let span = Span::dummy();
        let int_ty = Type::int(span);
        let float_ty = Type::float(span);
        let complex_ty = Type::complex(span);

        assert!(int_ty.is_compatible_with(&int_ty));
        assert!(int_ty.is_compatible_with(&float_ty));
        assert!(int_ty.is_compatible_with(&complex_ty));
        assert!(float_ty.is_compatible_with(&complex_ty));
    }

    #[test]
    fn test_common_supertype() {
        let span = Span::dummy();
        let int_ty = Type::int(span);
        let float_ty = Type::float(span);

        let common = int_ty.common_supertype(&float_ty);
        assert!(common.is_some());
        assert_eq!(common.unwrap().kind, TypeKind::Float);
    }

    #[test]
    fn test_layer_types() {
        let span = Span::dummy();
        let l0 = Type::layer(0, span);
        let l1 = Type::layer(1, span);

        assert!(l0.is_compatible_with(&l0));
        assert!(!l0.is_compatible_with(&l1));
    }

    #[test]
    fn test_numeric_types() {
        let span = Span::dummy();
        let int_ty = Type::int(span);
        let float_ty = Type::float(span);
        let bool_ty = Type::bool(span);

        assert!(int_ty.is_numeric());
        assert!(float_ty.is_numeric());
        assert!(!bool_ty.is_numeric());
    }
}
