//! # Quantum Gates GPU Execution
//!
//! Execução de gates quânticas em batch via GPU usando shaders WGSL.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │                    QuantumGpuExecutor                          │
//! │  ┌──────────────────────────────────────────────────────────┐  │
//! │  │  hadamard_pipeline     ← hadamard.wgsl (main)            │  │
//! │  │  hadamard_layer_pipeline ← hadamard.wgsl (hadamard_layer)│  │
//! │  │  gates_pipeline        ← quantum_gates.wgsl (apply_gate) │  │
//! │  │  matrix_pipeline       ← quantum_gates.wgsl (apply_matrix)│ │
//! │  └──────────────────────────────────────────────────────────┘  │
//! └────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_core::processors::gpu::quantum::QuantumGpuExecutor;
//!
//! let executor = QuantumGpuExecutor::new(&gpu_ctx)?;
//! let states = vec![GpuQuantumState::zero(); 1000];
//! let results = executor.apply_hadamard(&states).await?;
//! ```

use super::{GpuContext, GpuError, GpuResult};
use wgpu::{util::DeviceExt, BindGroupLayout, ComputePipeline};
use bytemuck::{Pod, Zeroable};

/// Workgroup size (deve corresponder ao shader)
const WORKGROUP_SIZE: u32 = 64;

/// Uniform para parâmetros do Hadamard
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct HadamardParams {
    pub num_states: u32,
    pub target_layer: u32,
    pub _padding: [f32; 2],
}

impl Default for HadamardParams {
    fn default() -> Self {
        Self {
            num_states: 0,
            target_layer: 0xFFFF, // Todas as camadas
            _padding: [0.0; 2],
        }
    }
}

/// Uniform para parâmetros de gate genérica
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GateParams {
    pub num_states: u32,
    pub gate_type: u32,
    pub theta: f32,
    pub phi: f32,
}

impl Default for GateParams {
    fn default() -> Self {
        Self {
            num_states: 0,
            gate_type: 0,
            theta: 0.0,
            phi: 0.0,
        }
    }
}

/// Uniform para matriz 2x2 complexa
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GateMatrix {
    pub m00_re: f32,
    pub m00_im: f32,
    pub m01_re: f32,
    pub m01_im: f32,
    pub m10_re: f32,
    pub m10_im: f32,
    pub m11_re: f32,
    pub m11_im: f32,
}

impl Default for GateMatrix {
    fn default() -> Self {
        // Identidade
        Self {
            m00_re: 1.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: 1.0,
            m11_im: 0.0,
        }
    }
}

/// Estado quântico em formato GPU (4 floats)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuQuantumState {
    /// α real
    pub alpha_re: f32,
    /// α imaginário
    pub alpha_im: f32,
    /// β real
    pub beta_re: f32,
    /// β imaginário
    pub beta_im: f32,
}

impl GpuQuantumState {
    /// Estado |0⟩
    pub fn zero() -> Self {
        Self {
            alpha_re: 1.0,
            alpha_im: 0.0,
            beta_re: 0.0,
            beta_im: 0.0,
        }
    }

    /// Estado |1⟩
    pub fn one() -> Self {
        Self {
            alpha_re: 0.0,
            alpha_im: 0.0,
            beta_re: 1.0,
            beta_im: 0.0,
        }
    }

    /// Estado |+⟩ = (|0⟩ + |1⟩)/√2
    pub fn plus() -> Self {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        Self {
            alpha_re: h,
            alpha_im: 0.0,
            beta_re: h,
            beta_im: 0.0,
        }
    }

    /// Estado |-⟩ = (|0⟩ - |1⟩)/√2
    pub fn minus() -> Self {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        Self {
            alpha_re: h,
            alpha_im: 0.0,
            beta_re: -h,
            beta_im: 0.0,
        }
    }

    /// Probabilidade de medir |0⟩
    pub fn prob_zero(&self) -> f32 {
        self.alpha_re * self.alpha_re + self.alpha_im * self.alpha_im
    }

    /// Probabilidade de medir |1⟩
    pub fn prob_one(&self) -> f32 {
        self.beta_re * self.beta_re + self.beta_im * self.beta_im
    }

    /// Verifica normalização
    pub fn is_normalized(&self, epsilon: f32) -> bool {
        let total = self.prob_zero() + self.prob_one();
        (total - 1.0).abs() < epsilon
    }
}

/// Executor de gates quânticas na GPU
pub struct QuantumGpuExecutor {
    // Pipelines
    hadamard_pipeline: ComputePipeline,
    gates_pipeline: ComputePipeline,
    matrix_pipeline: ComputePipeline,

    // Bind group layouts
    hadamard_layout: BindGroupLayout,
    gates_layout: BindGroupLayout,
}

impl QuantumGpuExecutor {
    /// Cria novo executor a partir de um GpuContext
    pub fn new(ctx: &GpuContext) -> GpuResult<Self> {
        let device = &ctx.device;

        // Compilar shader Hadamard
        let hadamard_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hadamard Shader"),
            source: wgpu::ShaderSource::Wgsl(super::shaders::HADAMARD_SHADER.into()),
        });

        // Compilar shader de gates genéricas
        let gates_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quantum Gates Shader"),
            source: wgpu::ShaderSource::Wgsl(super::shaders::QUANTUM_GATES_SHADER.into()),
        });

        // Layout para Hadamard: uniform + input + output
        let hadamard_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hadamard Bind Group Layout"),
            entries: &[
                // @binding(0): uniform params
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // @binding(1): input states (read-only)
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
                // @binding(2): output states (read-write)
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
            ],
        });

        // Layout para gates: params + matrix + input + output
        let gates_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gates Bind Group Layout"),
            entries: &[
                // @binding(0): uniform params
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // @binding(1): uniform matrix
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // @binding(2): input states (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // @binding(3): output states (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Pipeline layout para Hadamard
        let hadamard_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Hadamard Pipeline Layout"),
                bind_group_layouts: &[&hadamard_layout],
                push_constant_ranges: &[],
            });

        // Pipeline layout para gates
        let gates_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gates Pipeline Layout"),
                bind_group_layouts: &[&gates_layout],
                push_constant_ranges: &[],
            });

        // Pipeline Hadamard
        let hadamard_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Hadamard Pipeline"),
                layout: Some(&hadamard_pipeline_layout),
                module: &hadamard_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        // Pipeline para apply_gate (switch por tipo)
        let gates_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gates Pipeline"),
            layout: Some(&gates_pipeline_layout),
            module: &gates_shader,
            entry_point: Some("apply_gate"),
            compilation_options: Default::default(),
            cache: None,
        });

        // Pipeline para apply_matrix (matriz customizada)
        let matrix_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Matrix Pipeline"),
            layout: Some(&gates_pipeline_layout),
            module: &gates_shader,
            entry_point: Some("apply_matrix"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            hadamard_pipeline,
            gates_pipeline,
            matrix_pipeline,
            hadamard_layout,
            gates_layout,
        })
    }

    /// Aplica gate Hadamard em batch de estados
    pub async fn apply_hadamard(
        &self,
        ctx: &GpuContext,
        states: &[GpuQuantumState],
    ) -> GpuResult<Vec<GpuQuantumState>> {
        if states.is_empty() {
            return Ok(vec![]);
        }

        let device = &ctx.device;
        let queue = &ctx.queue;
        let num_states = states.len() as u32;

        // Parâmetros
        let params = HadamardParams {
            num_states,
            target_layer: 0xFFFF,
            _padding: [0.0; 2],
        };

        // Buffers
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hadamard Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hadamard Input Buffer"),
            contents: bytemuck::cast_slice(states),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hadamard Output Buffer"),
            size: (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hadamard Staging Buffer"),
            size: (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hadamard Bind Group"),
            layout: &self.hadamard_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch
        let workgroups = (num_states + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Hadamard Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Hadamard Pass"),
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.hadamard_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Copy to staging
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Read back
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        device.poll(wgpu::Maintain::Wait);

        rx.await
            .map_err(|_| GpuError::Timeout)?
            .map_err(|e| GpuError::DeviceCreation(e.to_string()))?;

        let data = buffer_slice.get_mapped_range();
        let result: Vec<GpuQuantumState> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(result)
    }

    /// Aplica gate por tipo (H, X, Y, Z, Rx, Ry, Rz, etc.)
    pub async fn apply_gate(
        &self,
        ctx: &GpuContext,
        states: &[GpuQuantumState],
        gate_type: u32,
        theta: f32,
    ) -> GpuResult<Vec<GpuQuantumState>> {
        if states.is_empty() {
            return Ok(vec![]);
        }

        let device = &ctx.device;
        let queue = &ctx.queue;
        let num_states = states.len() as u32;

        // Parâmetros
        let params = GateParams {
            num_states,
            gate_type,
            theta,
            phi: 0.0,
        };

        // Matriz identidade (não usada para gates built-in)
        let matrix = GateMatrix::default();

        // Buffers
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gate Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gate Matrix Buffer"),
            contents: bytemuck::bytes_of(&matrix),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gate Input Buffer"),
            contents: bytemuck::cast_slice(states),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gate Output Buffer"),
            size: (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gate Staging Buffer"),
            size: (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gate Bind Group"),
            layout: &self.gates_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch
        let workgroups = (num_states + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Gate Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Gate Pass"),
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.gates_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Copy to staging
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Read back
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        device.poll(wgpu::Maintain::Wait);

        rx.await
            .map_err(|_| GpuError::Timeout)?
            .map_err(|e| GpuError::DeviceCreation(e.to_string()))?;

        let data = buffer_slice.get_mapped_range();
        let result: Vec<GpuQuantumState> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(result)
    }

    /// Aplica matriz 2x2 customizada
    pub async fn apply_matrix(
        &self,
        ctx: &GpuContext,
        states: &[GpuQuantumState],
        matrix: GateMatrix,
    ) -> GpuResult<Vec<GpuQuantumState>> {
        if states.is_empty() {
            return Ok(vec![]);
        }

        let device = &ctx.device;
        let queue = &ctx.queue;
        let num_states = states.len() as u32;

        // Parâmetros (gate_type ignorado para apply_matrix)
        let params = GateParams {
            num_states,
            gate_type: 255, // Custom
            theta: 0.0,
            phi: 0.0,
        };

        // Buffers
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Matrix Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Matrix Buffer"),
            contents: bytemuck::bytes_of(&matrix),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Matrix Input Buffer"),
            contents: bytemuck::cast_slice(states),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Matrix Output Buffer"),
            size: (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Matrix Staging Buffer"),
            size: (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Matrix Bind Group"),
            layout: &self.gates_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch
        let workgroups = (num_states + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Matrix Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Matrix Pass"),
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.matrix_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Copy to staging
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            (states.len() * std::mem::size_of::<GpuQuantumState>()) as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Read back
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        device.poll(wgpu::Maintain::Wait);

        rx.await
            .map_err(|_| GpuError::Timeout)?
            .map_err(|e| GpuError::DeviceCreation(e.to_string()))?;

        let data = buffer_slice.get_mapped_range();
        let result: Vec<GpuQuantumState> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(result)
    }
}

/// Constantes de tipos de gate (correspondem ao shader)
pub mod gate_types {
    pub const HADAMARD: u32 = 0;
    pub const PAULI_X: u32 = 1;
    pub const PAULI_Y: u32 = 2;
    pub const PAULI_Z: u32 = 3;
    pub const ROTATION_X: u32 = 4;
    pub const ROTATION_Y: u32 = 5;
    pub const ROTATION_Z: u32 = 6;
    pub const PHASE: u32 = 7;
    pub const S_GATE: u32 = 8;
    pub const T_GATE: u32 = 9;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_state_zero() {
        let state = GpuQuantumState::zero();
        assert!((state.prob_zero() - 1.0).abs() < 1e-6);
        assert!(state.prob_one().abs() < 1e-6);
        assert!(state.is_normalized(1e-6));
    }

    #[test]
    fn test_quantum_state_plus() {
        let state = GpuQuantumState::plus();
        assert!((state.prob_zero() - 0.5).abs() < 1e-6);
        assert!((state.prob_one() - 0.5).abs() < 1e-6);
        assert!(state.is_normalized(1e-6));
    }

    #[test]
    fn test_gate_matrix_identity() {
        let matrix = GateMatrix::default();
        assert!((matrix.m00_re - 1.0).abs() < 1e-6);
        assert!((matrix.m11_re - 1.0).abs() < 1e-6);
        assert!(matrix.m01_re.abs() < 1e-6);
        assert!(matrix.m10_re.abs() < 1e-6);
    }

    #[test]
    fn test_hadamard_params() {
        let params = HadamardParams::default();
        assert_eq!(params.num_states, 0);
        assert_eq!(params.target_layer, 0xFFFF);
    }
}
