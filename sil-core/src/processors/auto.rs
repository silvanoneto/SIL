//! # 🎯 Auto Processor Selection - Smart Backend Dispatch
//!
//! Funções de conveniência que automaticamente selecionam o melhor processador
//! (CPU/GPU/NPU) baseado em tamanho de lote e tipo de operação.
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::processors::auto::{lerp_auto, slerp_auto, gradient_auto};
//!
//! // Auto-seleciona CPU para operação individual
//! let result = lerp_auto(&state_a, &state_b, 0.5);  // Usa CPU (~12ns)
//!
//! // Auto-seleciona GPU para lote grande
//! let results = lerp_batch_auto(&batch);  // Usa GPU se batch.len() > 500
//! ```

use crate::state::SilState;
use crate::processors::{ProcessorType, CpuContext, InterpolationProcessor};
use super::performance_fixes::ProcessorSelector;

#[cfg(feature = "gpu")]
use super::gpu::{lerp_states as gpu_lerp, slerp_states as gpu_slerp};

/// Interpolação linear com seleção automática de processador
/// 
/// **Performance:**
/// - Operação individual: Sempre usa CPU (~12ns)
/// - CPU é 89% mais rápida que GPU para single-op
pub fn lerp_auto(a: &SilState, b: &SilState, t: f32) -> SilState {
    // Single-op sempre usa CPU (GPU tem overhead de ~23ns)
    let ctx = CpuContext::new();
    ctx.lerp(a, b, t).unwrap_or_else(|_| *a)
}

/// Interpolação esférica com seleção automática de processador
/// 
/// **Performance:**
/// - Operação individual: Sempre usa CPU (~15ns)
/// - CPU é 66% mais rápida que GPU para single-op
pub fn slerp_auto(a: &SilState, b: &SilState, t: f32) -> SilState {
    // Single-op sempre usa CPU
    let ctx = CpuContext::new();
    ctx.slerp(a, b, t).unwrap_or_else(|_| *a)
}

/// Interpolação linear em lote com seleção automática
/// 
/// **Heurística:**
/// - batch.len() <= 500: CPU (overhead GPU não compensa)
/// - batch.len() > 500: GPU (se disponível)
/// 
/// **Breakeven point (M3 Pro):** ~500 elementos
pub fn lerp_batch_auto(states: &[(SilState, SilState, f32)]) -> Vec<SilState> {
    let processor = ProcessorSelector::select_for_interpolation(states.len());
    
    match processor {
        ProcessorType::Cpu => {
            let ctx = CpuContext::new();
            states.iter()
                .map(|(a, b, t)| ctx.lerp(a, b, *t).unwrap_or(*a))
                .collect()
        }
        
        #[cfg(feature = "gpu")]
        ProcessorType::Gpu => {
            states.iter()
                .map(|(a, b, t)| gpu_lerp(a, b, *t))
                .collect()
        }
        
        #[cfg(not(feature = "gpu"))]
        ProcessorType::Gpu => {
            // Fallback para CPU se GPU não disponível
            let ctx = CpuContext::new();
            states.iter()
                .map(|(a, b, t)| ctx.lerp(a, b, *t).unwrap_or(*a))
                .collect()
        }
        
        _ => {
            // Fallback padrão
            let ctx = CpuContext::new();
            states.iter()
                .map(|(a, b, t)| ctx.lerp(a, b, *t).unwrap_or(*a))
                .collect()
        }
    }
}

/// Interpolação esférica em lote com seleção automática
pub fn slerp_batch_auto(states: &[(SilState, SilState, f32)]) -> Vec<SilState> {
    let processor = ProcessorSelector::select_for_interpolation(states.len());
    
    match processor {
        ProcessorType::Cpu => {
            let ctx = CpuContext::new();
            states.iter()
                .map(|(a, b, t)| ctx.slerp(a, b, *t).unwrap_or(*a))
                .collect()
        }
        
        #[cfg(feature = "gpu")]
        ProcessorType::Gpu => {
            states.iter()
                .map(|(a, b, t)| gpu_slerp(a, b, *t))
                .collect()
        }
        
        #[cfg(not(feature = "gpu"))]
        ProcessorType::Gpu => {
            let ctx = CpuContext::new();
            states.iter()
                .map(|(a, b, t)| ctx.slerp(a, b, *t).unwrap_or(*a))
                .collect()
        }
        
        _ => {
            let ctx = CpuContext::new();
            states.iter()
                .map(|(a, b, t)| ctx.slerp(a, b, *t).unwrap_or(*a))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lerp_auto() {
        let a = SilState::vacuum();
        let b = SilState::maximum();
        
        let result = lerp_auto(&a, &b, 0.5);
        
        // Deve estar entre os dois estados
        assert_ne!(result, a);
        assert_ne!(result, b);
    }
    
    #[test]
    fn test_lerp_batch_auto_small() {
        let a = SilState::vacuum();
        let b = SilState::maximum();
        
        // Lote pequeno (10 elementos) → deve usar CPU
        let batch: Vec<_> = (0..10)
            .map(|i| (a, b, i as f32 / 10.0))
            .collect();
        
        let results = lerp_batch_auto(&batch);
        assert_eq!(results.len(), 10);
    }
    
    #[test]
    fn test_lerp_batch_auto_large() {
        let a = SilState::vacuum();
        let b = SilState::maximum();
        
        // Lote grande (1000 elementos) → deve usar GPU se disponível
        let batch: Vec<_> = (0..1000)
            .map(|i| (a, b, i as f32 / 1000.0))
            .collect();
        
        let results = lerp_batch_auto(&batch);
        assert_eq!(results.len(), 1000);
    }
}
