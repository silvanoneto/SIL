//! Contexto GPU — Inicialização wgpu

use wgpu::{
    Adapter, Device, Queue, Instance, InstanceDescriptor, 
    RequestAdapterOptions, DeviceDescriptor, Features, Limits,
    ComputePipeline, BindGroupLayout,
};
use super::{GpuError, GpuResult};
use super::pipeline_pool::GpuPipelinePool;
use std::sync::Arc;

/// Contexto de computação GPU
pub struct GpuContext {
    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    /// Device GPU (público para criar bind group layouts customizados)
    pub device: Device,
    pub(crate) queue: Queue,
    pub(crate) gradient_pipeline: ComputePipeline,
    pub(crate) gradient_bind_group_layout: BindGroupLayout,
    pub(crate) interpolate_pipeline: ComputePipeline,
    pub(crate) interpolate_bind_group_layout: BindGroupLayout,
    /// Pipeline pool para reutilização
    pub(crate) pipeline_pool: Arc<GpuPipelinePool>,
}

impl GpuContext {
    /// Cria novo contexto GPU
    /// 
    /// Seleciona automaticamente o melhor backend:
    /// - macOS: Metal
    /// - Windows: DX12 ou Vulkan
    /// - Linux: Vulkan
    pub async fn new() -> GpuResult<Self> {
        // Criar instância wgpu
        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Solicitar adaptador (GPU)
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;
        
        // Info do adaptador
        let info = adapter.get_info();
        log_adapter_info(&info);
        
        // Criar device e queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("SIL GPU Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| GpuError::DeviceCreation(e.to_string()))?;
        
        // Compilar shaders
        let gradient_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SIL Gradient Shader"),
            source: wgpu::ShaderSource::Wgsl(super::shaders::GRADIENT_SHADER.into()),
        });
        
        let interpolate_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SIL Interpolate Shader"),
            source: wgpu::ShaderSource::Wgsl(super::shaders::INTERPOLATE_SHADER.into()),
        });
        
        // Criar bind group layout para gradientes
        let gradient_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gradient Bind Group Layout"),
            entries: &[
                // Input: estados [u32; N * 4] (cada estado = 4 x u32 = 16 bytes = 128 bits)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output: gradientes [f32; N * 32] (cada gradiente = 16 camadas × 2 floats)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Uniforms: configuração
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Criar bind group layout para interpolação
        let interpolate_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Interpolate Bind Group Layout"),
            entries: &[
                // Input A: estados [u32; N * 4]
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Input B: estados [u32; N * 4]
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output: estados interpolados [u32; N * 4]
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Uniforms: params
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Criar pipeline layout para gradientes
        let gradient_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gradient Pipeline Layout"),
            bind_group_layouts: &[&gradient_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Criar pipeline layout para interpolação
        let interpolate_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Interpolate Pipeline Layout"),
            bind_group_layouts: &[&interpolate_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Criar compute pipeline para gradientes
        let gradient_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gradient Compute Pipeline"),
            layout: Some(&gradient_pipeline_layout),
            module: &gradient_shader,
            entry_point: Some("compute_gradient"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        // Criar compute pipeline para interpolação
        let interpolate_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Interpolate Compute Pipeline"),
            layout: Some(&interpolate_pipeline_layout),
            module: &interpolate_shader,
            entry_point: Some("interpolate"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        // Criar pipeline pool
        let pipeline_pool = Arc::new(GpuPipelinePool::new());
        
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            gradient_pipeline,
            gradient_bind_group_layout,
            interpolate_pipeline,
            interpolate_bind_group_layout,
            pipeline_pool,
        })
    }
    
    /// Cria contexto de forma síncrona (blocking)
    pub fn new_sync() -> GpuResult<Self> {
        pollster::block_on(Self::new())
    }
    
    /// Informações do adaptador
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
    
    /// Nome do backend (Metal, Vulkan, DX12, etc.)
    pub fn backend_name(&self) -> &'static str {
        match self.adapter.get_info().backend {
            wgpu::Backend::Empty => "Empty",
            wgpu::Backend::Vulkan => "Vulkan",
            wgpu::Backend::Metal => "Metal",
            wgpu::Backend::Dx12 => "DX12",
            wgpu::Backend::Gl => "OpenGL",
            wgpu::Backend::BrowserWebGpu => "WebGPU",
        }
    }
    
    /// Acesso ao pipeline pool
    pub fn pipeline_pool(&self) -> &Arc<GpuPipelinePool> {
        &self.pipeline_pool
    }
    
    /// Cria custom pipeline usando o pool
    ///
    /// # Performance
    /// - Primeira chamada: ~20ms (compilação)
    /// - Chamadas subsequentes: <50ns (cache hit)
    ///
    /// # Example
    /// ```ignore
    /// let pipeline = ctx.get_or_create_pipeline(
    ///     MY_SHADER_SOURCE,
    ///     "main",
    ///     &bind_group_layout,
    ///     Some("My Pipeline")
    /// );
    /// ```
    pub fn get_or_create_pipeline(
        &self,
        shader_source: &str,
        entry_point: &str,
        bind_group_layout: &BindGroupLayout,
        label: Option<&str>,
    ) -> Arc<ComputePipeline> {
        self.pipeline_pool.get_or_create_pipeline(
            &self.device,
            shader_source,
            entry_point,
            bind_group_layout,
            label,
        )
    }
}

fn log_adapter_info(info: &wgpu::AdapterInfo) {
    #[cfg(debug_assertions)]
    {
        eprintln!("🎮 GPU: {} ({:?})", info.name, info.backend);
        eprintln!("   Driver: {}", info.driver);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_context_creation() {
        // Este teste pode falhar em CI sem GPU
        if let Ok(ctx) = GpuContext::new_sync() {
            println!("GPU: {}", ctx.adapter_info().name);
            println!("Backend: {}", ctx.backend_name());
        } else {
            println!("GPU não disponível (ok em CI)");
        }
    }
}
