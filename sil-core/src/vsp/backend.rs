//! Backend abstraction do VSP
//!
//! Seleção automática e despacho para CPU, GPU, NPU.

use crate::processors::ProcessorType;
use super::state::VspState;
use super::error::VspResult;

/// Trait para backends de execução
pub trait VspBackend: Send + Sync {
    /// Nome do backend
    fn name(&self) -> &str;
    
    /// Tipo de processador
    fn processor_type(&self) -> ProcessorType;
    
    /// Verifica se está disponível
    fn is_available(&self) -> bool;
    
    /// Calcula gradiente
    fn compute_gradient(&self, state: &mut VspState) -> VspResult<()>;
    
    /// Executa emergência (inferência neural)
    fn emergence(&self, state: &mut VspState) -> VspResult<()>;
    
    /// Memory fence
    fn fence(&self) -> VspResult<()>;
}

/// Backend CPU (sempre disponível)
pub struct CpuBackend;

impl VspBackend for CpuBackend {
    fn name(&self) -> &str {
        "CPU"
    }
    
    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Cpu
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn compute_gradient(&self, state: &mut VspState) -> VspResult<()> {
        // Gradiente simplificado na CPU
        // Calcula diferença entre camadas adjacentes
        let mut grad = [0.0f32; 16];
        
        for i in 0..15 {
            let a = state.regs[i].to_complex();
            let b = state.regs[i + 1].to_complex();
            grad[i] = (b - a).norm() as f32;
        }
        grad[15] = 0.0;
        
        state.gradient = Some(grad);
        Ok(())
    }
    
    fn emergence(&self, state: &mut VspState) -> VspResult<()> {
        // Emergência simplificada: XOR das camadas de emergência
        let lb = state.regs[0xB];
        let lc = state.regs[0xC];
        state.regs[0xB] = lb.xor(&lc);
        Ok(())
    }
    
    fn fence(&self) -> VspResult<()> {
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// Backend GPU (opcional)
#[cfg(feature = "gpu")]
pub struct GpuBackend {
    #[allow(dead_code)]
    ctx: crate::processors::GpuContext,
}

#[cfg(feature = "gpu")]
impl GpuBackend {
    pub fn new() -> VspResult<Option<Self>> {
        match crate::processors::GpuContext::new_sync() {
            Ok(ctx) => Ok(Some(Self { ctx })),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(feature = "gpu")]
impl VspBackend for GpuBackend {
    fn name(&self) -> &str {
        "GPU"
    }
    
    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Gpu
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn compute_gradient(&self, state: &mut VspState) -> VspResult<()> {
        // TODO: usar compute shaders
        // Por enquanto, fallback para CPU
        CpuBackend.compute_gradient(state)
    }
    
    fn emergence(&self, state: &mut VspState) -> VspResult<()> {
        // TODO: inferência na GPU
        CpuBackend.emergence(state)
    }
    
    fn fence(&self) -> VspResult<()> {
        // GPU fence seria device.poll(wgpu::Maintain::Wait)
        Ok(())
    }
}

/// Backend NPU (opcional)
#[cfg(feature = "npu")]
pub struct NpuBackend {
    ctx: crate::processors::NpuContext,
}

#[cfg(feature = "npu")]
impl NpuBackend {
    pub fn new() -> VspResult<Option<Self>> {
        match crate::processors::NpuContext::new() {
            Ok(ctx) => Ok(Some(Self { ctx })),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(feature = "npu")]
impl VspBackend for NpuBackend {
    fn name(&self) -> &str {
        "NPU"
    }
    
    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Npu
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn compute_gradient(&self, state: &mut VspState) -> VspResult<()> {
        // NPU não é ideal para gradientes, fallback
        CpuBackend.compute_gradient(state)
    }
    
    fn emergence(&self, state: &mut VspState) -> VspResult<()> {
        // TODO: usar modelo neural no NPU
        CpuBackend.emergence(state)
    }
    
    fn fence(&self) -> VspResult<()> {
        Ok(())
    }
}

/// Seletor de backends
pub struct BackendSelector {
    cpu: CpuBackend,
    #[cfg(feature = "gpu")]
    gpu: Option<GpuBackend>,
    #[cfg(feature = "npu")]
    npu: Option<NpuBackend>,
    /// Batch mode ativo
    batch_mode: bool,
    /// Tamanho do batch
    batch_size: usize,
}

impl BackendSelector {
    /// Cria seletor de backends
    pub fn new(_enable_gpu: bool, _enable_npu: bool) -> VspResult<Self> {
        Ok(Self {
            cpu: CpuBackend,
            #[cfg(feature = "gpu")]
            gpu: if _enable_gpu { GpuBackend::new()? } else { None },
            #[cfg(feature = "npu")]
            npu: if _enable_npu { NpuBackend::new()? } else { None },
            batch_mode: false,
            batch_size: 0,
        })
    }
    
    /// Seleciona backend para gradiente
    pub fn select_for_gradient(&self, hint: Option<ProcessorType>) -> &dyn VspBackend {
        match hint {
            #[cfg(feature = "gpu")]
            Some(ProcessorType::Gpu) if self.gpu.is_some() => {
                self.gpu.as_ref().unwrap()
            }
            _ => &self.cpu,
        }
    }
    
    /// Seleciona backend para inferência
    pub fn select_for_inference(&self, hint: Option<ProcessorType>) -> &dyn VspBackend {
        match hint {
            #[cfg(feature = "npu")]
            Some(ProcessorType::Npu) if self.npu.is_some() => {
                self.npu.as_ref().unwrap()
            }
            #[cfg(feature = "gpu")]
            Some(ProcessorType::Gpu) if self.gpu.is_some() => {
                self.gpu.as_ref().unwrap()
            }
            _ => &self.cpu,
        }
    }
    
    /// Inicia modo batch
    pub fn begin_batch(&mut self, size: usize) -> VspResult<()> {
        self.batch_mode = true;
        self.batch_size = size;
        Ok(())
    }
    
    /// Finaliza modo batch
    pub fn end_batch(&mut self) -> VspResult<()> {
        self.batch_mode = false;
        self.batch_size = 0;
        Ok(())
    }
    
    /// Memory fence em todos os backends
    pub fn fence(&self) -> VspResult<()> {
        self.cpu.fence()?;
        
        #[cfg(feature = "gpu")]
        if let Some(ref gpu) = self.gpu {
            gpu.fence()?;
        }
        
        #[cfg(feature = "npu")]
        if let Some(ref npu) = self.npu {
            npu.fence()?;
        }
        
        Ok(())
    }
    
    /// Retorna informações dos backends
    pub fn info(&self) -> BackendInfo {
        BackendInfo {
            cpu_available: true,
            #[cfg(feature = "gpu")]
            gpu_available: self.gpu.is_some(),
            #[cfg(not(feature = "gpu"))]
            gpu_available: false,
            #[cfg(feature = "npu")]
            npu_available: self.npu.is_some(),
            #[cfg(not(feature = "npu"))]
            npu_available: false,
        }
    }
}

/// Informações dos backends
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub cpu_available: bool,
    pub gpu_available: bool,
    pub npu_available: bool,
}

impl std::fmt::Display for BackendInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backends: CPU={} GPU={} NPU={}",
            if self.cpu_available { "✓" } else { "✗" },
            if self.gpu_available { "✓" } else { "✗" },
            if self.npu_available { "✓" } else { "✗" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ByteSil;
    use super::super::state::SilMode;
    
    #[test]
    fn test_cpu_backend() {
        let backend = CpuBackend;
        assert!(backend.is_available());
        assert_eq!(backend.name(), "CPU");
    }
    
    #[test]
    fn test_cpu_gradient() {
        let backend = CpuBackend;
        let mut state = VspState::new(SilMode::Sil128);
        
        // Setup alguns valores
        state.regs[0] = ByteSil::new(0, 0);
        state.regs[1] = ByteSil::new(2, 4);
        
        backend.compute_gradient(&mut state).unwrap();
        
        assert!(state.gradient.is_some());
    }
    
    #[test]
    fn test_backend_selector() {
        let selector = BackendSelector::new(false, false).unwrap();
        let info = selector.info();
        
        assert!(info.cpu_available);
    }
}
