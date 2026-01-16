//! Type inference engine using bidirectional typing
//!
//! This module implements a bidirectional type inference algorithm that combines:
//! - **Synthesis** (bottom-up): Infer types from expressions
//! - **Checking** (top-down): Check expressions against expected types
//!
//! The bidirectional approach works well for:
//! - Hardware hints that need directed flow
//! - Complex numbers with log-polar representation
//! - Layer types with known constraints

use super::{Type, TypeKind};
use crate::ast::{self, BinOp, ExprKind, UnOp};
use crate::error::{Error, Result};
use crate::lexer::Span;
use std::collections::HashMap;

/// Type variable for unknown types during inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(usize);

/// Type constraint during inference
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Two types must be equal
    Equal(Type, Type, Span),

    /// First type must be a subtype of second
    Subtype(Type, Type, Span),

    /// Expression must have hardware hint
    Hardware(Type, ast::HardwareHint, Span),
}

/// Type inference context
pub struct TypeContext {
    /// Variable bindings: name -> type
    bindings: HashMap<std::string::String, Type>,

    /// Type alias definitions: name -> type
    type_aliases: HashMap<std::string::String, Type>,

    /// Constraints collected during inference
    constraints: Vec<Constraint>,

    /// Counter for generating fresh type variables
    fresh_counter: usize,

    /// Substitution map for type variables
    substitutions: HashMap<usize, Type>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            type_aliases: HashMap::new(),
            constraints: Vec::new(),
            fresh_counter: 0,
            substitutions: HashMap::new(),
        }
    }

    /// Register a type alias
    pub fn register_type_alias(&mut self, name: String, ty: Type) {
        self.type_aliases.insert(name, ty);
    }

    /// Resolve a type, expanding type aliases and substitutions
    pub fn resolve_type_alias(&self, ty: &Type) -> Type {
        match &ty.kind {
            TypeKind::Named(name) => {
                // Check if it's a type alias
                if let Some(resolved) = self.type_aliases.get(name) {
                    // Recursively resolve in case of nested aliases
                    self.resolve_type_alias(resolved)
                } else {
                    // Not a registered alias, return as-is
                    ty.clone()
                }
            }
            TypeKind::Function { params, ret } => {
                let resolved_params: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_type_alias(p))
                    .collect();
                let resolved_ret = self.resolve_type_alias(ret);
                Type::function(resolved_params, resolved_ret, ty.span)
            }
            TypeKind::Tuple(elems) => {
                let resolved_elems: Vec<Type> = elems
                    .iter()
                    .map(|e| self.resolve_type_alias(e))
                    .collect();
                Type::tuple(resolved_elems, ty.span)
            }
            _ => ty.clone(),
        }
    }

    /// Check for cycles in type alias definitions
    pub fn check_alias_cycle(&self, name: &str) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        self.check_alias_cycle_helper(name, &mut visited)
    }

    fn check_alias_cycle_helper(
        &self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if visited.contains(name) {
            return Err(Error::SemanticError {
                message: format!("Cyclic type alias detected: {}", name),
            });
        }

        if let Some(ty) = self.type_aliases.get(name) {
            visited.insert(name.to_string());
            self.check_type_for_cycles(ty, visited)?;
            visited.remove(name);
        }

        Ok(())
    }

    fn check_type_for_cycles(
        &self,
        ty: &Type,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        match &ty.kind {
            TypeKind::Named(ref_name) => {
                self.check_alias_cycle_helper(ref_name, visited)
            }
            TypeKind::Function { params, ret } => {
                for param in params {
                    self.check_type_for_cycles(param, visited)?;
                }
                self.check_type_for_cycles(ret, visited)
            }
            TypeKind::Tuple(elems) => {
                for elem in elems {
                    self.check_type_for_cycles(elem, visited)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_type_var(&mut self, span: Span) -> Type {
        let id = self.fresh_counter;
        self.fresh_counter += 1;
        Type::unknown(id, span)
    }

    /// Bind a variable to a type
    pub fn bind(&mut self, name: std::string::String, ty: Type) {
        self.bindings.insert(name, ty);
    }

    /// Look up a variable's type
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.bindings.get(name)
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Synthesize type from expression (bottom-up)
    pub fn synthesize_expr(&mut self, expr: &ast::Expr) -> Result<Type> {
        let span = expr.span;

        match &expr.kind {
            // Literals have known types
            ExprKind::Literal(lit) => Ok(self.synthesize_literal(lit, span)),

            // Variables look up their binding
            ExprKind::Ident(name) => {
                self.lookup(name)
                    .cloned()
                    .ok_or_else(|| Error::SemanticError {
                        message: format!("Undefined variable: {}", name),
                    })
            }

            // Binary operations
            ExprKind::Binary { left, op, right } => {
                self.synthesize_binary(left, *op, right, span)
            }

            // Unary operations
            ExprKind::Unary { op, expr } => {
                self.synthesize_unary(*op, expr, span)
            }

            // Function calls
            ExprKind::Call { name, args } => {
                self.synthesize_call(name, args, span)
            }

            // Layer access: state.L0 -> ByteSil
            ExprKind::LayerAccess { expr, layer } => {
                let state_ty = self.synthesize_expr(expr)?;
                self.check_state_type(&state_ty, span)?;
                self.check_layer_bounds(*layer, span)?;
                Ok(Type::bytesil(span))
            }

            // State construction
            ExprKind::StateConstruct { layers } => {
                for (layer, layer_expr) in layers {
                    self.check_layer_bounds(*layer, span)?;
                    let ty = self.synthesize_expr(layer_expr)?;
                    self.check_bytesil_compatible(&ty, span)?;
                }
                Ok(Type::state(span))
            }

            // Complex number: (rho, theta)
            ExprKind::Complex { rho, theta } => {
                let rho_ty = self.synthesize_expr(rho)?;
                let theta_ty = self.synthesize_expr(theta)?;

                // Both components should be numeric
                self.check_numeric(&rho_ty, span)?;
                self.check_numeric(&theta_ty, span)?;

                Ok(Type::complex(span))
            }

            // Tuple: (a, b, c)
            ExprKind::Tuple { elements } => {
                let element_types: Vec<Type> = elements
                    .iter()
                    .map(|e| self.synthesize_expr(e))
                    .collect::<Result<_>>()?;
                Ok(Type::tuple(element_types, span))
            }

            // Pipe: expr |> transform
            ExprKind::Pipe { expr, transform } => {
                let input_ty = self.synthesize_expr(expr)?;

                // Look up transform type
                let transform_ty = self.lookup(transform)
                    .cloned()
                    .ok_or_else(|| Error::SemanticError {
                        message: format!("Undefined transform: {}", transform),
                    })?;

                // Validate that transform accepts input type
                match &transform_ty.kind {
                    TypeKind::Function { params, ret } => {
                        if params.is_empty() {
                            return Err(Error::SemanticError {
                                message: format!("Transform '{}' expects no arguments", transform),
                            });
                        }
                        if !input_ty.is_compatible_with(&params[0]) {
                            return Err(Error::SemanticError {
                                message: format!(
                                    "Pipe type mismatch: '{}' expects {}, got {}",
                                    transform, params[0], input_ty
                                ),
                            });
                        }
                        Ok((**ret).clone())
                    }
                    _ => Err(Error::SemanticError {
                        message: format!("'{}' is not a function", transform),
                    }),
                }
            }

            // Feedback loop: type matches inner expression, must be State
            ExprKind::Feedback { expr } => {
                let inner_ty = self.synthesize_expr(expr)?;
                self.check_state_type(&inner_ty, span)?;
                Ok(inner_ty)
            }

            // Emerge: type matches inner expression, must be State
            ExprKind::Emerge { expr } => {
                let inner_ty = self.synthesize_expr(expr)?;
                self.check_state_type(&inner_ty, span)?;
                Ok(inner_ty)
            }
        }
    }

    /// Check expression against expected type (top-down)
    pub fn check_expr(&mut self, expr: &ast::Expr, expected: &Type) -> Result<()> {
        let span = expr.span;

        // Try to synthesize and unify
        let inferred = self.synthesize_expr(expr)?;

        if !inferred.is_compatible_with(expected) {
            return Err(Error::SemanticError {
                message: format!(
                    "Type mismatch at line {}: expected {}, found {}",
                    span.line, expected, inferred
                ),
            });
        }

        // Add equality constraint
        self.add_constraint(Constraint::Equal(inferred, expected.clone(), span));
        Ok(())
    }

    /// Synthesize type from literal
    fn synthesize_literal(&self, lit: &ast::Literal, span: Span) -> Type {
        match lit {
            ast::Literal::Int(_) => Type::int(span),
            ast::Literal::Float(_) => Type::float(span),
            ast::Literal::Bool(_) => Type::bool(span),
            ast::Literal::String(_) => Type::string(span),
        }
    }

    /// Synthesize type from binary operation
    fn synthesize_binary(
        &mut self,
        left: &ast::Expr,
        op: BinOp,
        right: &ast::Expr,
        span: Span,
    ) -> Result<Type> {
        let left_ty = self.synthesize_expr(left)?;
        let right_ty = self.synthesize_expr(right)?;

        match op {
            // Arithmetic: numeric -> numeric
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow => {
                self.check_numeric(&left_ty, span)?;
                self.check_numeric(&right_ty, span)?;

                // Return common supertype
                left_ty.common_supertype(&right_ty)
                    .ok_or_else(|| Error::SemanticError {
                        message: format!(
                            "Cannot perform {} on types {} and {}",
                            op, left_ty, right_ty
                        ),
                    })
            }

            // Comparison: numeric -> bool
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                self.check_numeric(&left_ty, span)?;
                self.check_numeric(&right_ty, span)?;
                Ok(Type::bool(span))
            }

            // Equality: any -> bool
            BinOp::Eq | BinOp::Ne => {
                if !left_ty.is_compatible_with(&right_ty) {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Cannot compare incompatible types {} and {}",
                            left_ty, right_ty
                        ),
                    });
                }
                Ok(Type::bool(span))
            }

            // Logical: bool -> bool
            BinOp::And | BinOp::Or => {
                self.check_bool(&left_ty, span)?;
                self.check_bool(&right_ty, span)?;
                Ok(Type::bool(span))
            }

            // Bitwise: int -> int, or ByteSil -> ByteSil
            BinOp::BitAnd | BinOp::BitOr | BinOp::Xor => {
                // ByteSil types support bitwise operations
                if matches!(left_ty.kind, TypeKind::ByteSil)
                    && matches!(right_ty.kind, TypeKind::ByteSil)
                {
                    return Ok(Type::bytesil(span));
                }

                // Layers cannot do direct bitwise ops - must extract ByteSil first
                if left_ty.is_layer() || right_ty.is_layer() {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Cannot perform bitwise operation on layer types at line {}. \
                             Use layer access (e.g., state.L0) to get ByteSil values first.",
                            span.line
                        ),
                    });
                }

                // Otherwise, require Int
                self.check_int(&left_ty, span)?;
                self.check_int(&right_ty, span)?;
                Ok(Type::int(span))
            }
        }
    }

    /// Synthesize type from unary operation
    fn synthesize_unary(&mut self, op: UnOp, expr: &ast::Expr, span: Span) -> Result<Type> {
        let expr_ty = self.synthesize_expr(expr)?;

        match op {
            // Negation: numeric -> same type
            UnOp::Neg => {
                self.check_numeric(&expr_ty, span)?;
                Ok(expr_ty)
            }

            // Logical not: bool -> bool
            UnOp::Not => {
                self.check_bool(&expr_ty, span)?;
                Ok(Type::bool(span))
            }

            // Complex conjugate: complex -> complex
            UnOp::Conj => {
                if !matches!(expr_ty.kind, TypeKind::Complex | TypeKind::ByteSil) {
                    return Err(Error::SemanticError {
                        message: format!("Cannot take conjugate of non-complex type {}", expr_ty),
                    });
                }
                Ok(expr_ty)
            }

            // Magnitude: complex -> float
            UnOp::Mag => {
                if !matches!(expr_ty.kind, TypeKind::Complex | TypeKind::ByteSil) {
                    return Err(Error::SemanticError {
                        message: format!("Cannot take magnitude of non-complex type {}", expr_ty),
                    });
                }
                Ok(Type::float(span))
            }
        }
    }

    /// Synthesize type from function call
    fn synthesize_call(&mut self, name: &str, args: &[ast::Expr], _span: Span) -> Result<Type> {
        // Look up function type
        let func_ty = self.lookup(name)
            .cloned()
            .ok_or_else(|| Error::SemanticError {
                message: format!("Undefined function: {}", name),
            })?;

        match &func_ty.kind {
            TypeKind::Function { params, ret } => {
                // Check argument count
                if args.len() != params.len() {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Function {} expects {} arguments, got {}",
                            name, params.len(), args.len()
                        ),
                    });
                }

                // Check each argument type
                for (arg, param_ty) in args.iter().zip(params) {
                    self.check_expr(arg, param_ty)?;
                }

                Ok((**ret).clone())
            }
            _ => Err(Error::SemanticError {
                message: format!("{} is not a function", name),
            }),
        }
    }

    // Validation helpers

    fn check_numeric(&self, ty: &Type, _span: Span) -> Result<()> {
        if !ty.is_numeric() && !ty.is_unknown() {
            return Err(Error::SemanticError {
                message: format!("Expected numeric type, found {}", ty),
            });
        }
        Ok(())
    }

    fn check_bool(&self, ty: &Type, _span: Span) -> Result<()> {
        if !matches!(ty.kind, TypeKind::Bool) && !ty.is_unknown() {
            return Err(Error::SemanticError {
                message: format!("Expected Bool, found {}", ty),
            });
        }
        Ok(())
    }

    fn check_int(&self, ty: &Type, _span: Span) -> Result<()> {
        if !matches!(ty.kind, TypeKind::Int) && !ty.is_unknown() {
            return Err(Error::SemanticError {
                message: format!("Expected Int, found {}", ty),
            });
        }
        Ok(())
    }

    fn check_state_type(&self, ty: &Type, _span: Span) -> Result<()> {
        if !matches!(ty.kind, TypeKind::State) && !ty.is_unknown() {
            return Err(Error::SemanticError {
                message: format!("Expected State, found {}", ty),
            });
        }
        Ok(())
    }

    fn check_bytesil_compatible(&self, ty: &Type, _span: Span) -> Result<()> {
        match &ty.kind {
            TypeKind::ByteSil | TypeKind::Complex | TypeKind::Float | TypeKind::Int
            | TypeKind::Unknown(_) => Ok(()),
            _ => Err(Error::SemanticError {
                message: format!("Type {} cannot be converted to ByteSil", ty),
            }),
        }
    }

    fn check_layer_bounds(&self, layer: u8, _span: Span) -> Result<()> {
        if layer > 0xF {
            return Err(Error::SemanticError {
                message: format!("Layer index out of bounds: L{:X} (max L{})", layer, 0xF),
            });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Type Unification (Robinson's Algorithm)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Unify two types, updating substitutions
    pub fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> Result<()> {
        let t1 = self.apply_substitution(t1);
        let t2 = self.apply_substitution(t2);

        match (&t1.kind, &t2.kind) {
            // Both are the same concrete type
            (TypeKind::Int, TypeKind::Int)
            | (TypeKind::Float, TypeKind::Float)
            | (TypeKind::Bool, TypeKind::Bool)
            | (TypeKind::String, TypeKind::String)
            | (TypeKind::Complex, TypeKind::Complex)
            | (TypeKind::ByteSil, TypeKind::ByteSil)
            | (TypeKind::State, TypeKind::State)
            | (TypeKind::Unit, TypeKind::Unit) => Ok(()),

            // Layer types must match
            (TypeKind::Layer(a), TypeKind::Layer(b)) if a == b => Ok(()),

            // Type variable on the left - bind it
            (TypeKind::Unknown(id), _) => {
                if self.occurs_check(*id, &t2) {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Infinite type detected: ?{} occurs in {}",
                            id, t2
                        ),
                    });
                }
                self.substitutions.insert(*id, t2);
                Ok(())
            }

            // Type variable on the right - bind it
            (_, TypeKind::Unknown(id)) => {
                if self.occurs_check(*id, &t1) {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Infinite type detected: ?{} occurs in {}",
                            id, t1
                        ),
                    });
                }
                self.substitutions.insert(*id, t1);
                Ok(())
            }

            // Function types - unify params and return
            (
                TypeKind::Function { params: p1, ret: r1 },
                TypeKind::Function { params: p2, ret: r2 },
            ) => {
                if p1.len() != p2.len() {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Function arity mismatch at line {}: expected {} params, found {}",
                            span.line, p1.len(), p2.len()
                        ),
                    });
                }
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    self.unify(param1, param2, span)?;
                }
                self.unify(r1, r2, span)
            }

            // Tuple types - unify element-wise
            (TypeKind::Tuple(elems1), TypeKind::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Tuple size mismatch at line {}: expected {} elements, found {}",
                            span.line, elems1.len(), elems2.len()
                        ),
                    });
                }
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    self.unify(e1, e2, span)?;
                }
                Ok(())
            }

            // Hardware hints are compatible if they match
            (TypeKind::Hardware(h1), TypeKind::Hardware(h2)) if h1 == h2 => Ok(()),

            // Numeric promotion: Int can unify with Float (result is Float)
            (TypeKind::Int, TypeKind::Float) | (TypeKind::Float, TypeKind::Int) => Ok(()),

            // Complex promotion: Int/Float can unify with Complex
            (TypeKind::Int, TypeKind::Complex)
            | (TypeKind::Float, TypeKind::Complex)
            | (TypeKind::Complex, TypeKind::Int)
            | (TypeKind::Complex, TypeKind::Float) => Ok(()),

            // ByteSil and Complex are interchangeable
            (TypeKind::Complex, TypeKind::ByteSil)
            | (TypeKind::ByteSil, TypeKind::Complex) => Ok(()),

            // Error type unifies with anything (for error recovery)
            (TypeKind::Error, _) | (_, TypeKind::Error) => Ok(()),

            // Named types - for now, assume compatible (should resolve aliases first)
            (TypeKind::Named(_), _) | (_, TypeKind::Named(_)) => Ok(()),

            // Otherwise, types don't unify
            _ => Err(Error::SemanticError {
                message: format!(
                    "Type mismatch at line {}: cannot unify {} with {}",
                    span.line, t1, t2
                ),
            }),
        }
    }

    /// Check if type variable occurs in a type (prevents infinite types)
    fn occurs_check(&self, var_id: usize, ty: &Type) -> bool {
        let ty = self.apply_substitution(ty);
        match &ty.kind {
            TypeKind::Unknown(id) => *id == var_id,
            TypeKind::Function { params, ret } => {
                params.iter().any(|p| self.occurs_check(var_id, p))
                    || self.occurs_check(var_id, ret)
            }
            TypeKind::Tuple(elems) => elems.iter().any(|e| self.occurs_check(var_id, e)),
            _ => false,
        }
    }

    /// Apply current substitutions to a type
    pub fn apply_substitution(&self, ty: &Type) -> Type {
        match &ty.kind {
            TypeKind::Unknown(id) => {
                if let Some(resolved) = self.substitutions.get(id) {
                    // Recursively apply substitutions
                    self.apply_substitution(resolved)
                } else {
                    ty.clone()
                }
            }
            TypeKind::Function { params, ret } => {
                let new_params: Vec<Type> = params
                    .iter()
                    .map(|p| self.apply_substitution(p))
                    .collect();
                let new_ret = self.apply_substitution(ret);
                Type::function(new_params, new_ret, ty.span)
            }
            TypeKind::Tuple(elems) => {
                let new_elems: Vec<Type> = elems
                    .iter()
                    .map(|e| self.apply_substitution(e))
                    .collect();
                Type::tuple(new_elems, ty.span)
            }
            _ => ty.clone(),
        }
    }

    /// Solve collected constraints using unification
    pub fn solve_constraints(&mut self) -> Result<()> {
        // Clone constraints to avoid borrow issues
        let constraints: Vec<Constraint> = self.constraints.drain(..).collect();

        for constraint in constraints {
            match constraint {
                Constraint::Equal(ty1, ty2, span) => {
                    self.unify(&ty1, &ty2, span)?;
                }
                Constraint::Subtype(sub, sup, span) => {
                    // For now, treat subtype as equality
                    // A more sophisticated system would handle variance
                    self.unify(&sub, &sup, span)?;
                }
                Constraint::Hardware(_ty, _hint, _span) => {
                    // Hardware hints are always valid (just metadata)
                }
            }
        }

        Ok(())
    }

    /// Get the resolved type for a type variable
    pub fn resolve_type(&self, ty: &Type) -> Type {
        self.apply_substitution(ty)
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, ExprKind, Literal};

    #[test]
    fn test_synthesize_literal() {
        let mut ctx = TypeContext::new();
        let expr = Expr::dummy(ExprKind::Literal(Literal::Int(42)));
        let ty = ctx.synthesize_expr(&expr).unwrap();
        assert_eq!(ty.kind, TypeKind::Int);
    }

    #[test]
    fn test_synthesize_binary() {
        let mut ctx = TypeContext::new();
        let expr = Expr::dummy(ExprKind::Binary {
            left: Box::new(Expr::dummy(ExprKind::Literal(Literal::Int(1)))),
            op: BinOp::Add,
            right: Box::new(Expr::dummy(ExprKind::Literal(Literal::Int(2)))),
        });
        let ty = ctx.synthesize_expr(&expr).unwrap();
        assert_eq!(ty.kind, TypeKind::Int);
    }

    #[test]
    fn test_type_mismatch() {
        let mut ctx = TypeContext::new();
        let expr = Expr::dummy(ExprKind::Binary {
            left: Box::new(Expr::dummy(ExprKind::Literal(Literal::Int(1)))),
            op: BinOp::And,
            right: Box::new(Expr::dummy(ExprKind::Literal(Literal::Int(2)))),
        });
        let result = ctx.synthesize_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_number() {
        let mut ctx = TypeContext::new();
        let expr = Expr::dummy(ExprKind::Complex {
            rho: Box::new(Expr::dummy(ExprKind::Literal(Literal::Float(1.0)))),
            theta: Box::new(Expr::dummy(ExprKind::Literal(Literal::Float(0.0)))),
        });
        let ty = ctx.synthesize_expr(&expr).unwrap();
        assert_eq!(ty.kind, TypeKind::Complex);
    }

    #[test]
    fn test_unify_same_types() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();
        let int1 = Type::int(span);
        let int2 = Type::int(span);
        assert!(ctx.unify(&int1, &int2, span).is_ok());
    }

    #[test]
    fn test_unify_type_var_with_concrete() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();
        let type_var = ctx.fresh_type_var(span);
        let int_ty = Type::int(span);

        assert!(ctx.unify(&type_var, &int_ty, span).is_ok());

        // After unification, type var should resolve to Int
        let resolved = ctx.resolve_type(&type_var);
        assert_eq!(resolved.kind, TypeKind::Int);
    }

    #[test]
    fn test_unify_numeric_promotion() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();
        let int_ty = Type::int(span);
        let float_ty = Type::float(span);

        // Int and Float should unify (numeric promotion)
        assert!(ctx.unify(&int_ty, &float_ty, span).is_ok());
    }

    #[test]
    fn test_unify_function_types() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        let fn1 = Type::function(
            vec![Type::int(span)],
            Type::bool(span),
            span,
        );
        let fn2 = Type::function(
            vec![Type::int(span)],
            Type::bool(span),
            span,
        );

        assert!(ctx.unify(&fn1, &fn2, span).is_ok());
    }

    #[test]
    fn test_unify_function_arity_mismatch() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        let fn1 = Type::function(
            vec![Type::int(span)],
            Type::bool(span),
            span,
        );
        let fn2 = Type::function(
            vec![Type::int(span), Type::int(span)],
            Type::bool(span),
            span,
        );

        assert!(ctx.unify(&fn1, &fn2, span).is_err());
    }

    #[test]
    fn test_unify_incompatible_types() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();
        let int_ty = Type::int(span);
        let bool_ty = Type::bool(span);

        // Int and Bool should not unify
        assert!(ctx.unify(&int_ty, &bool_ty, span).is_err());
    }

    #[test]
    fn test_occurs_check() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();
        let type_var = ctx.fresh_type_var(span);

        // Create a function type that contains the type variable
        let fn_ty = Type::function(vec![type_var.clone()], type_var.clone(), span);

        // Trying to unify type_var with fn_ty should fail (infinite type)
        assert!(ctx.unify(&type_var, &fn_ty, span).is_err());
    }

    #[test]
    fn test_tuple_unification() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        let tuple1 = Type::tuple(vec![Type::int(span), Type::bool(span)], span);
        let tuple2 = Type::tuple(vec![Type::int(span), Type::bool(span)], span);

        assert!(ctx.unify(&tuple1, &tuple2, span).is_ok());
    }

    #[test]
    fn test_tuple_size_mismatch() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        let tuple1 = Type::tuple(vec![Type::int(span), Type::bool(span)], span);
        let tuple2 = Type::tuple(vec![Type::int(span)], span);

        assert!(ctx.unify(&tuple1, &tuple2, span).is_err());
    }

    #[test]
    fn test_type_alias_resolution() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        // Register type alias: MyInt = Int
        ctx.register_type_alias("MyInt".to_string(), Type::int(span));

        // Create a Named type referring to the alias
        let named_ty = Type::named("MyInt".to_string(), span);

        // Resolve should return Int
        let resolved = ctx.resolve_type_alias(&named_ty);
        assert_eq!(resolved.kind, TypeKind::Int);
    }

    #[test]
    fn test_nested_type_alias_resolution() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        // Register: MyInt = Int, Number = MyInt
        ctx.register_type_alias("MyInt".to_string(), Type::int(span));
        ctx.register_type_alias("Number".to_string(), Type::named("MyInt".to_string(), span));

        // Resolve Number should return Int (through MyInt)
        let named_ty = Type::named("Number".to_string(), span);
        let resolved = ctx.resolve_type_alias(&named_ty);
        assert_eq!(resolved.kind, TypeKind::Int);
    }

    #[test]
    fn test_cyclic_type_alias_detection() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        // Create cycle: A = B, B = A
        ctx.register_type_alias("A".to_string(), Type::named("B".to_string(), span));
        ctx.register_type_alias("B".to_string(), Type::named("A".to_string(), span));

        // Should detect cycle
        let result = ctx.check_alias_cycle("A");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_type_alias_resolution() {
        let mut ctx = TypeContext::new();
        let span = Span::dummy();

        // Register: Callback = fn(Int) -> Bool
        let fn_ty = Type::function(vec![Type::int(span)], Type::bool(span), span);
        ctx.register_type_alias("Callback".to_string(), fn_ty);

        // Create a Named type
        let named_ty = Type::named("Callback".to_string(), span);
        let resolved = ctx.resolve_type_alias(&named_ty);

        // Should resolve to function type
        match resolved.kind {
            TypeKind::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].kind, TypeKind::Int);
                assert_eq!(ret.kind, TypeKind::Bool);
            }
            _ => panic!("Expected function type, got {:?}", resolved.kind),
        }
    }
}
