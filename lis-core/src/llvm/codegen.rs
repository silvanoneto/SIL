//! LLVM Code Generation for LIS AST
//!
//! Translates LIS AST to LLVM IR using inkwell.

use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

use crate::ast::{BinOp, Expr, Item, Literal, Program, Stmt, UnOp};
use crate::error::{Error, Result};
use super::intrinsics::Intrinsics;
use super::types::LlvmTypes;

/// LLVM Code Generator for LIS
pub struct LlvmCodegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    types: LlvmTypes<'ctx>,
    intrinsics: Intrinsics<'ctx>,

    /// Current function being compiled
    current_fn: Option<FunctionValue<'ctx>>,

    /// Variable name -> stack allocation
    variables: HashMap<String, PointerValue<'ctx>>,

    /// Function name -> LLVM function
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Loop context for break/continue
    loop_stack: Vec<LoopContext<'ctx>>,
}

/// Context for loop compilation (break/continue targets)
struct LoopContext<'ctx> {
    /// Block to jump to for `continue`
    continue_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// Block to jump to for `break`
    break_block: inkwell::basic_block::BasicBlock<'ctx>,
}

impl<'ctx> LlvmCodegen<'ctx> {
    /// Create a new code generator
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let types = LlvmTypes::new(context);
        let intrinsics = Intrinsics::new(context, &module, &types);

        Self {
            context,
            module,
            builder,
            types,
            intrinsics,
            current_fn: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Compile a LIS program to LLVM module
    pub fn compile(mut self, program: &Program) -> Result<Module<'ctx>> {
        // First pass: declare all functions
        for item in &program.items {
            self.declare_item(item)?;
        }

        // Second pass: compile function bodies
        for item in &program.items {
            self.compile_item(item)?;
        }

        // Verify the module
        self.module
            .verify()
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        Ok(self.module)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Item Compilation
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Declare a function (first pass)
    fn declare_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function { name, params, .. } | Item::Transform { name, params, .. } => {
                let param_types: Vec<_> = params
                    .iter()
                    .map(|_| self.types.int_type().into())
                    .collect();

                let fn_type = self.types.int_type().fn_type(&param_types, false);
                let function = self.module.add_function(name, fn_type, None);

                // Name parameters
                for (i, param) in params.iter().enumerate() {
                    if let Some(llvm_param) = function.get_nth_param(i as u32) {
                        llvm_param.set_name(&param.name);
                    }
                }

                self.functions.insert(name.clone(), function);
            }
            Item::TypeAlias { .. } => {
                // Type aliases are handled at type-check time
            }
        }
        Ok(())
    }

    /// Compile a function body (second pass)
    fn compile_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function { name, params, body } | Item::Transform { name, params, body } => {
                let function = *self.functions.get(name).unwrap();
                self.current_fn = Some(function);
                self.variables.clear();

                // Create entry block
                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Allocate space for parameters and store them
                for (i, param) in params.iter().enumerate() {
                    let llvm_param = function.get_nth_param(i as u32).unwrap();
                    let alloca = self.builder.build_alloca(self.types.int_type(), &param.name)
                        .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                    self.builder.build_store(alloca, llvm_param)
                        .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                    self.variables.insert(param.name.clone(), alloca);
                }

                // Compile body
                let mut has_terminator = false;
                for stmt in body {
                    self.compile_stmt(stmt)?;
                    if self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                        has_terminator = true;
                        break;
                    }
                }

                // Add implicit return 0 if no explicit return
                if !has_terminator {
                    let zero = self.types.int_type().const_int(0, false);
                    self.builder.build_return(Some(&zero))
                        .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                }

                self.current_fn = None;
            }
            Item::TypeAlias { .. } => {}
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Statement Compilation
    // ═══════════════════════════════════════════════════════════════════════════════

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.compile_expr(value)?;
                let alloca = self.builder.build_alloca(val.get_type(), name)
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                self.builder.build_store(alloca, val)
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                self.variables.insert(name.clone(), alloca);
            }

            Stmt::Assign { name, value } => {
                let val = self.compile_expr(value)?;
                let ptr = self.variables.get(name).ok_or_else(|| Error::CodeGenError {
                    message: format!("Undefined variable: {}", name),
                })?;
                self.builder.build_store(*ptr, val)
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
            }

            Stmt::Return(expr) => {
                match expr {
                    Some(e) => {
                        let val = self.compile_expr(e)?;
                        self.builder.build_return(Some(&val))
                            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                    }
                    None => {
                        let zero = self.types.int_type().const_int(0, false);
                        self.builder.build_return(Some(&zero))
                            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                    }
                }
            }

            Stmt::If { condition, then_body, else_body } => {
                self.compile_if(condition, then_body, else_body.as_deref())?;
            }

            Stmt::Loop { body } => {
                self.compile_loop(body)?;
            }

            Stmt::Break => {
                if let Some(ctx) = self.loop_stack.last() {
                    self.builder.build_unconditional_branch(ctx.break_block)
                        .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                } else {
                    return Err(Error::CodeGenError {
                        message: "Break outside of loop".to_string(),
                    });
                }
            }

            Stmt::Continue => {
                if let Some(ctx) = self.loop_stack.last() {
                    self.builder.build_unconditional_branch(ctx.continue_block)
                        .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                } else {
                    return Err(Error::CodeGenError {
                        message: "Continue outside of loop".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn compile_if(
        &mut self,
        condition: &Expr,
        then_body: &[Stmt],
        else_body: Option<&[Stmt]>,
    ) -> Result<()> {
        let function = self.current_fn.unwrap();

        let cond_val = self.compile_expr(condition)?;
        let cond_int = cond_val.into_int_value();

        // Compare to zero to get i1
        let cond_bool = self.builder.build_int_compare(
            IntPredicate::NE,
            cond_int,
            self.types.int_type().const_int(0, false),
            "ifcond",
        ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "ifcont");

        self.builder.build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        // Compile then block
        self.builder.position_at_end(then_block);
        for stmt in then_body {
            self.compile_stmt(stmt)?;
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(merge_block)
                .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
        }

        // Compile else block
        self.builder.position_at_end(else_block);
        if let Some(else_stmts) = else_body {
            for stmt in else_stmts {
                self.compile_stmt(stmt)?;
            }
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(merge_block)
                .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
        }

        // Continue at merge block
        self.builder.position_at_end(merge_block);
        Ok(())
    }

    fn compile_loop(&mut self, body: &[Stmt]) -> Result<()> {
        let function = self.current_fn.unwrap();

        let loop_block = self.context.append_basic_block(function, "loop");
        let after_block = self.context.append_basic_block(function, "afterloop");

        self.loop_stack.push(LoopContext {
            continue_block: loop_block,
            break_block: after_block,
        });

        // Jump to loop
        self.builder.build_unconditional_branch(loop_block)
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        // Compile loop body
        self.builder.position_at_end(loop_block);
        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(loop_block)
                .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
        }

        self.loop_stack.pop();

        // Continue after loop
        self.builder.position_at_end(after_block);
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Expression Compilation
    // ═══════════════════════════════════════════════════════════════════════════════

    fn compile_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Literal(lit) => self.compile_literal(lit),

            Expr::Ident(name) => {
                let ptr = self.variables.get(name).ok_or_else(|| Error::CodeGenError {
                    message: format!("Undefined variable: {}", name),
                })?;
                let val = self.builder.build_load(self.types.int_type(), *ptr, name)
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(val)
            }

            Expr::Binary { left, op, right } => {
                let lhs = self.compile_expr(left)?.into_int_value();
                let rhs = self.compile_expr(right)?.into_int_value();
                self.compile_binary_op(lhs, *op, rhs)
            }

            Expr::Unary { op, expr } => {
                let val = self.compile_expr(expr)?.into_int_value();
                self.compile_unary_op(*op, val)
            }

            Expr::Call { name, args } => {
                self.compile_call(name, args)
            }

            Expr::LayerAccess { expr, layer } => {
                self.compile_layer_access(expr, *layer)
            }

            Expr::StateConstruct { layers } => {
                self.compile_state_construct(layers)
            }

            Expr::Complex { rho, theta } => {
                self.compile_complex(rho, theta)
            }

            Expr::Pipe { expr, transform } => {
                // Pipe is sugar for transform(expr)
                let val = self.compile_expr(expr)?;
                self.compile_call(transform, &[Expr::Literal(Literal::Int(0))])
                    .or_else(|_| Ok(val))
            }

            Expr::Feedback { expr } => {
                // Feedback loop - compile as identity for now
                self.compile_expr(expr)
            }

            Expr::Emerge { expr } => {
                // Emergence - compile as identity for now
                self.compile_expr(expr)
            }
        }
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(n) => {
                Ok(self.types.int_type().const_int(*n as u64, true).into())
            }
            Literal::Float(f) => {
                // Convert float to fixed-point i64 (multiply by 1000)
                let scaled = (*f * 1000.0) as i64;
                Ok(self.types.int_type().const_int(scaled as u64, true).into())
            }
            Literal::Bool(b) => {
                let val = if *b { 1 } else { 0 };
                Ok(self.types.int_type().const_int(val, false).into())
            }
            Literal::String(s) => {
                let global = self.builder.build_global_string_ptr(s, "str")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                // Convert pointer to i64 for simplicity
                let ptr_val = self.builder.build_ptr_to_int(
                    global.as_pointer_value(),
                    self.types.int_type(),
                    "str_ptr",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(ptr_val.into())
            }
        }
    }

    fn compile_binary_op(
        &mut self,
        lhs: IntValue<'ctx>,
        op: BinOp,
        rhs: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>> {
        let result = match op {
            BinOp::Add => self.builder.build_int_add(lhs, rhs, "add"),
            BinOp::Sub => self.builder.build_int_sub(lhs, rhs, "sub"),
            BinOp::Mul => self.builder.build_int_mul(lhs, rhs, "mul"),
            BinOp::Div => self.builder.build_int_signed_div(lhs, rhs, "div"),
            BinOp::Pow => {
                // Power: use intrinsic or loop
                // For simplicity, use repeated multiplication for small powers
                // TODO: Use llvm.powi intrinsic
                return Ok(lhs.into()); // Placeholder
            }
            BinOp::Eq => {
                let cmp = self.builder.build_int_compare(IntPredicate::EQ, lhs, rhs, "eq")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return Ok(self.builder.build_int_z_extend(cmp, self.types.int_type(), "eq_ext")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?.into());
            }
            BinOp::Ne => {
                let cmp = self.builder.build_int_compare(IntPredicate::NE, lhs, rhs, "ne")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return Ok(self.builder.build_int_z_extend(cmp, self.types.int_type(), "ne_ext")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?.into());
            }
            BinOp::Lt => {
                let cmp = self.builder.build_int_compare(IntPredicate::SLT, lhs, rhs, "lt")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return Ok(self.builder.build_int_z_extend(cmp, self.types.int_type(), "lt_ext")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?.into());
            }
            BinOp::Le => {
                let cmp = self.builder.build_int_compare(IntPredicate::SLE, lhs, rhs, "le")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return Ok(self.builder.build_int_z_extend(cmp, self.types.int_type(), "le_ext")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?.into());
            }
            BinOp::Gt => {
                let cmp = self.builder.build_int_compare(IntPredicate::SGT, lhs, rhs, "gt")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return Ok(self.builder.build_int_z_extend(cmp, self.types.int_type(), "gt_ext")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?.into());
            }
            BinOp::Ge => {
                let cmp = self.builder.build_int_compare(IntPredicate::SGE, lhs, rhs, "ge")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return Ok(self.builder.build_int_z_extend(cmp, self.types.int_type(), "ge_ext")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?.into());
            }
            BinOp::And => self.builder.build_and(lhs, rhs, "and"),
            BinOp::Or => self.builder.build_or(lhs, rhs, "or"),
            BinOp::Xor => self.builder.build_xor(lhs, rhs, "xor"),
            BinOp::BitAnd => self.builder.build_and(lhs, rhs, "bitand"),
            BinOp::BitOr => self.builder.build_or(lhs, rhs, "bitor"),
        };

        result
            .map(|v| v.into())
            .map_err(|e| Error::CodeGenError { message: e.to_string() })
    }

    fn compile_unary_op(&mut self, op: UnOp, val: IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        let result = match op {
            UnOp::Neg => self.builder.build_int_neg(val, "neg"),
            UnOp::Not => self.builder.build_not(val, "not"),
            UnOp::Conj => {
                // Complex conjugate: negate theta (lower 8 bits)
                // For i64, this is a simplification
                self.builder.build_int_neg(val, "conj")
            }
            UnOp::Mag => {
                // Magnitude: abs(val)
                // Use select: val < 0 ? -val : val
                let zero = self.types.int_type().const_int(0, false);
                let is_neg = self.builder.build_int_compare(IntPredicate::SLT, val, zero, "is_neg")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let neg_val = self.builder.build_int_neg(val, "neg_val")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                return self.builder.build_select(is_neg, neg_val, val, "mag")
                    .map(|v| v.into())
                    .map_err(|e| Error::CodeGenError { message: e.to_string() });
            }
        };

        result
            .map(|v| v.into())
            .map_err(|e| Error::CodeGenError { message: e.to_string() })
    }

    fn compile_call(&mut self, name: &str, args: &[Expr]) -> Result<BasicValueEnum<'ctx>> {
        // Check for intrinsic first
        if let Some(result) = self.intrinsics.try_compile_intrinsic(&self.builder, name, args, self)? {
            return Ok(result);
        }

        // Look up user-defined function
        let function = self.functions.get(name).ok_or_else(|| Error::CodeGenError {
            message: format!("Undefined function: {}", name),
        })?;

        let compiled_args: Vec<_> = args
            .iter()
            .map(|a| self.compile_expr(a).map(|v| v.into()))
            .collect::<Result<Vec<_>>>()?;

        let call = self.builder.build_call(*function, &compiled_args, "call")
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        Ok(call.try_as_basic_value().left().unwrap_or_else(|| {
            self.types.int_type().const_int(0, false).into()
        }))
    }

    fn compile_layer_access(&mut self, expr: &Expr, layer: u8) -> Result<BasicValueEnum<'ctx>> {
        // For now, return the layer index as a placeholder
        // In full implementation, this would extract from State struct
        let _ = self.compile_expr(expr)?;
        Ok(self.types.int_type().const_int(layer as u64, false).into())
    }

    fn compile_state_construct(&mut self, layers: &[(u8, Expr)]) -> Result<BasicValueEnum<'ctx>> {
        // For now, return sum of layer values as a placeholder
        // In full implementation, this would construct State struct
        let mut sum = self.types.int_type().const_int(0, false);
        for (_, expr) in layers {
            let val = self.compile_expr(expr)?.into_int_value();
            sum = self.builder.build_int_add(sum, val, "layer_sum")
                .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
        }
        Ok(sum.into())
    }

    fn compile_complex(&mut self, rho: &Expr, theta: &Expr) -> Result<BasicValueEnum<'ctx>> {
        // Pack rho and theta into i64: (rho << 8) | theta
        let rho_val = self.compile_expr(rho)?.into_int_value();
        let theta_val = self.compile_expr(theta)?.into_int_value();

        let eight = self.types.int_type().const_int(8, false);
        let shifted = self.builder.build_left_shift(rho_val, eight, "rho_shift")
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
        let packed = self.builder.build_or(shifted, theta_val, "complex")
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        Ok(packed.into())
    }

    /// Get the types helper (for intrinsics)
    pub fn types(&self) -> &LlvmTypes<'ctx> {
        &self.types
    }
}
