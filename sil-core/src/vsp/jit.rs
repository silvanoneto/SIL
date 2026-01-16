//! Compilador JIT (Just-In-Time) para VSP
//!
//! Compila bytecode VSP para código nativo em runtime,
//! permitindo execução imediata sem pré-compilação.

use crate::vsp::{VspResult, VspError};
use crate::vsp::bytecode::SilcFile;
use crate::state::SilState;
use std::collections::HashMap;

#[cfg(feature = "jit")]
use cranelift_jit::{JITBuilder, JITModule};
#[cfg(feature = "jit")]
use cranelift_module::{Module, Linkage};
#[cfg(feature = "jit")]
use cranelift::prelude::*;
#[cfg(feature = "jit")]
use cranelift_native;

/// Compilador JIT do VSP
pub struct VspJit {
    #[cfg(feature = "jit")]
    module: JITModule,
    
    /// Funções compiladas (nome -> ponteiro)
    compiled_functions: HashMap<String, *const u8>,
    
    /// Estatísticas
    pub stats: JitStats,
}

/// Estatísticas de JIT
#[derive(Debug, Default, Clone)]
pub struct JitStats {
    /// Número de funções compiladas
    pub functions_compiled: usize,
    
    /// Tempo total de compilação (microsegundos)
    pub total_compile_time_us: u64,
    
    /// Número de execuções
    pub executions: usize,
    
    /// Código gerado (bytes)
    pub code_size_bytes: usize,
}

impl VspJit {
    /// Cria novo compilador JIT
    #[cfg(feature = "jit")]
    pub fn new() -> VspResult<Self> {
        let mut flag_builder = cranelift_codegen::settings::builder();
        
        // Configurar para ARM64
        flag_builder.set("is_pic", "true")
            .map_err(|e| VspError::CompilationError(format!("PIC flag failed: {}", e)))?;
        flag_builder.set("use_colocated_libcalls", "false")
            .map_err(|e| VspError::CompilationError(format!("Libcalls flag failed: {}", e)))?;
        flag_builder.set("opt_level", "speed")
            .map_err(|e| VspError::CompilationError(format!("Opt level failed: {}", e)))?;
        
        let flags = cranelift_codegen::settings::Flags::new(flag_builder);
        
        let isa_builder = cranelift_native::builder()
            .map_err(|e| VspError::CompilationError(format!("ISA builder failed: {}", e)))?;
        
        let isa = isa_builder.finish(flags)
            .map_err(|e| VspError::CompilationError(format!("ISA creation failed: {}", e)))?;
        
        // Criar builder com uma closure vazia para libcalls (evita PLT em ARM64)
        let libcall_names = Box::new(|_: cranelift_codegen::ir::LibCall| -> String {
            // Não queremos libcalls externos por enquanto
            String::new()
        });
        
        let builder = JITBuilder::with_isa(isa, libcall_names);
        let module = JITModule::new(builder);
        
        Ok(Self {
            module,
            compiled_functions: HashMap::new(),
            stats: JitStats::default(),
        })
    }
    
    #[cfg(not(feature = "jit"))]
    pub fn new() -> VspResult<Self> {
        Err(VspError::CompilationError(
            "JIT compilation requires 'jit' feature".to_string()
        ))
    }
    
    /// Compila bytecode para código nativo
    #[cfg(feature = "jit")]
    pub fn compile(&mut self, name: &str, bytecode: SilcFile) -> VspResult<()> {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Criar contexto de função
        let mut ctx = self.module.make_context();
        let mut fn_builder_ctx = FunctionBuilderContext::new();
        
        // Build function signature: fn(state: *mut SilState) -> i32
        let ptr_type = self.module.target_config().pointer_type();
        ctx.func.signature.params.push(cranelift_codegen::ir::AbiParam::new(ptr_type));
        ctx.func.signature.returns.push(cranelift_codegen::ir::AbiParam::new(cranelift_codegen::ir::types::I32));
        
        // Build IR usando função compartilhada
        super::codegen::build_vsp_function(&mut ctx, &mut fn_builder_ctx, &bytecode)?;
        
        // Declarar função
        let func_id = self.module
            .declare_function(name, Linkage::Export, &ctx.func.signature)
            .map_err(|e| VspError::CompilationError(format!("Function declaration failed: {}", e)))?;
        
        // Definir função
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| VspError::CompilationError(format!("Function definition failed: {}", e)))?;
        
        self.module.clear_context(&mut ctx);
        
        // Finalizar compilação
        self.module.finalize_definitions()
            .map_err(|e| VspError::CompilationError(format!("Finalization failed: {}", e)))?;
        
        // Obter ponteiro de código
        let code_ptr = self.module.get_finalized_function(func_id);
        self.compiled_functions.insert(name.to_string(), code_ptr);
        
        // Atualizar estatísticas
        let elapsed = start.elapsed();
        self.stats.functions_compiled += 1;
        self.stats.total_compile_time_us += elapsed.as_micros() as u64;
        self.stats.code_size_bytes += 256; // Estimativa (sem API direta para obter tamanho)
        
        Ok(())
    }
    
    #[cfg(not(feature = "jit"))]
    pub fn compile(&mut self, _name: &str, _bytecode: SilcFile) -> VspResult<()> {
        Err(VspError::CompilationError(
            "JIT compilation requires 'jit' feature".to_string()
        ))
    }
    
    /// Executa função compilada
    pub fn execute(&mut self, name: &str, state: &mut SilState) -> VspResult<i32> {
        let func_ptr = self.compiled_functions.get(name)
            .ok_or_else(|| VspError::CompilationError(
                format!("Function '{}' not compiled. Call compile() first.", name)
            ))?;
        
        // Cast pointer para função
        let func: unsafe extern "C" fn(*mut SilState) -> i32 = unsafe {
            std::mem::transmute(*func_ptr)
        };
        
        // Executar
        let result = unsafe { func(state) };
        
        // Atualizar estatísticas
        self.stats.executions += 1;
        
        Ok(result)
    }
    
    /// Compila e executa em uma operação
    pub fn compile_and_execute(
        &mut self,
        name: &str,
        bytecode: SilcFile,
        state: &mut SilState,
    ) -> VspResult<i32> {
        self.compile(name, bytecode)?;
        self.execute(name, state)
    }
    
    /// Verifica se função está compilada
    pub fn is_compiled(&self, name: &str) -> bool {
        self.compiled_functions.contains_key(name)
    }
    
    /// Limpa todas as funções compiladas
    pub fn clear(&mut self) {
        self.compiled_functions.clear();
        self.stats = JitStats::default();
    }
    
    /// Tempo médio de compilação
    pub fn avg_compile_time_us(&self) -> f64 {
        if self.stats.functions_compiled == 0 {
            0.0
        } else {
            self.stats.total_compile_time_us as f64 / self.stats.functions_compiled as f64
        }
    }
}

impl std::fmt::Debug for VspJit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VspJit")
            .field("compiled_functions", &self.compiled_functions.keys())
            .field("stats", &self.stats)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vsp::bytecode::SilcHeader;
    use crate::vsp::state::SilMode;
    
    fn create_test_bytecode() -> SilcFile {
        let mut header = SilcHeader::new(SilMode::Sil128);
        header.code_size = 16;
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_bytes());
        data.extend_from_slice(&[0u8; 16]); // NOPs
        
        SilcFile::from_bytes(&data).unwrap()
    }
    
    #[test]
    #[cfg(feature = "jit")]
    fn test_jit_compile() {
        let mut jit = VspJit::new().unwrap();
        let bytecode = create_test_bytecode();
        
        assert!(jit.compile("test", bytecode).is_ok());
        assert!(jit.is_compiled("test"));
        assert_eq!(jit.stats.functions_compiled, 1);
    }
    
    #[test]
    #[cfg(feature = "jit")]
    fn test_jit_execute() {
        let mut jit = VspJit::new().unwrap();
        let bytecode = create_test_bytecode();
        let mut state = SilState::neutral();
        
        jit.compile("test", bytecode).unwrap();
        let result = jit.execute("test", &mut state).unwrap();
        
        assert_eq!(result, 0); // Stub retorna 0
        assert_eq!(jit.stats.executions, 1);
    }
    
    #[test]
    #[cfg(feature = "jit")]
    fn test_jit_compile_and_execute() {
        let mut jit = VspJit::new().unwrap();
        let bytecode = create_test_bytecode();
        let mut state = SilState::neutral();
        
        let result = jit.compile_and_execute("test", bytecode, &mut state).unwrap();
        assert_eq!(result, 0);
    }
}
