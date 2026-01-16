//! # 🏃 lis-runtime — Runtime for LIS Programs
//!
//! Executa programas LIS compilados para bytecode VSP.
//!
//! ## Fluxo de Execução
//!
//! ```text
//! LIS source (.lis)
//!      ↓
//! LIS Compiler (lis-core)
//!      ↓
//! VSP Assembly (.sil)
//!      ↓
//! VSP Assembler (sil-core/vsp)
//!      ↓
//! VSP Bytecode (.silc)
//!      ↓
//! LIS Runtime (THIS!) ←─── Executa aqui
//!      ↓
//! SilState (resultado)
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use lis_runtime::{LisRuntime, RuntimeConfig};
//!
//! // Carregar programa LIS
//! let mut runtime = LisRuntime::new(RuntimeConfig::default());
//! runtime.load_lis_file("program.lis")?;
//!
//! // Executar
//! let result = runtime.run()?;
//! println!("Result: {:?}", result);
//! ```

pub mod error;
pub mod runtime;
pub mod loader;
pub mod executor;

pub use error::{RuntimeError, RuntimeResult};
pub use runtime::{LisRuntime, RuntimeConfig};
pub use loader::ProgramLoader;
pub use executor::Executor;

// Re-export core types
pub use sil_core::prelude::*;
pub use lis_core::{Compiler, Lexer, Parser};
