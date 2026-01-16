//! Type checker for LIS programs
//!
//! This module coordinates type inference and validation, producing a typed AST
//! with a complete type map.

use super::inference::TypeContext;
use super::{Type, TypeKind};
use crate::ast::{self, BinOp, ExprKind, UnOp};
use crate::error::{Error, Result};
use crate::lexer::Span;
use std::collections::HashMap;

/// Type-checked program with type information
#[derive(Debug)]
pub struct TypedProgram {
    pub items: Vec<TypedItem>,
    pub type_map: HashMap<NodeId, Type>,
}

/// Node identifier for type map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

/// Type-checked top-level item
#[derive(Debug, Clone)]
pub enum TypedItem {
    Function {
        name: std::string::String,
        params: Vec<(std::string::String, Type)>,
        ret_type: Type,
        body: Vec<TypedStmt>,
    },
    Transform {
        name: std::string::String,
        params: Vec<(std::string::String, Type)>,
        ret_type: Type,
        body: Vec<TypedStmt>,
    },
    TypeAlias {
        name: std::string::String,
        ty: Type,
    },
    /// Use statement (module import)
    Use {
        path: Vec<std::string::String>,
    },
    /// Module declaration
    Module {
        name: std::string::String,
    },
    /// External function (FFI)
    ExternFunction {
        name: std::string::String,
        param_types: Vec<Type>,
        ret_type: Type,
    },
}

/// Type-checked statement
#[derive(Debug, Clone)]
pub enum TypedStmt {
    Let {
        name: std::string::String,
        ty: Type,
        value: TypedExpr,
    },
    Assign {
        name: std::string::String,
        value: TypedExpr,
    },
    Expr(TypedExpr),
    Return(Option<TypedExpr>),
    Loop {
        body: Vec<TypedStmt>,
    },
    Break,
    Continue,
    If {
        condition: TypedExpr,
        then_body: Vec<TypedStmt>,
        else_body: Option<Vec<TypedStmt>>,
    },
}

/// Type-checked expression with inferred type
#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
    pub node_id: NodeId,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(ast::Literal),
    Ident(std::string::String),
    Binary {
        left: Box<TypedExpr>,
        op: BinOp,
        right: Box<TypedExpr>,
    },
    Unary {
        op: UnOp,
        expr: Box<TypedExpr>,
    },
    Call {
        name: std::string::String,
        args: Vec<TypedExpr>,
    },
    LayerAccess {
        expr: Box<TypedExpr>,
        layer: u8,
    },
    StateConstruct {
        layers: Vec<(u8, TypedExpr)>,
    },
    Complex {
        rho: Box<TypedExpr>,
        theta: Box<TypedExpr>,
    },
    Tuple {
        elements: Vec<TypedExpr>,
    },
    Pipe {
        expr: Box<TypedExpr>,
        transform: std::string::String,
    },
    Feedback {
        expr: Box<TypedExpr>,
    },
    Emerge {
        expr: Box<TypedExpr>,
    },
}

/// Type checker
pub struct TypeChecker {
    context: TypeContext,
    next_node_id: usize,
    type_map: HashMap<NodeId, Type>,
    /// Accumulated errors for error recovery mode
    errors: crate::error::Errors,
    /// Whether to continue checking after errors (error recovery mode)
    recovery_mode: bool,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut context = TypeContext::new();
        // Register all stdlib intrinsics
        super::intrinsics::register_stdlib_intrinsics(&mut context);

        Self {
            context,
            next_node_id: 0,
            type_map: HashMap::new(),
            errors: crate::error::Errors::new(),
            recovery_mode: false,
        }
    }

    /// Enable error recovery mode - collect all errors instead of failing early
    pub fn with_recovery(mut self) -> Self {
        self.recovery_mode = true;
        self
    }

    /// Record an error, optionally continuing if in recovery mode
    fn record_error(&mut self, error: Error) -> Result<()> {
        if self.recovery_mode {
            self.errors.push(error);
            Ok(())
        } else {
            Err(error)
        }
    }

    /// Get all collected errors
    pub fn collected_errors(&self) -> &crate::error::Errors {
        &self.errors
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check a complete program
    /// Check a complete program
    pub fn check_program(&mut self, program: &ast::Program) -> Result<TypedProgram> {
        let mut typed_items = Vec::new();

        // First pass: register all function signatures
        for item in &program.items {
            match item {
                ast::Item::Function { name, params, ret_ty, span, .. } |
                ast::Item::Transform { name, params, ret_ty, span, .. } => {
                    let param_types: Vec<Type> = params
                        .iter()
                        .map(|p| {
                            p.ty.clone()
                                .map(Type::from)
                                .unwrap_or_else(|| {
                                    self.context.fresh_type_var(*span)
                                })
                        })
                        .collect();

                    // Use declared return type or a fresh type variable
                    let ret_type = ret_ty.clone()
                        .map(Type::from)
                        .unwrap_or_else(|| self.context.fresh_type_var(*span));

                    let func_type = Type::function(param_types, ret_type, *span);
                    self.context.bind(name.clone(), func_type);
                }
                ast::Item::TypeAlias { name, ty, .. } => {
                    let resolved_ty = Type::from(ty.clone());
                    self.context.register_type_alias(name.clone(), resolved_ty.clone());
                    self.context.bind(name.clone(), resolved_ty);
                }
                ast::Item::Use(_) | ast::Item::Module(_) => {
                    // Use and module declarations handled by module resolver
                }
                ast::Item::ExternFunction(extern_fn) => {
                    // Register extern function signature
                    let span = extern_fn.span;
                    let param_types: Vec<Type> = extern_fn.params
                        .iter()
                        .map(|p| {
                            p.ty.clone()
                                .map(Type::from)
                                .unwrap_or_else(|| Type::unknown(0, span))
                        })
                        .collect();
                    let ret_type = extern_fn.ret_ty.clone()
                        .map(Type::from)
                        .unwrap_or_else(|| Type::unit(span));
                    let func_type = Type::function(param_types, ret_type, span);
                    self.context.bind(extern_fn.name.clone(), func_type);
                }
            }
        }

        // Check for cyclic type aliases
        for item in &program.items {
            if let ast::Item::TypeAlias { name, .. } = item {
                if let Err(e) = self.context.check_alias_cycle(name) {
                    self.record_error(e)?;
                }
            }
        }

        // Second pass: type check bodies
        for item in &program.items {
            match self.check_item(item) {
                Ok(typed_item) => typed_items.push(typed_item),
                Err(e) => {
                    self.record_error(e)?;
                    // In recovery mode, add a placeholder item to keep indices aligned
                    if self.recovery_mode {
                        // Skip this item but continue checking others
                    }
                }
            }
        }

        // Solve constraints
        if let Err(e) = self.context.solve_constraints() {
            self.record_error(e)?;
        }

        // If in recovery mode and there are errors, return them
        if self.recovery_mode && !self.errors.is_empty() {
            // Return the first error for API compatibility
            return Err(self.errors.errors().first().unwrap().clone());
        }

        Ok(TypedProgram {
            items: typed_items,
            type_map: self.type_map.clone(),
        })
    }

    /// Check a program and return all errors (recovery mode)
    pub fn check_program_with_recovery(
        &mut self,
        program: &ast::Program,
    ) -> std::result::Result<TypedProgram, crate::error::Errors> {
        self.recovery_mode = true;
        let mut typed_items = Vec::new();

        // First pass: register all function signatures
        for item in &program.items {
            match item {
                ast::Item::Function { name, params, ret_ty, span, .. } |
                ast::Item::Transform { name, params, ret_ty, span, .. } => {
                    let param_types: Vec<Type> = params
                        .iter()
                        .map(|p| {
                            p.ty.clone()
                                .map(Type::from)
                                .unwrap_or_else(|| {
                                    self.context.fresh_type_var(*span)
                                })
                        })
                        .collect();

                    let ret_type = ret_ty.clone()
                        .map(Type::from)
                        .unwrap_or_else(|| self.context.fresh_type_var(*span));

                    let func_type = Type::function(param_types, ret_type, *span);
                    self.context.bind(name.clone(), func_type);
                }
                ast::Item::TypeAlias { name, ty, .. } => {
                    let resolved_ty = Type::from(ty.clone());
                    self.context.register_type_alias(name.clone(), resolved_ty.clone());
                    self.context.bind(name.clone(), resolved_ty);
                }
                ast::Item::Use(_) | ast::Item::Module(_) => {
                    // Use and module declarations handled by module resolver
                }
                ast::Item::ExternFunction(extern_fn) => {
                    // Register extern function signature
                    let span = extern_fn.span;
                    let param_types: Vec<Type> = extern_fn.params
                        .iter()
                        .map(|p| {
                            p.ty.clone()
                                .map(Type::from)
                                .unwrap_or_else(|| Type::unknown(0, span))
                        })
                        .collect();
                    let ret_type = extern_fn.ret_ty.clone()
                        .map(Type::from)
                        .unwrap_or_else(|| Type::unit(span));
                    let func_type = Type::function(param_types, ret_type, span);
                    self.context.bind(extern_fn.name.clone(), func_type);
                }
            }
        }

        // Check for cyclic type aliases
        for item in &program.items {
            if let ast::Item::TypeAlias { name, .. } = item {
                if let Err(e) = self.context.check_alias_cycle(name) {
                    self.errors.push(e);
                }
            }
        }

        // Second pass: type check bodies
        for item in &program.items {
            match self.check_item(item) {
                Ok(typed_item) => typed_items.push(typed_item),
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }

        // Solve constraints
        if let Err(e) = self.context.solve_constraints() {
            self.errors.push(e);
        }

        if self.errors.is_empty() {
            Ok(TypedProgram {
                items: typed_items,
                type_map: self.type_map.clone(),
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Check a top-level item
    fn check_item(&mut self, item: &ast::Item) -> Result<TypedItem> {
        match item {
            ast::Item::Function { name, params, ret_ty, body, span, .. } => {
                self.check_function(name, params, ret_ty.as_ref(), body, *span, false)
            }
            ast::Item::Transform { name, params, ret_ty, body, span, .. } => {
                self.check_function(name, params, ret_ty.as_ref(), body, *span, true)
            }
            ast::Item::TypeAlias { name, ty, .. } => {
                Ok(TypedItem::TypeAlias {
                    name: name.clone(),
                    ty: Type::from(ty.clone()),
                })
            }
            ast::Item::Use(use_stmt) => {
                // Use statements don't produce typed items directly
                Ok(TypedItem::Use { path: use_stmt.path.clone() })
            }
            ast::Item::Module(mod_decl) => {
                // Module declarations don't produce typed items directly
                Ok(TypedItem::Module { name: mod_decl.name.clone() })
            }
            ast::Item::ExternFunction(extern_fn) => {
                // Extern functions registered in first pass
                let span = extern_fn.span;
                let param_types: Vec<Type> = extern_fn.params
                    .iter()
                    .map(|p| p.ty.clone().map(Type::from).unwrap_or_else(|| Type::unknown(0, span)))
                    .collect();
                let ret_type = extern_fn.ret_ty.clone()
                    .map(Type::from)
                    .unwrap_or_else(|| Type::unit(span));
                Ok(TypedItem::ExternFunction {
                    name: extern_fn.name.clone(),
                    param_types,
                    ret_type,
                })
            }
        }
    }

    /// Check a function or transform
    fn check_function(
        &mut self,
        name: &str,
        params: &[ast::Param],
        declared_ret_ty: Option<&ast::Type>,
        body: &[ast::Stmt],
        span: Span,
        _is_transform: bool,
    ) -> Result<TypedItem> {
        // Create new scope for function body
        let mut param_bindings = Vec::new();

        for param in params {
            let param_ty = param.ty.clone()
                .map(Type::from)
                .unwrap_or_else(|| self.context.fresh_type_var(span));

            self.context.bind(param.name.clone(), param_ty.clone());
            param_bindings.push((param.name.clone(), param_ty));
        }

        // Type check body
        let typed_body = self.check_stmts(body)?;

        // Infer return type from body
        let inferred_ret_type = self.infer_return_type(body)?;

        // Validate against declared return type
        let ret_type = if let Some(declared) = declared_ret_ty {
            let declared_type = Type::from(declared.clone());
            if !inferred_ret_type.is_compatible_with(&declared_type) {
                return Err(Error::SemanticError {
                    message: format!(
                        "Function '{}' declared to return {}, but returns {}",
                        name, declared_type, inferred_ret_type
                    ),
                });
            }
            declared_type
        } else {
            inferred_ret_type
        };

        // Validate that all paths return if declared return type is non-Unit
        self.validate_return_paths(name, &ret_type, body, span)?;

        if _is_transform {
            Ok(TypedItem::Transform {
                name: name.to_string(),
                params: param_bindings,
                ret_type,
                body: typed_body,
            })
        } else {
            Ok(TypedItem::Function {
                name: name.to_string(),
                params: param_bindings,
                ret_type,
                body: typed_body,
            })
        }
    }

    /// Check a list of statements
    fn check_stmts(&mut self, stmts: &[ast::Stmt]) -> Result<Vec<TypedStmt>> {
        let mut typed_stmts = Vec::new();

        for stmt in stmts {
            let typed_stmt = self.check_stmt(stmt)?;
            typed_stmts.push(typed_stmt);
        }

        Ok(typed_stmts)
    }

    /// Check a statement
    fn check_stmt(&mut self, stmt: &ast::Stmt) -> Result<TypedStmt> {
        match stmt {
            ast::Stmt::Let { name, ty, value, span } => {
                let value_expr = self.infer_expr(value)?;

                let declared_ty = if let Some(ty) = ty {
                    let expected = Type::from(ty.clone());
                    // Check compatibility
                    if !value_expr.ty.is_compatible_with(&expected) {
                        return Err(Error::SemanticError {
                            message: format!(
                                "Type mismatch in let binding at line {}: expected {}, found {}",
                                span.line, expected, value_expr.ty
                            ),
                        });
                    }
                    expected
                } else {
                    value_expr.ty.clone()
                };

                self.context.bind(name.clone(), declared_ty.clone());

                Ok(TypedStmt::Let {
                    name: name.clone(),
                    ty: declared_ty,
                    value: value_expr,
                })
            }

            ast::Stmt::Assign { name, value, span } => {
                let var_ty = self.context.lookup(name)
                    .ok_or_else(|| Error::SemanticError {
                        message: format!("Undefined variable '{}' at line {}", name, span.line),
                    })?
                    .clone();

                let value_expr = self.infer_expr(value)?;

                if !value_expr.ty.is_compatible_with(&var_ty) {
                    return Err(Error::SemanticError {
                        message: format!(
                            "Type mismatch in assignment at line {}: cannot assign {} to variable of type {}",
                            span.line, value_expr.ty, var_ty
                        ),
                    });
                }

                Ok(TypedStmt::Assign {
                    name: name.clone(),
                    value: value_expr,
                })
            }

            ast::Stmt::Expr(expr) => {
                let typed_expr = self.infer_expr(expr)?;
                Ok(TypedStmt::Expr(typed_expr))
            }

            ast::Stmt::Return(expr, _span) => {
                let typed_expr = expr.as_ref().map(|e| self.infer_expr(e)).transpose()?;
                Ok(TypedStmt::Return(typed_expr))
            }

            ast::Stmt::Loop { body, .. } => {
                let typed_body = self.check_stmts(body)?;
                Ok(TypedStmt::Loop { body: typed_body })
            }

            ast::Stmt::Break(_) => Ok(TypedStmt::Break),

            ast::Stmt::Continue(_) => Ok(TypedStmt::Continue),

            ast::Stmt::If { condition, then_body, else_body, span } => {
                let cond_expr = self.infer_expr(condition)?;

                if !matches!(cond_expr.ty.kind, TypeKind::Bool) {
                    return Err(Error::SemanticError {
                        message: format!(
                            "If condition at line {} must be Bool, found {}",
                            span.line, cond_expr.ty
                        ),
                    });
                }

                let typed_then = self.check_stmts(then_body)?;
                let typed_else = else_body.as_ref()
                    .map(|body| self.check_stmts(body))
                    .transpose()?;

                Ok(TypedStmt::If {
                    condition: cond_expr,
                    then_body: typed_then,
                    else_body: typed_else,
                })
            }
        }
    }

    /// Infer type of expression
    fn infer_expr(&mut self, expr: &ast::Expr) -> Result<TypedExpr> {
        let ty = self.context.synthesize_expr(expr)?;
        let node_id = self.alloc_node_id();
        self.type_map.insert(node_id, ty.clone());

        let typed_expr_kind = match &expr.kind {
            ExprKind::Literal(lit) => Expr::Literal(lit.clone()),

            ExprKind::Ident(name) => Expr::Ident(name.clone()),

            ExprKind::Binary { left, op, right } => {
                let left_typed = self.infer_expr(left)?;
                let right_typed = self.infer_expr(right)?;
                Expr::Binary {
                    left: Box::new(left_typed),
                    op: *op,
                    right: Box::new(right_typed),
                }
            }

            ExprKind::Unary { op, expr } => {
                let expr_typed = self.infer_expr(expr)?;
                Expr::Unary {
                    op: *op,
                    expr: Box::new(expr_typed),
                }
            }

            ExprKind::Call { name, args } => {
                let args_typed = args.iter()
                    .map(|arg| self.infer_expr(arg))
                    .collect::<Result<Vec<_>>>()?;
                Expr::Call {
                    name: name.clone(),
                    args: args_typed,
                }
            }

            ExprKind::LayerAccess { expr, layer } => {
                let expr_typed = self.infer_expr(expr)?;
                Expr::LayerAccess {
                    expr: Box::new(expr_typed),
                    layer: *layer,
                }
            }

            ExprKind::StateConstruct { layers } => {
                let layers_typed = layers.iter()
                    .map(|(l, e)| Ok((*l, self.infer_expr(e)?)))
                    .collect::<Result<Vec<_>>>()?;
                Expr::StateConstruct { layers: layers_typed }
            }

            ExprKind::Complex { rho, theta } => {
                let rho_typed = self.infer_expr(rho)?;
                let theta_typed = self.infer_expr(theta)?;
                Expr::Complex {
                    rho: Box::new(rho_typed),
                    theta: Box::new(theta_typed),
                }
            }

            ExprKind::Tuple { elements } => {
                let elements_typed = elements.iter()
                    .map(|e| self.infer_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Expr::Tuple { elements: elements_typed }
            }

            ExprKind::Pipe { expr, transform } => {
                let expr_typed = self.infer_expr(expr)?;
                Expr::Pipe {
                    expr: Box::new(expr_typed),
                    transform: transform.clone(),
                }
            }

            ExprKind::Feedback { expr } => {
                let expr_typed = self.infer_expr(expr)?;
                Expr::Feedback {
                    expr: Box::new(expr_typed),
                }
            }

            ExprKind::Emerge { expr } => {
                let expr_typed = self.infer_expr(expr)?;
                Expr::Emerge {
                    expr: Box::new(expr_typed),
                }
            }
        };

        Ok(TypedExpr {
            expr: typed_expr_kind,
            ty,
            node_id,
        })
    }

    /// Infer return type from function body
    fn infer_return_type(&mut self, body: &[ast::Stmt]) -> Result<Type> {
        // Look for explicit return statements
        for stmt in body {
            if let ast::Stmt::Return(Some(expr), _) = stmt {
                return self.context.synthesize_expr(expr);
            }
            // Check if statements (return could be inside a branch)
            if let ast::Stmt::If { then_body, else_body, .. } = stmt {
                if let Some(ty) = self.find_return_type_in_stmts(then_body) {
                    return Ok(ty);
                }
                if let Some(else_stmts) = else_body {
                    if let Some(ty) = self.find_return_type_in_stmts(else_stmts) {
                        return Ok(ty);
                    }
                }
            }
        }

        // No explicit return, assume Unit
        Ok(Type::unit(Span::dummy()))
    }

    /// Helper to find return type in nested statements
    fn find_return_type_in_stmts(&mut self, stmts: &[ast::Stmt]) -> Option<Type> {
        for stmt in stmts {
            if let ast::Stmt::Return(Some(expr), _) = stmt {
                return self.context.synthesize_expr(expr).ok();
            }
            if let ast::Stmt::If { then_body, else_body, .. } = stmt {
                if let Some(ty) = self.find_return_type_in_stmts(then_body) {
                    return Some(ty);
                }
                if let Some(else_stmts) = else_body {
                    if let Some(ty) = self.find_return_type_in_stmts(else_stmts) {
                        return Some(ty);
                    }
                }
            }
        }
        None
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Reachability Analysis
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check if all paths in a function body return a value
    /// Returns true if all execution paths return (or never terminate)
    fn all_paths_return(&self, body: &[ast::Stmt]) -> bool {
        for stmt in body {
            match stmt {
                // Explicit return always terminates
                ast::Stmt::Return(_, _) => return true,

                // If-else: both branches must return
                ast::Stmt::If { then_body, else_body, .. } => {
                    let then_returns = self.all_paths_return(then_body);
                    let else_returns = else_body
                        .as_ref()
                        .map(|body| self.all_paths_return(body))
                        .unwrap_or(false);

                    // Both branches return = all paths covered
                    if then_returns && else_returns {
                        return true;
                    }
                }

                // Infinite loop (without break) technically returns
                ast::Stmt::Loop { body, .. } => {
                    // If the loop has no break, it's an infinite loop
                    if !self.contains_break(body) {
                        return true;
                    }
                }

                // Other statements continue execution
                _ => {}
            }
        }

        false
    }

    /// Check if a statement list contains a break
    fn contains_break(&self, stmts: &[ast::Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                ast::Stmt::Break(_) => return true,
                ast::Stmt::If { then_body, else_body, .. } => {
                    if self.contains_break(then_body) {
                        return true;
                    }
                    if let Some(else_stmts) = else_body {
                        if self.contains_break(else_stmts) {
                            return true;
                        }
                    }
                }
                ast::Stmt::Loop { body, .. } => {
                    // Breaks in nested loops don't break outer loop
                    // but we still want to check the outer context
                    if self.contains_break(body) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Validate that a function with a non-Unit return type actually returns on all paths
    pub fn validate_return_paths(
        &self,
        name: &str,
        ret_type: &Type,
        body: &[ast::Stmt],
        span: Span,
    ) -> Result<()> {
        // If return type is Unit, no need to check
        if matches!(ret_type.kind, TypeKind::Unit) {
            return Ok(());
        }

        // Check if all paths return
        if !self.all_paths_return(body) {
            return Err(Error::SemanticError {
                message: format!(
                    "Function '{}' at line {} must return {} on all paths, but some paths have no return statement",
                    name, span.line, ret_type
                ),
            });
        }

        Ok(())
    }

    /// Allocate a new node ID
    fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr as AstExpr, ExprKind as AstExprKind, Item, Literal, Program, Stmt};
    use crate::lexer::Span;

    fn dummy_expr(kind: AstExprKind) -> AstExpr {
        AstExpr::new(kind, Span::dummy())
    }

    #[test]
    fn test_simple_function() {
        let mut checker = TypeChecker::new();

        let program = Program {
            items: vec![Item::Function {
                name: "add".to_string(),
                params: vec![
                    ast::Param { name: "a".to_string(), ty: Some(ast::Type::Named("Int".to_string())) },
                    ast::Param { name: "b".to_string(), ty: Some(ast::Type::Named("Int".to_string())) },
                ],
                ret_ty: Some(ast::Type::Named("Int".to_string())),
                body: vec![
                    Stmt::Return(Some(dummy_expr(AstExprKind::Binary {
                        left: Box::new(dummy_expr(AstExprKind::Ident("a".to_string()))),
                        op: BinOp::Add,
                        right: Box::new(dummy_expr(AstExprKind::Ident("b".to_string()))),
                    })), Span::dummy()),
                ],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = checker.check_program(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let mut checker = TypeChecker::new();

        let program = Program {
            items: vec![Item::Function {
                name: "bad".to_string(),
                params: vec![],
                ret_ty: None,
                body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        ty: Some(ast::Type::Named("Int".to_string())),
                        value: dummy_expr(AstExprKind::Literal(Literal::Bool(true))),
                        span: Span::dummy(),
                    },
                ],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = checker.check_program(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_paths_return_simple() {
        let checker = TypeChecker::new();
        let body = vec![
            Stmt::Return(Some(dummy_expr(AstExprKind::Literal(Literal::Int(42)))), Span::dummy()),
        ];
        assert!(checker.all_paths_return(&body));
    }

    #[test]
    fn test_all_paths_return_if_else() {
        let checker = TypeChecker::new();
        let body = vec![
            Stmt::If {
                condition: dummy_expr(AstExprKind::Literal(Literal::Bool(true))),
                then_body: vec![
                    Stmt::Return(Some(dummy_expr(AstExprKind::Literal(Literal::Int(1)))), Span::dummy()),
                ],
                else_body: Some(vec![
                    Stmt::Return(Some(dummy_expr(AstExprKind::Literal(Literal::Int(2)))), Span::dummy()),
                ]),
                span: Span::dummy(),
            },
        ];
        assert!(checker.all_paths_return(&body));
    }

    #[test]
    fn test_all_paths_return_if_without_else_fails() {
        let checker = TypeChecker::new();
        let body = vec![
            Stmt::If {
                condition: dummy_expr(AstExprKind::Literal(Literal::Bool(true))),
                then_body: vec![
                    Stmt::Return(Some(dummy_expr(AstExprKind::Literal(Literal::Int(1)))), Span::dummy()),
                ],
                else_body: None,
                span: Span::dummy(),
            },
        ];
        assert!(!checker.all_paths_return(&body));
    }

    #[test]
    fn test_all_paths_return_no_return() {
        let checker = TypeChecker::new();
        let body = vec![
            Stmt::Let {
                name: "x".to_string(),
                ty: None,
                value: dummy_expr(AstExprKind::Literal(Literal::Int(42))),
                span: Span::dummy(),
            },
        ];
        assert!(!checker.all_paths_return(&body));
    }

    #[test]
    fn test_reachability_validation() {
        let mut checker = TypeChecker::new();

        // Function with non-Unit return type but no return statement
        let program = Program {
            items: vec![Item::Function {
                name: "missing_return".to_string(),
                params: vec![],
                ret_ty: Some(ast::Type::Named("Int".to_string())),
                body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        ty: None,
                        value: dummy_expr(AstExprKind::Literal(Literal::Int(42))),
                        span: Span::dummy(),
                    },
                ],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = checker.check_program(&program);
        assert!(result.is_err(), "Expected error but got: {:?}", result);
        let err_msg = format!("{:?}", result.unwrap_err());
        // The error can be detected at declaration check (returns Unit) or reachability (all paths)
        assert!(
            err_msg.contains("returns Unit") || err_msg.contains("all paths") || err_msg.contains("must return"),
            "Expected type mismatch error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_reachability_with_if_else_return() {
        let mut checker = TypeChecker::new();

        // Function with return in both branches of if-else
        let program = Program {
            items: vec![Item::Function {
                name: "good_function".to_string(),
                params: vec![
                    ast::Param { name: "x".to_string(), ty: Some(ast::Type::Named("Bool".to_string())) },
                ],
                ret_ty: Some(ast::Type::Named("Int".to_string())),
                body: vec![
                    Stmt::If {
                        condition: dummy_expr(AstExprKind::Ident("x".to_string())),
                        then_body: vec![
                            Stmt::Return(Some(dummy_expr(AstExprKind::Literal(Literal::Int(1)))), Span::dummy()),
                        ],
                        else_body: Some(vec![
                            Stmt::Return(Some(dummy_expr(AstExprKind::Literal(Literal::Int(0)))), Span::dummy()),
                        ]),
                        span: Span::dummy(),
                    },
                ],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = checker.check_program(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_recovery_collects_multiple_errors() {
        let mut checker = TypeChecker::new();

        // Program with multiple errors
        let program = Program {
            items: vec![
                // First function with type mismatch
                Item::Function {
                    name: "bad1".to_string(),
                    params: vec![],
                    ret_ty: None,
                    body: vec![
                        Stmt::Let {
                            name: "x".to_string(),
                            ty: Some(ast::Type::Named("Int".to_string())),
                            value: dummy_expr(AstExprKind::Literal(Literal::Bool(true))),
                            span: Span::dummy(),
                        },
                    ],
                    hardware_hint: None,
                    is_pub: false,
                    span: Span::dummy(),
                },
                // Second function with another type mismatch
                Item::Function {
                    name: "bad2".to_string(),
                    params: vec![],
                    ret_ty: None,
                    body: vec![
                        Stmt::Let {
                            name: "y".to_string(),
                            ty: Some(ast::Type::Named("Bool".to_string())),
                            value: dummy_expr(AstExprKind::Literal(Literal::Int(42))),
                            span: Span::dummy(),
                        },
                    ],
                    hardware_hint: None,
                    is_pub: false,
                    span: Span::dummy(),
                },
            ],
        };

        let result = checker.check_program_with_recovery(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        // Should have collected multiple errors
        assert!(errors.len() >= 2, "Expected at least 2 errors, got {}", errors.len());
    }

    #[test]
    fn test_error_recovery_success() {
        let mut checker = TypeChecker::new();

        // Valid program
        let program = Program {
            items: vec![Item::Function {
                name: "good".to_string(),
                params: vec![],
                ret_ty: None,
                body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        ty: Some(ast::Type::Named("Int".to_string())),
                        value: dummy_expr(AstExprKind::Literal(Literal::Int(42))),
                        span: Span::dummy(),
                    },
                ],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = checker.check_program_with_recovery(&program);
        assert!(result.is_ok());
    }
}
