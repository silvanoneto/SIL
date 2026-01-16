//! Program loader for LIS files

use crate::error::{RuntimeError, RuntimeResult};
use lis_core::{Compiler, Lexer, Parser};
use std::fs;
use std::path::Path;

/// Carregador de programas LIS
pub struct ProgramLoader;

impl ProgramLoader {
    /// Carrega e compila arquivo LIS para VSP assembly
    pub fn load_lis_file(path: impl AsRef<Path>) -> RuntimeResult<String> {
        let source = fs::read_to_string(path.as_ref())
            .map_err(|_| RuntimeError::FileNotFound(path.as_ref().display().to_string()))?;

        Self::compile_lis(&source)
    }

    /// Compila código LIS para VSP assembly
    pub fn compile_lis(source: &str) -> RuntimeResult<String> {
        // Lexer
        let tokens = Lexer::new(source).tokenize()?;

        // Parser (use from_tokens for backward compatibility with plain Token vec)
        let program = Parser::from_tokens(tokens).parse()?;

        // Compiler
        let mut compiler = Compiler::new();
        let assembly = compiler.compile(&program)?;

        Ok(assembly)
    }

    /// Carrega arquivo VSP assembly diretamente
    pub fn load_vsp_file(path: impl AsRef<Path>) -> RuntimeResult<String> {
        fs::read_to_string(path.as_ref())
            .map_err(|_| RuntimeError::FileNotFound(path.as_ref().display().to_string()))
    }
}
