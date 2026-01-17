//! Instruções do VSP
//!
//! Decodificação e representação de instruções.

use super::opcode::Opcode;
use super::error::{VspError, VspResult};

/// Formato de instrução
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    /// Formato A: 8 bits (opcode apenas)
    FormatA,
    /// Formato B: 16 bits (opcode + registrador)
    FormatB,
    /// Formato C: 24 bits (opcode + 2 registradores + imm8)
    FormatC,
    /// Formato D: 32 bits (opcode + endereço/imediato 24 bits)
    FormatD,
}

impl InstructionFormat {
    /// Tamanho em bytes
    pub fn size(&self) -> usize {
        match self {
            Self::FormatA => 1,
            Self::FormatB => 2,
            Self::FormatC => 3,
            Self::FormatD => 4,
        }
    }
}

/// Instrução decodificada
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Opcode da instrução
    pub opcode: Opcode,
    /// Bytes raw da instrução
    raw: [u8; 4],
    /// Tamanho da instrução
    len: usize,
}

impl Instruction {
    /// Decodifica instrução de bytes
    pub fn decode(bytes: &[u8]) -> VspResult<Self> {
        if bytes.is_empty() {
            return Err(VspError::UnexpectedEof);
        }
        
        let opcode = Opcode::from_byte(bytes[0])
            .ok_or(VspError::InvalidOpcode(bytes[0]))?;
        
        let format = opcode.format();
        let len = format.size();
        
        if bytes.len() < len {
            return Err(VspError::InstructionTruncated {
                expected: len,
                found: bytes.len(),
            });
        }
        
        let mut raw = [0u8; 4];
        raw[..len].copy_from_slice(&bytes[..len]);
        
        Ok(Self { opcode, raw, len })
    }
    
    /// Cria instrução Formato A (opcode apenas)
    pub fn format_a(opcode: Opcode) -> Self {
        Self {
            opcode,
            raw: [opcode as u8, 0, 0, 0],
            len: 1,
        }
    }
    
    /// Cria instrução Formato B (opcode + reg)
    pub fn format_b(opcode: Opcode, reg: u8) -> Self {
        Self {
            opcode,
            raw: [opcode as u8, reg & 0x0F, 0, 0],
            len: 2,
        }
    }
    
    /// Cria instrução Formato C (opcode + ra + rb + imm8)
    pub fn format_c(opcode: Opcode, ra: u8, rb: u8, imm: u8) -> Self {
        Self {
            opcode,
            raw: [opcode as u8, (ra & 0x0F) | ((rb & 0x0F) << 4), imm, 0],
            len: 3,
        }
    }
    
    /// Cria instrução Formato D (opcode + addr24)
    pub fn format_d(opcode: Opcode, addr: u32) -> Self {
        Self {
            opcode,
            raw: [
                opcode as u8,
                (addr & 0xFF) as u8,
                ((addr >> 8) & 0xFF) as u8,
                ((addr >> 16) & 0xFF) as u8,
            ],
            len: 4,
        }
    }
    
    /// Tamanho da instrução em bytes
    pub fn size(&self) -> usize {
        self.len
    }
    
    /// Registrador A (bits 0-3 do byte 1)
    pub fn reg_a(&self) -> usize {
        (self.raw[1] & 0x0F) as usize
    }
    
    /// Registrador B (bits 4-7 do byte 1)
    pub fn reg_b(&self) -> usize {
        ((self.raw[1] >> 4) & 0x0F) as usize
    }
    
    /// Par de registradores (ra, rb)
    pub fn reg_pair(&self) -> (usize, usize) {
        (self.reg_a(), self.reg_b())
    }
    
    /// Imediato de 8 bits (byte 2)
    pub fn imm8(&self) -> u8 {
        self.raw[2]
    }
    
    /// Endereço ou imediato de 24 bits (bytes 1-3)
    pub fn addr_or_imm24(&self) -> u32 {
        (self.raw[1] as u32)
            | ((self.raw[2] as u32) << 8)
            | ((self.raw[3] as u32) << 16)
    }
    
    /// Bytes raw
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw[..self.len]
    }

    /// Retorna os bytes raw (alias para compatibilidade)
    pub fn raw_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    /// Retorna um byte específico da instrução
    pub fn raw_byte(&self, index: usize) -> u8 {
        if index < self.len {
            self.raw[index]
        } else {
            0
        }
    }

    /// Retorna o número de operandos baseado no formato e nos bytes
    /// Para FormatD: 1 operand se byte 1 parece endereço (> 0x0F), 2 se parece registrador
    pub fn operand_count(&self) -> usize {
        match self.opcode.format() {
            InstructionFormat::FormatA => 0,
            InstructionFormat::FormatB => 1,
            InstructionFormat::FormatC => {
                // FormatC: pode ter 1 ou 2 operandos
                // Heurística: se byte 1 é < 16, provavelmente é registrador
                if self.raw[1] < 16 { 2 } else { 1 }
            }
            InstructionFormat::FormatD => {
                // FormatD: pode ter 0, 1, 2 ou 3 operandos
                // Para JZ/JN: se byte 1 é < 16, é registrador (2 operandos)
                // senão é endereço direto (1 operando)
                if self.raw[1] < 16 { 2 } else { 1 }
            }
        }
    }
    
    /// Retorna o formato da instrução
    pub fn format(&self) -> InstructionFormat {
        self.opcode.format()
    }
    
    /// Registrador único (para Formato B)
    pub fn reg(&self) -> usize {
        self.reg_a()
    }
    
    /// Endereço de 24 bits (para Formato D)
    pub fn addr24(&self) -> u32 {
        self.addr_or_imm24()
    }
    
    /// Disassembly
    pub fn disassemble(&self) -> String {
        use Opcode::*;
        
        match self.opcode {
            // Formato A
            Nop | Hlt | Ret | Yield | Shiftl | Rotatl | Fold |
            HintCpu | HintGpu | HintNpu | HintAny | HintFpga | HintDsp |
            Unbatch | Fence => {
                self.opcode.mnemonic().to_string()
            }
            
            // Formato B
            Push | Pop | Inv | Conj | Mag | Phase | Notl |
            Spread | Gather | Grad | Emerge | Collapse |
            Setmode | Sense | Act | Sync |
            // Quantum / BitDeSil
            BitHadamard | BitPauliX | BitPauliY | BitPauliZ |
            BitCollapse | BitMeasure | BitRotateQ | BitNormalize => {
                format!("{} R{:X}", self.opcode.mnemonic(), self.reg_a())
            }
            
            // Formato C (2 regs)
            Mov | Xchg | Mul | Div | Add | Sub | Xorl | Andl | Orl | Entangle => {
                format!("{} R{:X}, R{:X}", self.opcode.mnemonic(), self.reg_a(), self.reg_b())
            }
            
            // Formato C (reg + imm)
            Movi | Pow | Root | Scale | Rotate | Lerp | Slerp | Descent |
            Promote | Demote | Truncate | Xordem | Avgdem | Maxdem | Compat |
            In | Out => {
                format!("{} R{:X}, 0x{:02X}", self.opcode.mnemonic(), self.reg_a(), self.imm8())
            }
            
            // Formato D
            Jmp | Jz | Jn | Jc | Jo | Call | Loop |
            Load | Store | Lstate | Sstate |
            Trans | Pipe | Broadcast | Receive |
            Batch | Prefetch | Syscall => {
                format!("{} 0x{:06X}", self.opcode.mnemonic(), self.addr_or_imm24())
            }

            // Formato C extended (opcodes Int/Float mode-aware)
            AddInt | SubInt | MulInt | DivInt |
            AddFloat | SubFloat | MulFloat | DivFloat |
            PowInt | PowFloat | SqrtFloat | ModInt |
            CmpInt | CmpFloat | TestInt |
            AndInt | OrInt | XorInt | NotInt |
            ShlInt | ShrInt | NegInt | AbsInt |
            NegFloat | AbsFloat | FloorFloat | CeilFloat |
            CvtIntToFloat | CvtFloatToInt |
            CvtIntToByteSil | CvtByteSilToInt |
            CvtFloatToByteSil | CvtByteSilToFloat => {
                format!("{} R{:X}, R{:X}", self.opcode.mnemonic(), self.reg_a(), self.reg_b())
            }
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.disassemble())
    }
}

/// Builder para construir instruções
pub struct InstructionBuilder {
    instructions: Vec<u8>,
}

impl InstructionBuilder {
    /// Cria novo builder
    pub fn new() -> Self {
        Self { instructions: Vec::new() }
    }
    
    /// Adiciona NOP
    pub fn nop(&mut self) -> &mut Self {
        self.instructions.push(Opcode::Nop as u8);
        self
    }
    
    /// Adiciona HLT
    pub fn hlt(&mut self) -> &mut Self {
        self.instructions.push(Opcode::Hlt as u8);
        self
    }
    
    /// Adiciona MOV Ra, Rb
    pub fn mov(&mut self, ra: u8, rb: u8) -> &mut Self {
        self.instructions.push(Opcode::Mov as u8);
        self.instructions.push((ra & 0x0F) | ((rb & 0x0F) << 4));
        self.instructions.push(0);
        self
    }
    
    /// Adiciona MOVI Ra, imm
    pub fn movi(&mut self, ra: u8, imm: u8) -> &mut Self {
        self.instructions.push(Opcode::Movi as u8);
        self.instructions.push(ra & 0x0F);
        self.instructions.push(imm);
        self
    }
    
    /// Adiciona MUL Ra, Rb
    pub fn mul(&mut self, ra: u8, rb: u8) -> &mut Self {
        self.instructions.push(Opcode::Mul as u8);
        self.instructions.push((ra & 0x0F) | ((rb & 0x0F) << 4));
        self.instructions.push(0);
        self
    }
    
    /// Adiciona ADD Ra, Rb
    pub fn add(&mut self, ra: u8, rb: u8) -> &mut Self {
        self.instructions.push(Opcode::Add as u8);
        self.instructions.push((ra & 0x0F) | ((rb & 0x0F) << 4));
        self.instructions.push(0);
        self
    }
    
    /// Adiciona JMP addr
    pub fn jmp(&mut self, addr: u32) -> &mut Self {
        self.instructions.push(Opcode::Jmp as u8);
        self.instructions.push((addr & 0xFF) as u8);
        self.instructions.push(((addr >> 8) & 0xFF) as u8);
        self.instructions.push(((addr >> 16) & 0xFF) as u8);
        self
    }
    
    /// Adiciona JZ addr
    pub fn jz(&mut self, addr: u32) -> &mut Self {
        self.instructions.push(Opcode::Jz as u8);
        self.instructions.push((addr & 0xFF) as u8);
        self.instructions.push(((addr >> 8) & 0xFF) as u8);
        self.instructions.push(((addr >> 16) & 0xFF) as u8);
        self
    }
    
    /// Adiciona CALL addr
    pub fn call(&mut self, addr: u32) -> &mut Self {
        self.instructions.push(Opcode::Call as u8);
        self.instructions.push((addr & 0xFF) as u8);
        self.instructions.push(((addr >> 8) & 0xFF) as u8);
        self.instructions.push(((addr >> 16) & 0xFF) as u8);
        self
    }
    
    /// Adiciona RET
    pub fn ret(&mut self) -> &mut Self {
        self.instructions.push(Opcode::Ret as u8);
        self
    }
    
    /// Adiciona COLLAPSE Ra
    pub fn collapse(&mut self, ra: u8) -> &mut Self {
        self.instructions.push(Opcode::Collapse as u8);
        self.instructions.push(ra & 0x0F);
        self
    }
    
    /// Adiciona LSTATE addr
    pub fn lstate(&mut self, addr: u32) -> &mut Self {
        self.instructions.push(Opcode::Lstate as u8);
        self.instructions.push((addr & 0xFF) as u8);
        self.instructions.push(((addr >> 8) & 0xFF) as u8);
        self.instructions.push(((addr >> 16) & 0xFF) as u8);
        self
    }
    
    /// Adiciona instrução raw
    pub fn raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.instructions.extend_from_slice(bytes);
        self
    }
    
    /// Retorna posição atual (para labels)
    pub fn pos(&self) -> u32 {
        self.instructions.len() as u32
    }
    
    /// Constrói bytecode
    pub fn build(self) -> Vec<u8> {
        self.instructions
    }
}

impl Default for InstructionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_nop() {
        let bytes = [0x00];
        let instr = Instruction::decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::Nop);
        assert_eq!(instr.size(), 1);
    }
    
    #[test]
    fn test_decode_movi() {
        let bytes = [0x21, 0x05, 0xAB];
        let instr = Instruction::decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::Movi);
        assert_eq!(instr.reg_a(), 5);
        assert_eq!(instr.imm8(), 0xAB);
    }
    
    #[test]
    fn test_decode_jmp() {
        let bytes = [0x10, 0x34, 0x12, 0x00];
        let instr = Instruction::decode(&bytes).unwrap();
        assert_eq!(instr.opcode, Opcode::Jmp);
        assert_eq!(instr.addr_or_imm24(), 0x001234);
    }
    
    #[test]
    fn test_builder() {
        let mut builder = InstructionBuilder::new();
        builder.movi(0, 0x55)
            .movi(1, 0xAA)
            .mul(0, 1)
            .hlt();
        let code = builder.build();
        
        assert_eq!(code.len(), 10); // 3 + 3 + 3 + 1
    }
    
    #[test]
    fn test_disassemble() {
        let instr = Instruction::format_c(Opcode::Movi, 5, 0, 0x42);
        assert_eq!(instr.disassemble(), "MOVI R5, 0x42");
    }
}
