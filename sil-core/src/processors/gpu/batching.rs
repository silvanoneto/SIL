//! Sistema de batching para operações GPU assíncronas
//!
//! Agrupa múltiplas operações para execução eficiente em batch.

use crate::state::SilState;
use super::{GpuContext, GpuResult, GpuError, SilGradient};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use wgpu::util::DeviceExt;

/// Tipo de operação GPU
#[derive(Debug)]
pub enum GpuOp {
    /// Calcular gradientes de estados
    ComputeGradients {
        states: Vec<SilState>,
        response: oneshot::Sender<GpuResult<Vec<SilGradient>>>,
    },
    
    /// Interpolar entre estados
    Interpolate {
        states_a: Vec<SilState>,
        states_b: Vec<SilState>,
        t: f32,
        use_slerp: bool,
        response: oneshot::Sender<GpuResult<Vec<SilState>>>,
    },
}

/// Batch de operações GPU
struct GpuBatch {
    ops: Vec<GpuOp>,
    total_states: usize,
}

impl GpuBatch {
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            total_states: 0,
        }
    }
    
    fn add(&mut self, op: GpuOp) {
        match &op {
            GpuOp::ComputeGradients { states, .. } => {
                self.total_states += states.len();
            }
            GpuOp::Interpolate { states_a, .. } => {
                self.total_states += states_a.len();
            }
        }
        self.ops.push(op);
    }
    
    fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
    
    fn should_flush(&self, max_batch_size: usize) -> bool {
        self.total_states >= max_batch_size
    }
}

/// Configuração do batching
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Tamanho máximo do batch (em número de estados)
    pub max_batch_size: usize,
    
    /// Tempo máximo de espera antes de flush (ms)
    pub max_wait_ms: u64,
    
    /// Tamanho do canal de operações pendentes
    pub channel_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1024,      // 1K estados por batch
            max_wait_ms: 5,             // 5ms max latência
            channel_size: 128,          // 128 operações na fila
        }
    }
}

/// Executor de operações GPU com batching
pub struct BatchedGpuExecutor {
    ctx: Arc<GpuContext>,
    tx: mpsc::Sender<GpuOp>,
    config: BatchConfig,
}

impl BatchedGpuExecutor {
    /// Cria novo executor com batching
    pub fn new(ctx: Arc<GpuContext>, config: BatchConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.channel_size);
        
        // Spawn batch processor
        let processor_ctx = ctx.clone();
        let processor_config = config.clone();
        tokio::spawn(async move {
            Self::batch_processor(processor_ctx, rx, processor_config).await;
        });
        
        Self { ctx, tx, config }
    }
    
    /// Cria com configuração padrão
    pub fn new_default(ctx: Arc<GpuContext>) -> Self {
        Self::new(ctx, BatchConfig::default())
    }
    
    /// Submete operação de cálculo de gradientes (async)
    pub async fn compute_gradients(
        &self,
        states: Vec<SilState>,
    ) -> GpuResult<Vec<SilGradient>> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(GpuOp::ComputeGradients {
            states,
            response: tx,
        })
        .await
        .map_err(|_| GpuError::Timeout)?;
        
        rx.await.map_err(|_| GpuError::Timeout)?
    }
    
    /// Submete operação de interpolação (async)
    pub async fn interpolate(
        &self,
        states_a: Vec<SilState>,
        states_b: Vec<SilState>,
        t: f32,
        use_slerp: bool,
    ) -> GpuResult<Vec<SilState>> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(GpuOp::Interpolate {
            states_a,
            states_b,
            t,
            use_slerp,
            response: tx,
        })
        .await
        .map_err(|_| GpuError::Timeout)?;
        
        rx.await.map_err(|_| GpuError::Timeout)?
    }
    
    /// Processador de batches (background task)
    async fn batch_processor(
        ctx: Arc<GpuContext>,
        mut rx: mpsc::Receiver<GpuOp>,
        config: BatchConfig,
    ) {
        let mut current_batch = GpuBatch::new();
        
        loop {
            // Espera por operações com timeout
            let timeout = tokio::time::Duration::from_millis(config.max_wait_ms);
            
            match tokio::time::timeout(timeout, rx.recv()).await {
                // Recebeu operação
                Ok(Some(op)) => {
                    current_batch.add(op);
                    
                    // Flush se batch está cheio
                    if current_batch.should_flush(config.max_batch_size) {
                        Self::execute_batch(&ctx, current_batch).await;
                        current_batch = GpuBatch::new();
                    }
                }
                
                // Timeout - flush batch pendente
                Ok(None) => break, // Canal fechado
                
                Err(_) => {
                    // Timeout - flush se há operações
                    if !current_batch.is_empty() {
                        Self::execute_batch(&ctx, current_batch).await;
                        current_batch = GpuBatch::new();
                    }
                }
            }
        }
    }
    
    /// Executa um batch de operações
    async fn execute_batch(ctx: &GpuContext, batch: GpuBatch) {
        for op in batch.ops {
            match op {
                GpuOp::ComputeGradients { states, response } => {
                    let result = Self::execute_gradients(ctx, &states).await;
                    let _ = response.send(result);
                }
                
                GpuOp::Interpolate { states_a, states_b, t, use_slerp, response } => {
                    let result = Self::execute_interpolate(ctx, &states_a, &states_b, t, use_slerp).await;
                    let _ = response.send(result);
                }
            }
        }
    }
    
    /// Executa cálculo de gradientes na GPU
    async fn execute_gradients(
        ctx: &GpuContext,
        states: &[SilState],
    ) -> GpuResult<Vec<SilGradient>> {
        let num_states = states.len();
        
        // Converter estados para formato GPU (u32 array)
        let mut state_data = Vec::with_capacity(num_states * 4);
        for state in states {
            let bytes = state.to_bytes();
            for chunk in bytes.chunks(4) {
                let mut word = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    word |= (byte as u32) << (i * 8);
                }
                state_data.push(word);
            }
        }
        
        // Criar buffers
        let states_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("States Buffer"),
            contents: bytemuck::cast_slice(&state_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        
        let gradients_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gradients Buffer"),
            size: (num_states * 16 * 2 * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Params uniform (u32 array, epsilon será convertido)
        let epsilon_bits = 0.001f32.to_bits();
        let params = [num_states as u32, epsilon_bits, 0u32, 0u32]; // num_states, epsilon(bits), padding
        let params_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::cast_slice(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        // Criar bind group
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gradient Bind Group"),
            layout: &ctx.gradient_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: states_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: gradients_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Encode e dispatch
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Gradient Encoder"),
        });
        
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Gradient Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&ctx.gradient_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch: 64 threads per workgroup
            let workgroups = (num_states + 63) / 64;
            cpass.dispatch_workgroups(workgroups as u32, 1, 1);
        }
        
        // Criar staging buffer para readback
        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: gradients_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        encoder.copy_buffer_to_buffer(
            &gradients_buffer,
            0,
            &staging_buffer,
            0,
            staging_buffer.size(),
        );
        
        ctx.queue.submit(Some(encoder.finish()));
        
        // Readback (async)
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = oneshot::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        
        ctx.device.poll(wgpu::Maintain::Wait);
        let map_result = rx.await.map_err(|_| GpuError::Timeout)?;
        map_result.map_err(|e| GpuError::ShaderCompilation(format!("Buffer map failed: {:?}", e)))?;
        
        // Copiar dados
        let data = buffer_slice.get_mapped_range();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        
        let mut gradients = Vec::with_capacity(num_states);
        for i in 0..num_states {
            let offset = i * 32; // 16 layers × 2 floats
            let mut layer_data = [0.0f32; 32];
            layer_data.copy_from_slice(&floats[offset..offset + 32]);
            gradients.push(SilGradient::from_floats(&layer_data));
        }
        
        drop(data);
        staging_buffer.unmap();
        
        Ok(gradients)
    }
    
    /// Executa interpolação na GPU
    async fn execute_interpolate(
        ctx: &GpuContext,
        states_a: &[SilState],
        states_b: &[SilState],
        t: f32,
        use_slerp: bool,
    ) -> GpuResult<Vec<SilState>> {
        let num_states = states_a.len();
        
        if states_b.len() != num_states {
            return Err(GpuError::BufferOverflow {
                expected: num_states,
                actual: states_b.len(),
            });
        }
        
        // Converter estados A para formato GPU
        let mut state_data_a = Vec::with_capacity(num_states * 4);
        for state in states_a {
            let bytes = state.to_bytes();
            for chunk in bytes.chunks(4) {
                let mut word = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    word |= (byte as u32) << (i * 8);
                }
                state_data_a.push(word);
            }
        }
        
        // Converter estados B para formato GPU
        let mut state_data_b = Vec::with_capacity(num_states * 4);
        for state in states_b {
            let bytes = state.to_bytes();
            for chunk in bytes.chunks(4) {
                let mut word = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    word |= (byte as u32) << (i * 8);
                }
                state_data_b.push(word);
            }
        }
        
        // Criar buffers
        let states_a_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("States A Buffer"),
            contents: bytemuck::cast_slice(&state_data_a),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        
        let states_b_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("States B Buffer"),
            contents: bytemuck::cast_slice(&state_data_b),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        
        let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (num_states * 4 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Params uniform
        let params = [
            num_states as u32,
            t.to_bits(),
            if use_slerp { 1u32 } else { 0u32 },
            0u32, // padding
        ];
        let params_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Interpolate Params Buffer"),
            contents: bytemuck::cast_slice(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        // Criar bind group
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Interpolate Bind Group"),
            layout: &ctx.interpolate_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: states_a_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: states_b_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Encode e dispatch
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Interpolate Encoder"),
        });
        
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Interpolate Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&ctx.interpolate_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch: 64 threads per workgroup
            let workgroups = (num_states + 63) / 64;
            cpass.dispatch_workgroups(workgroups as u32, 1, 1);
        }
        
        // Criar staging buffer para readback
        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            staging_buffer.size(),
        );
        
        ctx.queue.submit(Some(encoder.finish()));
        
        // Readback (async)
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = oneshot::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        
        ctx.device.poll(wgpu::Maintain::Wait);
        let map_result = rx.await.map_err(|_| GpuError::Timeout)?;
        map_result.map_err(|e| GpuError::ShaderCompilation(format!("Buffer map failed: {:?}", e)))?;
        
        // Copiar dados
        let data = buffer_slice.get_mapped_range();
        let words: &[u32] = bytemuck::cast_slice(&data);
        
        let mut results = Vec::with_capacity(num_states);
        for i in 0..num_states {
            let offset = i * 4;
            let word_data = &words[offset..offset + 4];
            
            // Converter de u32 para bytes
            let mut bytes = [0u8; 16];
            for (j, &word) in word_data.iter().enumerate() {
                let word_bytes = word.to_le_bytes();
                bytes[j * 4..(j + 1) * 4].copy_from_slice(&word_bytes);
            }
            
            results.push(SilState::from_bytes(&bytes));
        }
        
        drop(data);
        staging_buffer.unmap();
        
        Ok(results)
    }
}

/// Handle para operações GPU com auto-flush
#[derive(Clone)]
pub struct BatchedGpuHandle {
    executor: Arc<BatchedGpuExecutor>,
}

impl BatchedGpuHandle {
    pub fn new(ctx: Arc<GpuContext>) -> Self {
        Self {
            executor: Arc::new(BatchedGpuExecutor::new_default(ctx)),
        }
    }
    
    pub fn with_config(ctx: Arc<GpuContext>, config: BatchConfig) -> Self {
        Self {
            executor: Arc::new(BatchedGpuExecutor::new(ctx, config)),
        }
    }
    
    /// Compute gradients (async batched)
    pub async fn compute_gradients(&self, states: Vec<SilState>) -> GpuResult<Vec<SilGradient>> {
        self.executor.compute_gradients(states).await
    }
    
    /// Interpolate states (async batched)
    pub async fn interpolate(
        &self,
        states_a: Vec<SilState>,
        states_b: Vec<SilState>,
        t: f32,
        use_slerp: bool,
    ) -> GpuResult<Vec<SilState>> {
        self.executor.interpolate(states_a, states_b, t, use_slerp).await
    }
}
