//! Memória do VSP
//!
//! Segmentos de memória: código, heap de estados, stack e I/O.

use crate::state::{ByteSil, SilState};
use super::error::{VspError, VspResult};
use super::state::CallFrame;

/// Segmentos de memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySegment {
    /// Código (0x0000_0000 - 0x0FFF_FFFF)
    Code,
    /// Heap de estados (0x1000_0000 - 0x1FFF_FFFF)
    StateHeap,
    /// Stack de chamadas (0x2000_0000 - 0x2FFF_FFFF)
    CallStack,
    /// Tabela de transforms (0x3000_0000 - 0x3FFF_FFFF)
    TransformTable,
    /// I/O mapeado (0xF000_0000 - 0xFFFF_FFFF)
    IoMapped,
}

impl MemorySegment {
    /// Base do segmento
    pub fn base(&self) -> u32 {
        match self {
            Self::Code => 0x0000_0000,
            Self::StateHeap => 0x1000_0000,
            Self::CallStack => 0x2000_0000,
            Self::TransformTable => 0x3000_0000,
            Self::IoMapped => 0xF000_0000,
        }
    }
    
    /// Tamanho do segmento
    pub fn size(&self) -> u32 {
        match self {
            Self::Code => 0x1000_0000,
            Self::StateHeap => 0x1000_0000,
            Self::CallStack => 0x1000_0000,
            Self::TransformTable => 0xC000_0000,
            Self::IoMapped => 0x1000_0000,
        }
    }
    
    /// Determina segmento de um endereço
    pub fn from_addr(addr: u32) -> Option<Self> {
        match addr {
            0x0000_0000..=0x0FFF_FFFF => Some(Self::Code),
            0x1000_0000..=0x1FFF_FFFF => Some(Self::StateHeap),
            0x2000_0000..=0x2FFF_FFFF => Some(Self::CallStack),
            0x3000_0000..=0xEFFF_FFFF => Some(Self::TransformTable),
            0xF000_0000..=0xFFFF_FFFF => Some(Self::IoMapped),
        }
    }
}

/// Memória do VSP
pub struct VspMemory {
    /// Segmento de código
    code: Vec<u8>,
    /// Heap de estados
    state_heap: Vec<SilState>,
    /// Stack de estados (para PUSH/POP)
    state_stack: Vec<ByteSil>,
    /// Stack de chamadas
    call_stack: Vec<CallFrame>,
    /// Dados iniciais
    data: Vec<u8>,
    /// Tabela de transforms (IDs) - reservado para uso futuro
    _transforms: Vec<u32>,
    /// Buffer de I/O
    io_buffer: Vec<ByteSil>,
    /// Sensores virtuais
    sensors: [ByteSil; 16],
    /// Atuadores virtuais
    actuators: [ByteSil; 16],
    /// Buffer de output (registra cada ACT)
    output_buffer: Vec<ByteSil>,
    /// Buffer de input (dados de arquivo)
    input_buffer: Vec<u8>,
    /// Índice de leitura do input
    input_index: usize,
    /// Capacidade do heap
    heap_capacity: usize,
    /// Capacidade da stack
    stack_capacity: usize,
}

impl VspMemory {
    /// Cria nova memória
    pub fn new(heap_capacity: usize, stack_capacity: usize) -> VspResult<Self> {
        Ok(Self {
            code: Vec::new(),
            state_heap: Vec::with_capacity(heap_capacity),
            state_stack: Vec::with_capacity(stack_capacity * 16),
            call_stack: Vec::with_capacity(stack_capacity),
            data: Vec::new(),
            _transforms: Vec::new(),
            io_buffer: vec![ByteSil::NULL; 256],
            sensors: [ByteSil::NULL; 16],
            actuators: [ByteSil::NULL; 16],
            output_buffer: Vec::new(),
            input_buffer: Vec::new(),
            input_index: 0,
            heap_capacity,
            stack_capacity,
        })
    }
    
    /// Carrega dados de input (arquivo) para leitura via SENSE
    pub fn load_input(&mut self, data: &[u8]) -> VspResult<()> {
        self.input_buffer = data.to_vec();
        self.input_index = 0;
        Ok(())
    }
    
    /// Lê próximo byte do input buffer (para SENSE)
    pub fn read_input(&mut self) -> ByteSil {
        if self.input_index < self.input_buffer.len() {
            let byte = self.input_buffer[self.input_index];
            self.input_index += 1;
            ByteSil::from_u8(byte)
        } else {
            ByteSil::NULL  // EOF
        }
    }
    
    /// Verifica se ainda há dados no input
    pub fn has_input(&self) -> bool {
        self.input_index < self.input_buffer.len()
    }
    
    /// Retorna o buffer de output (valores escritos via ACT)
    pub fn output(&self) -> &[ByteSil] {
        &self.output_buffer
    }
    
    /// Limpa o buffer de output
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }
    
    /// Retorna referência ao segmento de código
    pub fn code(&self) -> &[u8] {
        &self.code
    }
    
    /// Retorna referência mutável ao segmento de código
    pub fn code_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }
    
    /// Retorna referência aos dados
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Retorna referência mutável aos dados
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    
    /// Carrega código
    pub fn load_code(&mut self, code: &[u8]) -> VspResult<()> {
        self.code = code.to_vec();
        Ok(())
    }
    
    /// Carrega dados
    pub fn load_data(&mut self, data: &[u8]) -> VspResult<()> {
        self.data = data.to_vec();
        
        // Parseia estados iniciais (cada 16 bytes = 1 SilState)
        for chunk in data.chunks(16) {
            if chunk.len() == 16 {
                let mut layers = [ByteSil::NULL; 16];
                for (i, &byte) in chunk.iter().enumerate() {
                    layers[i] = ByteSil::from_u8(byte);
                }
                self.state_heap.push(SilState { layers });
            }
        }
        
        Ok(())
    }
    
    /// Fetch de instrução
    pub fn fetch(&self, pc: u32) -> VspResult<&[u8]> {
        let offset = pc as usize;
        if offset >= self.code.len() {
            eprintln!("[FETCH] PC={:#x} out of bounds (code len={})", pc, self.code.len());
            return Err(VspError::AddressOutOfBounds(pc));
        }
        
        // Retorna até 4 bytes (tamanho máximo de instrução)
        let end = (offset + 4).min(self.code.len());
        Ok(&self.code[offset..end])
    }
    
    /// Carrega ByteSil de endereço
    pub fn load_byte_sil(&self, addr: u32) -> VspResult<ByteSil> {
        let segment = MemorySegment::from_addr(addr)
            .ok_or(VspError::AddressOutOfBounds(addr))?;
        
        match segment {
            MemorySegment::Code => {
                // Pode ser um dado na seção de dados (endereços baixos)
                let offset = addr as usize;
                if offset < self.data.len() {
                    // Ler da seção de dados
                    Ok(ByteSil::from_u8(self.data[offset]))
                } else {
                    Err(VspError::AddressOutOfBounds(addr))
                }
            }
            MemorySegment::StateHeap => {
                let offset = (addr - segment.base()) as usize;
                let state_idx = offset / 16;
                let layer_idx = offset % 16;
                
                self.state_heap.get(state_idx)
                    .map(|s| s.layers[layer_idx])
                    .ok_or(VspError::AddressOutOfBounds(addr))
            }
            MemorySegment::IoMapped => {
                let offset = (addr - segment.base()) as usize;
                self.io_buffer.get(offset)
                    .copied()
                    .ok_or(VspError::AddressOutOfBounds(addr))
            }
            _ => Err(VspError::InvalidSegment(segment)),
        }
    }
    
    /// Armazena ByteSil em endereço
    pub fn store_byte_sil(&mut self, addr: u32, value: ByteSil) -> VspResult<()> {
        let segment = MemorySegment::from_addr(addr)
            .ok_or(VspError::AddressOutOfBounds(addr))?;
        
        match segment {
            MemorySegment::StateHeap => {
                let offset = (addr - segment.base()) as usize;
                let state_idx = offset / 16;
                let layer_idx = offset % 16;
                
                // Expande heap se necessário
                while self.state_heap.len() <= state_idx {
                    if self.state_heap.len() >= self.heap_capacity {
                        return Err(VspError::HeapOverflow);
                    }
                    self.state_heap.push(SilState::neutral());
                }
                
                self.state_heap[state_idx].layers[layer_idx] = value;
                Ok(())
            }
            MemorySegment::IoMapped => {
                let offset = (addr - segment.base()) as usize;
                if offset < self.io_buffer.len() {
                    self.io_buffer[offset] = value;
                    Ok(())
                } else {
                    Err(VspError::AddressOutOfBounds(addr))
                }
            }
            MemorySegment::Code => Err(VspError::WriteToReadOnly(addr)),
            _ => Err(VspError::InvalidSegment(segment)),
        }
    }
    
    /// Carrega SilState completo
    pub fn load_sil_state(&self, addr: u32) -> VspResult<SilState> {
        let segment = MemorySegment::from_addr(addr)
            .ok_or(VspError::AddressOutOfBounds(addr))?;
        
        match segment {
            MemorySegment::StateHeap => {
                let offset = (addr - segment.base()) as usize;
                let state_idx = offset / 16;
                
                self.state_heap.get(state_idx)
                    .cloned()
                    .ok_or(VspError::AddressOutOfBounds(addr))
            }
            _ => {
                // Lê 16 ByteSils consecutivos
                let mut layers = [ByteSil::NULL; 16];
                for i in 0..16 {
                    layers[i] = self.load_byte_sil(addr + i as u32)?;
                }
                Ok(SilState { layers })
            }
        }
    }
    
    /// Armazena SilState completo
    pub fn store_sil_state(&mut self, addr: u32, state: &SilState) -> VspResult<()> {
        let segment = MemorySegment::from_addr(addr)
            .ok_or(VspError::AddressOutOfBounds(addr))?;
        
        match segment {
            MemorySegment::StateHeap => {
                let offset = (addr - segment.base()) as usize;
                let state_idx = offset / 16;
                
                while self.state_heap.len() <= state_idx {
                    if self.state_heap.len() >= self.heap_capacity {
                        return Err(VspError::HeapOverflow);
                    }
                    self.state_heap.push(SilState::neutral());
                }
                
                self.state_heap[state_idx] = state.clone();
                Ok(())
            }
            _ => {
                for (i, &layer) in state.layers.iter().enumerate() {
                    self.store_byte_sil(addr + i as u32, layer)?;
                }
                Ok(())
            }
        }
    }
    
    /// Carrega u8
    pub fn load_u8(&self, addr: u32) -> VspResult<u8> {
        let segment = MemorySegment::from_addr(addr)
            .ok_or(VspError::AddressOutOfBounds(addr))?;
        
        match segment {
            MemorySegment::Code => {
                let offset = addr as usize;
                self.code.get(offset)
                    .copied()
                    .ok_or(VspError::AddressOutOfBounds(addr))
            }
            _ => {
                let bs = self.load_byte_sil(addr)?;
                Ok(bs.to_u8())
            }
        }
    }
    
    /// Carrega u32
    pub fn load_u32(&self, addr: u32) -> VspResult<u32> {
        let b0 = self.load_u8(addr)? as u32;
        let b1 = self.load_u8(addr + 1)? as u32;
        let b2 = self.load_u8(addr + 2)? as u32;
        let b3 = self.load_u8(addr + 3)? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }
    
    /// Carrega pipeline de transforms
    pub fn load_pipeline(&self, addr: u32) -> VspResult<Vec<u32>> {
        let count = self.load_u32(addr)? as usize;
        let mut pipeline = Vec::with_capacity(count);
        
        for i in 0..count {
            pipeline.push(self.load_u32(addr + 4 + (i as u32) * 4)?);
        }
        
        Ok(pipeline)
    }
    
    /// Push estado na stack
    pub fn push_state(&mut self, state: ByteSil) -> VspResult<()> {
        if self.state_stack.len() >= self.stack_capacity * 16 {
            return Err(VspError::StackOverflow);
        }
        self.state_stack.push(state);
        Ok(())
    }
    
    /// Pop estado da stack
    pub fn pop_state(&mut self) -> VspResult<ByteSil> {
        self.state_stack.pop()
            .ok_or(VspError::StackUnderflow)
    }
    
    /// Push frame de chamada
    pub fn push_frame(&mut self, return_addr: u32, prev_fp: u32) -> VspResult<()> {
        if self.call_stack.len() >= self.stack_capacity {
            return Err(VspError::StackOverflow);
        }
        self.call_stack.push(CallFrame::new(return_addr, prev_fp));
        Ok(())
    }
    
    /// Pop frame de chamada
    pub fn pop_frame(&mut self) -> VspResult<CallFrame> {
        self.call_stack.pop()
            .ok_or(VspError::StackUnderflow)
    }
    
    /// I/O read
    pub fn io_read(&self, port: u32) -> VspResult<ByteSil> {
        let idx = port as usize;
        if idx < self.io_buffer.len() {
            Ok(self.io_buffer[idx])
        } else {
            Err(VspError::InvalidPort(port))
        }
    }
    
    /// I/O write
    pub fn io_write(&mut self, port: u32, value: ByteSil) -> VspResult<()> {
        let idx = port as usize;
        if idx < self.io_buffer.len() {
            self.io_buffer[idx] = value;
            Ok(())
        } else {
            Err(VspError::InvalidPort(port))
        }
    }
    
    /// Lê sensor (prioriza input buffer se disponível)
    /// Retorna (valor, eof_flag)
    pub fn sense(&mut self, sensor_id: usize) -> VspResult<(ByteSil, bool)> {
        // Se há dados no input buffer, lê de lá
        if self.has_input() {
            return Ok((self.read_input(), false));
        }
        
        // EOF - não há mais dados no input
        if self.input_buffer.len() > 0 && self.input_index >= self.input_buffer.len() {
            return Ok((ByteSil::NULL, true));
        }
        
        // Senão, lê do sensor
        if sensor_id < 16 {
            Ok((self.sensors[sensor_id], false))
        } else {
            Err(VspError::InvalidSensor(sensor_id))
        }
    }
    
    /// Escreve atuador
    pub fn actuate(&mut self, actuator_id: usize, value: ByteSil) -> VspResult<()> {
        if actuator_id < 16 {
            self.actuators[actuator_id] = value;
            // Registrar no buffer de output
            self.output_buffer.push(value);
            Ok(())
        } else {
            Err(VspError::InvalidActuator(actuator_id))
        }
    }
    
    /// Define valor de sensor (para simulação)
    pub fn set_sensor(&mut self, sensor_id: usize, value: ByteSil) -> VspResult<()> {
        if sensor_id < 16 {
            self.sensors[sensor_id] = value;
            Ok(())
        } else {
            Err(VspError::InvalidSensor(sensor_id))
        }
    }
    
    /// Lê valor de atuador (para verificação)
    pub fn get_actuator(&self, actuator_id: usize) -> VspResult<ByteSil> {
        if actuator_id < 16 {
            Ok(self.actuators[actuator_id])
        } else {
            Err(VspError::InvalidActuator(actuator_id))
        }
    }
    
    /// Broadcast estado (stub)
    pub fn broadcast(&self, _addr: u32, _state: &SilState) -> VspResult<()> {
        // TODO: implementar broadcast distribuído
        Ok(())
    }
    
    /// Recebe estado (stub)
    pub fn receive(&self, _addr: u32) -> VspResult<Option<SilState>> {
        // TODO: implementar receive distribuído
        Ok(None)
    }
    
    /// Entangle (stub)
    pub fn entangle(&mut self, _reg: usize, _node_id: u32, _value: ByteSil) -> VspResult<()> {
        // TODO: implementar entanglement
        Ok(())
    }
    
    /// Prefetch (stub)
    pub fn prefetch(&self, _addr: u32) -> VspResult<()> {
        // Hint para pré-carregamento
        Ok(())
    }
    
    /// Reset da memória
    pub fn reset(&mut self) {
        self.state_heap.clear();
        self.state_stack.clear();
        self.call_stack.clear();
        self.io_buffer.fill(ByteSil::NULL);
        self.sensors = [ByteSil::NULL; 16];
        self.actuators = [ByteSil::NULL; 16];
    }
    
    /// Estatísticas
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            code_size: self.code.len(),
            heap_used: self.state_heap.len(),
            heap_capacity: self.heap_capacity,
            stack_used: self.state_stack.len(),
            call_depth: self.call_stack.len(),
        }
    }
}

/// Estatísticas de memória
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub code_size: usize,
    pub heap_used: usize,
    pub heap_capacity: usize,
    pub stack_used: usize,
    pub call_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_creation() {
        let mem = VspMemory::new(1024, 64);
        assert!(mem.is_ok());
    }
    
    #[test]
    fn test_code_load_and_fetch() {
        let mut mem = VspMemory::new(1024, 64).unwrap();
        mem.load_code(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        
        let fetched = mem.fetch(0).unwrap();
        assert_eq!(fetched, &[0x00, 0x01, 0x02, 0x03]);
    }
    
    #[test]
    fn test_state_heap() {
        let mut mem = VspMemory::new(1024, 64).unwrap();
        let state = SilState::neutral();
        let addr = 0x1000_0000;
        
        mem.store_sil_state(addr, &state).unwrap();
        let loaded = mem.load_sil_state(addr).unwrap();
        
        assert_eq!(loaded.layers, state.layers);
    }
    
    #[test]
    fn test_state_stack() {
        let mut mem = VspMemory::new(1024, 64).unwrap();
        let value = ByteSil::new(3, 7);
        
        mem.push_state(value).unwrap();
        let popped = mem.pop_state().unwrap();
        
        assert_eq!(popped, value);
    }
    
    #[test]
    fn test_call_stack() {
        let mut mem = VspMemory::new(1024, 64).unwrap();
        
        mem.push_frame(0x1234, 0x5678).unwrap();
        let frame = mem.pop_frame().unwrap();
        
        assert_eq!(frame.return_addr, 0x1234);
        assert_eq!(frame.prev_fp, 0x5678);
    }
    
    #[test]
    fn test_sensors_actuators() {
        let mut mem = VspMemory::new(1024, 64).unwrap();
        
        mem.set_sensor(0, ByteSil::new(5, 10)).unwrap();
        let (value, eof) = mem.sense(0).unwrap();
        assert_eq!(value, ByteSil::new(5, 10));
        assert!(!eof);
        
        mem.actuate(3, ByteSil::new(2, 4)).unwrap();
        assert_eq!(mem.get_actuator(3).unwrap(), ByteSil::new(2, 4));
    }
}
