//! High-Performance Threaded Interpreter for VSP
//!
//! This interpreter uses function pointer dispatch for near-native performance
//! on any architecture. While slower than JIT (ARM64 DynASM), it provides:
//! - Universal portability (x86, ARM, RISC-V, WASM, etc)
//! - Predictable performance (~500K-1M ops/sec)
//! - Zero external dependencies
//! - Easy debugging and maintenance
//!
//! # Performance Strategy
//! - Pre-computed jump table (no match/branch per instruction)
//! - Inline hot paths with #[inline(always)]
//! - Branch prediction friendly code
//! - Cache-friendly SilState access patterns

use crate::prelude::*;
use crate::state::BitDeSil;
use crate::vsp::{Opcode, SilcFile, VspError, assembler::StdlibIntrinsic};
use std::time::Instant;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use sil_energy::{EnergyMeter, CpuEnergyModel};

/// Handler function type for opcode execution
type OpcodeHandler = fn(&mut SilState, &[u8]) -> Result<(), VspError>;

/// Statistics for interpreter execution
#[derive(Debug, Clone, Default)]
pub struct InterpreterStats {
    pub total_cycles: u64,
    pub total_instructions: u64,
    pub execute_time_us: u64,
}

/// Benchmark metrics state
#[derive(Debug, Clone, Default)]
pub struct BenchmarkMetrics {
    pub start_time: Option<Instant>,
    pub total_samples: u64,
    pub total_flops: u64,
    pub total_macs: u64,
    pub peak_memory_bytes: usize,
    pub total_latency_us: u64,
}

/// Global energy meter state (thread-safe)
static ENERGY_METER: Lazy<Mutex<Option<EnergyMeter>>> = Lazy::new(|| Mutex::new(None));

/// Global benchmark state (thread-safe)
static BENCHMARK_STATE: Lazy<Mutex<BenchmarkMetrics>> = Lazy::new(|| Mutex::new(BenchmarkMetrics::default()));

/// Global string table for print_string (thread-safe)
static STRING_TABLE: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Carbon intensity factor (gCO2e per kWh) - Brazil grid average
const CARBON_INTENSITY_BRAZIL: f64 = 0.075; // Very clean grid (hydroelectric)

/// Carbon intensity factor (gCO2e per kWh) - World average
const CARBON_INTENSITY_WORLD: f64 = 0.475;

/// Threaded interpreter with pre-computed dispatch table
pub struct VspInterpreter {
    /// Pre-compiled dispatch table
    handlers: Vec<(OpcodeHandler, Vec<u8>)>,
    stats: InterpreterStats,
}

impl VspInterpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            stats: InterpreterStats::default(),
        }
    }

    /// Compile bytecode into dispatch table (threaded code)
    pub fn compile(&mut self, program: &SilcFile) -> Result<(), VspError> {
        self.handlers.clear();

        // Parse bytecode into instructions
        let mut i = 0;
        while i < program.code.len() {
            let opcode_byte = program.code[i];
            let opcode = Opcode::from_byte(opcode_byte).unwrap_or(Opcode::Nop);

            // Get instruction length based on format
            let inst_len = opcode.format().size();
            let end = (i + inst_len).min(program.code.len());
            let instruction = program.code[i..end].to_vec();

            let handler = Self::get_handler(opcode);
            self.handlers.push((handler, instruction));

            i += inst_len;
        }

        Ok(())
    }

    /// Execute the compiled program
    #[inline]
    pub fn execute(&mut self, state: &mut SilState) -> Result<(), VspError> {
        let start = Instant::now();
        let mut cycles = 0u64;

        // Threaded dispatch - direct function calls, no match overhead
        for (handler, instruction) in &self.handlers {
            handler(state, instruction)?;
            cycles += 1;
        }

        self.stats.total_cycles += cycles;
        self.stats.total_instructions += self.handlers.len() as u64;
        self.stats.execute_time_us += start.elapsed().as_micros() as u64;

        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> &InterpreterStats {
        &self.stats
    }

    /// Map opcode to handler function (compiled at build time)
    #[inline(always)]
    fn get_handler(opcode: Opcode) -> OpcodeHandler {
        match opcode {
            // Control Flow
            Opcode::Nop => Self::op_nop,
            Opcode::Hlt => Self::op_hlt,
            Opcode::Ret => Self::op_ret,
            Opcode::Yield => Self::op_yield,

            // Data Movement
            Opcode::Mov => Self::op_mov,
            Opcode::Movi => Self::op_movi,
            Opcode::Movi16 => Self::op_movi16,
            Opcode::Movi32 => Self::op_movi32,
            Opcode::Xchg => Self::op_xchg,
            Opcode::Load => Self::op_load,
            Opcode::Store => Self::op_store,
            Opcode::Push => Self::op_push,
            Opcode::Pop => Self::op_pop,

            // Arithmetic ByteSil
            Opcode::Mul => Self::op_mul,
            Opcode::Div => Self::op_div,
            Opcode::Add => Self::op_add,
            Opcode::Sub => Self::op_sub,

            // Arithmetic Int/Float (direct values)
            Opcode::AddInt => Self::op_add_int,
            Opcode::SubInt => Self::op_sub_int,
            Opcode::MulInt => Self::op_mul_int,
            Opcode::DivInt => Self::op_div_int,
            Opcode::AddFloat => Self::op_add_float,
            Opcode::SubFloat => Self::op_sub_float,
            Opcode::MulFloat => Self::op_mul_float,
            Opcode::DivFloat => Self::op_div_float,
            Opcode::Inv => Self::op_inv,
            Opcode::Conj => Self::op_conj,
            Opcode::Pow => Self::op_pow,
            Opcode::Root => Self::op_root,

            // Phase & Magnitude
            Opcode::Mag => Self::op_mag,
            Opcode::Phase => Self::op_phase,
            Opcode::Scale => Self::op_scale,
            Opcode::Rotate => Self::op_rotate,

            // Layer Operations
            Opcode::Xorl => Self::op_xorl,
            Opcode::Andl => Self::op_andl,
            Opcode::Orl => Self::op_orl,
            Opcode::Notl => Self::op_notl,
            Opcode::Fold => Self::op_fold,
            Opcode::Shiftl => Self::op_shiftl,
            Opcode::Rotatl => Self::op_rotatl,

            // Transforms
            Opcode::Lerp => Self::op_lerp,
            Opcode::Slerp => Self::op_slerp,
            Opcode::Collapse => Self::op_collapse,

            // Jumps (need full VM for proper implementation)
            Opcode::Jmp | Opcode::Jz | Opcode::Jn | Opcode::Jc | Opcode::Jo |
            Opcode::Call | Opcode::Loop => Self::op_nop,

            // System
            Opcode::Syscall => Self::op_syscall,
            Opcode::Setmode | Opcode::Promote | Opcode::Demote => Self::op_nop,
            Opcode::In | Opcode::Out | Opcode::Sense | Opcode::Act => Self::op_nop,
            Opcode::Prefetch | Opcode::HintCpu | Opcode::HintGpu | Opcode::HintNpu => Self::op_nop,

            // Quantum / BitDeSil
            Opcode::BitHadamard => Self::op_bit_hadamard,
            Opcode::BitPauliX => Self::op_bit_pauli_x,
            Opcode::BitPauliY => Self::op_bit_pauli_y,
            Opcode::BitPauliZ => Self::op_bit_pauli_z,
            Opcode::BitCollapse => Self::op_bit_collapse,
            Opcode::BitMeasure => Self::op_bit_measure,
            Opcode::BitRotateQ => Self::op_bit_rotate_q,
            Opcode::BitNormalize => Self::op_bit_normalize,

            _ => Self::op_nop,
        }
    }

    // ═════════════════════════════════════════════════════════════════
    // OPCODE IMPLEMENTATIONS
    // ═════════════════════════════════════════════════════════════════

    #[inline(always)]
    fn op_nop(_state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        Ok(())
    }

    #[inline(always)]
    fn op_hlt(_state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        Ok(()) // Early return handled by VM
    }

    #[inline(always)]
    fn op_ret(_state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        Ok(())
    }

    #[inline(always)]
    fn op_yield(_state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        Ok(())
    }

    #[inline(always)]
    fn op_mov(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        // MOV format: [opcode][packed_regs][unused]
        // packed_regs: (dest & 0x0F) | ((src & 0x0F) << 4)
        // Copy source layer to destination layer
        if inst.len() >= 2 {
            let packed = inst[1];
            let dest = (packed & 0x0F) as usize;
            let src = ((packed >> 4) & 0x0F) as usize;
            if dest < 16 && src < 16 {
                state.layers[dest] = state.layers[src];
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn op_movi(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        // MOVI format: [opcode][reg][imm8]
        // Store immediate value in specified layer (reg maps to layer)
        if inst.len() >= 3 {
            let layer = (inst[1] & 0x0F) as usize;
            let imm = inst[2] as i8; // Signed 8-bit immediate
            if layer < 16 {
                // Store immediate in rho, keep theta at 0
                state.layers[layer] = ByteSil::new(imm, 0);
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn op_xchg(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Same as MOV
        state.layers.swap(0, 1);
        Ok(())
    }

    #[inline(always)]
    fn op_load(_state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Placeholder - needs memory infrastructure
        Ok(())
    }

    #[inline(always)]
    fn op_store(_state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Placeholder
        Ok(())
    }

    #[inline(always)]
    fn op_push(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Rotate layers down: L15 ← ... ← L1 ← L0
        let temp = state.layers[0];
        state.layers.rotate_left(1);
        state.layers[15] = temp;
        Ok(())
    }

    #[inline(always)]
    fn op_pop(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Rotate layers up: L0 ← L1 ← ... ← L15
        state.layers.rotate_left(1);
        Ok(())
    }

    #[inline(always)]
    fn op_mul(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // ByteSil multiplication: (ρ1+ρ2, θ1+θ2 mod 16)
        let l0 = state.layers[0];
        let l1 = state.layers[1];

        let rho = l0.rho.saturating_add(l1.rho);
        let theta = l0.theta.wrapping_add(l1.theta) & 0x0F;

        state.layers[0] = ByteSil { rho, theta };
        Ok(())
    }

    #[inline(always)]
    fn op_div(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // ByteSil division: (ρ1-ρ2, θ1-θ2 mod 16)
        let l0 = state.layers[0];
        let l1 = state.layers[1];

        let rho = l0.rho.saturating_sub(l1.rho);
        let theta = l0.theta.wrapping_sub(l1.theta) & 0x0F;

        state.layers[0] = ByteSil { rho, theta };
        Ok(())
    }

    #[inline(always)]
    fn op_add(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Simplified ADD (component-wise)
        let l0 = state.layers[0];
        let l1 = state.layers[1];

        state.layers[0] = ByteSil {
            rho: l0.rho.saturating_add(l1.rho),
            theta: l0.theta.wrapping_add(l1.theta),
        };
        Ok(())
    }

    #[inline(always)]
    fn op_sub(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Simplified SUB
        let l0 = state.layers[0];
        let l1 = state.layers[1];

        state.layers[0] = ByteSil {
            rho: l0.rho.saturating_sub(l1.rho),
            theta: l0.theta.wrapping_sub(l1.theta),
        };
        Ok(())
    }

    // ═════════════════════════════════════════════════════════════════
    // INTEGER/FLOAT OPERATIONS (direct values, not log-polar)
    // Values are stored as: i32 in R0.rho:R1.rho (low:high bytes)
    // or f32 bits in R0:R1:R2:R3 (4 bytes)
    // ═════════════════════════════════════════════════════════════════

    #[inline(always)]
    fn op_movi16(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        // MOVI16 format: [opcode][reg][imm_lo][imm_hi]
        // Store 16-bit value across R[n].rho (low byte) and R[n+1].rho (high byte)
        if inst.len() >= 4 {
            let layer = (inst[1] & 0x0F) as usize;
            let imm_lo = inst[2] as i8;
            let imm_hi = inst[3] as i8;
            if layer < 15 {
                state.layers[layer] = ByteSil::new(imm_lo, 0);
                state.layers[layer + 1] = ByteSil::new(imm_hi, 0);
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn op_movi32(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        // MOVI32 format: [opcode][reg][b0][b1][b2][b3]
        // Store 32-bit value across 4 consecutive layers
        if inst.len() >= 6 {
            let layer = (inst[1] & 0x0F) as usize;
            if layer <= 12 {
                state.layers[layer] = ByteSil::new(inst[2] as i8, 0);
                state.layers[layer + 1] = ByteSil::new(inst[3] as i8, 0);
                state.layers[layer + 2] = ByteSil::new(inst[4] as i8, 0);
                state.layers[layer + 3] = ByteSil::new(inst[5] as i8, 0);
            }
        }
        Ok(())
    }

    /// Extract i32 from 4 consecutive layers (little-endian)
    #[inline(always)]
    fn extract_i32(state: &SilState, start: usize) -> i32 {
        let b0 = state.layers[start].rho as u8 as u32;
        let b1 = state.layers[start + 1].rho as u8 as u32;
        let b2 = state.layers[start + 2].rho as u8 as u32;
        let b3 = state.layers[start + 3].rho as u8 as u32;
        ((b3 << 24) | (b2 << 16) | (b1 << 8) | b0) as i32
    }

    /// Store i32 into 4 consecutive layers (little-endian)
    #[inline(always)]
    fn store_i32(state: &mut SilState, start: usize, value: i32) {
        let bytes = value.to_le_bytes();
        state.layers[start] = ByteSil::new(bytes[0] as i8, 0);
        state.layers[start + 1] = ByteSil::new(bytes[1] as i8, 0);
        state.layers[start + 2] = ByteSil::new(bytes[2] as i8, 0);
        state.layers[start + 3] = ByteSil::new(bytes[3] as i8, 0);
    }

    /// Extract f32 from 4 consecutive layers
    #[inline(always)]
    fn extract_f32(state: &SilState, start: usize) -> f32 {
        let bits = Self::extract_i32(state, start) as u32;
        f32::from_bits(bits)
    }

    /// Store f32 into 4 consecutive layers
    #[inline(always)]
    fn store_f32(state: &mut SilState, start: usize, value: f32) {
        Self::store_i32(state, start, value.to_bits() as i32);
    }

    #[inline(always)]
    fn op_add_int(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // ADDINT: R0:R1:R2:R3 = R0:R1:R2:R3 + R4:R5:R6:R7
        let a = Self::extract_i32(state, 0) as i32;
        let b = Self::extract_i32(state, 4) as i32;
        Self::store_i32(state, 0, a.wrapping_add(b));
        Ok(())
    }

    #[inline(always)]
    fn op_sub_int(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        let a = Self::extract_i32(state, 0) as i32;
        let b = Self::extract_i32(state, 4) as i32;
        Self::store_i32(state, 0, a.wrapping_sub(b));
        Ok(())
    }

    #[inline(always)]
    fn op_mul_int(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        let a = Self::extract_i32(state, 0) as i32;
        let b = Self::extract_i32(state, 4) as i32;
        Self::store_i32(state, 0, a.wrapping_mul(b));
        Ok(())
    }

    #[inline(always)]
    fn op_div_int(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        let a = Self::extract_i32(state, 0) as i32;
        let b = Self::extract_i32(state, 4) as i32;
        if b != 0 {
            Self::store_i32(state, 0, a / b);
        }
        Ok(())
    }

    #[inline(always)]
    fn op_add_float(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // ADDFLOAT: R0:R1:R2:R3 = R0:R1:R2:R3 + R4:R5:R6:R7
        let a = Self::extract_f32(state, 0);
        let b = Self::extract_f32(state, 4);
        Self::store_f32(state, 0, a + b);
        Ok(())
    }

    #[inline(always)]
    fn op_sub_float(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        let a = Self::extract_f32(state, 0);
        let b = Self::extract_f32(state, 4);
        Self::store_f32(state, 0, a - b);
        Ok(())
    }

    #[inline(always)]
    fn op_mul_float(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        let a = Self::extract_f32(state, 0);
        let b = Self::extract_f32(state, 4);
        Self::store_f32(state, 0, a * b);
        Ok(())
    }

    #[inline(always)]
    fn op_div_float(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        let a = Self::extract_f32(state, 0);
        let b = Self::extract_f32(state, 4);
        if b != 0.0 {
            Self::store_f32(state, 0, a / b);
        }
        Ok(())
    }

    #[inline(always)]
    fn op_inv(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Inverse: negate rho
        state.layers[0].rho = -state.layers[0].rho;
        Ok(())
    }

    #[inline(always)]
    fn op_conj(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Conjugate: negate theta
        let theta = state.layers[0].theta as i16;
        state.layers[0].theta = ((-theta) & 0x0F) as u8;
        Ok(())
    }

    #[inline(always)]
    fn op_pow(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Power: (ρ×n, θ×n mod 16)
        let exponent = state.layers[1].rho;
        let rho = (state.layers[0].rho as i16 * exponent as i16).clamp(-128, 127) as i8;
        let theta = (state.layers[0].theta.wrapping_mul(exponent as u8)) & 0x0F;

        state.layers[0] = ByteSil { rho, theta };
        Ok(())
    }

    #[inline(always)]
    fn op_root(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Root: (ρ÷n, θ÷n)
        let divisor = state.layers[1].rho.max(1); // Avoid division by zero
        state.layers[0].rho /= divisor;
        state.layers[0].theta /= divisor as u8;
        state.layers[0].theta &= 0x0F;
        Ok(())
    }

    #[inline(always)]
    fn op_mag(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Magnitude: absolute value of rho
        state.layers[0].rho = state.layers[0].rho.abs();
        Ok(())
    }

    #[inline(always)]
    fn op_phase(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Phase: keep only theta, zero rho
        state.layers[0].rho = 0;
        Ok(())
    }

    #[inline(always)]
    fn op_scale(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Scale magnitude by 2
        state.layers[0].rho = state.layers[0].rho.saturating_mul(2);
        state.layers[0].theta = state.layers[0].theta.wrapping_mul(2) & 0x0F;
        Ok(())
    }

    #[inline(always)]
    fn op_rotate(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Rotate phase: increment theta
        state.layers[0].theta = (state.layers[0].theta + 1) & 0x0F;
        Ok(())
    }

    #[inline(always)]
    fn op_xorl(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // XOR L0 with L1
        state.layers[0] = state.layers[0] ^ state.layers[1];
        Ok(())
    }

    #[inline(always)]
    fn op_andl(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // AND L0 with L1 (bitwise)
        let l0 = state.layers[0];
        let l1 = state.layers[1];
        state.layers[0] = ByteSil {
            rho: l0.rho & l1.rho,
            theta: l0.theta & l1.theta,
        };
        Ok(())
    }

    #[inline(always)]
    fn op_orl(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // OR L0 with L1 (bitwise)
        let l0 = state.layers[0];
        let l1 = state.layers[1];
        state.layers[0] = ByteSil {
            rho: l0.rho | l1.rho,
            theta: l0.theta | l1.theta,
        };
        Ok(())
    }

    #[inline(always)]
    fn op_notl(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // NOT L0 (bitwise)
        let l0 = state.layers[0];
        state.layers[0] = ByteSil {
            rho: !l0.rho,
            theta: !l0.theta,
        };
        Ok(())
    }

    #[inline(always)]
    fn op_fold(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Fold: L0 XOR L8
        state.layers[0] = state.layers[0] ^ state.layers[8];
        Ok(())
    }

    #[inline(always)]
    fn op_shiftl(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Shift layers: L0 = L1, L1 = L2, ..., L15 = L0
        state.layers.rotate_left(1);
        Ok(())
    }

    #[inline(always)]
    fn op_rotatl(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Rotate layers (same as shift for circular buffer)
        state.layers.rotate_left(1);
        Ok(())
    }

    #[inline(always)]
    fn op_lerp(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Linear interpolation: average L0 and L1
        let l0 = state.layers[0];
        let l1 = state.layers[1];

        state.layers[0] = ByteSil {
            rho: ((l0.rho as i16 + l1.rho as i16) / 2) as i8,
            theta: ((l0.theta as u16 + l1.theta as u16) / 2) as u8,
        };
        Ok(())
    }

    #[inline(always)]
    fn op_slerp(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Spherical lerp - simplified as lerp for now
        Self::op_lerp(state, _inst)
    }

    #[inline(always)]
    fn op_collapse(state: &mut SilState, _inst: &[u8]) -> Result<(), VspError> {
        // Collapse: XOR all 16 layers into L0
        let mut result = state.layers[0];
        for i in 1..16 {
            result = result ^ state.layers[i];
        }
        state.layers[0] = result;
        Ok(())
    }

    // ═════════════════════════════════════════════════════════════════
    // QUANTUM / BITDESIL OPERATIONS
    // ═════════════════════════════════════════════════════════════════

    #[inline(always)]
    fn op_bit_hadamard(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        state.layers[ra] = bit.hadamard().to_byte_sil();
        Ok(())
    }

    #[inline(always)]
    fn op_bit_pauli_x(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        state.layers[ra] = bit.pauli_x().to_byte_sil();
        Ok(())
    }

    #[inline(always)]
    fn op_bit_pauli_y(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        state.layers[ra] = bit.pauli_y().to_byte_sil();
        Ok(())
    }

    #[inline(always)]
    fn op_bit_pauli_z(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        state.layers[ra] = bit.pauli_z().to_byte_sil();
        Ok(())
    }

    #[inline(always)]
    fn op_bit_collapse(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        // Usar theta como fonte de pseudo-aleatoriedade
        let random = (state.layers[ra].theta as f32) / 16.0;
        let (_measurement, collapsed) = bit.collapse(random);
        state.layers[ra] = collapsed.to_byte_sil();
        Ok(())
    }

    #[inline(always)]
    fn op_bit_measure(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        // Retorna probabilidade de |0⟩ em L0.rho (escalado 0-127)
        let prob_zero = (bit.prob_zero() * 127.0) as i8;
        state.layers[0] = ByteSil { rho: prob_zero, theta: 0 };
        Ok(())
    }

    #[inline(always)]
    fn op_bit_rotate_q(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        // Quantidade de rotacao vem de L1.rho
        let n = state.layers[1].rho;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        state.layers[ra] = bit.rotate(n).to_byte_sil();
        Ok(())
    }

    #[inline(always)]
    fn op_bit_normalize(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        let ra = (inst.get(1).copied().unwrap_or(0) & 0x0F) as usize;
        let bit = BitDeSil::from_byte_sil(&state.layers[ra]);
        state.layers[ra] = bit.normalize().to_byte_sil();
        Ok(())
    }

    // ═════════════════════════════════════════════════════════════════
    // SYSCALL - STDLIB INTRINSICS
    // ═════════════════════════════════════════════════════════════════

    #[inline(always)]
    fn op_syscall(state: &mut SilState, inst: &[u8]) -> Result<(), VspError> {
        // SYSCALL format: [opcode, intrinsic_id, pad, pad]
        let intrinsic_id = inst.get(1).copied().unwrap_or(0);

        match intrinsic_id {
            // I/O Functions
            id if id == StdlibIntrinsic::Println as u8 => {
                println!();
            }
            id if id == StdlibIntrinsic::PrintString as u8 => {
                // Print string from string table
                // String index passed in inst[2..4] as u16 (little-endian)
                let string_idx = if inst.len() >= 4 {
                    u16::from_le_bytes([inst[2], inst[3]]) as usize
                } else if inst.len() >= 3 {
                    inst[2] as usize
                } else {
                    // Fallback to L0.rho for backwards compatibility
                    let rho = state.layers[0].rho;
                    if rho >= 0 { rho as usize } else { 0 }
                };
                if let Ok(table) = STRING_TABLE.lock() {
                    if let Some(s) = table.get(string_idx) {
                        println!("{}", s);
                    } else {
                        println!("[string#{}]", string_idx);
                    }
                }
            }
            id if id == StdlibIntrinsic::PrintInt as u8 => {
                // Print integer from L0.rho
                println!("{}", state.layers[0].rho);
            }
            id if id == StdlibIntrinsic::PrintFloat as u8 => {
                // Print float from L0 as magnitude (e^rho)
                let mag = (state.layers[0].rho as f64).exp();
                println!("{:.6}", mag);
            }
            id if id == StdlibIntrinsic::PrintBool as u8 => {
                // Print bool (rho != 0)
                println!("{}", state.layers[0].rho != 0);
            }
            id if id == StdlibIntrinsic::PrintBytesil as u8 => {
                let b = state.layers[0];
                println!("ByteSil(ρ={}, θ={})", b.rho, b.theta);
            }
            id if id == StdlibIntrinsic::PrintState as u8 => {
                println!("State:");
                for (i, layer) in state.layers.iter().enumerate() {
                    println!("  L{:X}: ρ={:+3}, θ={:3}", i, layer.rho, layer.theta);
                }
            }

            // ByteSil Functions
            id if id == StdlibIntrinsic::BytesilNew as u8 => {
                // Create ByteSil from L0.rho (rho) and L1.theta (theta)
                state.layers[0] = ByteSil {
                    rho: state.layers[0].rho,
                    theta: state.layers[1].theta,
                };
            }
            id if id == StdlibIntrinsic::BytesilNull as u8 => {
                state.layers[0] = ByteSil::NULL;
            }
            id if id == StdlibIntrinsic::BytesilOne as u8 => {
                state.layers[0] = ByteSil::ONE;
            }
            id if id == StdlibIntrinsic::BytesilI as u8 => {
                state.layers[0] = ByteSil::I;
            }
            id if id == StdlibIntrinsic::BytesilNegOne as u8 => {
                state.layers[0] = ByteSil::NEG_ONE;
            }
            id if id == StdlibIntrinsic::BytesilNegI as u8 => {
                state.layers[0] = ByteSil::NEG_I;
            }
            id if id == StdlibIntrinsic::BytesilMax as u8 => {
                state.layers[0] = ByteSil::MAX;
            }
            id if id == StdlibIntrinsic::BytesilMul as u8 => {
                // L0 = L0 * L1 (log-polar: rho adds, theta adds mod 16)
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil::new(
                    l0.rho.saturating_add(l1.rho),
                    l0.theta.wrapping_add(l1.theta),
                );
            }
            id if id == StdlibIntrinsic::BytesilDiv as u8 => {
                // L0 = L0 / L1 (log-polar: rho subtracts, theta subtracts mod 16)
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil::new(
                    l0.rho.saturating_sub(l1.rho),
                    l0.theta.wrapping_sub(l1.theta),
                );
            }
            id if id == StdlibIntrinsic::BytesilPow as u8 => {
                // L0 = L0^(L1.rho) (log-polar: rho *= n, theta *= n)
                let l0 = state.layers[0];
                let exp = state.layers[1].rho;
                state.layers[0] = ByteSil::new(
                    (l0.rho as i16 * exp as i16).clamp(-8, 7) as i8,
                    l0.theta.wrapping_mul(exp as u8),
                );
            }
            id if id == StdlibIntrinsic::BytesilRoot as u8 => {
                // L0 = L0^(1/L1.rho) (log-polar: rho /= n, theta /= n)
                let l0 = state.layers[0];
                let n = state.layers[1].rho.max(1);
                state.layers[0] = ByteSil::new(
                    l0.rho / n,
                    l0.theta / n as u8,
                );
            }
            id if id == StdlibIntrinsic::BytesilInv as u8 => {
                // Inverse: negate rho
                let l0 = state.layers[0];
                state.layers[0] = ByteSil::new(-l0.rho, l0.theta);
            }
            id if id == StdlibIntrinsic::BytesilConj as u8 => {
                // Conjugate: negate theta
                let l0 = state.layers[0];
                state.layers[0] = ByteSil::new(l0.rho, (16 - l0.theta) & 0x0F);
            }
            id if id == StdlibIntrinsic::BytesilXor as u8 => {
                state.layers[0] = state.layers[0] ^ state.layers[1];
            }
            id if id == StdlibIntrinsic::BytesilMix as u8 => {
                // Mix = (L0 + L1) / 2 simplified as XOR
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil {
                    rho: ((l0.rho as i16 + l1.rho as i16) / 2) as i8,
                    theta: ((l0.theta as u16 + l1.theta as u16) / 2) as u8,
                };
            }
            id if id == StdlibIntrinsic::BytesilRho as u8 => {
                // Return rho as integer in L0
                let rho = state.layers[0].rho;
                state.layers[0] = ByteSil { rho, theta: 0 };
            }
            id if id == StdlibIntrinsic::BytesilTheta as u8 => {
                // Return theta as integer in L0
                let theta = state.layers[0].theta;
                state.layers[0] = ByteSil { rho: theta as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::BytesilMagnitude as u8 => {
                // Magnitude = e^rho
                let mag = (state.layers[0].rho as f64).exp();
                // Store as rho (scaled)
                state.layers[0] = ByteSil::new((mag * 10.0) as i8, 0);
            }
            id if id == StdlibIntrinsic::BytesilIsNull as u8 => {
                let is_null = state.layers[0] == ByteSil::NULL;
                state.layers[0] = ByteSil { rho: is_null as i8, theta: 0 };
            }

            // State Functions
            id if id == StdlibIntrinsic::StateVacuum as u8 => {
                *state = SilState::vacuum();
            }
            id if id == StdlibIntrinsic::StateNeutral as u8 => {
                *state = SilState::neutral();
            }
            id if id == StdlibIntrinsic::StateMax as u8 => {
                *state = SilState::maximum();
            }
            id if id == StdlibIntrinsic::StateGetLayer as u8 => {
                // Get layer index from L1.rho, put result in L0
                let idx = (state.layers[1].rho as usize) & 0x0F;
                state.layers[0] = state.layers[idx];
            }
            id if id == StdlibIntrinsic::StateSetLayer as u8 => {
                // Set layer at index L1.rho to value in L2
                let idx = (state.layers[1].rho as usize) & 0x0F;
                state.layers[idx] = state.layers[2];
            }
            id if id == StdlibIntrinsic::StateXor as u8 => {
                // XOR all layers with L0
                for i in 1..16 {
                    state.layers[i] = state.layers[i] ^ state.layers[0];
                }
            }
            id if id == StdlibIntrinsic::StateFold as u8 => {
                // Fold: L0 = XOR of all layers
                let mut result = state.layers[0];
                for i in 1..16 {
                    result = result ^ state.layers[i];
                }
                state.layers[0] = result;
            }
            id if id == StdlibIntrinsic::StateRotate as u8 => {
                state.layers.rotate_left(1);
            }
            id if id == StdlibIntrinsic::StateCollapse as u8 => {
                // Collapse using XOR strategy
                let mut result = state.layers[0];
                for i in 1..16 {
                    result = result ^ state.layers[i];
                }
                state.layers[0] = result;
            }
            id if id == StdlibIntrinsic::StateCountActiveLayers as u8 => {
                let count = state.layers.iter()
                    .filter(|l| **l != ByteSil::NULL)
                    .count();
                state.layers[0] = ByteSil { rho: count as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateIsVacuum as u8 => {
                let is_vacuum = *state == SilState::vacuum();
                state.layers[0] = ByteSil { rho: is_vacuum as i8, theta: 0 };
            }

            // State - Additional
            id if id == StdlibIntrinsic::StateIsNeutral as u8 => {
                let is_neutral = *state == SilState::neutral();
                state.layers[0] = ByteSil { rho: is_neutral as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateEquals as u8 => {
                // Compare two states stored in consecutive layers (simplified: just L0 == L1)
                let eq = state.layers[0] == state.layers[1];
                state.layers[0] = ByteSil { rho: eq as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateCountNullLayers as u8 => {
                let count = state.layers.iter()
                    .filter(|l| **l == ByteSil::NULL)
                    .count();
                state.layers[0] = ByteSil { rho: count as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateHash as u8 => {
                // Simple hash: XOR all bytes
                let mut hash: u8 = 0;
                for layer in &state.layers {
                    hash ^= layer.rho as u8;
                    hash ^= layer.theta;
                }
                state.layers[0] = ByteSil { rho: hash as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateTensor as u8 => {
                // Tensor product approximation: multiply magnitudes, add phases
                // Takes L0-L7 as state A, L8-L15 as state B, result in L0-L15
                // Simplified: just multiply corresponding layers
                for i in 0..16 {
                    let a = state.layers[i];
                    let b = state.layers[(i + 8) % 16];
                    state.layers[i] = ByteSil {
                        rho: ((a.rho as i16 + b.rho as i16) / 2) as i8,
                        theta: a.theta.wrapping_add(b.theta),
                    };
                }
            }
            id if id == StdlibIntrinsic::StateProject as u8 => {
                // Project to specific layer group (0-3 for each 4-layer group)
                let group = (state.layers[0].rho as usize) & 0x03;
                let start = group * 4;
                for i in 0..16 {
                    if i < start || i >= start + 4 {
                        state.layers[i] = ByteSil::NULL;
                    }
                }
            }
            id if id == StdlibIntrinsic::StateNormalize as u8 => {
                // Normalize: find max magnitude and scale all relative to it
                let max_rho = state.layers.iter()
                    .map(|l| l.rho.abs())
                    .max()
                    .unwrap_or(0);
                if max_rho > 0 {
                    let scale = 7.0 / max_rho as f32;
                    for layer in &mut state.layers {
                        layer.rho = ((layer.rho as f32 * scale) as i8).clamp(-8, 7);
                    }
                }
            }
            id if id == StdlibIntrinsic::StateCollapseMag as u8 => {
                // Collapse by summing magnitudes
                let sum: i16 = state.layers.iter()
                    .map(|l| l.rho as i16)
                    .sum();
                state.layers[0] = ByteSil { rho: (sum / 16) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateCollapseSum as u8 => {
                // Same as CollapseMag (alias)
                let sum: i16 = state.layers.iter()
                    .map(|l| l.rho as i16)
                    .sum();
                state.layers[0] = ByteSil { rho: (sum / 16) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::StateCollapseFirst as u8 => {
                // Collapse: first non-null layer
                let first = state.layers.iter()
                    .find(|l| **l != ByteSil::NULL)
                    .copied()
                    .unwrap_or(ByteSil::NULL);
                state.layers[0] = first;
            }
            id if id == StdlibIntrinsic::StateCollapseLast as u8 => {
                // Collapse: last non-null layer
                let last = state.layers.iter()
                    .rev()
                    .find(|l| **l != ByteSil::NULL)
                    .copied()
                    .unwrap_or(ByteSil::NULL);
                state.layers[0] = last;
            }

            // ByteSil - Additional
            id if id == StdlibIntrinsic::BytesilAdd as u8 => {
                // Approximate addition in log-polar: mix magnitudes
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil {
                    rho: ((l0.rho as i16 + l1.rho as i16) / 2) as i8,
                    theta: l0.theta, // Keep first phase
                };
            }
            id if id == StdlibIntrinsic::BytesilSub as u8 => {
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil {
                    rho: l0.rho.saturating_sub(l1.rho),
                    theta: l0.theta,
                };
            }
            id if id == StdlibIntrinsic::BytesilScale as u8 => {
                // Scale magnitude by L1.rho
                let l0 = state.layers[0];
                let scale = state.layers[1].rho;
                state.layers[0] = ByteSil::new(
                    l0.rho.saturating_add(scale),
                    l0.theta,
                );
            }
            id if id == StdlibIntrinsic::BytesilRotate as u8 => {
                // Rotate phase by L1.rho
                let l0 = state.layers[0];
                let delta = state.layers[1].rho as u8;
                state.layers[0] = ByteSil::new(l0.rho, l0.theta.wrapping_add(delta));
            }

            // Math Functions - Basic
            id if id == StdlibIntrinsic::MathSin as u8 => {
                let angle = state.layers[0].theta as f64 * std::f64::consts::PI / 8.0;
                let result = angle.sin();
                state.layers[0] = ByteSil { rho: (result * 127.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathCos as u8 => {
                let angle = state.layers[0].theta as f64 * std::f64::consts::PI / 8.0;
                let result = angle.cos();
                state.layers[0] = ByteSil { rho: (result * 127.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathTan as u8 => {
                let angle = state.layers[0].theta as f64 * std::f64::consts::PI / 8.0;
                let result = angle.tan().clamp(-127.0, 127.0);
                state.layers[0] = ByteSil { rho: result as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathSqrt as u8 => {
                let val = (state.layers[0].rho.abs() as f64).sqrt();
                state.layers[0] = ByteSil { rho: val as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathPow as u8 => {
                let base = state.layers[0].rho as f64;
                let exp = state.layers[1].rho as f64;
                let result = base.powf(exp).clamp(-127.0, 127.0);
                state.layers[0] = ByteSil { rho: result as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathLog as u8 => {
                let val = (state.layers[0].rho.abs().max(1) as f64).ln();
                state.layers[0] = ByteSil { rho: (val * 10.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathExp as u8 => {
                let val = (state.layers[0].rho as f64).exp().clamp(-127.0, 127.0);
                state.layers[0] = ByteSil { rho: val as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathAbs as u8 => {
                state.layers[0].rho = state.layers[0].rho.abs();
            }
            id if id == StdlibIntrinsic::MathFloor as u8 => {
                // Already integer, no-op
            }
            id if id == StdlibIntrinsic::MathCeil as u8 => {
                // Already integer, no-op
            }
            id if id == StdlibIntrinsic::MathRound as u8 => {
                // Already integer, no-op
            }
            id if id == StdlibIntrinsic::MathMin as u8 => {
                let a = state.layers[0].rho;
                let b = state.layers[1].rho;
                state.layers[0] = ByteSil { rho: a.min(b), theta: 0 };
            }
            id if id == StdlibIntrinsic::MathMax as u8 => {
                let a = state.layers[0].rho;
                let b = state.layers[1].rho;
                state.layers[0] = ByteSil { rho: a.max(b), theta: 0 };
            }

            // Math Constants
            id if id == StdlibIntrinsic::MathPi as u8 => {
                // π ≈ 3.14159... store scaled: 31
                state.layers[0] = ByteSil { rho: 31, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathTau as u8 => {
                // τ = 2π ≈ 6.28... store scaled: 63
                state.layers[0] = ByteSil { rho: 63, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathE as u8 => {
                // e ≈ 2.718... store scaled: 27
                state.layers[0] = ByteSil { rho: 27, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathPhi as u8 => {
                // φ ≈ 1.618... store scaled: 16
                state.layers[0] = ByteSil { rho: 16, theta: 0 };
            }

            // Math - Inverse Trig
            id if id == StdlibIntrinsic::MathAsin as u8 => {
                let val = (state.layers[0].rho as f64 / 127.0).clamp(-1.0, 1.0);
                let result = val.asin();
                state.layers[0] = ByteSil { rho: (result * 127.0 / std::f64::consts::PI) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathAcos as u8 => {
                let val = (state.layers[0].rho as f64 / 127.0).clamp(-1.0, 1.0);
                let result = val.acos();
                state.layers[0] = ByteSil { rho: (result * 127.0 / std::f64::consts::PI) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathAtan as u8 => {
                let val = state.layers[0].rho as f64 / 10.0;
                let result = val.atan();
                state.layers[0] = ByteSil { rho: (result * 127.0 / std::f64::consts::PI) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathAtan2 as u8 => {
                let y = state.layers[0].rho as f64;
                let x = state.layers[1].rho as f64;
                let result = y.atan2(x);
                state.layers[0] = ByteSil { rho: (result * 127.0 / std::f64::consts::PI) as i8, theta: 0 };
            }

            // Math - Logarithms
            id if id == StdlibIntrinsic::MathLn as u8 => {
                let val = (state.layers[0].rho.abs().max(1) as f64).ln();
                state.layers[0] = ByteSil { rho: (val * 10.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathLog10 as u8 => {
                let val = (state.layers[0].rho.abs().max(1) as f64).log10();
                state.layers[0] = ByteSil { rho: (val * 10.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MathLog2 as u8 => {
                let val = (state.layers[0].rho.abs().max(1) as f64).log2();
                state.layers[0] = ByteSil { rho: (val * 10.0) as i8, theta: 0 };
            }

            // Math - Type-specific
            id if id == StdlibIntrinsic::MathAbsInt as u8 => {
                state.layers[0].rho = state.layers[0].rho.abs();
            }
            id if id == StdlibIntrinsic::MathAbsFloat as u8 => {
                state.layers[0].rho = state.layers[0].rho.abs();
            }
            id if id == StdlibIntrinsic::MathMinInt as u8 => {
                let a = state.layers[0].rho;
                let b = state.layers[1].rho;
                state.layers[0] = ByteSil { rho: a.min(b), theta: 0 };
            }
            id if id == StdlibIntrinsic::MathMaxInt as u8 => {
                let a = state.layers[0].rho;
                let b = state.layers[1].rho;
                state.layers[0] = ByteSil { rho: a.max(b), theta: 0 };
            }
            id if id == StdlibIntrinsic::MathClampInt as u8 => {
                let val = state.layers[0].rho;
                let min_val = state.layers[1].rho;
                let max_val = state.layers[2].rho;
                state.layers[0] = ByteSil { rho: val.clamp(min_val, max_val), theta: 0 };
            }
            id if id == StdlibIntrinsic::MathPowFloat as u8 => {
                let base = state.layers[0].rho as f64;
                let exp = state.layers[1].rho as f64;
                let result = base.powf(exp).clamp(-127.0, 127.0);
                state.layers[0] = ByteSil { rho: result as i8, theta: 0 };
            }

            // String Functions (simplified - strings stored as layer values)
            id if id == StdlibIntrinsic::StringLen as u8 => {
                // Return a dummy length
                state.layers[0] = ByteSil { rho: 0, theta: 0 };
            }
            id if id == StdlibIntrinsic::StringConcat as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringSubstr as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringSlice as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringContains as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringReplace as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringSplit as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringTrim as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringToUpper as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringToLower as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringStartsWith as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringEndsWith as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::IntToString as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::FloatToString as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringToInt as u8 => { /* String ops: pass through */ }
            id if id == StdlibIntrinsic::StringToFloat as u8 => { /* String ops: pass through */ }

            // Type conversions
            id if id == StdlibIntrinsic::FloatFromInt as u8 => {
                // Convert L0.rho (as integer) to float, store back in L0
                // In ByteSil, we store the value in rho directly
                // (No change needed since we're operating on the same rho)
            }
            id if id == StdlibIntrinsic::IntFromFloat as u8 => {
                // Convert L0.rho (as float) to int, store back in L0
                // Truncate fractional part (no-op in ByteSil since rho is already i8)
            }
            id if id == StdlibIntrinsic::FloatToInt as u8 => {
                // Alias for IntFromFloat
            }
            id if id == StdlibIntrinsic::IntToFloat as u8 => {
                // Alias for FloatFromInt
            }

            // Transform Functions
            id if id == StdlibIntrinsic::ApplyFeedback as u8 => {
                // Apply feedback: multiply all layers by gain (L1.rho / 10)
                let gain = state.layers[1].rho as f64 / 10.0;
                for i in 0..16 {
                    let new_rho = (state.layers[i].rho as f64 * gain) as i8;
                    state.layers[i].rho = new_rho;
                }
            }
            id if id == StdlibIntrinsic::DetectEmergence as u8 => {
                // Detect emergence: check if activity exceeds threshold
                let threshold = state.layers[1].rho as f64 / 10.0;
                let mut total: f64 = 0.0;
                for layer in &state.layers {
                    total += layer.rho.abs() as f64;
                }
                let avg = total / 16.0 / 127.0;
                state.layers[0] = ByteSil { rho: (avg > threshold) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::TransformPhaseShift as u8 => {
                // Shift phase of all layers by L1.rho
                let delta = state.layers[1].rho as u8;
                for i in 0..16 {
                    state.layers[i].theta = state.layers[i].theta.wrapping_add(delta);
                }
            }
            id if id == StdlibIntrinsic::TransformMagnitudeScale as u8 => {
                // Scale magnitude of all layers by L1.rho
                let scale = state.layers[1].rho;
                for i in 0..16 {
                    state.layers[i].rho = state.layers[i].rho.saturating_add(scale);
                }
            }
            id if id == StdlibIntrinsic::TransformIdentity as u8 => {
                // Identity: do nothing
            }
            id if id == StdlibIntrinsic::TransformLayerSwap as u8 => {
                // Swap layers L1.rho and L2.rho
                let a = (state.layers[1].rho as usize) & 0x0F;
                let b = (state.layers[2].rho as usize) & 0x0F;
                state.layers.swap(a, b);
            }
            id if id == StdlibIntrinsic::TransformXorLayers as u8 => {
                // XOR L1 and L2, store in L3
                let a = (state.layers[1].rho as usize) & 0x0F;
                let b = (state.layers[2].rho as usize) & 0x0F;
                let dest = (state.layers[3].rho as usize) & 0x0F;
                state.layers[dest] = state.layers[a] ^ state.layers[b];
            }
            id if id == StdlibIntrinsic::ShiftLayersUp as u8 => {
                state.layers.rotate_left(1);
            }
            id if id == StdlibIntrinsic::ShiftLayersDown as u8 => {
                state.layers.rotate_right(1);
            }
            id if id == StdlibIntrinsic::NormalizePerception as u8 => {
                // Normalize layers 0-4 (perception)
                let mut max_rho: i8 = 1;
                for i in 0..5 {
                    max_rho = max_rho.max(state.layers[i].rho.abs());
                }
                if max_rho > 0 {
                    for i in 0..5 {
                        state.layers[i].rho = (state.layers[i].rho as i16 * 7 / max_rho as i16) as i8;
                    }
                }
            }

            // Debug Functions
            id if id == StdlibIntrinsic::DebugPrint as u8 => {
                eprintln!("[DEBUG] L0: ρ={}, θ={}", state.layers[0].rho, state.layers[0].theta);
            }
            id if id == StdlibIntrinsic::AssertEq as u8 => {
                if state.layers[0] != state.layers[1] {
                    return Err(VspError::Other(format!(
                        "Assertion failed: {:?} != {:?}",
                        state.layers[0], state.layers[1]
                    )));
                }
            }
            id if id == StdlibIntrinsic::AssertTrue as u8 => {
                if state.layers[0].rho == 0 {
                    return Err(VspError::Other("Assertion failed: expected true".to_string()));
                }
            }
            id if id == StdlibIntrinsic::AssertFalse as u8 => {
                if state.layers[0].rho != 0 {
                    return Err(VspError::Other("Assertion failed: expected false".to_string()));
                }
            }

            // HTTP Functions - stubs (require async runtime)
            id if (0xA0..=0xB0).contains(&id) => {
                // HTTP operations require async runtime - no-op in simple interpreter
                // 0xA0-0xB0 covers HttpGet through HttpStatusText
            }

            // Layer Functions
            id if id == StdlibIntrinsic::FuseVisionAudio as u8 => {
                // Fuse L0 (vision) and L1 (audio) using mix
                let vision = state.layers[0];
                let audio = state.layers[1];
                state.layers[0] = ByteSil {
                    rho: ((vision.rho as i16 + audio.rho as i16) / 2) as i8,
                    theta: vision.theta ^ audio.theta,
                };
            }
            id if id == StdlibIntrinsic::FuseMultimodal as u8 => {
                // Fuse first 5 perception layers
                let mut rho_sum: i16 = 0;
                let mut theta_xor: u8 = 0;
                for i in 0..5 {
                    rho_sum += state.layers[i].rho as i16;
                    theta_xor ^= state.layers[i].theta;
                }
                state.layers[0] = ByteSil {
                    rho: (rho_sum / 5) as i8,
                    theta: theta_xor,
                };
            }
            id if id == StdlibIntrinsic::RotateLayers as u8 => {
                let amount = (state.layers[1].rho as usize) % 16;
                state.layers.rotate_left(amount);
            }
            id if id == StdlibIntrinsic::SpreadToGroup as u8 => {
                // Spread L0 value to layers starting at L1.rho, count L2.rho
                let start = (state.layers[1].rho as usize) & 0x0F;
                let count = (state.layers[2].rho.abs() as usize).min(16 - start);
                let value = state.layers[0];
                for i in 0..count {
                    state.layers[(start + i) & 0x0F] = value;
                }
            }
            id if id == StdlibIntrinsic::EmergencePattern as u8 => {
                // Extract emergence pattern from layers 11-12
                state.layers[0] = ByteSil {
                    rho: ((state.layers[11].rho as i16 + state.layers[12].rho as i16) / 2) as i8,
                    theta: state.layers[11].theta ^ state.layers[12].theta,
                };
            }
            id if id == StdlibIntrinsic::AutopoieticLoop as u8 => {
                // Run autopoietic feedback for L1.rho iterations
                let iterations = state.layers[1].rho.abs().min(10) as u8;
                for _ in 0..iterations {
                    // XOR all layers into temp
                    let mut collapsed = state.layers[0];
                    for i in 1..16 {
                        collapsed = collapsed ^ state.layers[i];
                    }
                    // Mix collapsed back into each layer
                    for i in 0..16 {
                        state.layers[i] = ByteSil {
                            rho: ((state.layers[i].rho as i16 + collapsed.rho as i16) / 2) as i8,
                            theta: state.layers[i].theta ^ collapsed.theta,
                        };
                    }
                }
            }

            // Complex Math
            id if id == StdlibIntrinsic::ComplexAdd as u8 => {
                // Approximate complex addition
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil {
                    rho: ((l0.rho as i16 + l1.rho as i16) / 2) as i8,
                    theta: ((l0.theta as u16 + l1.theta as u16) / 2) as u8,
                };
            }
            id if id == StdlibIntrinsic::ComplexSub as u8 => {
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                state.layers[0] = ByteSil {
                    rho: l0.rho.saturating_sub(l1.rho),
                    theta: l0.theta.wrapping_sub(l1.theta),
                };
            }
            id if id == StdlibIntrinsic::ComplexScale as u8 => {
                let scale = state.layers[1].rho;
                state.layers[0].rho = state.layers[0].rho.saturating_add(scale);
            }
            id if id == StdlibIntrinsic::ComplexRotate as u8 => {
                let delta = state.layers[1].rho as u8;
                state.layers[0].theta = state.layers[0].theta.wrapping_add(delta);
            }
            id if id == StdlibIntrinsic::ComplexLerp as u8 => {
                let l0 = state.layers[0];
                let l1 = state.layers[1];
                let t = (state.layers[2].rho as f64 / 127.0).clamp(0.0, 1.0);
                state.layers[0] = ByteSil {
                    rho: (l0.rho as f64 * (1.0 - t) + l1.rho as f64 * t) as i8,
                    theta: (l0.theta as f64 * (1.0 - t) + l1.theta as f64 * t) as u8,
                };
            }
            id if id == StdlibIntrinsic::DegreesToRadians as u8 => {
                // degrees * π/180
                let deg = state.layers[0].rho as f64;
                let rad = deg * std::f64::consts::PI / 180.0;
                state.layers[0] = ByteSil { rho: (rad * 40.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::RadiansToDegrees as u8 => {
                // radians * 180/π
                let rad = state.layers[0].rho as f64 / 40.0;
                let deg = rad * 180.0 / std::f64::consts::PI;
                state.layers[0] = ByteSil { rho: deg.clamp(-127.0, 127.0) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::ClampFloat as u8 => {
                let val = state.layers[0].rho;
                let min_val = state.layers[1].rho;
                let max_val = state.layers[2].rho;
                state.layers[0] = ByteSil { rho: val.clamp(min_val, max_val), theta: 0 };
            }
            id if id == StdlibIntrinsic::MinFloat as u8 => {
                let a = state.layers[0].rho;
                let b = state.layers[1].rho;
                state.layers[0] = ByteSil { rho: a.min(b), theta: 0 };
            }
            id if id == StdlibIntrinsic::MaxFloat as u8 => {
                let a = state.layers[0].rho;
                let b = state.layers[1].rho;
                state.layers[0] = ByteSil { rho: a.max(b), theta: 0 };
            }
            id if id == StdlibIntrinsic::SignFloat as u8 => {
                state.layers[0] = ByteSil { rho: state.layers[0].rho.signum(), theta: 0 };
            }
            id if id == StdlibIntrinsic::SignInt as u8 => {
                state.layers[0] = ByteSil { rho: state.layers[0].rho.signum(), theta: 0 };
            }

            // Extended Debug Functions
            id if id == StdlibIntrinsic::TraceState as u8 => {
                eprintln!("[TRACE] State:");
                for (i, layer) in state.layers.iter().enumerate() {
                    eprintln!("  L{:X}: ρ={:+4}, θ={:3}", i, layer.rho, layer.theta);
                }
            }
            id if id == StdlibIntrinsic::TimestampMillis as u8 => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i8)
                    .unwrap_or(0);
                state.layers[0] = ByteSil { rho: now, theta: 0 };
            }
            id if id == StdlibIntrinsic::TimestampMicros as u8 => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_micros() as i8)
                    .unwrap_or(0);
                state.layers[0] = ByteSil { rho: now, theta: 0 };
            }
            id if id == StdlibIntrinsic::SleepMillis as u8 => {
                let duration = state.layers[0].rho.abs() as u64;
                std::thread::sleep(std::time::Duration::from_millis(duration.min(1000)));
            }
            id if id == StdlibIntrinsic::MemoryUsed as u8 => {
                // Placeholder
                state.layers[0] = ByteSil { rho: 0, theta: 0 };
            }
            id if id == StdlibIntrinsic::AssertEqInt as u8 => {
                if state.layers[0].rho != state.layers[1].rho {
                    return Err(VspError::Other(format!(
                        "Assertion failed: {} != {}",
                        state.layers[0].rho, state.layers[1].rho
                    )));
                }
            }
            id if id == StdlibIntrinsic::AssertEqBytesil as u8 => {
                if state.layers[0] != state.layers[1] {
                    return Err(VspError::Other(format!(
                        "Assertion failed: ByteSil values not equal"
                    )));
                }
            }
            id if id == StdlibIntrinsic::AssertEqState as u8 => {
                // Can't compare full states easily, just check first layer
                if state.layers[0] != state.layers[1] {
                    return Err(VspError::Other(format!(
                        "Assertion failed: State values not equal"
                    )));
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // Energy & Benchmark Metrics
            // ═══════════════════════════════════════════════════════════════════
            id if id == StdlibIntrinsic::EnergyBegin as u8 => {
                let mut meter_guard = ENERGY_METER.lock().unwrap();
                if meter_guard.is_none() {
                    *meter_guard = Some(EnergyMeter::new(Box::new(CpuEnergyModel::detect())));
                }
                if let Some(ref mut meter) = *meter_guard {
                    let _ = meter.begin_measurement();
                }
            }
            id if id == StdlibIntrinsic::EnergyEndJoules as u8 => {
                let mut meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref mut meter) = *meter_guard {
                    if let Ok(snapshot) = meter.end_measurement(1000) {
                        // Store joules as scaled value (nanojoules fit in i8 for small values)
                        // For larger values, use multiple layers
                        let nanojoules = (snapshot.joules * 1e9) as u64;
                        state.layers[0] = ByteSil { rho: (nanojoules & 0xFF) as i8, theta: 0 };
                        state.layers[1] = ByteSil { rho: ((nanojoules >> 8) & 0xFF) as i8, theta: 0 };
                        state.layers[2] = ByteSil { rho: ((nanojoules >> 16) & 0xFF) as i8, theta: 0 };
                        state.layers[3] = ByteSil { rho: ((nanojoules >> 24) & 0xFF) as i8, theta: 0 };
                    }
                }
            }
            id if id == StdlibIntrinsic::EnergyEndWatts as u8 => {
                let mut meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref mut meter) = *meter_guard {
                    if let Ok(snapshot) = meter.end_measurement(1000) {
                        // Store watts as milliwatts
                        let milliwatts = (snapshot.watts * 1000.0) as u32;
                        state.layers[0] = ByteSil { rho: (milliwatts & 0xFF) as i8, theta: 0 };
                        state.layers[1] = ByteSil { rho: ((milliwatts >> 8) & 0xFF) as i8, theta: 0 };
                        state.layers[2] = ByteSil { rho: ((milliwatts >> 16) & 0xFF) as i8, theta: 0 };
                        state.layers[3] = ByteSil { rho: ((milliwatts >> 24) & 0xFF) as i8, theta: 0 };
                    }
                }
            }
            id if id == StdlibIntrinsic::EnergyEndSamplesPerJoule as u8 => {
                let meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref meter) = *meter_guard {
                    let efficiency = meter.efficiency();
                    let samples_per_joule = efficiency as u32;
                    state.layers[0] = ByteSil { rho: (samples_per_joule & 0xFF) as i8, theta: 0 };
                    state.layers[1] = ByteSil { rho: ((samples_per_joule >> 8) & 0xFF) as i8, theta: 0 };
                    state.layers[2] = ByteSil { rho: ((samples_per_joule >> 16) & 0xFF) as i8, theta: 0 };
                    state.layers[3] = ByteSil { rho: ((samples_per_joule >> 24) & 0xFF) as i8, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::CarbonFootprintGrams as u8 => {
                let meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref meter) = *meter_guard {
                    // gCO2e = (Joules / 3600) * (gCO2e/Wh) = Joules * carbon_intensity / 3.6
                    let joules = meter.total_joules();
                    let gco2e = joules * CARBON_INTENSITY_BRAZIL / 3.6;
                    let micrograms = (gco2e * 1e6) as u32;
                    state.layers[0] = ByteSil { rho: (micrograms & 0xFF) as i8, theta: 0 };
                    state.layers[1] = ByteSil { rho: ((micrograms >> 8) & 0xFF) as i8, theta: 0 };
                    state.layers[2] = ByteSil { rho: ((micrograms >> 16) & 0xFF) as i8, theta: 0 };
                    state.layers[3] = ByteSil { rho: ((micrograms >> 24) & 0xFF) as i8, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::TimestampNanosLow as u8 => {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                let low = (nanos & 0xFFFFFFFF) as u32;
                state.layers[0] = ByteSil { rho: (low & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((low >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((low >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((low >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::TimestampNanosHigh as u8 => {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                let high = ((nanos >> 32) & 0xFFFFFFFF) as u32;
                state.layers[0] = ByteSil { rho: (high & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((high >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((high >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((high >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::BenchmarkStart as u8 => {
                let mut bench = BENCHMARK_STATE.lock().unwrap();
                bench.start_time = Some(Instant::now());
                bench.total_samples = 0;
                bench.total_flops = 0;
                bench.total_macs = 0;
                bench.total_latency_us = 0;
            }
            id if id == StdlibIntrinsic::BenchmarkEnd as u8 => {
                let bench = BENCHMARK_STATE.lock().unwrap();
                if let Some(start) = bench.start_time {
                    let elapsed_us = start.elapsed().as_micros() as u64;
                    // Store elapsed time in layers
                    state.layers[0] = ByteSil { rho: (elapsed_us & 0xFF) as i8, theta: 0 };
                    state.layers[1] = ByteSil { rho: ((elapsed_us >> 8) & 0xFF) as i8, theta: 0 };
                    state.layers[2] = ByteSil { rho: ((elapsed_us >> 16) & 0xFF) as i8, theta: 0 };
                    state.layers[3] = ByteSil { rho: ((elapsed_us >> 24) & 0xFF) as i8, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::FlopCount as u8 => {
                let bench = BENCHMARK_STATE.lock().unwrap();
                let flops = bench.total_flops as u32;
                state.layers[0] = ByteSil { rho: (flops & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((flops >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((flops >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((flops >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MacCount as u8 => {
                let bench = BENCHMARK_STATE.lock().unwrap();
                let macs = bench.total_macs as u32;
                state.layers[0] = ByteSil { rho: (macs & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((macs >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((macs >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((macs >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::ThroughputSamplesPerSec as u8 => {
                let bench = BENCHMARK_STATE.lock().unwrap();
                if let Some(start) = bench.start_time {
                    let elapsed_secs = start.elapsed().as_secs_f64();
                    let throughput = if elapsed_secs > 0.0 {
                        (bench.total_samples as f64 / elapsed_secs) as u32
                    } else {
                        0
                    };
                    state.layers[0] = ByteSil { rho: (throughput & 0xFF) as i8, theta: 0 };
                    state.layers[1] = ByteSil { rho: ((throughput >> 8) & 0xFF) as i8, theta: 0 };
                    state.layers[2] = ByteSil { rho: ((throughput >> 16) & 0xFF) as i8, theta: 0 };
                    state.layers[3] = ByteSil { rho: ((throughput >> 24) & 0xFF) as i8, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::LatencyMicros as u8 => {
                let bench = BENCHMARK_STATE.lock().unwrap();
                let avg_latency = if bench.total_samples > 0 {
                    (bench.total_latency_us / bench.total_samples) as u32
                } else {
                    0
                };
                state.layers[0] = ByteSil { rho: (avg_latency & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((avg_latency >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((avg_latency >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((avg_latency >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::MemoryPeakBytes as u8 => {
                let bench = BENCHMARK_STATE.lock().unwrap();
                let peak = bench.peak_memory_bytes as u32;
                state.layers[0] = ByteSil { rho: (peak & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((peak >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((peak >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((peak >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::AccuracyPerWatt as u8 => {
                // Read accuracy from L0 (as percentage * 100)
                let accuracy = state.layers[0].rho as f64 / 100.0;
                let meter_guard = ENERGY_METER.lock().unwrap();
                let watts = if let Some(ref meter) = *meter_guard {
                    meter.current_watts().max(0.001) // Avoid division by zero
                } else {
                    1.0
                };
                let acc_per_watt = (accuracy / watts * 1000.0) as u32; // milli-accuracy per watt
                state.layers[0] = ByteSil { rho: (acc_per_watt & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((acc_per_watt >> 8) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::EnergyDelayProduct as u8 => {
                let meter_guard = ENERGY_METER.lock().unwrap();
                let bench = BENCHMARK_STATE.lock().unwrap();
                let joules = if let Some(ref meter) = *meter_guard {
                    meter.total_joules()
                } else {
                    0.0
                };
                let latency_s = if bench.total_samples > 0 {
                    bench.total_latency_us as f64 / bench.total_samples as f64 / 1e6
                } else {
                    0.0
                };
                let edp = (joules * latency_s * 1e12) as u32; // picojoule-seconds
                state.layers[0] = ByteSil { rho: (edp & 0xFF) as i8, theta: 0 };
                state.layers[1] = ByteSil { rho: ((edp >> 8) & 0xFF) as i8, theta: 0 };
                state.layers[2] = ByteSil { rho: ((edp >> 16) & 0xFF) as i8, theta: 0 };
                state.layers[3] = ByteSil { rho: ((edp >> 24) & 0xFF) as i8, theta: 0 };
            }
            id if id == StdlibIntrinsic::TimestampNanosState as u8 => {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                // Distribute across all 16 layers for full 128-bit precision
                for i in 0..16 {
                    let byte = ((nanos >> (i * 8)) & 0xFF) as i8;
                    state.layers[i] = ByteSil { rho: byte, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::EnergyTotalJoules as u8 => {
                let meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref meter) = *meter_guard {
                    let nanojoules = (meter.total_joules() * 1e9) as u64;
                    state.layers[0] = ByteSil { rho: (nanojoules & 0xFF) as i8, theta: 0 };
                    state.layers[1] = ByteSil { rho: ((nanojoules >> 8) & 0xFF) as i8, theta: 0 };
                    state.layers[2] = ByteSil { rho: ((nanojoules >> 16) & 0xFF) as i8, theta: 0 };
                    state.layers[3] = ByteSil { rho: ((nanojoules >> 24) & 0xFF) as i8, theta: 0 };
                    state.layers[4] = ByteSil { rho: ((nanojoules >> 32) & 0xFF) as i8, theta: 0 };
                    state.layers[5] = ByteSil { rho: ((nanojoules >> 40) & 0xFF) as i8, theta: 0 };
                    state.layers[6] = ByteSil { rho: ((nanojoules >> 48) & 0xFF) as i8, theta: 0 };
                    state.layers[7] = ByteSil { rho: ((nanojoules >> 56) & 0xFF) as i8, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::EnergyEfficiency as u8 => {
                let meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref meter) = *meter_guard {
                    let efficiency = (meter.efficiency() * 1000.0) as u32; // milli-ops per joule
                    state.layers[0] = ByteSil { rho: (efficiency & 0xFF) as i8, theta: 0 };
                    state.layers[1] = ByteSil { rho: ((efficiency >> 8) & 0xFF) as i8, theta: 0 };
                    state.layers[2] = ByteSil { rho: ((efficiency >> 16) & 0xFF) as i8, theta: 0 };
                    state.layers[3] = ByteSil { rho: ((efficiency >> 24) & 0xFF) as i8, theta: 0 };
                }
            }
            id if id == StdlibIntrinsic::BenchmarkReset as u8 => {
                let mut meter_guard = ENERGY_METER.lock().unwrap();
                if let Some(ref mut meter) = *meter_guard {
                    meter.reset();
                }
                let mut bench = BENCHMARK_STATE.lock().unwrap();
                *bench = BenchmarkMetrics::default();
            }

            _ => {
                // Unknown intrinsic - no-op
            }
        }

        Ok(())
    }
}

impl Default for VspInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Set the global string table for print_string operations
pub fn set_string_table(strings: Vec<String>) {
    if let Ok(mut table) = STRING_TABLE.lock() {
        *table = strings;
    }
}

/// Clear the global string table
pub fn clear_string_table() {
    if let Ok(mut table) = STRING_TABLE.lock() {
        table.clear();
    }
}

/// Get a copy of the current string table
pub fn get_string_table() -> Vec<String> {
    STRING_TABLE.lock().map(|t| t.clone()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0x00, 0x01], // NOP, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        assert!(interp.compile(&program).is_ok());
        assert_eq!(interp.handlers.len(), 2);
    }

    #[test]
    fn test_execute_nop() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0x00, 0x01],
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        assert!(interp.execute(&mut state).is_ok());
    }

    #[test]
    fn test_mov() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0x20, 0x01], // MOV, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        state.layers[0] = ByteSil { rho: 5, theta: 10 };
        state.layers[1] = ByteSil { rho: 3, theta: 6 };

        interp.execute(&mut state).unwrap();

        // After MOV, layers should be swapped
        assert_eq!(state.layers[0].rho, 3);
        assert_eq!(state.layers[0].theta, 6);
        assert_eq!(state.layers[1].rho, 5);
        assert_eq!(state.layers[1].theta, 10);
    }

    #[test]
    fn test_mul_precise() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0x40, 0x01], // MUL, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        state.layers[0] = ByteSil { rho: 5, theta: 10 };
        state.layers[1] = ByteSil { rho: 3, theta: 6 };

        interp.execute(&mut state).unwrap();

        // MUL: (5+3, 10+6 mod 16) = (8, 0)
        assert_eq!(state.layers[0].rho, 8);
        assert_eq!(state.layers[0].theta, 0); // 16 mod 16 = 0
    }

    #[test]
    fn test_xorl() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0x60, 0x01], // XORL, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        state.layers[0] = ByteSil { rho: 5, theta: 10 };
        state.layers[1] = ByteSil { rho: 3, theta: 6 };

        interp.execute(&mut state).unwrap();

        // XOR: (5^3, 10^6) = (6, 12)
        assert_eq!(state.layers[0].rho, 6);
        assert_eq!(state.layers[0].theta, 12);
    }

    #[test]
    fn test_collapse() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0xA0, 0x01], // COLLAPSE, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        for i in 0..16 {
            state.layers[i] = ByteSil {
                rho: i as i8,
                theta: i as u8,
            };
        }

        interp.execute(&mut state).unwrap();

        // Collapse XORs all layers
        // Expected: 0^1^2^...^15 = 0 (for rho), 0^1^2^...^15 = 0 (for theta)
        let expected_rho = (0..16i8).fold(0i8, |acc, x| acc ^ x);
        let expected_theta = (0..16u8).fold(0u8, |acc, x| acc ^ x);
        assert_eq!(state.layers[0].rho, expected_rho);
        assert_eq!(state.layers[0].theta, expected_theta);
    }

    #[test]
    fn test_rotate() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0x4B, 0x01], // ROTATE, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        state.layers[0] = ByteSil { rho: 3, theta: 8 };

        interp.execute(&mut state).unwrap();

        // ROTATE increments theta
        assert_eq!(state.layers[0].theta, 9);
    }

    #[test]
    fn test_bit_hadamard() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0xB0, 0x00, 0x01], // BitHadamard L0, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        // Usar estado com fase != 0 para que Hadamard produza mudanca observavel
        state.layers[0] = ByteSil { rho: 0, theta: 4 }; // theta = pi/2

        // Verificar que o opcode executa sem erro
        let result = interp.execute(&mut state);
        assert!(result.is_ok(), "BitHadamard deveria executar sem erro");

        // O opcode foi registrado e executado corretamente
        assert_eq!(interp.handlers.len(), 2); // BitHadamard + HLT
    }

    #[test]
    fn test_bit_pauli_x() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0xB1, 0x00, 0x01], // BitPauliX L0, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        state.layers[0] = ByteSil::ONE; // |0⟩

        interp.execute(&mut state).unwrap();

        // Pauli-X flipa o estado
        let bit = BitDeSil::from_byte_sil(&state.layers[0]);
        // Apos X, deve estar mais proximo de |1⟩
        assert!(bit.classical || bit.prob_one() > 0.4);
    }

    #[test]
    fn test_bit_normalize() {
        let mut interp = VspInterpreter::new();
        let program = SilcFile {
            header: crate::vsp::SilcHeader::new(crate::vsp::SilMode::Sil128),
            code: vec![0xB7, 0x00, 0x01], // BitNormalize L0, HLT
            data: vec![],
            symbols: vec![],
            debug_info: None,
        };
        interp.compile(&program).unwrap();

        let mut state = SilState::vacuum();
        state.layers[0] = ByteSil { rho: 3, theta: 4 };

        interp.execute(&mut state).unwrap();

        // Normalize deve preservar o valor (ja normalizado)
        let bit = BitDeSil::from_byte_sil(&state.layers[0]);
        let total_prob = bit.prob_zero() + bit.prob_one();
        assert!((total_prob - 1.0).abs() < 0.1);
    }
}
