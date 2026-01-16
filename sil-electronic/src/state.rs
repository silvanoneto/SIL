//! Estado do processador eletrônico

use serde::{Deserialize, Serialize};
use sil_core::state::SilState;

/// Estado interno do processador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorState {
    /// Contador de programa
    pub pc: u32,
    /// Número de ciclos executados
    pub cycles: u64,
    /// Código carregado
    pub bytecode: Vec<u8>,
    /// Dados iniciais
    pub data: Vec<u8>,
    /// Flag de halt
    pub halted: bool,
    /// Flag de erro
    pub error: Option<String>,
    /// Timestamp da última execução
    pub last_execution_ns: u64,
    /// Estado SIL acumulado
    pub accumulated_state: Option<SilState>,
}

impl ProcessorState {
    /// Cria novo estado
    pub fn new() -> Self {
        Self {
            pc: 0,
            cycles: 0,
            bytecode: Vec::new(),
            data: Vec::new(),
            halted: false,
            error: None,
            last_execution_ns: 0,
            accumulated_state: None,
        }
    }

    /// Define bytecode
    pub fn with_bytecode(mut self, bytecode: Vec<u8>) -> Self {
        self.bytecode = bytecode;
        self
    }

    /// Define dados
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Reseta estado, limpando bytecode
    pub fn reset(&mut self) {
        self.pc = 0;
        self.cycles = 0;
        self.bytecode.clear();
        self.data.clear();
        self.halted = false;
        self.error = None;
        self.accumulated_state = None;
    }

    /// Marca como halt
    pub fn mark_halted(&mut self) {
        self.halted = true;
    }

    /// Marca como erro
    pub fn mark_error(&mut self, msg: String) {
        self.error = Some(msg);
        self.halted = true;
    }

    /// Verifica se está pronto para executar
    pub fn is_ready(&self) -> bool {
        !self.bytecode.is_empty() && !self.halted && self.error.is_none()
    }

    /// Retorna informações de execução
    pub fn execution_info(&self) -> ExecutionInfo {
        ExecutionInfo {
            pc: self.pc,
            cycles: self.cycles,
            bytecode_size: self.bytecode.len(),
            data_size: self.data.len(),
            halted: self.halted,
            has_error: self.error.is_some(),
        }
    }
}

impl Default for ProcessorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Informações de execução
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub pc: u32,
    pub cycles: u64,
    pub bytecode_size: usize,
    pub data_size: usize,
    pub halted: bool,
    pub has_error: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_state_new() {
        let state = ProcessorState::new();
        assert_eq!(state.pc, 0);
        assert_eq!(state.cycles, 0);
        assert!(!state.halted);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_processor_state_with_bytecode() {
        let bytecode = vec![0x00, 0x01, 0x02];
        let state = ProcessorState::new().with_bytecode(bytecode.clone());
        assert_eq!(state.bytecode, bytecode);
    }

    #[test]
    fn test_processor_state_reset() {
        let mut state = ProcessorState::new().with_bytecode(vec![1, 2, 3]);
        state.pc = 100;
        state.cycles = 50;
        state.halted = true;
        state.reset();
        assert_eq!(state.pc, 0);
        assert_eq!(state.cycles, 0);
        assert!(!state.halted);
    }

    #[test]
    fn test_processor_state_is_ready() {
        let mut state = ProcessorState::new();
        assert!(!state.is_ready()); // sem bytecode

        state = state.with_bytecode(vec![1, 2]);
        assert!(state.is_ready());

        state.halted = true;
        assert!(!state.is_ready());
    }

    #[test]
    fn test_execution_info() {
        let state = ProcessorState::new()
            .with_bytecode(vec![1, 2, 3])
            .with_data(vec![4, 5]);
        let info = state.execution_info();
        assert_eq!(info.bytecode_size, 3);
        assert_eq!(info.data_size, 2);
        assert!(!info.halted);
    }
}
