//! Implementação concreta do processador eletrônico

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::Processor;
use sil_core::vsp::{Vsp, VspConfig};
use crate::error::{ElectronicError, ElectronicResult};
use crate::state::ProcessorState;

/// Processador eletrônico concreto (L5)
#[derive(Clone)]
pub struct ElectronicProcessor {
    /// Estado interno
    state: Arc<Mutex<ProcessorState>>,
    /// Instância do VSP
    vsp: Arc<Mutex<Vsp>>,
    /// ID único
    id: u128,
    /// Configuração
    config: ElectronicConfig,
    /// Timestamp de criação
    created_at: u64,
}

impl std::fmt::Debug for ElectronicProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElectronicProcessor")
            .field("id", &self.id)
            .field("created_at", &self.created_at)
            .field("config", &self.config)
            .finish()
    }
}

/// Configuração do processador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicConfig {
    /// Tamanho máximo do heap (estados)
    pub heap_size: usize,
    /// Tamanho máximo da stack (frames)
    pub stack_size: usize,
    /// Ativar GPU
    pub enable_gpu: bool,
    /// Ativar NPU
    pub enable_npu: bool,
    /// Limite de ciclos por execução
    pub max_cycles: u64,
    /// Debug mode
    pub debug: bool,
}

impl Default for ElectronicConfig {
    fn default() -> Self {
        Self {
            heap_size: 65536,
            stack_size: 1024,
            enable_gpu: true,
            enable_npu: true,
            max_cycles: 1_000_000,
            debug: false,
        }
    }
}

impl ElectronicProcessor {
    /// Cria novo processador eletrônico
    pub fn new() -> ElectronicResult<Self> {
        Self::with_config(ElectronicConfig::default())
    }

    /// Cria com configuração específica
    pub fn with_config(config: ElectronicConfig) -> ElectronicResult<Self> {
        let vsp_config = VspConfig {
            heap_size: config.heap_size,
            stack_size: config.stack_size,
            enable_gpu: config.enable_gpu,
            enable_npu: config.enable_npu,
            debug: config.debug,
            ..Default::default()
        };

        let vsp = Vsp::new(vsp_config)
            .map_err(|e| ElectronicError::VspError(e.to_string()))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Gerar ID simples: converter timestamp para u128
        let id = (timestamp as u128) ^ ((timestamp as u128) << 32);

        Ok(Self {
            state: Arc::new(Mutex::new(ProcessorState::new())),
            vsp: Arc::new(Mutex::new(vsp)),
            id,
            config,
            created_at: timestamp,
        })
    }

    /// Carrega bytecode .silc de arquivo
    pub fn load_silc(&mut self, path: &std::path::Path) -> ElectronicResult<()> {
        let mut vsp = self.vsp.lock().unwrap();
        vsp.load_silc(path)
            .map_err(|e| ElectronicError::VspError(e.to_string()))?;

        let mut state = self.state.lock().unwrap();
        state.bytecode = std::fs::read(path)
            .map_err(|e| ElectronicError::InvalidBytecode(e.to_string()))?;
        Ok(())
    }

    /// Carrega bytecode de bytes
    pub fn load_bytes(&mut self, code: &[u8], data: &[u8]) -> ElectronicResult<()> {
        let mut vsp = self.vsp.lock().unwrap();
        vsp.load_bytes(code, data)
            .map_err(|e| ElectronicError::VspError(e.to_string()))?;

        let mut state = self.state.lock().unwrap();
        state.bytecode = code.to_vec();
        state.data = data.to_vec();
        Ok(())
    }

    /// Retorna estado interno
    pub fn state(&self) -> ElectronicResult<ProcessorState> {
        let state = self.state.lock().unwrap();
        Ok(state.clone())
    }

    /// Reseta o processador
    pub fn reset(&mut self) -> ElectronicResult<()> {
        let mut state = self.state.lock().unwrap();
        state.reset();

        let mut vsp = self.vsp.lock().unwrap();
        *vsp = Vsp::new(VspConfig {
            heap_size: self.config.heap_size,
            stack_size: self.config.stack_size,
            enable_gpu: self.config.enable_gpu,
            enable_npu: self.config.enable_npu,
            debug: self.config.debug,
            ..Default::default()
        })
        .map_err(|e| ElectronicError::VspError(e.to_string()))?;

        Ok(())
    }

    /// Executa um passo
    pub fn step(&mut self) -> ElectronicResult<bool> {
        let mut vsp = self.vsp.lock().unwrap();
        let running = vsp.step()
            .map_err(|e| ElectronicError::VspError(e.to_string()))?;

        let mut state = self.state.lock().unwrap();
        state.cycles += 1;

        if !running {
            state.halted = true;
        }

        Ok(running)
    }

    /// Executa até halt
    pub fn run(&mut self) -> ElectronicResult<SilState> {
        let start_cycles = {
            let state = self.state.lock().unwrap();
            state.cycles
        };

        loop {
            let cycles = {
                let state = self.state.lock().unwrap();
                state.cycles - start_cycles
            };

            if cycles >= self.config.max_cycles {
                let mut state = self.state.lock().unwrap();
                state.mark_error("Execution limit exceeded".into());
                return Err(ElectronicError::ExecutionLimitExceeded {
                    reason: format!("Max {} cycles reached", self.config.max_cycles),
                });
            }

            let running = self.step()?;
            if !running {
                break;
            }
        }

        let vsp = self.vsp.lock().unwrap();
        let result = vsp.to_sil_state();
        
        let mut state = self.state.lock().unwrap();
        state.accumulated_state = Some(result.clone());

        Ok(result)
    }

    /// Obtém informações de execução
    pub fn execution_info(&self) -> ElectronicResult<crate::state::ExecutionInfo> {
        let state = self.state.lock().unwrap();
        Ok(state.execution_info())
    }
}

impl Default for ElectronicProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create ElectronicProcessor")
    }
}

/// Implementação do trait Processor
impl Processor for ElectronicProcessor {
    fn execute(&mut self, _state: &SilState) -> Result<SilState, ProcessorError> {
        // Executa o programa carregado
        self.run().map_err(|e| ProcessorError::ExecutionFailed(e.to_string()))
    }
}

/// Implementação do trait SilComponent
impl SilComponent for ElectronicProcessor {
    fn name(&self) -> &str {
        "ElectronicProcessor"
    }

    fn layers(&self) -> &[LayerId] {
        &[5] // L5 - Eletrônico
    }

    fn version(&self) -> &str {
        "2026.1.11"
    }

    fn is_ready(&self) -> bool {
        self.state.lock().unwrap().is_ready()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_processor() {
        let proc = ElectronicProcessor::new();
        assert!(proc.is_ok());
    }

    #[test]
    fn test_processor_default() {
        let proc = ElectronicProcessor::default();
        assert_eq!(proc.layers(), &[5]);
    }

    #[test]
    fn test_processor_config() {
        let config = ElectronicConfig {
            heap_size: 32768,
            stack_size: 512,
            ..Default::default()
        };
        let proc = ElectronicProcessor::with_config(config);
        assert!(proc.is_ok());
    }

    #[test]
    fn test_processor_load_bytes() {
        let mut proc = ElectronicProcessor::new().unwrap();
        let code = vec![0x00, 0x01, 0x02];
        let data = vec![0x03, 0x04];
        let result = proc.load_bytes(&code, &data);
        assert!(result.is_ok());

        let state = proc.state().unwrap();
        assert_eq!(state.bytecode.len(), 3);
        assert_eq!(state.data.len(), 2);
    }

    #[test]
    fn test_processor_status() {
        let mut proc = ElectronicProcessor::new().unwrap();
        
        // Sem bytecode
        let state = proc.state().unwrap();
        assert!(state.error.is_none());

        // Com bytecode
        proc.load_bytes(&[0x00], &[]).unwrap();
        let state = proc.state().unwrap();
        assert!(state.bytecode.len() > 0);
    }

    #[test]
    fn test_processor_reset() {
        let mut proc = ElectronicProcessor::new().unwrap();
        proc.load_bytes(&[1, 2, 3], &[4, 5]).unwrap();

        let state_before = proc.state().unwrap();
        assert!(!state_before.bytecode.is_empty());

        proc.reset().unwrap();
        let state_after = proc.state().unwrap();
        assert_eq!(state_after.pc, 0);
        assert_eq!(state_after.cycles, 0);
        assert!(!state_after.halted);
    }
}

