//! LLVM Backend for LIS
//!
//! This module provides JIT and AOT compilation via LLVM.
//!
//! ## Architecture
//!
//! ```text
//! Typed AST
//!     ↓ LlvmCodegen
//! LLVM IR (inkwell Module)
//!     ↓ JitEngine / AotCompiler
//! Native Code
//! ```
//!
//! ## Type Mapping
//!
//! | LIS Type   | LLVM Type              | Notes                          |
//! |------------|------------------------|--------------------------------|
//! | Int        | i64                    | 64-bit signed integer          |
//! | Float      | f64                    | 64-bit IEEE 754 double         |
//! | Bool       | i1                     | Single bit boolean             |
//! | ByteSil    | { i8, i8 }             | Struct: (rho, theta)           |
//! | State      | [16 x { i8, i8 }]      | Array of 16 ByteSils           |
//! | String     | i8*                    | Pointer to null-terminated     |

mod codegen;
mod intrinsics;
mod jit;
mod types;

pub use codegen::LlvmCodegen;
pub use jit::{JitEngine, JitFunction};
pub use types::LlvmTypes;

use crate::error::Result;
use crate::Program;

/// Compile LIS program to LLVM IR (as string for debugging)
pub fn compile_to_llvm_ir(source: &str) -> Result<String> {
    let tokens = crate::Lexer::new(source).tokenize()?;
    let ast = crate::Parser::new(tokens).parse()?;

    let context = inkwell::context::Context::create();
    let codegen = LlvmCodegen::new(&context, "lis_module");
    let module = codegen.compile(&ast)?;

    Ok(module.print_to_string().to_string())
}

/// JIT compile and execute LIS program
pub fn jit_execute(source: &str) -> Result<i64> {
    let tokens = crate::Lexer::new(source).tokenize()?;
    let ast = crate::Parser::new(tokens).parse()?;

    let context = inkwell::context::Context::create();
    let codegen = LlvmCodegen::new(&context, "lis_jit");
    let module = codegen.compile(&ast)?;

    let engine = JitEngine::new(module)?;
    engine.run_main()
}

/// AOT compile LIS program to object file
pub fn compile_to_object(source: &str, output_path: &str) -> Result<()> {
    let tokens = crate::Lexer::new(source).tokenize()?;
    let ast = crate::Parser::new(tokens).parse()?;

    let context = inkwell::context::Context::create();
    let codegen = LlvmCodegen::new(&context, "lis_aot");
    let module = codegen.compile(&ast)?;

    use inkwell::targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
    };

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| crate::Error::CodeGenError { message: e.to_string() })?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| crate::Error::CodeGenError { message: e.to_string() })?;

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            inkwell::OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| crate::Error::CodeGenError {
            message: "Failed to create target machine".to_string(),
        })?;

    machine
        .write_to_file(&module, FileType::Object, std::path::Path::new(output_path))
        .map_err(|e| crate::Error::CodeGenError { message: e.to_string() })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_to_ir() {
        let source = r#"
            fn main() {
                let x = 42;
                return x;
            }
        "#;
        let ir = compile_to_llvm_ir(source).unwrap();
        assert!(ir.contains("define"));
        assert!(ir.contains("main"));
    }

    #[test]
    fn test_jit_simple() {
        let source = r#"
            fn main() {
                return 42;
            }
        "#;
        let result = jit_execute(source).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_jit_arithmetic() {
        let source = r#"
            fn main() {
                let a = 10;
                let b = 32;
                return a + b;
            }
        "#;
        let result = jit_execute(source).unwrap();
        assert_eq!(result, 42);
    }
}
