//! # LIS - Language for Intelligent Systems
//!
//! LIS is a programming language for modeling non-linear systems that compiles to SIL VSP bytecode.
//!
//! ## Core Philosophy
//!
//! - **Non-linear by design**: Native support for feedback loops, topology, and emergence
//! - **Self-compiling**: Reflexive metaprogramming and adaptive optimization
//! - **Hardware-aware**: Type system reflects computation substrate (CPU/GPU/NPU)
//! - **Edge-native**: Distributed execution on the swarm
//!
//! ## Example
//!
//! ```lis
//! fn main() {
//!     let state = sense();
//!     let processed = transform(state);
//!     act(processed);
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! LIS Source (.lis)
//!     ↓ lexer
//! Token Stream
//!     ↓ parser
//! AST
//!     ↓ compiler
//! VSP Assembly (.sil)
//!     ↓ assembler (sil-core)
//! Bytecode (.silc)
//!     ↓ VSP runtime
//! Execution
//! ```

pub mod ast;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod manifest;
pub mod parser;
pub mod resolver;
pub mod types;

#[cfg(feature = "jsil")]
pub mod stdlib;

#[cfg(feature = "python")]
pub mod python_bindings;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "llvm")]
pub mod llvm;

pub use ast::{Expr, ExprKind, Item, Literal, Program, Stmt, UseStatement, ModuleDecl, ExternFn};
pub use compiler::Compiler;
pub use error::{Error, Result, TypeErrorKind};
pub use lexer::{Lexer, Span, SpannedToken, Token};
pub use manifest::{Manifest, Package, Dependency, BuildConfig};
pub use parser::Parser;
pub use resolver::{ModuleResolver, ResolvedModule, resolve_single_file};
pub use types::checker::TypeChecker;
pub use types::{Type, TypeKind};

/// Parse LIS source code into an AST
pub fn parse(source: &str) -> Result<Program> {
    let tokens = Lexer::new(source).tokenize_with_spans()?;
    Parser::new(tokens).parse()
}

/// Compile LIS source code to VSP assembly
pub fn compile(source: &str) -> Result<String> {
    let ast = parse(source)?;
    let mut compiler = Compiler::new();
    compiler.compile(&ast)
}

/// Compile LIS source code to VSP bytecode (via sil-core assembler)
#[cfg(feature = "bytecode")]
pub fn compile_to_bytecode(source: &str) -> Result<Vec<u8>> {
    let assembly = compile(source)?;
    // TODO: integrate with sil-core assembler
    todo!("Integration with sil-core assembler")
}

/// Compile LIS source code directly to JSIL format
#[cfg(feature = "jsil")]
pub fn compile_to_jsil(
    source: &str,
    output_path: &str,
    compression: Option<sil_core::io::jsil::CompressionMode>,
) -> Result<JsilStats> {
    use sil_core::io::jsil::{JsilWriter, JsilCompressor, CompressionMode};

    // 1. Parse source
    let tokens = Lexer::new(source).tokenize_with_spans()?;
    let ast = Parser::new(tokens).parse()?;

    // 2. Compile to JSONL records
    let mut compiler = Compiler::new();
    let records = compiler.compile_to_jsonl(&ast)?;

    // 3. Write to JSIL file
    let mode = compression.unwrap_or(CompressionMode::XorRotate);
    let compressor = JsilCompressor::new(mode, 0x5A);
    let mut writer = JsilWriter::new(compressor);

    for record in records {
        writer
            .write_record(&record)
            .map_err(|e| Error::CodeGenError {
                message: format!("Failed to write JSIL record: {}", e),
            })?;
    }

    let header = writer.save(output_path).map_err(|e| Error::IoError {
        message: format!("Failed to save JSIL file: {}", e),
    })?;

    Ok(JsilStats {
        uncompressed_size: header.uncompressed_size as usize,
        compressed_size: header.compressed_size as usize,
        record_count: header.record_count as usize,
        compression_ratio: header.compression_ratio(),
    })
}

/// Statistics from JSIL compilation
#[cfg(feature = "jsil")]
#[derive(Debug, Clone)]
pub struct JsilStats {
    pub uncompressed_size: usize,
    pub compressed_size: usize,
    pub record_count: usize,
    pub compression_ratio: f64,
}

#[cfg(feature = "jsil")]
impl JsilStats {
    pub fn report(&self) -> String {
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
}

// PyO3 module is exposed directly via the pymodule attribute
// No need to re-export it

// ═══════════════════════════════════════════════════════════════════════════════
// LLVM JIT/AOT Compilation
// ═══════════════════════════════════════════════════════════════════════════════

/// Compile LIS source code to LLVM IR (for debugging)
#[cfg(feature = "llvm")]
pub fn compile_to_llvm_ir(source: &str) -> Result<String> {
    llvm::compile_to_llvm_ir(source)
}

/// JIT compile and execute LIS program, returning main()'s result
#[cfg(feature = "llvm")]
pub fn jit_execute(source: &str) -> Result<i64> {
    llvm::jit_execute(source)
}

/// AOT compile LIS program to native object file (.o)
#[cfg(feature = "llvm")]
pub fn compile_to_object(source: &str, output_path: &str) -> Result<()> {
    llvm::compile_to_object(source, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_empty() {
        let result = compile("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_simple() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let result = compile(source);
        assert!(result.is_ok());
    }
}
