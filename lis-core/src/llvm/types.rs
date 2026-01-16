//! LLVM Type definitions for LIS types
//!
//! Maps LIS types to LLVM IR types.

use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};

/// Helper struct for creating LLVM types from LIS types
pub struct LlvmTypes<'ctx> {
    context: &'ctx Context,
    bytesil_type: StructType<'ctx>,
    state_type: inkwell::types::ArrayType<'ctx>,
}

impl<'ctx> LlvmTypes<'ctx> {
    /// Create a new LlvmTypes helper
    pub fn new(context: &'ctx Context) -> Self {
        // ByteSil = { i8 rho, i8 theta }
        let i8_type = context.i8_type();
        let bytesil_type = context.struct_type(&[i8_type.into(), i8_type.into()], false);

        // State = [16 x ByteSil]
        let state_type = bytesil_type.array_type(16);

        Self {
            context,
            bytesil_type,
            state_type,
        }
    }

    /// Get the LLVM context
    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Primitive Types
    // ═══════════════════════════════════════════════════════════════════════════════

    /// i1 (boolean)
    pub fn bool_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.bool_type()
    }

    /// i8
    pub fn i8_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.i8_type()
    }

    /// i64 (LIS Int)
    pub fn int_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.i64_type()
    }

    /// f64 (LIS Float)
    pub fn float_type(&self) -> inkwell::types::FloatType<'ctx> {
        self.context.f64_type()
    }

    /// void
    pub fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.context.void_type()
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // LIS-Specific Types
    // ═══════════════════════════════════════════════════════════════════════════════

    /// ByteSil = { i8 rho, i8 theta }
    pub fn bytesil_type(&self) -> StructType<'ctx> {
        self.bytesil_type
    }

    /// State = [16 x ByteSil]
    pub fn state_type(&self) -> inkwell::types::ArrayType<'ctx> {
        self.state_type
    }

    /// Pointer to i8 (for strings)
    pub fn string_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(inkwell::AddressSpace::default())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Type Conversion
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Convert LIS AST type to LLVM type
    pub fn from_lis_type(&self, ty: &crate::ast::Type) -> BasicTypeEnum<'ctx> {
        match ty {
            crate::ast::Type::ByteSil => self.bytesil_type.into(),
            crate::ast::Type::State => self.state_type.into(),
            crate::ast::Type::Layer(_) => self.bytesil_type.into(),
            crate::ast::Type::Hardware(_) => self.int_type().into(), // Hardware hints don't change type
            crate::ast::Type::Function { .. } => {
                // Function pointers - use i64 as placeholder
                self.int_type().into()
            }
            crate::ast::Type::Named(name) => {
                match name.as_str() {
                    "Int" | "int" => self.int_type().into(),
                    "Float" | "float" => self.float_type().into(),
                    "Bool" | "bool" => self.bool_type().into(),
                    "String" | "string" => self.string_type().into(),
                    "ByteSil" => self.bytesil_type.into(),
                    "State" => self.state_type.into(),
                    _ => self.int_type().into(), // Default to int
                }
            }
        }
    }

    /// Convert to BasicMetadataTypeEnum (for function parameters)
    pub fn to_metadata_type(&self, ty: BasicTypeEnum<'ctx>) -> BasicMetadataTypeEnum<'ctx> {
        ty.into()
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ByteSil Operations (Log-Polar Math)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Create a ByteSil constant { rho, theta }
    pub fn bytesil_const(&self, rho: u8, theta: u8) -> inkwell::values::StructValue<'ctx> {
        let rho_val = self.i8_type().const_int(rho as u64, false);
        let theta_val = self.i8_type().const_int(theta as u64, false);
        self.bytesil_type.const_named_struct(&[rho_val.into(), theta_val.into()])
    }

    /// ByteSil null (0, 0) - represents zero
    pub fn bytesil_null(&self) -> inkwell::values::StructValue<'ctx> {
        self.bytesil_const(0, 0)
    }

    /// ByteSil one (1, 0) - represents 1.0
    pub fn bytesil_one(&self) -> inkwell::values::StructValue<'ctx> {
        self.bytesil_const(1, 0)
    }

    /// ByteSil i (1, 64) - represents i (90° = 64 in 256-phase)
    pub fn bytesil_i(&self) -> inkwell::values::StructValue<'ctx> {
        self.bytesil_const(1, 64)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // State Operations
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Create a neutral state (all layers = null)
    pub fn state_neutral(&self) -> inkwell::values::ArrayValue<'ctx> {
        let null = self.bytesil_null();
        self.state_type.const_array(&[
            null, null, null, null,
            null, null, null, null,
            null, null, null, null,
            null, null, null, null,
        ])
    }
}
