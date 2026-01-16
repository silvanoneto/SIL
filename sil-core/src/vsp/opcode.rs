//! Opcodes do VSP
//!
//! Definição de todos os opcodes da ISA do Virtual Sil Processor.

/// Categorias de opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpcodeCategory {
    /// Controle de fluxo (0x00-0x1F)
    Control = 0x00,
    /// Movimento de dados (0x20-0x3F)
    Data = 0x20,
    /// Aritmética ByteSil (0x40-0x5F)
    Arithmetic = 0x40,
    /// Operações de camada (0x60-0x7F)
    Layer = 0x60,
    /// Transformações (0x80-0x9F)
    Transform = 0x80,
    /// Compatibilidade (0xA0-0xBF)
    Compat = 0xA0,
    /// I/O e Sistema (0xC0-0xDF)
    System = 0xC0,
    /// Hardware Hints (0xE0-0xFF)
    Hint = 0xE0,
}

impl OpcodeCategory {
    /// Obtém categoria de um opcode
    pub fn from_opcode(opcode: u8) -> Self {
        match opcode & 0xE0 {
            0x00 => Self::Control,
            0x20 => Self::Data,
            0x40 => Self::Arithmetic,
            0x60 => Self::Layer,
            0x80 => Self::Transform,
            0xA0 => Self::Compat,
            0xC0 => Self::System,
            0xE0 => Self::Hint,
            _ => Self::Control, // fallback
        }
    }
}

/// Opcodes do VSP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // ═══════════════════════════════════════════════════════════════
    // CONTROLE DE FLUXO (0x00-0x1F)
    // ═══════════════════════════════════════════════════════════════
    
    /// No operation
    Nop = 0x00,
    /// Halt execution
    Hlt = 0x01,
    /// Return from call
    Ret = 0x02,
    /// Yield to scheduler
    Yield = 0x03,
    
    /// Jump incondicional
    Jmp = 0x10,
    /// Jump if zero
    Jz = 0x11,
    /// Jump if negative
    Jn = 0x12,
    /// Jump if collapse
    Jc = 0x13,
    /// Jump if overflow
    Jo = 0x14,
    /// Call subroutine
    Call = 0x15,
    /// Loop (decrement and jump if not zero)
    Loop = 0x16,
    
    // ═══════════════════════════════════════════════════════════════
    // MOVIMENTO DE DADOS (0x20-0x3F)
    // ═══════════════════════════════════════════════════════════════
    
    /// Move register to register
    Mov = 0x20,
    /// Move immediate to register
    Movi = 0x21,
    /// Load from memory
    Load = 0x22,
    /// Store to memory
    Store = 0x23,
    /// Push to stack
    Push = 0x24,
    /// Pop from stack
    Pop = 0x25,
    /// Exchange registers
    Xchg = 0x26,
    /// Load full state (128 bits)
    Lstate = 0x27,
    /// Store full state (128 bits)
    Sstate = 0x28,
    
    // ═══════════════════════════════════════════════════════════════
    // ARITMÉTICA BYTESIL (0x40-0x5F)
    // ═══════════════════════════════════════════════════════════════
    
    /// Multiplicação (ρ+, θ+)
    Mul = 0x40,
    /// Divisão (ρ-, θ-)
    Div = 0x41,
    /// Potência (ρ×n, θ×n)
    Pow = 0x42,
    /// Raiz (ρ÷n, θ÷n)
    Root = 0x43,
    /// Inverso (-ρ, -θ)
    Inv = 0x44,
    /// Conjugado (-θ)
    Conj = 0x45,
    /// Adição (cartesiana)
    Add = 0x46,
    /// Subtração (cartesiana)
    Sub = 0x47,
    /// Extrai magnitude (θ=0)
    Mag = 0x48,
    /// Extrai fase (ρ=0)
    Phase = 0x49,
    /// Escala magnitude
    Scale = 0x4A,
    /// Rotaciona fase
    Rotate = 0x4B,
    
    // ═══════════════════════════════════════════════════════════════
    // OPERAÇÕES DE CAMADA (0x60-0x7F)
    // ═══════════════════════════════════════════════════════════════
    
    /// XOR de camadas
    Xorl = 0x60,
    /// AND de camadas
    Andl = 0x61,
    /// OR de camadas
    Orl = 0x62,
    /// NOT de camada
    Notl = 0x63,
    /// Shift de camadas
    Shiftl = 0x64,
    /// Rotate circular de camadas
    Rotatl = 0x65,
    /// Fold (R[i] ⊕ R[i+8])
    Fold = 0x66,
    /// Spread para grupo
    Spread = 0x67,
    /// Gather de grupo
    Gather = 0x68,
    
    // ═══════════════════════════════════════════════════════════════
    // TRANSFORMAÇÕES (0x80-0x9F)
    // ═══════════════════════════════════════════════════════════════
    
    /// Aplica transformação
    Trans = 0x80,
    /// Pipeline de transformações
    Pipe = 0x81,
    /// Interpolação linear
    Lerp = 0x82,
    /// Interpolação esférica
    Slerp = 0x83,
    /// Calcula gradiente
    Grad = 0x84,
    /// Gradient descent
    Descent = 0x85,
    /// Emergência (NPU)
    Emerge = 0x86,
    /// Colapso
    Collapse = 0x87,
    
    // ═══════════════════════════════════════════════════════════════
    // COMPATIBILIDADE (0xA0-0xAF)
    // ═══════════════════════════════════════════════════════════════

    /// Define modo SIL
    Setmode = 0xA0,
    /// Promove para modo maior
    Promote = 0xA1,
    /// Demove para modo menor
    Demote = 0xA2,
    /// Demote por truncamento
    Truncate = 0xA3,
    /// Demote por XOR
    Xordem = 0xA4,
    /// Demote por média
    Avgdem = 0xA5,
    /// Demote por máximo
    Maxdem = 0xA6,
    /// Negocia compatibilidade
    Compat = 0xA7,

    // ═══════════════════════════════════════════════════════════════
    // QUANTUM / BITDESIL (0xB0-0xBF)
    // ═══════════════════════════════════════════════════════════════

    /// Porta Hadamard (superposicao)
    BitHadamard = 0xB0,
    /// Porta Pauli-X (NOT quantico)
    BitPauliX = 0xB1,
    /// Porta Pauli-Y (rotacao + flip)
    BitPauliY = 0xB2,
    /// Porta Pauli-Z (flip de fase)
    BitPauliZ = 0xB3,
    /// Colapso quantico (medicao)
    BitCollapse = 0xB4,
    /// Medir probabilidade sem colapso
    BitMeasure = 0xB5,
    /// Rotacao por n quanta de fase
    BitRotateQ = 0xB6,
    /// Normalizar amplitudes quanticas
    BitNormalize = 0xB7,
    
    // ═══════════════════════════════════════════════════════════════
    // I/O E SISTEMA (0xC0-0xDF)
    // ═══════════════════════════════════════════════════════════════
    
    /// Input de porta
    In = 0xC0,
    /// Output para porta
    Out = 0xC1,
    /// Lê sensor
    Sense = 0xC2,
    /// Escreve atuador
    Act = 0xC3,
    /// Sincroniza com nó
    Sync = 0xC4,
    /// Broadcast estado
    Broadcast = 0xC5,
    /// Recebe estado
    Receive = 0xC6,
    /// Entangle com nó remoto
    Entangle = 0xC7,
    
    // ═══════════════════════════════════════════════════════════════
    // HARDWARE HINTS (0xE0-0xFF)
    // ═══════════════════════════════════════════════════════════════
    
    /// Preferir CPU
    HintCpu = 0xE0,
    /// Preferir GPU
    HintGpu = 0xE1,
    /// Preferir NPU
    HintNpu = 0xE2,
    /// Qualquer backend
    HintAny = 0xE3,
    /// Início de batch
    Batch = 0xE4,
    /// Fim de batch
    Unbatch = 0xE5,
    /// Prefetch
    Prefetch = 0xE6,
    /// Memory fence
    Fence = 0xE7,
    /// Preferir FPGA
    HintFpga = 0xE8,
    /// Preferir DSP
    HintDsp = 0xE9,
    /// Syscall
    Syscall = 0xFF,
}

impl Opcode {
    /// Decodifica opcode de byte
    pub fn from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            // Controle
            0x00 => Self::Nop,
            0x01 => Self::Hlt,
            0x02 => Self::Ret,
            0x03 => Self::Yield,
            0x10 => Self::Jmp,
            0x11 => Self::Jz,
            0x12 => Self::Jn,
            0x13 => Self::Jc,
            0x14 => Self::Jo,
            0x15 => Self::Call,
            0x16 => Self::Loop,
            
            // Dados
            0x20 => Self::Mov,
            0x21 => Self::Movi,
            0x22 => Self::Load,
            0x23 => Self::Store,
            0x24 => Self::Push,
            0x25 => Self::Pop,
            0x26 => Self::Xchg,
            0x27 => Self::Lstate,
            0x28 => Self::Sstate,
            
            // Aritmética
            0x40 => Self::Mul,
            0x41 => Self::Div,
            0x42 => Self::Pow,
            0x43 => Self::Root,
            0x44 => Self::Inv,
            0x45 => Self::Conj,
            0x46 => Self::Add,
            0x47 => Self::Sub,
            0x48 => Self::Mag,
            0x49 => Self::Phase,
            0x4A => Self::Scale,
            0x4B => Self::Rotate,
            
            // Camada
            0x60 => Self::Xorl,
            0x61 => Self::Andl,
            0x62 => Self::Orl,
            0x63 => Self::Notl,
            0x64 => Self::Shiftl,
            0x65 => Self::Rotatl,
            0x66 => Self::Fold,
            0x67 => Self::Spread,
            0x68 => Self::Gather,
            
            // Transformação
            0x80 => Self::Trans,
            0x81 => Self::Pipe,
            0x82 => Self::Lerp,
            0x83 => Self::Slerp,
            0x84 => Self::Grad,
            0x85 => Self::Descent,
            0x86 => Self::Emerge,
            0x87 => Self::Collapse,
            
            // Compatibilidade
            0xA0 => Self::Setmode,
            0xA1 => Self::Promote,
            0xA2 => Self::Demote,
            0xA3 => Self::Truncate,
            0xA4 => Self::Xordem,
            0xA5 => Self::Avgdem,
            0xA6 => Self::Maxdem,
            0xA7 => Self::Compat,

            // Quantum / BitDeSil
            0xB0 => Self::BitHadamard,
            0xB1 => Self::BitPauliX,
            0xB2 => Self::BitPauliY,
            0xB3 => Self::BitPauliZ,
            0xB4 => Self::BitCollapse,
            0xB5 => Self::BitMeasure,
            0xB6 => Self::BitRotateQ,
            0xB7 => Self::BitNormalize,

            // Sistema
            0xC0 => Self::In,
            0xC1 => Self::Out,
            0xC2 => Self::Sense,
            0xC3 => Self::Act,
            0xC4 => Self::Sync,
            0xC5 => Self::Broadcast,
            0xC6 => Self::Receive,
            0xC7 => Self::Entangle,
            
            // Hints
            0xE0 => Self::HintCpu,
            0xE1 => Self::HintGpu,
            0xE2 => Self::HintNpu,
            0xE3 => Self::HintAny,
            0xE4 => Self::Batch,
            0xE5 => Self::Unbatch,
            0xE6 => Self::Prefetch,
            0xE7 => Self::Fence,
            0xE8 => Self::HintFpga,
            0xE9 => Self::HintDsp,
            0xFF => Self::Syscall,

            _ => return None,
        })
    }
    
    /// Retorna categoria do opcode
    pub fn category(&self) -> OpcodeCategory {
        OpcodeCategory::from_opcode(*self as u8)
    }
    
    /// Retorna formato da instrução
    pub fn format(&self) -> super::InstructionFormat {
        use super::InstructionFormat::*;
        match self {
            // Formato A (8 bits)
            Self::Nop | Self::Hlt | Self::Ret | Self::Yield |
            Self::Shiftl | Self::Rotatl | Self::Fold |
            Self::HintCpu | Self::HintGpu | Self::HintNpu | Self::HintAny |
            Self::HintFpga | Self::HintDsp |
            Self::Unbatch | Self::Fence => FormatA,
            
            // Formato B (16 bits)
            Self::Push | Self::Pop | Self::Inv | Self::Conj |
            Self::Mag | Self::Phase | Self::Notl | Self::Spread | Self::Gather |
            Self::Grad | Self::Emerge | Self::Collapse |
            Self::Setmode | Self::Sense | Self::Act | Self::Sync |
            // Quantum / BitDeSil
            Self::BitHadamard | Self::BitPauliX | Self::BitPauliY | Self::BitPauliZ |
            Self::BitCollapse | Self::BitMeasure | Self::BitRotateQ | Self::BitNormalize => FormatB,
            
            // Formato C (24 bits)
            Self::Mov | Self::Movi | Self::Xchg |
            Self::Mul | Self::Div | Self::Pow | Self::Root |
            Self::Add | Self::Sub | Self::Scale | Self::Rotate |
            Self::Xorl | Self::Andl | Self::Orl |
            Self::Lerp | Self::Slerp | Self::Descent |
            Self::Promote | Self::Demote | Self::Truncate |
            Self::Xordem | Self::Avgdem | Self::Maxdem | Self::Compat |
            Self::In | Self::Out | Self::Entangle => FormatC,
            
            // Formato D (32 bits)
            Self::Jmp | Self::Jz | Self::Jn | Self::Jc | Self::Jo |
            Self::Call | Self::Loop |
            Self::Load | Self::Store | Self::Lstate | Self::Sstate |
            Self::Trans | Self::Pipe |
            Self::Broadcast | Self::Receive |
            Self::Batch | Self::Prefetch | Self::Syscall => FormatD,
        }
    }
    
    /// Retorna mnemônico
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Self::Nop => "NOP",
            Self::Hlt => "HLT",
            Self::Ret => "RET",
            Self::Yield => "YIELD",
            Self::Jmp => "JMP",
            Self::Jz => "JZ",
            Self::Jn => "JN",
            Self::Jc => "JC",
            Self::Jo => "JO",
            Self::Call => "CALL",
            Self::Loop => "LOOP",
            Self::Mov => "MOV",
            Self::Movi => "MOVI",
            Self::Load => "LOAD",
            Self::Store => "STORE",
            Self::Push => "PUSH",
            Self::Pop => "POP",
            Self::Xchg => "XCHG",
            Self::Lstate => "LSTATE",
            Self::Sstate => "SSTATE",
            Self::Mul => "MUL",
            Self::Div => "DIV",
            Self::Pow => "POW",
            Self::Root => "ROOT",
            Self::Inv => "INV",
            Self::Conj => "CONJ",
            Self::Add => "ADD",
            Self::Sub => "SUB",
            Self::Mag => "MAG",
            Self::Phase => "PHASE",
            Self::Scale => "SCALE",
            Self::Rotate => "ROTATE",
            Self::Xorl => "XORL",
            Self::Andl => "ANDL",
            Self::Orl => "ORL",
            Self::Notl => "NOTL",
            Self::Shiftl => "SHIFTL",
            Self::Rotatl => "ROTATL",
            Self::Fold => "FOLD",
            Self::Spread => "SPREAD",
            Self::Gather => "GATHER",
            Self::Trans => "TRANS",
            Self::Pipe => "PIPE",
            Self::Lerp => "LERP",
            Self::Slerp => "SLERP",
            Self::Grad => "GRAD",
            Self::Descent => "DESCENT",
            Self::Emerge => "EMERGE",
            Self::Collapse => "COLLAPSE",
            Self::Setmode => "SETMODE",
            Self::Promote => "PROMOTE",
            Self::Demote => "DEMOTE",
            Self::Truncate => "TRUNCATE",
            Self::Xordem => "XORDEM",
            Self::Avgdem => "AVGDEM",
            Self::Maxdem => "MAXDEM",
            Self::Compat => "COMPAT",
            // Quantum / BitDeSil
            Self::BitHadamard => "BIT.H",
            Self::BitPauliX => "BIT.X",
            Self::BitPauliY => "BIT.Y",
            Self::BitPauliZ => "BIT.Z",
            Self::BitCollapse => "BIT.COLLAPSE",
            Self::BitMeasure => "BIT.MEASURE",
            Self::BitRotateQ => "BIT.ROTQ",
            Self::BitNormalize => "BIT.NORM",
            Self::In => "IN",
            Self::Out => "OUT",
            Self::Sense => "SENSE",
            Self::Act => "ACT",
            Self::Sync => "SYNC",
            Self::Broadcast => "BROADCAST",
            Self::Receive => "RECEIVE",
            Self::Entangle => "ENTANGLE",
            Self::HintCpu => "HINT.CPU",
            Self::HintGpu => "HINT.GPU",
            Self::HintNpu => "HINT.NPU",
            Self::HintAny => "HINT.ANY",
            Self::HintFpga => "HINT.FPGA",
            Self::HintDsp => "HINT.DSP",
            Self::Batch => "BATCH",
            Self::Unbatch => "UNBATCH",
            Self::Prefetch => "PREFETCH",
            Self::Fence => "FENCE",
            Self::Syscall => "SYSCALL",
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opcode_roundtrip() {
        for byte in 0..=255u8 {
            if let Some(op) = Opcode::from_byte(byte) {
                assert_eq!(op as u8, byte);
            }
        }
    }
    
    #[test]
    fn test_categories() {
        assert_eq!(Opcode::Nop.category(), OpcodeCategory::Control);
        assert_eq!(Opcode::Mov.category(), OpcodeCategory::Data);
        assert_eq!(Opcode::Mul.category(), OpcodeCategory::Arithmetic);
        assert_eq!(Opcode::Xorl.category(), OpcodeCategory::Layer);
        assert_eq!(Opcode::Trans.category(), OpcodeCategory::Transform);
        assert_eq!(Opcode::Setmode.category(), OpcodeCategory::Compat);
        assert_eq!(Opcode::In.category(), OpcodeCategory::System);
        assert_eq!(Opcode::HintCpu.category(), OpcodeCategory::Hint);
    }
}
