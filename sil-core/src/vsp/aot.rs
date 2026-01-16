//! Compilador AOT (Ahead-Of-Time) para VSP
//!
//! Compila bytecode VSP para código nativo em tempo de build,
//! eliminando overhead de interpretação e JIT compilation.

use crate::vsp::{VspResult, VspError};
use crate::vsp::bytecode::SilcFile;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
#[cfg(feature = "jit")]
use cranelift_module::{Module, Linkage};
#[cfg(feature = "jit")]
use cranelift_object::{ObjectModule, ObjectBuilder};
#[cfg(feature = "jit")]
use cranelift_codegen::settings::{self, Configurable};
#[cfg(feature = "jit")]
use cranelift_native;

/// Resultado de compilação AOT
#[derive(Debug)]
pub struct AotCompilation {
    /// Nome do módulo
    pub name: String,
    
    /// Bytecode original (placeholder)
    pub bytecode_size: usize,
    
    /// Objeto compilado (ELF/Mach-O/PE)
    pub object_data: Vec<u8>,
    
    /// Símbolos exportados
    pub symbols: Vec<String>,
    
    /// Metadados
    pub metadata: CompilationMetadata,
}

impl AotCompilation {
    /// Salva compilação em arquivo
    pub fn save<P: AsRef<Path>>(&self, path: P) -> VspResult<()> {
        let path = path.as_ref();
        
        // Salvar objeto
        fs::write(path, &self.object_data)?;
        
        // Salvar metadados
        let metadata_path = path.with_extension("meta");
        let metadata_json = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| VspError::SerializationError(e.to_string()))?;
        fs::write(metadata_path, metadata_json)?;
        
        Ok(())
    }
    
    /// Carrega compilação de arquivo
    pub fn load<P: AsRef<Path>>(path: P) -> VspResult<Self> {
        let path = path.as_ref();
        
        // Carregar objeto
        let object_data = fs::read(path)?;
        
        // Carregar metadados
        let metadata_path = path.with_extension("meta");
        let metadata_json = fs::read_to_string(&metadata_path)?;
        let metadata: CompilationMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| VspError::SerializationError(e.to_string()))?;
        
        // Extrair nome do arquivo
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(AotCompilation {
            name,
            bytecode_size: 0, // Unknown from file
            object_data,
            symbols: vec![],  // TODO: Parse from object
            metadata,
        })
    }
}

/// Metadados de compilação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationMetadata {
    /// Timestamp de compilação
    pub compiled_at: u64,
    
    /// Versão do compilador
    pub compiler_version: String,
    
    /// Target triple (x86_64-apple-darwin, etc.)
    pub target_triple: String,
    
    /// Otimizações aplicadas
    pub optimization_level: OptLevel,
    
    /// Tamanho do código
    pub code_size: usize,
}

/// Nível de otimização
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptLevel {
    /// Sem otimização (debug)
    None,
    
    /// Otimizações básicas
    Speed,
    
    /// Otimizações agressivas
    SpeedAndSize,
}

/// Compilador AOT
pub struct AotCompiler {
    /// Target architecture
    target_triple: String,
    
    /// Nível de otimização
    opt_level: OptLevel,
    
    /// Cache de compilações
    cache_dir: Option<PathBuf>,
}

impl AotCompiler {
    /// Cria novo compilador AOT
    pub fn new() -> Self {
        Self {
            target_triple: target_lexicon::HOST.to_string(),
            opt_level: OptLevel::Speed,
            cache_dir: None,
        }
    }
    
    /// Configura nível de otimização
    pub fn with_opt_level(mut self, level: OptLevel) -> Self {
        self.opt_level = level;
        self
    }
    
    /// Configura diretório de cache
    pub fn with_cache<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }
    
    /// Compila bytecode para código nativo
    #[cfg(feature = "jit")]
    pub fn compile(&self, name: &str, bytecode: SilcFile) -> VspResult<AotCompilation> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Setup Cranelift
        let isa_builder = cranelift_native::builder()
            .map_err(|e| VspError::CompilationError(format!("ISA builder failed: {}", e)))?;
        
        let isa = isa_builder
            .finish(self.cranelift_flags())
            .map_err(|e| VspError::CompilationError(format!("ISA creation failed: {}", e)))?;
        
        let builder = ObjectBuilder::new(
            isa,
            name.as_bytes().to_vec(),
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| VspError::CompilationError(format!("Object builder failed: {}", e)))?;
        
        let mut module = ObjectModule::new(builder);
        
        // Compilar função principal (stub por enquanto)
        let _func_id = self.compile_function_stub(&mut module, name, &bytecode)?;
        
        // Finalizar módulo
        let object_product = module.finish();
        let object_data = object_product.emit()
            .map_err(|e| VspError::CompilationError(format!("Object emit failed: {}", e)))?;
        
        // Metadados
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let metadata = CompilationMetadata {
            compiled_at: timestamp,
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            target_triple: self.target_triple.clone(),
            optimization_level: self.opt_level,
            code_size: object_data.len(),
        };
        
        Ok(AotCompilation {
            name: name.to_string(),
            bytecode_size: bytecode.to_bytes().len(),
            object_data,
            symbols: vec![name.to_string()],
            metadata,
        })
    }
    
    #[cfg(not(feature = "jit"))]
    pub fn compile(&self, _name: &str, _bytecode: SilcFile) -> VspResult<AotCompilation> {
        Err(VspError::CompilationError(
            "AOT compilation requires 'jit' feature".to_string()
        ))
    }
    
    /// Compila função stub (placeholder)
    #[cfg(feature = "jit")]
    fn compile_function_stub(
        &self,
        module: &mut ObjectModule,
        name: &str,
        bytecode: &SilcFile,
    ) -> VspResult<cranelift_module::FuncId> {
        use cranelift::prelude::*;
        
        let mut sig = module.make_signature();
        
        // Signature: fn(state: *mut SilState) -> i32
        let ptr_type = module.target_config().pointer_type();
        sig.params.push(AbiParam::new(ptr_type));
        sig.returns.push(AbiParam::new(types::I32));
        
        let func_id = module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| VspError::CompilationError(format!("Function declaration failed: {}", e)))?;
        
        let mut ctx = module.make_context();
        ctx.func.signature = sig;
        
        // Build IR usando função compartilhada
        let mut fn_builder_ctx = FunctionBuilderContext::new();
        super::codegen::build_vsp_function(&mut ctx, &mut fn_builder_ctx, bytecode)?;
        
        // Definir função
        module
            .define_function(func_id, &mut ctx)
            .map_err(|e| VspError::CompilationError(format!("Function definition failed: {}", e)))?;
        
        module.clear_context(&mut ctx);
        
        Ok(func_id)
    }
    
    /// Flags do Cranelift baseadas no opt level
    #[cfg(feature = "jit")]
    fn cranelift_flags(&self) -> settings::Flags {
        let mut builder = settings::builder();
        
        match self.opt_level {
            OptLevel::None => {
                builder.set("opt_level", "none").unwrap();
            }
            OptLevel::Speed => {
                builder.set("opt_level", "speed").unwrap();
            }
            OptLevel::SpeedAndSize => {
                builder.set("opt_level", "speed_and_size").unwrap();
            }
        }
        
        settings::Flags::new(builder)
    }
    
    /// Salva compilação em arquivo
    pub fn save(&self, compilation: &AotCompilation, output: &Path) -> VspResult<()> {
        fs::write(output, &compilation.object_data)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        // Salvar metadados em arquivo .meta
        let meta_path = output.with_extension("meta");
        let meta_json = serde_json::to_string_pretty(&compilation.metadata)
            .map_err(|e| VspError::SerializationError(e.to_string()))?;
        
        fs::write(meta_path, meta_json)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Carrega compilação de arquivo
    pub fn load(&self, path: &Path) -> VspResult<Vec<u8>> {
        fs::read(path)
            .map_err(|e| VspError::IoError(e.to_string()))
    }
    
    /// Verifica se há compilação em cache
    pub fn cached_compilation(&self, name: &str) -> Option<PathBuf> {
        let cache_dir = self.cache_dir.as_ref()?;
        let cached = cache_dir.join(format!("{}.o", name));
        
        if cached.exists() {
            Some(cached)
        } else {
            None
        }
    }
}

impl Default for AotCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache de código compilado
pub struct AotCache {
    /// Diretório de cache
    cache_dir: PathBuf,
    
    /// Índice de compilações
    index: HashMap<String, PathBuf>,
}

impl AotCache {
    /// Cria novo cache
    pub fn new<P: Into<PathBuf>>(cache_dir: P) -> VspResult<Self> {
        let cache_dir = cache_dir.into();
        
        fs::create_dir_all(&cache_dir)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        let mut cache = Self {
            cache_dir,
            index: HashMap::new(),
        };
        
        cache.rebuild_index()?;
        
        Ok(cache)
    }
    
    /// Reconstrói índice do cache
    fn rebuild_index(&mut self) -> VspResult<()> {
        self.index.clear();
        
        for entry in fs::read_dir(&self.cache_dir)
            .map_err(|e| VspError::IoError(e.to_string()))? 
        {
            let entry = entry.map_err(|e| VspError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("o") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    self.index.insert(name.to_string(), path);
                }
            }
        }
        
        Ok(())
    }
    
    /// Busca compilação no cache
    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.index.get(name)
    }
    
    /// Adiciona compilação ao cache
    pub fn put(&mut self, compilation: &AotCompilation) -> VspResult<PathBuf> {
        let output = self.cache_dir.join(format!("{}.o", compilation.name));
        
        let compiler = AotCompiler::new();
        compiler.save(compilation, &output)?;
        
        self.index.insert(compilation.name.clone(), output.clone());
        
        Ok(output)
    }
    
    /// Remove compilação do cache
    pub fn remove(&mut self, name: &str) -> VspResult<bool> {
        if let Some(path) = self.index.remove(name) {
            fs::remove_file(&path)
                .map_err(|e| VspError::IoError(e.to_string()))?;
            
            // Remover metadados também
            let meta_path = path.with_extension("meta");
            if meta_path.exists() {
                let _ = fs::remove_file(meta_path);
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Limpa todo o cache
    pub fn clear(&mut self) -> VspResult<()> {
        for (_name, path) in self.index.drain() {
            let _ = fs::remove_file(&path);
            
            let meta_path = path.with_extension("meta");
            if meta_path.exists() {
                let _ = fs::remove_file(meta_path);
            }
        }
        
        Ok(())
    }
    
    /// Estatísticas do cache
    pub fn stats(&self) -> CacheStats {
        let mut total_size = 0u64;
        
        for path in self.index.values() {
            if let Ok(metadata) = fs::metadata(path) {
                total_size += metadata.len();
            }
        }
        
        CacheStats {
            num_entries: self.index.len(),
            total_size_bytes: total_size,
        }
    }
}

/// Estatísticas do cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub num_entries: usize,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aot_compiler_creation() {
        let compiler = AotCompiler::new();
        assert_eq!(compiler.opt_level, OptLevel::Speed);
    }
    
    #[test]
    fn test_cache_creation() {
        let temp_dir = std::env::temp_dir().join("sil_aot_cache_test");
        let cache = AotCache::new(&temp_dir);
        assert!(cache.is_ok());
        
        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
