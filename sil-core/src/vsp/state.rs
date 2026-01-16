//! Estado do VSP
//!
//! Registradores, flags e modos de compatibilidade.

use crate::state::{ByteSil, SilState};
use super::error::{VspError, VspResult};

/// Modo SIL (compatibilidade)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SilMode {
    /// 1 camada, 8 bits
    Sil8 = 0,
    /// 2 camadas, 16 bits
    Sil16 = 1,
    /// 4 camadas, 32 bits
    Sil32 = 2,
    /// 8 camadas, 64 bits
    Sil64 = 3,
    /// 16 camadas, 128 bits
    Sil128 = 4,
}

impl SilMode {
    /// Número de camadas ativas
    pub fn layer_count(&self) -> usize {
        match self {
            Self::Sil8 => 1,
            Self::Sil16 => 2,
            Self::Sil32 => 4,
            Self::Sil64 => 8,
            Self::Sil128 => 16,
        }
    }
    
    /// Número de bits
    pub fn bit_count(&self) -> usize {
        self.layer_count() * 8
    }
    
    /// Cria de bits
    pub fn from_bits(bits: u8) -> VspResult<Self> {
        Ok(match bits {
            0 | 8 => Self::Sil8,
            1 | 16 => Self::Sil16,
            2 | 32 => Self::Sil32,
            3 | 64 => Self::Sil64,
            4 | 128 => Self::Sil128,
            _ => return Err(VspError::InvalidMode(bits)),
        })
    }
    
    /// Negocia modo comum (menor dos dois)
    pub fn negotiate(&self, other: Self) -> Self {
        if *self as u8 <= other as u8 {
            *self
        } else {
            other
        }
    }
}

impl std::fmt::Display for SilMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SIL-{}", self.bit_count())
    }
}

/// Estratégia de demoção
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoteStrategy {
    /// Trunca camadas superiores
    Truncate,
    /// XOR das camadas
    Xor,
    /// Média das camadas
    Average,
    /// Máximo das camadas
    Max,
}

/// Status Register
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusRegister {
    /// Zero flag - resultado é ByteSil::NULL
    pub zero: bool,
    /// Negative flag - ρ < 0
    pub negative: bool,
    /// Overflow flag - ρ saturou
    pub overflow: bool,
    /// Collapse flag - RF colapsou
    pub collapse: bool,
    /// Halt flag - execução parada
    pub halt: bool,
    /// Interrupt flag - interrupção pendente
    pub interrupt: bool,
    /// Error flag - erro de execução
    pub error: bool,
    /// Mode change flag - mudança de modo pendente
    pub mode_change: bool,
}

impl StatusRegister {
    /// Cria novo SR zerado
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Converte para byte
    pub fn to_u8(&self) -> u8 {
        let mut byte = 0u8;
        if self.zero { byte |= 0x01; }
        if self.negative { byte |= 0x02; }
        if self.overflow { byte |= 0x04; }
        if self.collapse { byte |= 0x08; }
        if self.halt { byte |= 0x10; }
        if self.interrupt { byte |= 0x20; }
        if self.error { byte |= 0x40; }
        if self.mode_change { byte |= 0x80; }
        byte
    }
    
    /// Cria de byte
    pub fn from_u8(byte: u8) -> Self {
        Self {
            zero: byte & 0x01 != 0,
            negative: byte & 0x02 != 0,
            overflow: byte & 0x04 != 0,
            collapse: byte & 0x08 != 0,
            halt: byte & 0x10 != 0,
            interrupt: byte & 0x20 != 0,
            error: byte & 0x40 != 0,
            mode_change: byte & 0x80 != 0,
        }
    }
    
    /// Reset
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Estado completo do VSP
#[derive(Debug, Clone)]
pub struct VspState {
    /// 16 registradores de estado (R0-RF)
    pub regs: [ByteSil; 16],
    /// Program Counter
    pub pc: u32,
    /// Stack Pointer
    pub sp: u32,
    /// Frame Pointer
    pub fp: u32,
    /// Status Register
    pub sr: StatusRegister,
    /// Modo SIL atual
    pub mode: SilMode,
    /// Gradiente calculado (opcional)
    pub gradient: Option<[f32; 16]>,
}

impl VspState {
    /// Cria novo estado
    pub fn new(mode: SilMode) -> Self {
        Self {
            regs: [ByteSil::NULL; 16],
            pc: 0,
            sp: 0,
            fp: 0,
            sr: StatusRegister::new(),
            mode,
            gradient: None,
        }
    }
    
    /// Cria estado neutro (ρ=0, θ=0 em todas as camadas)
    pub fn neutral(mode: SilMode) -> Self {
        Self {
            regs: [ByteSil::ONE; 16],
            pc: 0,
            sp: 0,
            fp: 0,
            sr: StatusRegister::new(),
            mode,
            gradient: None,
        }
    }
    
    /// Converte para SilState
    pub fn to_sil_state(&self) -> SilState {
        SilState { layers: self.regs }
    }
    
    /// Carrega de SilState
    pub fn from_sil_state(&mut self, state: &SilState) {
        self.regs = state.layers;
    }
    
    /// Promove para modo maior
    pub fn promote(&mut self, target: SilMode) -> VspResult<()> {
        if target as u8 <= self.mode as u8 {
            return Ok(()); // Já está no modo ou maior
        }
        
        // Camadas novas são NULL
        // (já estão NULL por padrão)
        
        self.mode = target;
        self.sr.mode_change = true;
        Ok(())
    }
    
    /// Demove para modo menor
    pub fn demote(&mut self, target: SilMode, strategy: DemoteStrategy) -> VspResult<()> {
        if target as u8 >= self.mode as u8 {
            return Ok(()); // Já está no modo ou menor
        }
        
        let target_layers = target.layer_count();
        let current_layers = self.mode.layer_count();
        
        match strategy {
            DemoteStrategy::Truncate => {
                // Apenas ignora camadas superiores
                for i in target_layers..16 {
                    self.regs[i] = ByteSil::NULL;
                }
            }
            
            DemoteStrategy::Xor => {
                // XOR camadas: R[i] = R[i] ⊕ R[i + target_layers]
                let fold = current_layers / target_layers;
                for i in 0..target_layers {
                    let mut acc = self.regs[i];
                    for f in 1..fold {
                        let idx = i + f * target_layers;
                        if idx < 16 {
                            acc = acc.xor(&self.regs[idx]);
                        }
                    }
                    self.regs[i] = acc;
                }
                for i in target_layers..16 {
                    self.regs[i] = ByteSil::NULL;
                }
            }
            
            DemoteStrategy::Average => {
                // Média das camadas
                let fold = current_layers / target_layers;
                for i in 0..target_layers {
                    let mut rho_sum = self.regs[i].rho as i32;
                    let mut theta_sum = self.regs[i].theta as u32;
                    let mut count = 1;
                    
                    for f in 1..fold {
                        let idx = i + f * target_layers;
                        if idx < 16 {
                            rho_sum += self.regs[idx].rho as i32;
                            theta_sum += self.regs[idx].theta as u32;
                            count += 1;
                        }
                    }
                    
                    self.regs[i] = ByteSil::new(
                        (rho_sum / count) as i8,
                        ((theta_sum / count as u32) % 16) as u8,
                    );
                }
                for i in target_layers..16 {
                    self.regs[i] = ByteSil::NULL;
                }
            }
            
            DemoteStrategy::Max => {
                // Máximo (maior magnitude)
                let fold = current_layers / target_layers;
                for i in 0..target_layers {
                    let mut max = self.regs[i];
                    
                    for f in 1..fold {
                        let idx = i + f * target_layers;
                        if idx < 16 && self.regs[idx].rho > max.rho {
                            max = self.regs[idx];
                        }
                    }
                    
                    self.regs[i] = max;
                }
                for i in target_layers..16 {
                    self.regs[i] = ByteSil::NULL;
                }
            }
        }
        
        self.mode = target;
        self.sr.mode_change = true;
        Ok(())
    }
    
    /// Retorna camadas ativas baseado no modo
    pub fn active_layers(&self) -> &[ByteSil] {
        &self.regs[..self.mode.layer_count()]
    }
    
    /// Retorna camadas ativas mutáveis
    pub fn active_layers_mut(&mut self) -> &mut [ByteSil] {
        let count = self.mode.layer_count();
        &mut self.regs[..count]
    }
    
    /// Serializa estado (incluindo registradores especiais)
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(32 + 16);
        
        // Registradores (16 bytes)
        for r in &self.regs {
            data.push(r.to_u8());
        }
        
        // PC (4 bytes)
        data.extend_from_slice(&self.pc.to_le_bytes());
        
        // SP (4 bytes)
        data.extend_from_slice(&self.sp.to_le_bytes());
        
        // FP (4 bytes)
        data.extend_from_slice(&self.fp.to_le_bytes());
        
        // SR (1 byte)
        data.push(self.sr.to_u8());
        
        // Mode (1 byte)
        data.push(self.mode as u8);
        
        data
    }
    
    /// Deserializa estado
    pub fn deserialize(data: &[u8]) -> VspResult<Self> {
        if data.len() < 30 {
            return Err(VspError::InvalidState);
        }
        
        let mut regs = [ByteSil::NULL; 16];
        for i in 0..16 {
            regs[i] = ByteSil::from_u8(data[i]);
        }
        
        let pc = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let sp = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let fp = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let sr = StatusRegister::from_u8(data[28]);
        let mode = SilMode::from_bits(data[29])?;
        
        Ok(Self {
            regs,
            pc,
            sp,
            fp,
            sr,
            mode,
            gradient: None,
        })
    }
}

impl std::fmt::Display for VspState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "VSP State [{}]", self.mode)?;
        writeln!(f, "  PC: 0x{:08X}  SP: 0x{:08X}  FP: 0x{:08X}", self.pc, self.sp, self.fp)?;
        writeln!(f, "  SR: Z={} N={} O={} C={} H={}",
            self.sr.zero as u8,
            self.sr.negative as u8,
            self.sr.overflow as u8,
            self.sr.collapse as u8,
            self.sr.halt as u8,
        )?;
        writeln!(f, "  Registers:")?;
        for (i, r) in self.regs.iter().enumerate() {
            if i < self.mode.layer_count() {
                write!(f, "    R{:X}: (ρ={:+2}, θ={:2})", i, r.rho, r.theta)?;
                if (i + 1) % 4 == 0 {
                    writeln!(f)?;
                } else {
                    write!(f, "  ")?;
                }
            }
        }
        Ok(())
    }
}

/// Frame de chamada
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Endereço de retorno
    pub return_addr: u32,
    /// Frame pointer anterior
    pub prev_fp: u32,
    /// Estado local salvo (opcional)
    pub saved_state: Option<[ByteSil; 16]>,
}

impl CallFrame {
    /// Cria novo frame
    pub fn new(return_addr: u32, prev_fp: u32) -> Self {
        Self {
            return_addr,
            prev_fp,
            saved_state: None,
        }
    }
    
    /// Tamanho serializado
    pub fn serialized_size(&self) -> usize {
        8 + if self.saved_state.is_some() { 16 } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sil_mode() {
        assert_eq!(SilMode::Sil8.layer_count(), 1);
        assert_eq!(SilMode::Sil128.layer_count(), 16);
        assert_eq!(SilMode::Sil64.bit_count(), 64);
    }
    
    #[test]
    fn test_mode_negotiation() {
        let a = SilMode::Sil128;
        let b = SilMode::Sil32;
        assert_eq!(a.negotiate(b), SilMode::Sil32);
    }
    
    #[test]
    fn test_status_register() {
        let mut sr = StatusRegister::new();
        sr.zero = true;
        sr.collapse = true;
        
        let byte = sr.to_u8();
        let sr2 = StatusRegister::from_u8(byte);
        
        assert!(sr2.zero);
        assert!(sr2.collapse);
        assert!(!sr2.halt);
    }
    
    #[test]
    fn test_demote_xor() {
        let mut state = VspState::new(SilMode::Sil64);
        state.regs[0] = ByteSil::new(2, 3);
        state.regs[4] = ByteSil::new(1, 5);
        
        state.demote(SilMode::Sil32, DemoteStrategy::Xor).unwrap();
        
        // R0 = R0 ^ R4
        let expected = ByteSil::new(2, 3).xor(&ByteSil::new(1, 5));
        assert_eq!(state.regs[0], expected);
        assert_eq!(state.mode, SilMode::Sil32);
    }
    
    #[test]
    fn test_serialize_roundtrip() {
        let mut state = VspState::new(SilMode::Sil128);
        state.regs[0] = ByteSil::new(3, 7);
        state.pc = 0x1234;
        state.sr.zero = true;
        
        let data = state.serialize();
        let state2 = VspState::deserialize(&data).unwrap();
        
        assert_eq!(state2.regs[0], state.regs[0]);
        assert_eq!(state2.pc, state.pc);
        assert!(state2.sr.zero);
    }
}
