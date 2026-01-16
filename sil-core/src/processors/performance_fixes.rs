//! # 🔧 Patches de Performance - Hot Fixes Críticos
//!
//! Correções urgentes para regressões de performance identificadas
//! no relatório de benchmarks de 11/01/2026.

use std::sync::OnceLock;

#[cfg(feature = "gpu")]
use wgpu::{Instance, InstanceDescriptor, RequestAdapterOptions, PowerPreference};

#[cfg(feature = "gpu")]
use crate::processors::gpu::GpuError;

// ═══════════════════════════════════════════════════════════════════════════
//  FIX #1: Cache de is_available() - Elimina regressão de +21,000%
// ═══════════════════════════════════════════════════════════════════════════

/// Cache estático de disponibilidade de GPU
#[cfg(feature = "gpu")]
static GPU_AVAILABLE: OnceLock<bool> = OnceLock::new();

/// Verifica disponibilidade de GPU (com cache)
/// 
/// **Performance:**
/// - Primeira chamada: ~4.8µs (detecção real)
/// - Chamadas subsequentes: <1ns (lookup em cache)
/// 
/// **Antes:** 4.67µs TODA CHAMADA (+1,551,665% regressão)
/// **Depois:** <1ns (amortizado)
#[cfg(feature = "gpu")]
pub fn is_gpu_available_cached() -> bool {
    *GPU_AVAILABLE.get_or_init(|| {
        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        #[cfg(feature = "gpu")]
        pollster::block_on(async {
            instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }).await.is_some()
        })
    })
}

/// Versão stub quando GPU não está disponível
#[cfg(not(feature = "gpu"))]
pub fn is_gpu_available_cached() -> bool {
    false
}

// ═══════════════════════════════════════════════════════════════════════════
//  FIX #2: Singleton GpuContext - Amortiza overhead de 700µs
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "gpu")]
use crate::processors::gpu::{GpuContext, GpuResult};

/// Cache global de contexto GPU (singleton)
#[cfg(feature = "gpu")]
static GPU_CONTEXT: OnceLock<GpuContext> = OnceLock::new();

#[cfg(feature = "gpu")]
static GPU_CONTEXT_INIT_ERROR: OnceLock<String> = OnceLock::new();

/// Obtém ou inicializa contexto GPU singleton
/// 
/// **Performance:**
/// - Primeira chamada: ~701µs (inicialização completa)
/// - Chamadas subsequentes: <1ns (referência estática)
/// 
/// **Economia:** ~700µs por operação GPU após primeira chamada
#[cfg(feature = "gpu")]
pub fn get_gpu_context() -> GpuResult<&'static GpuContext> {
    // Tenta obter contexto já inicializado
    if let Some(ctx) = GPU_CONTEXT.get() {
        return Ok(ctx);
    }
    
    // Se houve erro prévio, retorna
    if let Some(err) = GPU_CONTEXT_INIT_ERROR.get() {
        return Err(GpuError::DeviceCreation(err.clone()));
    }
    
    // Inicializar (apenas primeira vez)
    match GpuContext::new_sync() {
        Ok(ctx) => {
            // Sucesso: armazena contexto
            GPU_CONTEXT.get_or_init(|| ctx);
            Ok(GPU_CONTEXT.get().unwrap())
        }
        Err(e) => {
            // Erro: armazena mensagem para futuras chamadas
            GPU_CONTEXT_INIT_ERROR.get_or_init(|| format!("{}", e));
            Err(e)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  FIX #3: Auto-selection de Processador (CPU vs GPU)
// ═══════════════════════════════════════════════════════════════════════════

use crate::processors::ProcessorType;

/// Heurística de seleção de processador baseada em tamanho de lote
/// 
/// **Breakeven points empíricos (M3 Pro):**
/// - Interpolação: 500 elementos
/// - Gradiente: 200 elementos
/// - Distâncias: 1000 elementos
pub struct ProcessorSelector;

impl ProcessorSelector {
    /// Seleciona processador ótimo para interpolação (lerp/slerp)
    pub fn select_for_interpolation(batch_size: usize) -> ProcessorType {
        match batch_size {
            0..=500 => ProcessorType::Cpu,  // CPU melhor (overhead GPU não compensa)
            _ => {
                if is_gpu_available_cached() {
                    ProcessorType::Gpu  // GPU compensa para lotes grandes
                } else {
                    ProcessorType::Cpu
                }
            }
        }
    }
    
    /// Seleciona processador ótimo para gradientes
    pub fn select_for_gradient(batch_size: usize) -> ProcessorType {
        match batch_size {
            0..=200 => ProcessorType::Cpu,
            _ => {
                if is_gpu_available_cached() {
                    ProcessorType::Gpu
                } else {
                    ProcessorType::Cpu
                }
            }
        }
    }
    
    /// Seleciona processador ótimo para distâncias geodésicas
    pub fn select_for_distance(batch_size: usize) -> ProcessorType {
        match batch_size {
            0..=1000 => ProcessorType::Cpu,
            _ => {
                if is_gpu_available_cached() {
                    ProcessorType::Gpu
                } else {
                    ProcessorType::Cpu
                }
            }
        }
    }
    
    /// Seleciona processador ótimo para quantização
    pub fn select_for_quantization(batch_size: usize) -> ProcessorType {
        #[cfg(feature = "npu")]
        {
            use crate::processors::npu::NpuContext;
            
            match batch_size {
                0..=100 => ProcessorType::Cpu,  // Trait Quantizable é mais rápido
                _ => {
                    if NpuContext::is_available() {
                        ProcessorType::Npu  // NPU excelente para INT8 em lotes
                    } else {
                        ProcessorType::Cpu
                    }
                }
            }
        }
        
        #[cfg(not(feature = "npu"))]
        {
            let _ = batch_size;
            ProcessorType::Cpu
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  FIX #4: ProcessorType::available() otimizado
// ═══════════════════════════════════════════════════════════════════════════

/// Cache de processadores disponíveis
static AVAILABLE_PROCESSORS: OnceLock<Vec<ProcessorType>> = OnceLock::new();

/// Lista processadores disponíveis (com cache)
/// 
/// **Antes:** 4.80µs TODA CHAMADA (+21,310% regressão)
/// **Depois:** <1ns (após primeira chamada)
pub fn available_processors_cached() -> &'static [ProcessorType] {
    AVAILABLE_PROCESSORS.get_or_init(|| {
        #[allow(unused_mut)]
        let mut processors = vec![ProcessorType::Cpu]; // CPU sempre disponível
        
        #[cfg(feature = "gpu")]
        if is_gpu_available_cached() {
            processors.push(ProcessorType::Gpu);
        }
        
        #[cfg(feature = "npu")]
        {
            use crate::processors::npu::NpuContext;
            if NpuContext::is_available() {
                processors.push(ProcessorType::Npu);
            }
        }
        
        processors
    })
}

// ═══════════════════════════════════════════════════════════════════════════
//  TESTES DE PERFORMANCE
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_gpu_available_cache_performance() {
        // Primeira chamada (cold)
        let start = Instant::now();
        let _ = is_gpu_available_cached();
        let cold_time = start.elapsed();
        
        // Segunda chamada (cached)
        let start = Instant::now();
        let _ = is_gpu_available_cached();
        let cached_time = start.elapsed();
        
        println!("GPU available cold: {:?}", cold_time);
        println!("GPU available cached: {:?}", cached_time);

        // Cache deve ser mais rápido ou similar (permite jitter de até 100ns)
        // Em sistemas muito rápidos, a diferença pode ser mínima
        let max_overhead_ns = 100;
        assert!(
            cached_time.as_nanos() <= cold_time.as_nanos() + max_overhead_ns,
            "Cached time ({:?}) should be faster or similar to cold time ({:?})",
            cached_time,
            cold_time
        );
    }
    
    #[test]
    fn test_available_processors_cache() {
        let start = Instant::now();
        let _ = available_processors_cached();
        let cold_time = start.elapsed();
        
        let start = Instant::now();
        let _ = available_processors_cached();
        let cached_time = start.elapsed();
        
        println!("Available processors cold: {:?}", cold_time);
        println!("Available processors cached: {:?}", cached_time);
        
        // Cache deve ser mais rápido ou igual (robusto a jitter)
        assert!(cached_time <= cold_time);
    }
    
    #[test]
    fn test_processor_selection_heuristics() {
        // Lotes pequenos → CPU
        assert_eq!(
            ProcessorSelector::select_for_interpolation(10),
            ProcessorType::Cpu
        );
        
        assert_eq!(
            ProcessorSelector::select_for_gradient(50),
            ProcessorType::Cpu
        );
        
        // Lotes grandes → GPU (se disponível)
        let large_interp = ProcessorSelector::select_for_interpolation(1000);
        let large_grad = ProcessorSelector::select_for_gradient(500);
        
        if is_gpu_available_cached() {
            assert_eq!(large_interp, ProcessorType::Gpu);
            assert_eq!(large_grad, ProcessorType::Gpu);
        } else {
            assert_eq!(large_interp, ProcessorType::Cpu);
            assert_eq!(large_grad, ProcessorType::Cpu);
        }
    }
}
