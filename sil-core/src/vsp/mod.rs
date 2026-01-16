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
                if self.state.sr.zero {
                    self.state.pc = instr.addr_or_imm24();
                }
            }
            
            Jn => {
                if self.state.sr.negative {
                    self.state.pc = instr.addr_or_imm24();
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
                let imm = instr.imm8();
                self.state.regs[ra] = ByteSil::from_u8(imm);
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
                let syscall_num = instr.addr_or_imm24();
                self.syscall(syscall_num)?;
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
    fn syscall(&mut self, num: u32) -> VspResult<()> {
        use StdlibIntrinsic::*;
        let id = num as u8;

        match id {
            // ────── I/O Functions ──────
            id if id == Println as u8 => {
                println!();
            }
            id if id == PrintString as u8 => {
                // Print value from R0.rho
                print!("{}", self.state.regs[0].rho);
            }
            id if id == PrintInt as u8 => {
                println!("{}", self.state.regs[0].rho);
            }
            id if id == PrintFloat as u8 => {
                let mag = (self.state.regs[0].rho as f64).exp();
                println!("{:.6}", mag);
            }
            id if id == PrintBool as u8 => {
                println!("{}", self.state.regs[0].rho != 0);
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
}
