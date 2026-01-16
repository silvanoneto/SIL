//! Main runtime implementation

use crate::error::{RuntimeError, RuntimeResult};
use crate::loader::ProgramLoader;
use crate::executor::Executor;
use sil_core::prelude::*;
use std::path::Path;

/// Configuração do runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Modo de execução (SIL-128, SIL-256, etc.)
    pub mode: String,

    /// Limite de ciclos de execução
    pub max_cycles: usize,

    /// Debug mode
    pub debug: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            mode: "SIL-128".to_string(),
            max_cycles: 10000,
            debug: false,
        }
    }
}

/// Runtime principal para programas LIS
pub struct LisRuntime {
    config: RuntimeConfig,
    executor: Executor,
    assembly: Option<String>,
}

impl LisRuntime {
    /// Cria novo runtime com configuração padrão
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            executor: Executor::new(),
            assembly: None,
        }
    }

    /// Carrega programa LIS de arquivo
    pub fn load_lis_file(&mut self, path: impl AsRef<Path>) -> RuntimeResult<()> {
        if self.config.debug {
            println!("📂 Loading LIS file: {}", path.as_ref().display());
        }

        let assembly = ProgramLoader::load_lis_file(path)?;

        if self.config.debug {
            println!("✓ Compiled to VSP assembly ({} bytes)", assembly.len());
            println!("\n{}\n", "═".repeat(60));
            println!("{}", assembly);
            println!("{}\n", "═".repeat(60));
        }

        self.assembly = Some(assembly);
        Ok(())
    }

    /// Carrega programa VSP assembly diretamente
    pub fn load_vsp_file(&mut self, path: impl AsRef<Path>) -> RuntimeResult<()> {
        if self.config.debug {
            println!("📂 Loading VSP assembly: {}", path.as_ref().display());
        }

        let assembly = ProgramLoader::load_vsp_file(path)?;
        self.assembly = Some(assembly);
        Ok(())
    }

    /// Carrega código LIS source diretamente
    pub fn load_lis_source(&mut self, source: &str) -> RuntimeResult<()> {
        if self.config.debug {
            println!("📝 Compiling LIS source...");
        }

        let assembly = ProgramLoader::compile_lis(source)?;

        if self.config.debug {
            println!("✓ Compiled to VSP assembly");
        }

        self.assembly = Some(assembly);
        Ok(())
    }

    /// Executa programa carregado
    pub fn run(&mut self) -> RuntimeResult<SilState> {
        let assembly = self.assembly.as_ref()
            .ok_or_else(|| RuntimeError::ExecutionError(
                "No program loaded".to_string()
            ))?;

        if self.config.debug {
            println!("🚀 Executing program...\n");
        }

        let result = self.executor.execute(assembly)?;

        if self.config.debug {
            println!("\n✅ Execution complete");
            println!("\n📊 Final state:");
            Self::print_state(&result);
            println!("\n🔧 Registers:");
            Self::print_registers(self.executor.registers());
        }

        Ok(result)
    }

    /// Executa arquivo LIS diretamente (load + run)
    pub fn run_file(&mut self, path: impl AsRef<Path>) -> RuntimeResult<SilState> {
        self.load_lis_file(path)?;
        self.run()
    }

    /// Executa código LIS source diretamente
    pub fn run_source(&mut self, source: &str) -> RuntimeResult<SilState> {
        self.load_lis_source(source)?;
        self.run()
    }

    /// Print state summary
    fn print_state(state: &SilState) {
        for layer in 0..16 {
            let byte = state.get(layer);
            if byte.rho != 0 || byte.theta != 0 {
                println!("   L{:X}: ρ={:+3}, θ={:3}", layer, byte.rho, byte.theta);
            }
        }
    }

    /// Print registers
    fn print_registers(registers: &[ByteSil; 16]) {
        for (i, reg) in registers.iter().enumerate() {
            if reg.rho != 0 || reg.theta != 0 {
                println!("   R{:X}: ρ={:+3}, θ={:3}", i, reg.rho, reg.theta);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_simple_program() {
        let source = r#"
            fn main() {
                let x = 10;
                let y = 20;
                let z = x + y;
            }
        "#;

        let mut runtime = LisRuntime::new(RuntimeConfig::default());
        let result = runtime.run_source(source).unwrap();

        // O resultado deve estar em algum registrador
        assert_eq!(result, SilState::neutral()); // Por enquanto, estado neutral
    }
}
