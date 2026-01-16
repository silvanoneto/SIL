//! LLVM Intrinsics for LIS Standard Library
//!
//! Maps LIS stdlib functions to LLVM intrinsics or generated code.

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;

use crate::ast::Expr;
use crate::error::{Error, Result};
use super::codegen::LlvmCodegen;
use super::types::LlvmTypes;

/// Intrinsic function registry
pub struct Intrinsics<'ctx> {
    // Math intrinsics
    llvm_abs: FunctionValue<'ctx>,
    llvm_sqrt: FunctionValue<'ctx>,
    llvm_pow: FunctionValue<'ctx>,
    llvm_sin: FunctionValue<'ctx>,
    llvm_cos: FunctionValue<'ctx>,
    llvm_exp: FunctionValue<'ctx>,
    llvm_log: FunctionValue<'ctx>,
    llvm_floor: FunctionValue<'ctx>,
    llvm_ceil: FunctionValue<'ctx>,

    // I/O (external C functions)
    printf: FunctionValue<'ctx>,

    // Types helper
    types: LlvmTypes<'ctx>,
}

impl<'ctx> Intrinsics<'ctx> {
    /// Create and register all intrinsics
    pub fn new(context: &'ctx Context, module: &Module<'ctx>, types: &LlvmTypes<'ctx>) -> Self {
        let i64_type = context.i64_type();
        let f64_type = context.f64_type();
        let ptr_type = context.ptr_type(AddressSpace::default());

        // ═══════════════════════════════════════════════════════════════════════════════
        // LLVM Math Intrinsics
        // ═══════════════════════════════════════════════════════════════════════════════

        // llvm.abs.i64
        let abs_type = i64_type.fn_type(&[i64_type.into(), context.bool_type().into()], false);
        let llvm_abs = module.add_function("llvm.abs.i64", abs_type, None);

        // llvm.sqrt.f64
        let sqrt_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_sqrt = module.add_function("llvm.sqrt.f64", sqrt_type, None);

        // llvm.pow.f64
        let pow_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        let llvm_pow = module.add_function("llvm.pow.f64", pow_type, None);

        // llvm.sin.f64
        let sin_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_sin = module.add_function("llvm.sin.f64", sin_type, None);

        // llvm.cos.f64
        let cos_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_cos = module.add_function("llvm.cos.f64", cos_type, None);

        // llvm.exp.f64
        let exp_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_exp = module.add_function("llvm.exp.f64", exp_type, None);

        // llvm.log.f64
        let log_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_log = module.add_function("llvm.log.f64", log_type, None);

        // llvm.floor.f64
        let floor_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_floor = module.add_function("llvm.floor.f64", floor_type, None);

        // llvm.ceil.f64
        let ceil_type = f64_type.fn_type(&[f64_type.into()], false);
        let llvm_ceil = module.add_function("llvm.ceil.f64", ceil_type, None);

        // ═══════════════════════════════════════════════════════════════════════════════
        // External C Functions
        // ═══════════════════════════════════════════════════════════════════════════════

        // printf
        let printf_type = i64_type.fn_type(&[ptr_type.into()], true);
        let printf = module.add_function("printf", printf_type, None);

        Self {
            llvm_abs,
            llvm_sqrt,
            llvm_pow,
            llvm_sin,
            llvm_cos,
            llvm_exp,
            llvm_log,
            llvm_floor,
            llvm_ceil,
            printf,
            types: LlvmTypes::new(context),
        }
    }

    /// Try to compile a function call as an intrinsic
    pub fn try_compile_intrinsic(
        &self,
        builder: &Builder<'ctx>,
        name: &str,
        args: &[Expr],
        codegen: &mut LlvmCodegen<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
        match name {
            // ═══════════════════════════════════════════════════════════════════════════════
            // Math Intrinsics
            // ═══════════════════════════════════════════════════════════════════════════════

            "abs_int" | "abs" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires 1 argument", name),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let false_val = codegen.types().context().bool_type().const_int(0, false);
                let call = builder.build_call(self.llvm_abs, &[val.into(), false_val.into()], "abs")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(call.try_as_basic_value().left().unwrap()))
            }

            "sqrt" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "sqrt requires 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                // Convert i64 to f64
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_sqrt, &[f64_val.into()], "sqrt")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                // Convert back to i64
                let result = call.try_as_basic_value().left().unwrap();
                let i64_result = builder.build_float_to_signed_int(
                    result.into_float_value(),
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "sin" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "sin requires 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                // Scale down (assuming fixed-point * 1000)
                let scale = self.types.float_type().const_float(1000.0);
                let scaled = builder.build_float_div(f64_val, scale, "scale_down")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_sin, &[scaled.into()], "sin")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                // Scale up result
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "cos" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "cos requires 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let scale = self.types.float_type().const_float(1000.0);
                let scaled = builder.build_float_div(f64_val, scale, "scale_down")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_cos, &[scaled.into()], "cos")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "exp" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "exp requires 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let scale = self.types.float_type().const_float(1000.0);
                let scaled = builder.build_float_div(f64_val, scale, "scale_down")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_exp, &[scaled.into()], "exp")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "ln" | "log" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires 1 argument", name),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let scale = self.types.float_type().const_float(1000.0);
                let scaled = builder.build_float_div(f64_val, scale, "scale_down")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_log, &[scaled.into()], "log")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "pow_float" | "pow" => {
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires 2 arguments", name),
                    });
                }
                let base = codegen.compile_expr(&args[0])?.into_int_value();
                let exp = codegen.compile_expr(&args[1])?.into_int_value();
                let base_f64 = builder.build_signed_int_to_float(base, self.types.float_type(), "base_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let exp_f64 = builder.build_signed_int_to_float(exp, self.types.float_type(), "exp_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let scale = self.types.float_type().const_float(1000.0);
                let base_scaled = builder.build_float_div(base_f64, scale, "base_scaled")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let exp_scaled = builder.build_float_div(exp_f64, scale, "exp_scaled")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_pow, &[base_scaled.into(), exp_scaled.into()], "pow")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "floor" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "floor requires 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let scale = self.types.float_type().const_float(1000.0);
                let scaled = builder.build_float_div(f64_val, scale, "scale_down")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_floor, &[scaled.into()], "floor")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            "ceil" => {
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "ceil requires 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let f64_val = builder.build_signed_int_to_float(val, self.types.float_type(), "to_f64")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let scale = self.types.float_type().const_float(1000.0);
                let scaled = builder.build_float_div(f64_val, scale, "scale_down")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let call = builder.build_call(self.llvm_ceil, &[scaled.into()], "ceil")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = call.try_as_basic_value().left().unwrap();
                let scaled_up = builder.build_float_mul(result.into_float_value(), scale, "scale_up")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let i64_result = builder.build_float_to_signed_int(
                    scaled_up,
                    self.types.int_type(),
                    "to_i64",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(i64_result.into()))
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // Integer Math (no conversion needed)
            // ═══════════════════════════════════════════════════════════════════════════════

            "min_int" | "min" => {
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires 2 arguments", name),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();
                let b = codegen.compile_expr(&args[1])?.into_int_value();
                let cmp = builder.build_int_compare(inkwell::IntPredicate::SLT, a, b, "min_cmp")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_select(cmp, a, b, "min")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(result))
            }

            "max_int" | "max" => {
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires 2 arguments", name),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();
                let b = codegen.compile_expr(&args[1])?.into_int_value();
                let cmp = builder.build_int_compare(inkwell::IntPredicate::SGT, a, b, "max_cmp")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_select(cmp, a, b, "max")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(result))
            }

            "clamp_int" | "clamp" => {
                if args.len() != 3 {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires 3 arguments", name),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let min = codegen.compile_expr(&args[1])?.into_int_value();
                let max = codegen.compile_expr(&args[2])?.into_int_value();

                // clamp = max(min, min(val, max))
                let cmp1 = builder.build_int_compare(inkwell::IntPredicate::SLT, val, max, "clamp_max_cmp")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let min_val_max = builder.build_select(cmp1, val, max, "clamp_max")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let cmp2 = builder.build_int_compare(
                    inkwell::IntPredicate::SGT,
                    min_val_max.into_int_value(),
                    min,
                    "clamp_min_cmp",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_select(cmp2, min_val_max, min.into(), "clamp")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(result))
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // I/O Intrinsics
            // ═══════════════════════════════════════════════════════════════════════════════

            "print_int" | "println" => {
                if args.is_empty() {
                    return Err(Error::CodeGenError {
                        message: format!("{} requires at least 1 argument", name),
                    });
                }
                let val = codegen.compile_expr(&args[0])?.into_int_value();
                let fmt = builder.build_global_string_ptr("%lld\n", "int_fmt")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                builder.build_call(self.printf, &[fmt.as_pointer_value().into(), val.into()], "printf")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(self.types.int_type().const_int(0, false).into()))
            }

            "print_string" => {
                if args.is_empty() {
                    return Err(Error::CodeGenError {
                        message: "print_string requires at least 1 argument".to_string(),
                    });
                }
                let val = codegen.compile_expr(&args[0])?;
                // Convert i64 back to pointer
                let ptr = builder.build_int_to_ptr(
                    val.into_int_value(),
                    codegen.types().context().ptr_type(AddressSpace::default()),
                    "str_ptr",
                ).map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let fmt = builder.build_global_string_ptr("%s", "str_fmt")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                builder.build_call(self.printf, &[fmt.as_pointer_value().into(), ptr.into()], "printf")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(self.types.int_type().const_int(0, false).into()))
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // ByteSil Intrinsics (Log-Polar Math - O(1) operations)
            // ═══════════════════════════════════════════════════════════════════════════════

            "bytesil_mul" => {
                // (ρ₁, θ₁) × (ρ₂, θ₂) = (ρ₁ + ρ₂, θ₁ + θ₂)
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: "bytesil_mul requires 2 arguments".to_string(),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();
                let b = codegen.compile_expr(&args[1])?.into_int_value();

                // Extract rho and theta (packed as (rho << 8) | theta)
                let mask_8 = self.types.int_type().const_int(0xFF, false);
                let eight = self.types.int_type().const_int(8, false);

                let rho_a = builder.build_right_shift(a, eight, false, "rho_a")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_a = builder.build_and(a, mask_8, "theta_a")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let rho_b = builder.build_right_shift(b, eight, false, "rho_b")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_b = builder.build_and(b, mask_8, "theta_b")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                // Add (O(1) operation!)
                let rho_sum = builder.build_int_add(rho_a, rho_b, "rho_sum")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_sum = builder.build_int_add(theta_a, theta_b, "theta_sum")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                // Pack result
                let rho_shifted = builder.build_left_shift(rho_sum, eight, "rho_shift")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_or(rho_shifted, theta_sum, "bytesil_mul")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                Ok(Some(result.into()))
            }

            "bytesil_div" => {
                // (ρ₁, θ₁) ÷ (ρ₂, θ₂) = (ρ₁ - ρ₂, θ₁ - θ₂)
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: "bytesil_div requires 2 arguments".to_string(),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();
                let b = codegen.compile_expr(&args[1])?.into_int_value();

                let mask_8 = self.types.int_type().const_int(0xFF, false);
                let eight = self.types.int_type().const_int(8, false);

                let rho_a = builder.build_right_shift(a, eight, false, "rho_a")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_a = builder.build_and(a, mask_8, "theta_a")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let rho_b = builder.build_right_shift(b, eight, false, "rho_b")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_b = builder.build_and(b, mask_8, "theta_b")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                // Subtract (O(1) operation!)
                let rho_diff = builder.build_int_sub(rho_a, rho_b, "rho_diff")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_diff = builder.build_int_sub(theta_a, theta_b, "theta_diff")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                let rho_shifted = builder.build_left_shift(rho_diff, eight, "rho_shift")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_or(rho_shifted, theta_diff, "bytesil_div")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                Ok(Some(result.into()))
            }

            "bytesil_pow" => {
                // (ρ, θ)^n = (n·ρ, n·θ)
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: "bytesil_pow requires 2 arguments".to_string(),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();
                let n = codegen.compile_expr(&args[1])?.into_int_value();

                let mask_8 = self.types.int_type().const_int(0xFF, false);
                let eight = self.types.int_type().const_int(8, false);

                let rho = builder.build_right_shift(a, eight, false, "rho")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta = builder.build_and(a, mask_8, "theta")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                // Multiply by n (O(1) operation!)
                let rho_pow = builder.build_int_mul(rho, n, "rho_pow")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta_pow = builder.build_int_mul(theta, n, "theta_pow")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                let rho_shifted = builder.build_left_shift(rho_pow, eight, "rho_shift")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_or(rho_shifted, theta_pow, "bytesil_pow")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                Ok(Some(result.into()))
            }

            "bytesil_conj" => {
                // Conjugate: (ρ, θ) → (ρ, -θ)
                if args.len() != 1 {
                    return Err(Error::CodeGenError {
                        message: "bytesil_conj requires 1 argument".to_string(),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();

                let mask_8 = self.types.int_type().const_int(0xFF, false);
                let eight = self.types.int_type().const_int(8, false);

                let rho = builder.build_right_shift(a, eight, false, "rho")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let theta = builder.build_and(a, mask_8, "theta")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let neg_theta = builder.build_int_neg(theta, "neg_theta")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                let rho_shifted = builder.build_left_shift(rho, eight, "rho_shift")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_or(rho_shifted, neg_theta, "bytesil_conj")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                Ok(Some(result.into()))
            }

            "bytesil_xor" => {
                // XOR: (ρ₁ ⊕ ρ₂, θ₁ ⊕ θ₂)
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: "bytesil_xor requires 2 arguments".to_string(),
                    });
                }
                let a = codegen.compile_expr(&args[0])?.into_int_value();
                let b = codegen.compile_expr(&args[1])?.into_int_value();
                let result = builder.build_xor(a, b, "bytesil_xor")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                Ok(Some(result.into()))
            }

            // ═══════════════════════════════════════════════════════════════════════════════
            // State Intrinsics
            // ═══════════════════════════════════════════════════════════════════════════════

            "state_neutral" | "state_vacuum" => {
                // Return 0 (neutral state representation)
                Ok(Some(self.types.int_type().const_int(0, false).into()))
            }

            "state_get_layer" => {
                if args.len() != 2 {
                    return Err(Error::CodeGenError {
                        message: "state_get_layer requires 2 arguments".to_string(),
                    });
                }
                let state = codegen.compile_expr(&args[0])?.into_int_value();
                let layer = codegen.compile_expr(&args[1])?.into_int_value();

                // Extract layer: (state >> (layer * 4)) & 0xF
                let four = self.types.int_type().const_int(4, false);
                let mask = self.types.int_type().const_int(0xF, false);
                let shift = builder.build_int_mul(layer, four, "layer_shift")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let shifted = builder.build_right_shift(state, shift, false, "state_shifted")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;
                let result = builder.build_and(shifted, mask, "layer_val")
                    .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

                Ok(Some(result.into()))
            }

            // Not an intrinsic - return None to use regular function call
            _ => Ok(None),
        }
    }
}
