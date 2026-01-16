//! Erros do VSP

use super::state::SilMode;
use super::memory::MemorySegment;
use std::fmt;

/// Tipo de resultado do VSP
pub type VspResult<T> = Result<T, VspError>;

/// Erros do VSP
#[derive(Debug)]
pub enum VspError {
    /// Opcode inválido
    InvalidOpcode(u8),
    
    /// Instrução truncada
    InstructionTruncated {
        expected: usize,
        found: usize,
    },
    
    /// Fim inesperado do código
    UnexpectedEof,
    
    /// Endereço fora dos limites
    AddressOutOfBounds(u32),
    
    /// Escrita em memória somente leitura
    WriteToReadOnly(u32),
    
    /// Segmento inválido para operação
    InvalidSegment(MemorySegment),
    
    /// Stack overflow
    StackOverflow,
    
    /// Stack underflow
    StackUnderflow,
    
    /// Heap overflow
    HeapOverflow,
    
    /// Modo inválido
    InvalidMode(u8),
    
    /// Modos incompatíveis
    IncompatibleMode {
        expected: SilMode,
        found: SilMode,
    },
    
    /// Porta de I/O inválida
    InvalidPort(u32),
    
    /// Sensor inválido
    InvalidSensor(usize),
    
    /// Atuador inválido
    InvalidActuator(usize),
    
    /// Syscall inválida
    InvalidSyscall(u32),
    
    /// Bytecode inválido
    InvalidBytecode(String),
    
    /// Estado inválido
    InvalidState,
    
    /// Erro de I/O
    IoError(String),
    
    /// Backend não disponível
    BackendNotAvailable(String),
    
    /// Erro de assembler
    AssemblerError(String),
    
    /// Erro de compilação (JIT/AOT)
    CompilationError(String),
    
    /// Erro de serialização
    SerializationError(String),
    
    /// Entanglement quebrado
    EntanglementBroken,
    
    /// Erro genérico
    Other(String),
}

impl fmt::Display for VspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            Self::InstructionTruncated { expected, found } => {
                write!(f, "Instruction truncated: expected {} bytes, found {}", expected, found)
            }
            Self::UnexpectedEof => write!(f, "Unexpected end of code"),
            Self::AddressOutOfBounds(addr) => write!(f, "Address out of bounds: 0x{:08X}", addr),
            Self::WriteToReadOnly(addr) => write!(f, "Write to read-only memory: 0x{:08X}", addr),
            Self::InvalidSegment(seg) => write!(f, "Invalid segment for operation: {:?}", seg),
            Self::StackOverflow => write!(f, "Stack overflow"),
            Self::StackUnderflow => write!(f, "Stack underflow"),
            Self::HeapOverflow => write!(f, "Heap overflow"),
            Self::InvalidMode(mode) => write!(f, "Invalid SIL mode: {}", mode),
            Self::IncompatibleMode { expected, found } => {
                write!(f, "Incompatible modes: expected {}, found {}", expected, found)
            }
            Self::InvalidPort(port) => write!(f, "Invalid I/O port: {}", port),
            Self::InvalidSensor(id) => write!(f, "Invalid sensor: {}", id),
            Self::InvalidActuator(id) => write!(f, "Invalid actuator: {}", id),
            Self::InvalidSyscall(num) => write!(f, "Invalid syscall: {}", num),
            Self::InvalidBytecode(msg) => write!(f, "Invalid bytecode: {}", msg),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::BackendNotAvailable(name) => write!(f, "Backend not available: {}", name),
            Self::AssemblerError(msg) => write!(f, "Assembler error: {}", msg),
            Self::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::EntanglementBroken => write!(f, "Entanglement broken"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for VspError {}

impl From<std::io::Error> for VspError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = VspError::InvalidOpcode(0xFF);
        assert!(err.to_string().contains("0xFF"));
        
        let err = VspError::StackOverflow;
        assert!(err.to_string().contains("overflow"));
    }
}
