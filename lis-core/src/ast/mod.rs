//! Abstract Syntax Tree definitions for LIS
//!
//! The AST represents the structure of LIS programs after parsing.

use std::fmt;
use crate::lexer::Span;

/// A complete LIS program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items (functions, structs, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Function definition: `fn name(params) -> Type @hint { body }`
    Function {
        name: String,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Vec<Stmt>,
        hardware_hint: Option<HardwareHint>,
        is_pub: bool,
        span: Span,
    },

    /// Transform definition: `transform name(params) -> Type @hint { body }`
    Transform {
        name: String,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Vec<Stmt>,
        hardware_hint: Option<HardwareHint>,
        is_pub: bool,
        span: Span,
    },

    /// Type alias: `type Name = Type`
    TypeAlias { name: String, ty: Type, is_pub: bool, span: Span },

    /// Use statement: `use path::to::module;` or `pub use ...`
    Use(UseStatement),

    /// Module declaration: `mod name;` or `mod name { ... }`
    Module(ModuleDecl),

    /// External function declaration: `extern fn name(params) -> Type;`
    ExternFunction(ExternFn),
}

/// Use statement for importing modules
#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    /// Path segments: `["neural", "layers"]` for `use neural::layers`
    pub path: Vec<String>,
    /// Optional alias: `as name`
    pub alias: Option<String>,
    /// Specific items to import: `{Dense, Conv2D}`
    pub items: Option<Vec<String>>,
    /// Whether this is a pub use (re-export)
    pub is_pub: bool,
    pub span: Span,
}

/// Module declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: String,
    /// None = external module (mod name;), Some = inline module (mod name { ... })
    pub items: Option<Vec<Item>>,
    pub is_pub: bool,
    pub span: Span,
}

/// External function declaration (FFI to Rust)
#[derive(Debug, Clone, PartialEq)]
pub struct ExternFn {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
}

/// Type annotations
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// ByteSil - single complex value
    ByteSil,

    /// SilState - 16-layer state
    State,

    /// Layer reference (L0-LF)
    Layer(u8),

    /// Hardware annotation
    Hardware(HardwareHint),

    /// Function type: (params) -> return
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },

    /// Tuple type: (T1, T2, ...)
    Tuple(Vec<Type>),

    /// Named type
    Named(String),
}

/// Hardware execution hints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HardwareHint {
    Cpu,
    Gpu,
    Npu,
    Simd,
    Photonic,
}

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Let binding: `let name = expr;`
    Let { name: String, ty: Option<Type>, value: Expr, span: Span },

    /// Assignment: `name = expr;`
    Assign { name: String, value: Expr, span: Span },

    /// Expression statement: `expr;`
    Expr(Expr),

    /// Return statement: `return expr;`
    Return(Option<Expr>, Span),

    /// Loop: `loop { body }`
    Loop { body: Vec<Stmt>, span: Span },

    /// Break from loop
    Break(Span),

    /// Continue loop
    Continue(Span),

    /// If statement: `if cond { then_body } else { else_body }`
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
}

/// Expression with source span
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create expression with dummy span (for backwards compatibility)
    pub fn dummy(kind: ExprKind) -> Self {
        Self { kind, span: Span::dummy() }
    }
}

/// Expression kinds
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Literal value
    Literal(Literal),

    /// Variable reference
    Ident(String),

    /// Binary operation: `left op right`
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    /// Unary operation: `op expr`
    Unary { op: UnOp, expr: Box<Expr> },

    /// Function call: `name(args)`
    Call { name: String, args: Vec<Expr> },

    /// Layer access: `state.L0`
    LayerAccess { expr: Box<Expr>, layer: u8 },

    /// State construction: `State { L0: expr0, L1: expr1, ... }`
    StateConstruct { layers: Vec<(u8, Expr)> },

    /// Complex number: `(rho, theta)` or `expr + i*expr`
    Complex { rho: Box<Expr>, theta: Box<Expr> },

    /// Tuple: `(a, b, c)`
    Tuple { elements: Vec<Expr> },

    /// Transform application: `expr |> transform`
    Pipe { expr: Box<Expr>, transform: String },

    /// Feedback loop: `feedback expr`
    Feedback { expr: Box<Expr> },

    /// Emergence: `emerge expr`
    Emerge { expr: Box<Expr> },
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer: `42`
    Int(i64),

    /// Float: `3.14`
    Float(f64),

    /// Boolean: `true` or `false`
    Bool(bool),

    /// String: `"hello"`
    String(String),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Pow,      // **

    // Comparison
    Eq,       // ==
    Ne,       // !=
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=

    // Logical
    And,      // &&
    Or,       // ||

    // Bitwise/Layer
    Xor,      // ^
    BitAnd,   // &
    BitOr,    // |
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,      // -
    Not,      // !
    Conj,     // ~ (complex conjugate)
    Mag,      // |x| (magnitude)
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Pow => write!(f, "**"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Le => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Xor => write!(f, "^"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
            UnOp::Conj => write!(f, "~"),
            UnOp::Mag => write!(f, "|"),
        }
    }
}
