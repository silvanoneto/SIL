//! # 🖥️ VSP — Virtual Sil Processor
//!
//! Máquina virtual para execução de bytecode SIL com abstração de hardware.
//!
//! > *"A JVM só que realmente aberta. O bytecode que é estado."*
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         VSP CORE                                │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────────┐   │
//! │  │  16×R   │ │  Stack  │ │  Heap   │ │   Instruction Unit   │   │
//! │  │ (regs)  │ │ (frames)│ │ (states)│ │   (decode+dispatch)  │   │
//! │  └─────────┘ └─────────┘ └─────────┘ └─────────────────────┘   │
//! │                            │                                    │
//! │                            ▼                                    │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
//! │  │   CPU   │ │   GPU   │ │   NPU   │ │  FPGA   │              │
//! │  └─────────┘ └─────────┘ └─────────┘ └─────────┘              │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Módulos
//!
//! - [`opcode`] - ISA com 70+ opcodes
//! - [`instruction`] - Decodificação e encoding
//! - [`memory`] - Segmentos de memória
//! - [`state`] - Registradores e flags
//! - [`backend`] - Abstração CPU/GPU/NPU
//! - [`bytecode`] - Formato .silc
//! - [`assembler`] - Compilador .sil → .silc
//! - [`repl`] - Console interativo
//! - [`debugger`] - Debugging visual (DAP)
//! - [`entanglement`] - Sincronização distribuída
//! - [`lsp`] - Language Server Protocol
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::vsp::{Vsp, VspConfig};
//!
//! // Criar VM
//! let mut vsp = Vsp::new(VspConfig::default())?;
//!
//! // Carregar programa
//! vsp.load_silc("program.silc")?;
//!
//! // Executar
//! let result = vsp.run()?;
//! ```

pub mod assembler;
pub mod backend;
pub mod bytecode;
pub mod debugger;
pub mod entanglement;
pub mod error;
pub mod instruction;
pub mod memory;
pub mod opcode;
pub mod repl;
pub mod state;
pub mod lsp;

#[cfg(feature = "jit")]
pub mod codegen;

#[cfg(feature = "jit")]
pub mod jit;

#[cfg(feature = "jit")]
pub mod aot;

#[cfg(feature = "dynasm")]
pub mod dynasm;

// Native Rust interpreter (universal fallback)
pub mod interpreter;

// Re-exports
pub use opcode::{Opcode, OpcodeCategory};
pub use instruction::{Instruction, InstructionFormat};
pub use memory::{VspMemory, MemorySegment};
pub use state::{VspState, StatusRegister, SilMode};
pub use backend::{VspBackend, BackendSelector};
pub use bytecode::{SilcFile, SilcHeader, SilcBuilder};
pub use error::{VspError, VspResult};
pub use assembler::{Assembler, assemble, disassemble, StdlibIntrinsic};
pub use repl::Repl;
pub use debugger::{Debugger, Breakpoint, DebugEvent, DebuggerState};
pub use entanglement::{EntanglementManager, NodeId, PairId, SyncMessage};
pub use lsp::{SilLanguageServer, LspConfig};

use crate::state::{ByteSil, SilState};

/// Configuração do VSP
#[derive(Debug, Clone)]
pub struct VspConfig {
    /// Tamanho máximo do heap (em estados)
    pub heap_size: usize,
    /// Tamanho máximo da stack (em frames)
    pub stack_size: usize,
    /// Modo SIL padrão
    pub default_mode: SilMode,
    /// Habilitar GPU
    pub enable_gpu: bool,
    /// Habilitar NPU
    pub enable_npu: bool,
    /// Modo de debug
    pub debug: bool,
}

impl Default for VspConfig {
    fn default() -> Self {
        Self {
            heap_size: 65536,      // 64K estados
            stack_size: 1024,      // 1K frames
            default_mode: SilMode::Sil128,
            enable_gpu: true,
            enable_npu: true,
            debug: false,
        }
    }
}

impl VspConfig {
    /// Define o modo SIL
    pub fn with_mode(mut self, mode: SilMode) -> Self {
        self.default_mode = mode;
        self
    }
}

/// Virtual Sil Processor
pub struct Vsp {
    /// Estado da VM
    state: VspState,
    /// Memória
    memory: VspMemory,
    /// Seletor de backends
    backends: BackendSelector,
    /// Configuração
    config: VspConfig,
    /// Contador de ciclos
    cycles: u64,
    /// Hint de backend atual
    backend_hint: Option<crate::processors::ProcessorType>,
}

impl Vsp {
    /// Cria nova instância do VSP
    pub fn new(config: VspConfig) -> VspResult<Self> {
        let state = VspState::new(config.default_mode);
        let memory = VspMemory::new(config.heap_size, config.stack_size)?;
        let backends = BackendSelector::new(config.enable_gpu, config.enable_npu)?;
        
        Ok(Self {
            state,
            memory,
            backends,
            config,
            cycles: 0,
            backend_hint: None,
        })
    }
    
    /// Carrega programa de bytes .silc
    pub fn load(&mut self, bytes: &[u8]) -> VspResult<()> {
        let silc = SilcFile::from_bytes(bytes)?;
        
        // Verifica compatibilidade de modo
        if silc.header.mode as u8 > self.state.mode as u8 {
            return Err(VspError::IncompatibleMode {
                expected: self.state.mode,
                found: silc.header.mode,
            });
        }
        
        // Carrega código
        self.memory.load_code(&silc.code)?;
        
        // Carrega dados iniciais
        self.memory.load_data(&silc.data)?;
        
        // Define entry point
        self.state.pc = silc.header.entry_point;
        
        Ok(())
    }
    
    /// Carrega programa .silc de arquivo
    pub fn load_silc(&mut self, path: &std::path::Path) -> VspResult<()> {
        let silc = SilcFile::load(path)?;
        
        // Verifica compatibilidade de modo
        if silc.header.mode as u8 > self.state.mode as u8 {
            return Err(VspError::IncompatibleMode {
                expected: self.state.mode,
                found: silc.header.mode,
            });
        }
        
        // Carrega código
        self.memory.load_code(&silc.code)?;
        
        // Carrega dados iniciais
        self.memory.load_data(&silc.data)?;
        
        // Define entry point
        self.state.pc = silc.header.entry_point;
        
        Ok(())
    }
    
    /// Carrega programa de bytes
    pub fn load_bytes(&mut self, code: &[u8], data: &[u8]) -> VspResult<()> {
        self.memory.load_code(code)?;
        self.memory.load_data(data)?;
        self.state.pc = 0;
        Ok(())
    }
    
    /// Executa até halt ou collapse
    pub fn run(&mut self) -> VspResult<SilState> {
        loop {
            // Fetch
            let raw = self.memory.fetch(self.state.pc)?;

            // Decode
            let instr = Instruction::decode(&raw)?;
            self.state.pc += instr.size() as u32;

            // Execute
            self.execute(&instr)?;
            self.cycles += 1;

            // Check termination
            if self.state.sr.halt || self.state.sr.collapse {
                break;
            }
        }

        Ok(self.to_sil_state())
    }
    
    /// Executa um único passo
    pub fn step(&mut self) -> VspResult<bool> {
        let raw = self.memory.fetch(self.state.pc)?;
        let instr = Instruction::decode(&raw)?;
        self.state.pc += instr.size() as u32;

        self.execute(&instr)?;
        self.cycles += 1;

        Ok(!self.state.sr.halt && !self.state.sr.collapse)
    }
    
    /// Executa instrução
    fn execute(&mut self, instr: &Instruction) -> VspResult<()> {
        use Opcode::*;
        
        match instr.opcode {
            // ─────────────────────────────────────────────────────────
            // Controle de Fluxo
            // ─────────────────────────────────────────────────────────
            Nop => {}
            
            Hlt => {
                self.state.sr.halt = true;
            }
            
            Ret => {
                let frame = self.memory.pop_frame()?;
                self.state.pc = frame.return_addr;
                self.state.fp = frame.prev_fp;
            }
            
            Yield => {
                // Yield para scheduler externo
                // Por enquanto, apenas incrementa ciclo
            }
            
            Jmp => {
                self.state.pc = instr.addr_or_imm24();
            }
            
            Jz => {
                // JZ can work in two modes:
                // 1. Single operand (label): uses sr.zero flag
                // 2. Two operands (Ra, label): tests if Ra.rho == 0
                let operands = instr.operand_count();
                let should_jump = if operands >= 2 {
                    // Two operand mode: test Ra.rho == 0
                    let ra = instr.reg_a();
                    self.state.regs[ra].rho == 0
                } else {
                    // Single operand mode: use status register
                    self.state.sr.zero
                };
                if should_jump {
                    // For 2-operand mode, address is in bytes 2-3 (16-bit)
                    // For 1-operand mode, address is in bytes 1-3 (24-bit)
                    let addr = if operands >= 2 {
                        (instr.raw_byte(2) as u32) | ((instr.raw_byte(3) as u32) << 8)
                    } else {
                        instr.addr_or_imm24()
                    };
                    self.state.pc = addr;
                }
            }

            Jn => {
                // JN can work in two modes:
                // 1. Single operand (label): uses sr.negative flag
                // 2. Two operands (Ra, label): tests if Ra.rho < 0
                let operands = instr.operand_count();
                let should_jump = if operands >= 2 {
                    let ra = instr.reg_a();
                    self.state.regs[ra].rho < 0
                } else {
                    self.state.sr.negative
                };
                if should_jump {
                    let addr = if operands >= 2 {
                        (instr.raw_byte(2) as u32) | ((instr.raw_byte(3) as u32) << 8)
                    } else {
                        instr.addr_or_imm24()
                    };
                    self.state.pc = addr;
                }
            }
            
            Jc => {
                if self.state.sr.collapse {
                    self.state.pc = instr.addr_or_imm24();
                }
            }
            
            Jo => {
                if self.state.sr.overflow {
                    self.state.pc = instr.addr_or_imm24();
                }
            }
            
            Call => {
                let addr = instr.addr_or_imm24();
                self.memory.push_frame(self.state.pc, self.state.fp)?;
                self.state.fp = self.state.sp;
                self.state.pc = addr;
            }
            
            Loop => {
                // Decrementa RC e salta se não zero
                let rc = &mut self.state.regs[0xC];
                if rc.rho > ByteSil::RHO_MIN {
                    rc.rho -= 1;
                    self.state.pc = instr.addr_or_imm24();
                }
            }
            
            // ─────────────────────────────────────────────────────────
            // Movimento de Dados
            // ─────────────────────────────────────────────────────────
            Mov => {
                let (ra, rb) = instr.reg_pair();
                self.state.regs[ra] = self.state.regs[rb];
            }
            
            Movi => {
                let ra = instr.reg_a();
                let imm = instr.imm8() as i8; // Signed immediate value
                // Store immediate as rho value with theta=0
                // This allows storing string indices and small integers directly
                self.state.regs[ra] = ByteSil::new(imm, 0);
            }
            
            Load => {
                let ra = instr.reg_a();
                let addr = instr.addr_or_imm24();
                self.state.regs[ra] = self.memory.load_byte_sil(addr)?;
            }
            
            Store => {
                let ra = instr.reg_a();
                let addr = instr.addr_or_imm24();
                self.memory.store_byte_sil(addr, self.state.regs[ra])?;
            }
            
            Push => {
                let ra = instr.reg_a();
                self.memory.push_state(self.state.regs[ra])?;
                self.state.sp += 1;
            }
            
            Pop => {
                let ra = instr.reg_a();
                self.state.regs[ra] = self.memory.pop_state()?;
                self.state.sp -= 1;
            }
            
            Xchg => {
                let (ra, rb) = instr.reg_pair();
                let temp = self.state.regs[ra];
                self.state.regs[ra] = self.state.regs[rb];
                self.state.regs[rb] = temp;
            }
            
            Lstate => {
                let addr = instr.addr_or_imm24();
                let state = self.memory.load_sil_state(addr)?;
                self.state.regs = state.layers;
            }
            
            Sstate => {
                let addr = instr.addr_or_imm24();
                let state = SilState { layers: self.state.regs };
                self.memory.store_sil_state(addr, &state)?;
            }
            
            // ─────────────────────────────────────────────────────────
            // Operações Aritméticas ByteSil
            // ─────────────────────────────────────────────────────────
            Mul => {
                let (ra, rb) = instr.reg_pair();
                self.state.regs[ra] = self.state.regs[ra].mul(&self.state.regs[rb]);
                self.update_flags(ra);
            }
            
            Div => {
                let (ra, rb) = instr.reg_pair();
                self.state.regs[ra] = self.state.regs[ra].div(&self.state.regs[rb]);
                self.update_flags(ra);
            }
            
            Pow => {
                let ra = instr.reg_a();
                let n = instr.imm8() as i32;
                self.state.regs[ra] = self.state.regs[ra].pow(n);
                self.update_flags(ra);
            }
            
            Root => {
                let ra = instr.reg_a();
                let n = instr.imm8() as i32;
                self.state.regs[ra] = self.state.regs[ra].root(n);
                self.update_flags(ra);
            }
            
            Inv => {
                let ra = instr.reg_a();
                self.state.regs[ra] = self.state.regs[ra].inv();
                self.update_flags(ra);
            }
            
            Conj => {
                let ra = instr.reg_a();
                self.state.regs[ra] = self.state.regs[ra].conj();
            }
            
            Add => {
                let (ra, rb) = instr.reg_pair();
                // Soma em coordenadas cartesianas
                let za = self.state.regs[ra].to_complex();
                let zb = self.state.regs[rb].to_complex();
                self.state.regs[ra] = ByteSil::from_complex(za + zb);
                self.update_flags(ra);
            }
            
            Sub => {
                let (ra, rb) = instr.reg_pair();
                let za = self.state.regs[ra].to_complex();
                let zb = self.state.regs[rb].to_complex();
                self.state.regs[ra] = ByteSil::from_complex(za - zb);
                self.update_flags(ra);
            }
            
            Mag => {
                let ra = instr.reg_a();
                self.state.regs[ra].theta = 0;
            }
            
            Phase => {
                let ra = instr.reg_a();
                self.state.regs[ra].rho = 0;
            }
            
            Scale => {
                let ra = instr.reg_a();
                let delta = instr.imm8() as i8;
                let new_rho = (self.state.regs[ra].rho as i16 + delta as i16)
                    .clamp(ByteSil::RHO_MIN as i16, ByteSil::RHO_MAX as i16) as i8;
                self.state.regs[ra].rho = new_rho;
                self.update_flags(ra);
            }
            
            Rotate => {
                let ra = instr.reg_a();
                let delta = instr.imm8();
                self.state.regs[ra].theta = (self.state.regs[ra].theta + delta) % 16;
            }

            // ─────────────────────────────────────────────────────────
            // Operações de Inteiros (mode-aware)
            // ─────────────────────────────────────────────────────────
            AddInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.store_int_mode(ra, a.wrapping_add(b));
            }

            SubInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.store_int_mode(ra, a.wrapping_sub(b));
            }

            MulInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.store_int_mode(ra, a.wrapping_mul(b));
            }

            DivInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                if b != 0 {
                    self.store_int_mode(ra, a / b);
                } else {
                    self.state.sr.overflow = true;
                }
            }

            ModInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                if b != 0 {
                    self.store_int_mode(ra, a % b);
                } else {
                    self.state.sr.overflow = true;
                }
            }

            PowInt => {
                let (ra, rb) = instr.reg_pair();
                let base = self.load_int_mode(ra);
                let exp = self.load_int_mode(rb) as u32;
                self.store_int_mode(ra, base.pow(exp));
            }

            NegInt => {
                let ra = instr.reg_a();
                let a = self.load_int_mode(ra);
                self.store_int_mode(ra, -a);
            }

            AbsInt => {
                let ra = instr.reg_a();
                let a = self.load_int_mode(ra);
                self.store_int_mode(ra, a.abs());
            }

            // Bitwise de inteiros
            AndInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.store_int_mode(ra, a & b);
            }

            OrInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.store_int_mode(ra, a | b);
            }

            XorInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.store_int_mode(ra, a ^ b);
            }

            NotInt => {
                let ra = instr.reg_a();
                let a = self.load_int_mode(ra);
                self.store_int_mode(ra, !a);
            }

            ShlInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let shift = self.load_int_mode(rb) as u32;
                self.store_int_mode(ra, a << shift);
            }

            ShrInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let shift = self.load_int_mode(rb) as u32;
                self.store_int_mode(ra, a >> shift);
            }

            // Comparação de inteiros
            CmpInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                self.state.sr.zero = a == b;
                self.state.sr.negative = a < b;
            }

            TestInt => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_int_mode(ra);
                let b = self.load_int_mode(rb);
                let result = a & b;
                self.state.sr.zero = result == 0;
            }

            // ─────────────────────────────────────────────────────────
            // Operações de Float (mode-aware)
            // ─────────────────────────────────────────────────────────
            AddFloat => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_float_mode(ra);
                let b = self.load_float_mode(rb);
                self.store_float_mode(ra, a + b);
            }

            SubFloat => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_float_mode(ra);
                let b = self.load_float_mode(rb);
                self.store_float_mode(ra, a - b);
            }

            MulFloat => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_float_mode(ra);
                let b = self.load_float_mode(rb);
                self.store_float_mode(ra, a * b);
            }

            DivFloat => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_float_mode(ra);
                let b = self.load_float_mode(rb);
                self.store_float_mode(ra, a / b);
            }

            PowFloat => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_float_mode(ra);
                let b = self.load_float_mode(rb);
                self.store_float_mode(ra, a.powf(b));
            }

            SqrtFloat => {
                let ra = instr.reg_a();
                let a = self.load_float_mode(ra);
                self.store_float_mode(ra, a.sqrt());
            }

            NegFloat => {
                let ra = instr.reg_a();
                let a = self.load_float_mode(ra);
                self.store_float_mode(ra, -a);
            }

            AbsFloat => {
                let ra = instr.reg_a();
                let a = self.load_float_mode(ra);
                self.store_float_mode(ra, a.abs());
            }

            FloorFloat => {
                let ra = instr.reg_a();
                let a = self.load_float_mode(ra);
                self.store_float_mode(ra, a.floor());
            }

            CeilFloat => {
                let ra = instr.reg_a();
                let a = self.load_float_mode(ra);
                self.store_float_mode(ra, a.ceil());
            }

            CmpFloat => {
                let (ra, rb) = instr.reg_pair();
                let a = self.load_float_mode(ra);
                let b = self.load_float_mode(rb);
                self.state.sr.zero = (a - b).abs() < f64::EPSILON;
                self.state.sr.negative = a < b;
            }

            // ─────────────────────────────────────────────────────────
            // Conversões
            // ─────────────────────────────────────────────────────────
            CvtIntToFloat => {
                let ra = instr.reg_a();
                let i = self.load_int_mode(ra);
                self.store_float_mode(ra, i as f64);
            }

            CvtFloatToInt => {
                let ra = instr.reg_a();
                let f = self.load_float_mode(ra);
                self.store_int_mode(ra, f as i64);
            }

            CvtIntToByteSil => {
                let ra = instr.reg_a();
                let i = self.load_int_mode(ra);
                // Converte inteiro para ByteSil: usa log para rho
                let rho = if i == 0 {
                    ByteSil::RHO_MIN
                } else {
                    ((i.abs() as f64).ln().round() as i8).clamp(ByteSil::RHO_MIN, ByteSil::RHO_MAX)
                };
                let theta = if i < 0 { 8 } else { 0 }; // 180° se negativo
                self.state.regs[ra] = ByteSil::new(rho, theta);
            }

            CvtByteSilToInt => {
                let ra = instr.reg_a();
                let bs = self.state.regs[ra];
                let mag = (bs.rho as f64).exp();
                let sign = if bs.theta >= 8 { -1.0 } else { 1.0 };
                self.store_int_mode(ra, (mag * sign) as i64);
            }

            CvtFloatToByteSil => {
                let ra = instr.reg_a();
                let f = self.load_float_mode(ra);
                self.state.regs[ra] = ByteSil::from_complex(
                    num_complex::Complex64::new(f, 0.0)
                );
            }

            CvtByteSilToFloat => {
                let ra = instr.reg_a();
                let bs = self.state.regs[ra];
                let f = bs.to_complex().norm();
                self.store_float_mode(ra, f);
            }

            // ─────────────────────────────────────────────────────────
            // Operações de Camada
            // ─────────────────────────────────────────────────────────
            Xorl => {
                let (ra, rb) = instr.reg_pair();
                // XOR de bytes (como ANDL e ORL)
                let a = self.state.regs[ra].to_u8();
                let b = self.state.regs[rb].to_u8();
                self.state.regs[ra] = ByteSil::from_u8(a ^ b);
            }
            
            Andl => {
                let (ra, rb) = instr.reg_pair();
                let a = self.state.regs[ra].to_u8();
                let b = self.state.regs[rb].to_u8();
                self.state.regs[ra] = ByteSil::from_u8(a & b);
            }
            
            Orl => {
                let (ra, rb) = instr.reg_pair();
                let a = self.state.regs[ra].to_u8();
                let b = self.state.regs[rb].to_u8();
                self.state.regs[ra] = ByteSil::from_u8(a | b);
            }
            
            Notl => {
                let ra = instr.reg_a();
                let a = self.state.regs[ra].to_u8();
                self.state.regs[ra] = ByteSil::from_u8(!a);
            }
            
            Shiftl => {
                // Shift todas as camadas para cima
                let overflow = self.state.regs[15];
                for i in (1..16).rev() {
                    self.state.regs[i] = self.state.regs[i - 1];
                }
                self.state.regs[0] = ByteSil::NULL;
                if !overflow.is_null() {
                    self.state.sr.overflow = true;
                }
            }
            
            Rotatl => {
                // Rotate circular
                let temp = self.state.regs[15];
                for i in (1..16).rev() {
                    self.state.regs[i] = self.state.regs[i - 1];
                }
                self.state.regs[0] = temp;
            }
            
            Fold => {
                // Fold: R[i] ⊕ R[i+8]
                for i in 0..8 {
                    self.state.regs[i] = self.state.regs[i].xor(&self.state.regs[i + 8]);
                }
            }
            
            Spread => {
                // Spread: copia Ra para todas as camadas do grupo
                let ra = instr.reg_a();
                let group = ra / 4; // 0-3
                let base = group * 4;
                let value = self.state.regs[ra];
                for i in base..(base + 4).min(16) {
                    self.state.regs[i] = value;
                }
            }
            
            Gather => {
                // Gather: reduz grupo para Ra
                let ra = instr.reg_a();
                let group = ra / 4;
                let base = group * 4;
                let mut acc = ByteSil::ONE;
                for i in base..(base + 4).min(16) {
                    acc = acc.mul(&self.state.regs[i]);
                }
                self.state.regs[ra] = acc;
            }
            
            // ─────────────────────────────────────────────────────────
            // Transformações
            // ─────────────────────────────────────────────────────────
            Trans => {
                let addr = instr.addr_or_imm24();
                let transform_id = self.memory.load_u32(addr)?;
                self.apply_transform(transform_id)?;
            }
            
            Pipe => {
                let addr = instr.addr_or_imm24();
                let pipeline = self.memory.load_pipeline(addr)?;
                for transform_id in pipeline {
                    self.apply_transform(transform_id)?;
                }
            }
            
            Lerp => {
                let (ra, rb) = instr.reg_pair();
                let t = instr.imm8() as f32 / 255.0;
                let za = self.state.regs[ra].to_complex();
                let zb = self.state.regs[rb].to_complex();
                let result = za * (1.0 - t as f64) + zb * t as f64;
                self.state.regs[ra] = ByteSil::from_complex(result);
            }
            
            Slerp => {
                let (ra, rb) = instr.reg_pair();
                let t = instr.imm8() as f32 / 255.0;
                // Interpolação esférica simplificada
                let a = &self.state.regs[ra];
                let b = &self.state.regs[rb];
                let rho = ((1.0 - t) * a.rho as f32 + t * b.rho as f32).round() as i8;
                let theta = (((1.0 - t) * a.theta as f32 + t * b.theta as f32).round() as u8) % 16;
                self.state.regs[ra] = ByteSil::new(rho, theta);
            }
            
            Grad => {
                let _ra = instr.reg_a();
                // Delega para backend (GPU preferido)
                let backend = self.backends.select_for_gradient(self.backend_hint);
                backend.compute_gradient(&mut self.state)?;
            }
            
            Descent => {
                let ra = instr.reg_a();
                let lr = instr.imm8() as f32 / 255.0;
                if let Some(ref grad) = self.state.gradient {
                    let g = grad[ra];
                    self.state.regs[ra].rho = (self.state.regs[ra].rho as f32 - lr * g)
                        .clamp(ByteSil::RHO_MIN as f32, ByteSil::RHO_MAX as f32) as i8;
                }
            }
            
            Emerge => {
                let _ra = instr.reg_a();
                // Delega para NPU
                let backend = self.backends.select_for_inference(self.backend_hint);
                backend.emergence(&mut self.state)?;
            }
            
            Collapse => {
                let ra = instr.reg_a();
                // Colapso: RF = resultado, marca flag
                self.state.regs[0xF] = self.state.regs[ra];
                if self.state.regs[0xF].rho == ByteSil::RHO_MIN {
                    self.state.sr.collapse = true;
                }
            }
            
            // ─────────────────────────────────────────────────────────
            // Compatibilidade
            // ─────────────────────────────────────────────────────────
            Setmode => {
                let mode_bits = instr.imm8();
                self.state.mode = SilMode::from_bits(mode_bits)?;
            }
            
            Promote => {
                let target = SilMode::from_bits(instr.imm8())?;
                self.state.promote(target)?;
            }
            
            Demote => {
                let target = SilMode::from_bits(instr.imm8())?;
                self.state.demote(target, state::DemoteStrategy::Xor)?;
            }
            
            Truncate => {
                let target = SilMode::from_bits(instr.imm8())?;
                self.state.demote(target, state::DemoteStrategy::Truncate)?;
            }
            
            Xordem => {
                let target = SilMode::from_bits(instr.imm8())?;
                self.state.demote(target, state::DemoteStrategy::Xor)?;
            }
            
            Avgdem => {
                let target = SilMode::from_bits(instr.imm8())?;
                self.state.demote(target, state::DemoteStrategy::Average)?;
            }
            
            Maxdem => {
                let target = SilMode::from_bits(instr.imm8())?;
                self.state.demote(target, state::DemoteStrategy::Max)?;
            }
            
            Compat => {
                // Negocia compatibilidade com estado externo
                let addr = instr.addr_or_imm24();
                let external_mode = self.memory.load_u8(addr)?;
                let external = SilMode::from_bits(external_mode)?;
                self.state.mode = self.state.mode.negotiate(external);
            }
            
            // ─────────────────────────────────────────────────────────
            // I/O e Sistema
            // ─────────────────────────────────────────────────────────
            In => {
                let ra = instr.reg_a();
                let port = instr.imm8() as u32;
                self.state.regs[ra] = self.memory.io_read(port)?;
            }
            
            Out => {
                let ra = instr.reg_a();
                let port = instr.imm8() as u32;
                self.memory.io_write(port, self.state.regs[ra])?;
            }
            
            Sense => {
                let ra = instr.reg_a();
                // Mapeia registrador para sensor
                let (value, eof) = self.memory.sense(ra)?;
                self.state.regs[ra] = value;
                // Seta flag zero em EOF para permitir JZ detectar fim de input
                self.state.sr.zero = eof;
            }
            
            Act => {
                let ra = instr.reg_a();
                self.memory.actuate(ra, self.state.regs[ra])?;
            }
            
            Sync => {
                // Sincroniza com nó remoto
                let _node_id = instr.imm8();
                // TODO: implementar sincronização distribuída
            }
            
            Broadcast => {
                let addr = instr.addr_or_imm24();
                let state = SilState { layers: self.state.regs };
                self.memory.broadcast(addr, &state)?;
            }
            
            Receive => {
                let addr = instr.addr_or_imm24();
                if let Some(state) = self.memory.receive(addr)? {
                    self.state.regs = state.layers;
                }
            }
            
            Entangle => {
                let (ra, rb) = instr.reg_pair();
                // Entangle registrador com nó remoto
                self.memory.entangle(ra, rb as u32, self.state.regs[ra])?;
            }
            
            // ─────────────────────────────────────────────────────────
            // Hardware Hints
            // ─────────────────────────────────────────────────────────
            HintCpu => {
                self.backend_hint = Some(crate::processors::ProcessorType::Cpu);
            }
            
            HintGpu => {
                self.backend_hint = Some(crate::processors::ProcessorType::Gpu);
            }
            
            HintNpu => {
                self.backend_hint = Some(crate::processors::ProcessorType::Npu);
            }
            
            HintAny => {
                self.backend_hint = None;
            }

            HintFpga => {
                self.backend_hint = Some(crate::processors::ProcessorType::Fpga);
            }

            HintDsp => {
                // DSP não tem ProcessorType específico, usa CPU com SIMD
                self.backend_hint = Some(crate::processors::ProcessorType::Cpu);
            }

            Batch => {
                let count = instr.addr_or_imm24();
                self.backends.begin_batch(count as usize)?;
            }
            
            Unbatch => {
                self.backends.end_batch()?;
            }
            
            Prefetch => {
                let addr = instr.addr_or_imm24();
                self.memory.prefetch(addr)?;
            }
            
            Fence => {
                self.backends.fence()?;
            }
            
            Syscall => {
                let syscall_data = instr.addr_or_imm24();
                // byte 1 = intrinsic_id, bytes 2-3 = arg (e.g., string_id)
                let intrinsic_id = (syscall_data & 0xFF) as u8;
                let syscall_arg = ((syscall_data >> 8) & 0xFFFF) as u16;
                self.syscall(intrinsic_id, syscall_arg)?;
            }

            // ─────────────────────────────────────────────────────────
            // Quantum / BitDeSil
            // ─────────────────────────────────────────────────────────
            BitHadamard => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                self.state.regs[ra] = bit.hadamard().to_byte_sil();
                self.update_flags(ra);
            }

            BitPauliX => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                self.state.regs[ra] = bit.pauli_x().to_byte_sil();
                self.update_flags(ra);
            }

            BitPauliY => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                self.state.regs[ra] = bit.pauli_y().to_byte_sil();
                self.update_flags(ra);
            }

            BitPauliZ => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                self.state.regs[ra] = bit.pauli_z().to_byte_sil();
                self.update_flags(ra);
            }

            BitCollapse => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                let random = (self.state.regs[ra].theta as f32) / 16.0;
                let (measurement, collapsed) = bit.collapse(random);
                self.state.regs[ra] = collapsed.to_byte_sil();
                self.state.sr.zero = !measurement;
                self.update_flags(ra);
            }

            BitMeasure => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                let prob_zero = (bit.prob_zero() * 127.0) as i8;
                self.state.regs[0] = ByteSil { rho: prob_zero, theta: 0 };
            }

            BitRotateQ => {
                let ra = instr.reg_a();
                let n = self.state.regs[1].rho;
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                self.state.regs[ra] = bit.rotate(n).to_byte_sil();
                self.update_flags(ra);
            }

            BitNormalize => {
                let ra = instr.reg_a();
                let bit = crate::state::BitDeSil::from_byte_sil(&self.state.regs[ra]);
                self.state.regs[ra] = bit.normalize().to_byte_sil();
                self.update_flags(ra);
            }
        }

        Ok(())
    }
    
    /// Atualiza flags baseado no registrador
    fn update_flags(&mut self, ra: usize) {
        let r = &self.state.regs[ra];
        self.state.sr.zero = r.is_null();
        self.state.sr.negative = r.rho < 0;
        self.state.sr.overflow = r.rho == ByteSil::RHO_MAX || r.rho == ByteSil::RHO_MIN;
        
        // Collapse quando RF fica nulo
        if ra == 0xF && r.is_null() {
            self.state.sr.collapse = true;
        }
    }
    
    /// Aplica transformação registrada
    fn apply_transform(&mut self, _transform_id: u32) -> VspResult<()> {
        // TODO: lookup na tabela de transforms
        Ok(())
    }
    
    /// Executa syscall
    fn syscall(&mut self, id: u8, arg: u16) -> VspResult<()> {
        use StdlibIntrinsic::*;

        match id {
            // ────── I/O Functions ──────
            id if id == Println as u8 => {
                println!();
            }
            id if id == PrintString as u8 => {
                // Print string from global string table
                // arg contains the string index directly
                let string_idx = if arg > 0 {
                    arg as usize
                } else {
                    // Fallback to R0.rho for backwards compatibility
                    self.state.regs[0].rho.max(0) as usize
                };
                let table = interpreter::get_string_table();
                if let Some(s) = table.get(string_idx) {
                    println!("{}", s);
                } else {
                    println!("[string#{}]", string_idx);
                }
            }
            id if id == PrintInt as u8 => {
                // Print integer mode-aware (uses current SilMode)
                let value = self.load_int_mode(0);
                print!("{}", value);
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            id if id == PrintFloat as u8 => {
                // Print float mode-aware (uses current SilMode)
                let value = self.load_float_mode(0);
                print!("{:.6}", value);
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            id if id == PrintBool as u8 => {
                print!("{}", self.state.regs[0].rho != 0);
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            id if id == PrintBytesil as u8 => {
                let b = self.state.regs[0];
                println!("ByteSil(ρ={}, θ={})", b.rho, b.theta);
            }
            id if id == PrintState as u8 => {
                println!("State:");
                for (i, r) in self.state.regs.iter().enumerate() {
                    println!("  L{:X}: ρ={:+3}, θ={:2}", i, r.rho, r.theta);
                }
            }

            // ────── ByteSil Functions ──────
            id if id == BytesilNew as u8 => {
                // Create from R0.rho and R1.theta
                self.state.regs[0] = ByteSil::new(
                    self.state.regs[0].rho,
                    self.state.regs[1].theta,
                );
            }
            id if id == BytesilNull as u8 => {
                self.state.regs[0] = ByteSil::NULL;
            }
            id if id == BytesilOne as u8 => {
                self.state.regs[0] = ByteSil::ONE;
            }
            id if id == BytesilI as u8 => {
                self.state.regs[0] = ByteSil::I;
            }
            id if id == BytesilNegOne as u8 => {
                self.state.regs[0] = ByteSil::NEG_ONE;
            }
            id if id == BytesilNegI as u8 => {
                self.state.regs[0] = ByteSil::NEG_I;
            }
            id if id == BytesilMax as u8 => {
                self.state.regs[0] = ByteSil::MAX;
            }
            id if id == BytesilMul as u8 => {
                // R0 = R0 * R1
                let a = self.state.regs[0];
                let b = self.state.regs[1];
                self.state.regs[0] = ByteSil::new(
                    a.rho.saturating_add(b.rho),
                    a.theta.wrapping_add(b.theta),
                );
            }
            id if id == BytesilDiv as u8 => {
                let a = self.state.regs[0];
                let b = self.state.regs[1];
                self.state.regs[0] = ByteSil::new(
                    a.rho.saturating_sub(b.rho),
                    a.theta.wrapping_sub(b.theta),
                );
            }
            id if id == BytesilPow as u8 => {
                let a = self.state.regs[0];
                let n = self.state.regs[1].rho;
                self.state.regs[0] = ByteSil::new(
                    (a.rho as i16 * n as i16).clamp(-8, 7) as i8,
                    a.theta.wrapping_mul(n as u8),
                );
            }
            id if id == BytesilRoot as u8 => {
                let a = self.state.regs[0];
                let n = self.state.regs[1].rho.max(1);
                self.state.regs[0] = ByteSil::new(a.rho / n, a.theta / n as u8);
            }
            id if id == BytesilXor as u8 => {
                self.state.regs[0] = self.state.regs[0] ^ self.state.regs[1];
            }
            id if id == BytesilMix as u8 => {
                let a = self.state.regs[0];
                let b = self.state.regs[1];
                self.state.regs[0] = ByteSil::new(
                    ((a.rho as i16 + b.rho as i16) / 2) as i8,
                    ((a.theta as u16 + b.theta as u16) / 2) as u8,
                );
            }
            id if id == BytesilRho as u8 => {
                // Retorna rho como inteiro em R0.rho
                // R0.rho já contém o valor, não precisa fazer nada
            }
            id if id == BytesilTheta as u8 => {
                // Retorna theta como inteiro em R0.rho
                let theta = self.state.regs[0].theta;
                self.state.regs[0] = ByteSil::new(theta as i8, 0);
            }
            id if id == BytesilMagnitude as u8 => {
                // Retorna magnitude = e^rho como Float em R0
                // Para Float, armazenamos ln(mag) em rho, então o resultado é o próprio rho
                // Mas quando impresso com print_float, será convertido via to_complex().norm()
                let mag = (self.state.regs[0].rho as f64).exp();
                // Armazenar como ByteSil: rho = ln(mag) (que é o mesmo valor original)
                // Isso parece circular, mas print_float fará e^rho novamente
                // A solução correta é manter a magnitude como está (rho contém ln(mag))
                // NÃO modificamos R0 - a magnitude já está codificada em rho
            }
            id if id == BytesilPhaseDegrees as u8 => {
                // Retorna fase em graus: theta * (360/256) = theta * 1.40625
                let degrees = (self.state.regs[0].theta as f64) * 1.40625;
                // Armazenar como scaled int
                self.state.regs[0] = ByteSil::new((degrees as i8).clamp(-128, 127) as i8, 0);
            }
            id if id == BytesilPhaseRadians as u8 => {
                // Retorna fase em radianos: theta * (2*PI/256)
                let radians = (self.state.regs[0].theta as f64) * std::f64::consts::TAU / 256.0;
                self.state.regs[0] = ByteSil::new((radians * 10.0) as i8, 0);
            }
            id if id == BytesilIsNull as u8 => {
                let is_null = self.state.regs[0] == ByteSil::NULL;
                self.state.regs[0] = ByteSil::new(is_null as i8, 0);
            }
            id if id == BytesilNorm as u8 => {
                // Normaliza para magnitude 1 (rho=0)
                self.state.regs[0] = ByteSil::new(0, self.state.regs[0].theta);
            }

            // ────── State Functions ──────
            id if id == StateVacuum as u8 => {
                for r in &mut self.state.regs {
                    *r = ByteSil::NULL;
                }
            }
            id if id == StateNeutral as u8 => {
                for r in &mut self.state.regs {
                    *r = ByteSil::ONE;
                }
            }
            id if id == StateGetLayer as u8 => {
                let idx = (self.state.regs[1].rho as usize) & 0x0F;
                self.state.regs[0] = self.state.regs[idx];
            }
            id if id == StateSetLayer as u8 => {
                let idx = (self.state.regs[1].rho as usize) & 0x0F;
                self.state.regs[idx] = self.state.regs[2];
            }
            id if id == StateCollapse as u8 => {
                let mut result = self.state.regs[0];
                for i in 1..16 {
                    result = result ^ self.state.regs[i];
                }
                self.state.regs[0] = result;
            }
            id if id == StateCountActiveLayers as u8 => {
                let count = self.state.regs.iter()
                    .filter(|r| **r != ByteSil::NULL)
                    .count();
                self.state.regs[0] = ByteSil::new(count as i8, 0);
            }

            // ────── Math Functions ──────
            id if id == MathSin as u8 => {
                let angle = self.state.regs[0].theta as f64 * std::f64::consts::PI / 8.0;
                let result = angle.sin();
                self.state.regs[0] = ByteSil::new((result * 127.0) as i8, 0);
            }
            id if id == MathCos as u8 => {
                let angle = self.state.regs[0].theta as f64 * std::f64::consts::PI / 8.0;
                let result = angle.cos();
                self.state.regs[0] = ByteSil::new((result * 127.0) as i8, 0);
            }
            id if id == MathSqrt as u8 => {
                let val = (self.state.regs[0].rho.abs() as f64).sqrt();
                self.state.regs[0] = ByteSil::new(val as i8, 0);
            }
            id if id == MathAbs as u8 => {
                self.state.regs[0].rho = self.state.regs[0].rho.abs();
            }

            // ────── Transform Functions ──────
            id if id == ApplyFeedback as u8 => {
                self.state.regs[0] = self.state.regs[0] ^ self.state.regs[15];
            }
            id if id == DetectEmergence as u8 => {
                let pattern = self.state.regs[0];
                let found = self.state.regs[1..].iter().any(|r| *r == pattern);
                self.state.regs[0] = ByteSil::new(found as i8, 0);
            }

            // ────── Debug Functions ──────
            id if id == DebugPrint as u8 => {
                eprintln!("[DEBUG] R0: ρ={}, θ={}", self.state.regs[0].rho, self.state.regs[0].theta);
            }
            id if id == AssertEq as u8 => {
                if self.state.regs[0] != self.state.regs[1] {
                    return Err(VspError::Other(format!(
                        "Assertion failed: {:?} != {:?}",
                        self.state.regs[0], self.state.regs[1]
                    )));
                }
            }
            id if id == AssertTrue as u8 => {
                if self.state.regs[0].rho == 0 {
                    return Err(VspError::Other("Assertion failed: expected true".to_string()));
                }
            }

            // ────── Energy Functions ──────
            0xE0 => { // EnergyBegin
                // No-op: energia é medida externamente pelo runtime
            }
            0xE1 => { // EnergyEndJoules
                // Retorna 0 joules (placeholder)
                self.state.regs[0] = ByteSil::new(0, 0);
            }
            0xE2 => { // EnergyEndWatts
                self.state.regs[0] = ByteSil::new(0, 0);
            }
            0xE3 => { // EnergyEndSamplesPerJoule
                self.state.regs[0] = ByteSil::new(0, 0);
            }

            // ────── Time Functions ──────
            0xF6 => { // TimestampMicros
                use std::time::{SystemTime, UNIX_EPOCH};
                let micros = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_micros() as i64)
                    .unwrap_or(0);
                // Store low 8 bits in R0.rho (limited precision for ByteSil)
                self.state.regs[0] = ByteSil::new((micros & 0x7F) as i8, 0);
            }

            // ────── Legacy Syscalls ──────
            0 => self.state.sr.halt = true, // exit

            _ => {} // Unknown syscall - no-op
        }
        Ok(())
    }
    
    /// Converte estado interno para SilState
    pub fn to_sil_state(&self) -> SilState {
        SilState { layers: self.state.regs }
    }
    
    /// Carrega estado externo
    pub fn from_sil_state(&mut self, state: &SilState) {
        self.state.regs = state.layers;
    }
    
    /// Retorna contagem de ciclos
    pub fn cycles(&self) -> u64 {
        self.cycles
    }
    
    /// Retorna referência ao estado
    pub fn state(&self) -> &VspState {
        &self.state
    }
    
    /// Retorna referência mutável ao estado
    pub fn state_mut(&mut self) -> &mut VspState {
        &mut self.state
    }
    
    /// Retorna referência à memória
    pub fn memory(&self) -> &VspMemory {
        &self.memory
    }
    
    /// Retorna referência mutável à memória
    pub fn memory_mut(&mut self) -> &mut VspMemory {
        &mut self.memory
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Helpers para Int/Float mode-aware
    // ═══════════════════════════════════════════════════════════════════════

    /// Carrega Int16 de um registrador (rho + theta = 16 bits)
    fn load_int16(&self, reg: usize) -> i16 {
        let lo = self.state.regs[reg].rho as u8 as u16;
        let hi = (self.state.regs[reg].theta as u16) << 8;
        (lo | hi) as i16
    }

    /// Armazena Int16 em um registrador
    fn store_int16(&mut self, reg: usize, val: i16) {
        let v = val as u16;
        self.state.regs[reg] = ByteSil::new(
            (v & 0xFF) as i8,
            ((v >> 8) & 0xFF) as u8,
        );
    }

    /// Carrega Int32 de 2 registradores consecutivos
    fn load_int32(&self, reg: usize) -> i32 {
        let lo = self.load_int16(reg) as u16 as u32;
        let hi = (self.load_int16(reg + 1) as u16 as u32) << 16;
        (lo | hi) as i32
    }

    /// Armazena Int32 em 2 registradores consecutivos
    fn store_int32(&mut self, reg: usize, val: i32) {
        self.store_int16(reg, val as i16);
        self.store_int16(reg + 1, (val >> 16) as i16);
    }

    /// Carrega Float32 de 2 registradores consecutivos
    fn load_float32(&self, reg: usize) -> f32 {
        f32::from_bits(self.load_int32(reg) as u32)
    }

    /// Armazena Float32 em 2 registradores consecutivos
    fn store_float32(&mut self, reg: usize, val: f32) {
        self.store_int32(reg, val.to_bits() as i32);
    }

    /// Carrega Int64 de 4 registradores consecutivos
    fn load_int64(&self, reg: usize) -> i64 {
        let lo = self.load_int32(reg) as u32 as u64;
        let hi = (self.load_int32(reg + 2) as u32 as u64) << 32;
        (lo | hi) as i64
    }

    /// Armazena Int64 em 4 registradores consecutivos
    fn store_int64(&mut self, reg: usize, val: i64) {
        self.store_int32(reg, val as i32);
        self.store_int32(reg + 2, (val >> 32) as i32);
    }

    /// Carrega Float64 de 4 registradores consecutivos
    fn load_float64(&self, reg: usize) -> f64 {
        f64::from_bits(self.load_int64(reg) as u64)
    }

    /// Armazena Float64 em 4 registradores consecutivos
    fn store_float64(&mut self, reg: usize, val: f64) {
        self.store_int64(reg, val.to_bits() as i64);
    }

    /// Carrega inteiro mode-aware (baseado no SilMode atual)
    fn load_int_mode(&self, reg: usize) -> i64 {
        match self.state.mode {
            SilMode::Sil8 => self.state.regs[reg].rho as i64,
            SilMode::Sil16 => self.load_int16(reg) as i64,
            SilMode::Sil32 => self.load_int32(reg) as i64,
            SilMode::Sil64 | SilMode::Sil128 => self.load_int64(reg),
        }
    }

    /// Armazena inteiro mode-aware
    fn store_int_mode(&mut self, reg: usize, val: i64) {
        match self.state.mode {
            SilMode::Sil8 => {
                self.state.regs[reg] = ByteSil::new(val as i8, 0);
            }
            SilMode::Sil16 => self.store_int16(reg, val as i16),
            SilMode::Sil32 => self.store_int32(reg, val as i32),
            SilMode::Sil64 | SilMode::Sil128 => self.store_int64(reg, val),
        }
    }

    /// Carrega float mode-aware
    fn load_float_mode(&self, reg: usize) -> f64 {
        match self.state.mode {
            SilMode::Sil8 => self.state.regs[reg].to_complex().norm(),
            SilMode::Sil16 => {
                // Float16 (half precision)
                let bits = self.load_int16(reg) as u16;
                half::f16::from_bits(bits).to_f64()
            }
            SilMode::Sil32 => self.load_float32(reg) as f64,
            SilMode::Sil64 | SilMode::Sil128 => self.load_float64(reg),
        }
    }

    /// Armazena float mode-aware
    fn store_float_mode(&mut self, reg: usize, val: f64) {
        match self.state.mode {
            SilMode::Sil8 => {
                self.state.regs[reg] = ByteSil::from_complex(
                    num_complex::Complex64::new(val, 0.0)
                );
            }
            SilMode::Sil16 => {
                let f16_val = half::f16::from_f64(val);
                self.store_int16(reg, f16_val.to_bits() as i16);
            }
            SilMode::Sil32 => self.store_float32(reg, val as f32),
            SilMode::Sil64 | SilMode::Sil128 => self.store_float64(reg, val),
        }
    }

    /// Retorna o processador atual (hint)
    pub fn current_processor(&self) -> &str {
        match self.backend_hint {
            Some(crate::processors::ProcessorType::Cpu) => "CPU",
            Some(crate::processors::ProcessorType::Gpu) => "GPU",
            Some(crate::processors::ProcessorType::Npu) => "NPU",
            Some(crate::processors::ProcessorType::Fpga) => "FPGA",
            Some(crate::processors::ProcessorType::Hybrid) => "HYBRID",
            None => "AUTO",
        }
    }
    
    /// Retorna o hint de backend atual
    pub fn backend_hint(&self) -> Option<crate::processors::ProcessorType> {
        self.backend_hint
    }
    
    /// Retorna o buffer de output (valores escritos via ACT)
    pub fn output(&self) -> &[crate::state::ByteSil] {
        self.memory.output()
    }
    
    /// Reset da VM
    pub fn reset(&mut self) {
        self.state = VspState::new(self.config.default_mode);
        self.cycles = 0;
        self.backend_hint = None;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // API Pública Mode-Aware
    // ═══════════════════════════════════════════════════════════════════════

    /// Define o modo SIL (determina tamanho de Int/Float)
    pub fn set_mode(&mut self, mode: SilMode) {
        self.state.mode = mode;
    }

    /// Retorna o modo SIL atual
    pub fn mode(&self) -> SilMode {
        self.state.mode
    }

    /// Armazena inteiro no registrador (tamanho baseado no modo)
    /// - Sil8: 8 bits (i8)
    /// - Sil16: 16 bits (i16)
    /// - Sil32: 32 bits (i32)
    /// - Sil64/Sil128: 64 bits (i64)
    pub fn set_int(&mut self, reg: usize, value: i64) {
        self.store_int_mode(reg, value);
    }

    /// Lê inteiro do registrador (tamanho baseado no modo)
    pub fn get_int(&self, reg: usize) -> i64 {
        self.load_int_mode(reg)
    }

    /// Armazena float no registrador (tamanho baseado no modo)
    /// - Sil8: ByteSil (log-polar)
    /// - Sil16: f16 (half precision)
    /// - Sil32: f32
    /// - Sil64/Sil128: f64
    pub fn set_float(&mut self, reg: usize, value: f64) {
        self.store_float_mode(reg, value);
    }

    /// Lê float do registrador (tamanho baseado no modo)
    pub fn get_float(&self, reg: usize) -> f64 {
        self.load_float_mode(reg)
    }

    /// Armazena ByteSil diretamente no registrador
    pub fn set_bytesil(&mut self, reg: usize, value: ByteSil) {
        self.state.regs[reg] = value;
    }

    /// Lê ByteSil do registrador
    pub fn get_bytesil(&self, reg: usize) -> ByteSil {
        self.state.regs[reg]
    }

    /// Retorna quantos registradores um valor Int/Float ocupa no modo atual
    pub fn regs_per_value(&self) -> usize {
        match self.state.mode {
            SilMode::Sil8 => 1,
            SilMode::Sil16 => 1,
            SilMode::Sil32 => 2,
            SilMode::Sil64 | SilMode::Sil128 => 4,
        }
    }

    /// Retorna o tamanho em bits do modo atual
    pub fn mode_bits(&self) -> usize {
        match self.state.mode {
            SilMode::Sil8 => 8,
            SilMode::Sil16 => 16,
            SilMode::Sil32 => 32,
            SilMode::Sil64 => 64,
            SilMode::Sil128 => 128,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsp_creation() {
        let vsp = Vsp::new(VspConfig::default());
        assert!(vsp.is_ok());
    }

    #[test]
    fn test_simple_program() {
        let mut vsp = Vsp::new(VspConfig::default()).unwrap();

        // Programa: MOVI R0, 0x55; HLT
        let code = vec![
            0x21, 0x00, 0x55, // MOVI R0, 0x55
            0x01,             // HLT
        ];

        vsp.load_bytes(&code, &[]).unwrap();
        let result = vsp.run().unwrap();

        assert_eq!(result.layers[0].to_u8(), 0x55);
    }

    #[test]
    fn test_mode_aware_int_sil8() {
        let mut vsp = Vsp::new(VspConfig::default().with_mode(SilMode::Sil8)).unwrap();
        vsp.set_int(0, 42);
        assert_eq!(vsp.get_int(0), 42);
        assert_eq!(vsp.regs_per_value(), 1);
    }

    #[test]
    fn test_mode_aware_int_sil16() {
        let mut vsp = Vsp::new(VspConfig::default().with_mode(SilMode::Sil16)).unwrap();
        vsp.set_int(0, 1000);
        assert_eq!(vsp.get_int(0), 1000);
        assert_eq!(vsp.regs_per_value(), 1);
    }

    #[test]
    fn test_mode_aware_int_sil32() {
        let mut vsp = Vsp::new(VspConfig::default().with_mode(SilMode::Sil32)).unwrap();
        vsp.set_int(0, 100_000);
        assert_eq!(vsp.get_int(0), 100_000);
        assert_eq!(vsp.regs_per_value(), 2);
    }

    #[test]
    fn test_mode_aware_int_sil64() {
        let mut vsp = Vsp::new(VspConfig::default().with_mode(SilMode::Sil64)).unwrap();
        vsp.set_int(0, 10_000_000_000i64);
        assert_eq!(vsp.get_int(0), 10_000_000_000i64);
        assert_eq!(vsp.regs_per_value(), 4);
    }

    #[test]
    fn test_mode_aware_float_sil32() {
        let mut vsp = Vsp::new(VspConfig::default().with_mode(SilMode::Sil32)).unwrap();
        vsp.set_float(0, 3.14159);
        let result = vsp.get_float(0);
        assert!((result - 3.14159).abs() < 0.001); // f32 precision
    }

    #[test]
    fn test_mode_aware_float_sil64() {
        let mut vsp = Vsp::new(VspConfig::default().with_mode(SilMode::Sil64)).unwrap();
        vsp.set_float(0, std::f64::consts::PI);
        let result = vsp.get_float(0);
        assert!((result - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_mode_switch() {
        let mut vsp = Vsp::new(VspConfig::default()).unwrap();

        // Começa em Sil128 (default)
        assert_eq!(vsp.mode(), SilMode::Sil128);

        // Muda para Sil32
        vsp.set_mode(SilMode::Sil32);
        assert_eq!(vsp.mode(), SilMode::Sil32);
        assert_eq!(vsp.mode_bits(), 32);

        // Testa int no novo modo
        vsp.set_int(0, 42);
        assert_eq!(vsp.get_int(0), 42);
    }
}
