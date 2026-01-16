//! Debugger visual para VSP
//!
//! Fornece capacidades de debugging:
//! - Breakpoints (condicionais)
//! - Step-through execution
//! - Inspeção de estado
//! - Watch expressions
//! - Call stack

use std::collections::{HashMap, HashSet};
use std::fmt;

use super::{
    Vsp, VspConfig, VspResult,
    Opcode,
    state::VspState,
};
use crate::state::ByteSil;

// ═══════════════════════════════════════════════════════════════════════════════
// BREAKPOINT
// ═══════════════════════════════════════════════════════════════════════════════

/// Tipo de breakpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
    /// Breakpoint de execução (para no endereço)
    Execution,
    /// Breakpoint de leitura de memória
    Read,
    /// Breakpoint de escrita de memória
    Write,
    /// Breakpoint de acesso (leitura ou escrita)
    Access,
}

/// Condição de breakpoint
#[derive(Debug, Clone)]
pub enum BreakCondition {
    /// Sempre para
    Always,
    /// Para quando registrador == valor
    RegisterEquals(u8, u8),
    /// Para quando registrador != valor
    RegisterNotEquals(u8, u8),
    /// Para após N hits
    HitCount(u32),
    /// Expressão customizada
    Expression(String),
}

/// Breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// ID único
    pub id: u32,
    /// Endereço
    pub address: u32,
    /// Tipo
    pub bp_type: BreakpointType,
    /// Condição
    pub condition: BreakCondition,
    /// Habilitado
    pub enabled: bool,
    /// Contador de hits
    pub hit_count: u32,
    /// Log message (logpoint)
    pub log_message: Option<String>,
    /// Temporário (remove após primeiro hit)
    pub temporary: bool,
}

impl Breakpoint {
    pub fn new(id: u32, address: u32) -> Self {
        Self {
            id,
            address,
            bp_type: BreakpointType::Execution,
            condition: BreakCondition::Always,
            enabled: true,
            hit_count: 0,
            log_message: None,
            temporary: false,
        }
    }
    
    pub fn with_type(mut self, bp_type: BreakpointType) -> Self {
        self.bp_type = bp_type;
        self
    }
    
    pub fn with_condition(mut self, condition: BreakCondition) -> Self {
        self.condition = condition;
        self
    }
    
    pub fn with_log(mut self, message: String) -> Self {
        self.log_message = Some(message);
        self
    }
    
    pub fn temporary(mut self) -> Self {
        self.temporary = true;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WATCH
// ═══════════════════════════════════════════════════════════════════════════════

/// Watch expression
#[derive(Debug, Clone)]
pub struct Watch {
    /// ID único
    pub id: u32,
    /// Expressão
    pub expression: WatchExpression,
    /// Último valor
    pub last_value: Option<WatchValue>,
}

/// Tipo de expressão de watch
#[derive(Debug, Clone)]
pub enum WatchExpression {
    /// Registrador (R0-R15)
    Register(u8),
    /// Memória em endereço
    Memory(u32),
    /// Range de memória
    MemoryRange(u32, u32),
    /// Estado completo
    State,
    /// PC
    ProgramCounter,
    /// SP
    StackPointer,
    /// Flags
    Flags,
    /// Expressão customizada
    Custom(String),
}

/// Valor de watch
#[derive(Debug, Clone)]
pub enum WatchValue {
    Byte(u8),
    Word(u16),
    Dword(u32),
    Qword(u64),
    Bytes(Vec<u8>),
    State(Box<VspState>),
    Text(String),
}

impl fmt::Display for WatchValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatchValue::Byte(v) => write!(f, "0x{:02X}", v),
            WatchValue::Word(v) => write!(f, "0x{:04X}", v),
            WatchValue::Dword(v) => write!(f, "0x{:08X}", v),
            WatchValue::Qword(v) => write!(f, "0x{:016X}", v),
            WatchValue::Bytes(b) => {
                write!(f, "[")?;
                for (i, byte) in b.iter().take(16).enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{:02X}", byte)?;
                }
                if b.len() > 16 {
                    write!(f, " ...")?;
                }
                write!(f, "]")
            }
            WatchValue::State(s) => write!(f, "State {{ pc: 0x{:08X} }}", s.pc),
            WatchValue::Text(t) => write!(f, "{}", t),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STACK FRAME
// ═══════════════════════════════════════════════════════════════════════════════

/// Frame de call stack
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Índice do frame (0 = topo)
    pub index: u32,
    /// Endereço de retorno
    pub return_address: u32,
    /// PC quando chamou
    pub call_site: u32,
    /// Nome da função (se disponível)
    pub function_name: Option<String>,
    /// Argumentos (se disponível)
    pub arguments: Vec<u8>,
    /// Variáveis locais (se disponível)
    pub locals: Vec<u8>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEBUG EVENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Evento de debug
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Hit em breakpoint
    BreakpointHit {
        breakpoint_id: u32,
        address: u32,
    },
    /// Step completo
    StepComplete {
        address: u32,
        instruction: Opcode,
    },
    /// Watch mudou
    WatchChanged {
        watch_id: u32,
        old_value: WatchValue,
        new_value: WatchValue,
    },
    /// Exception
    Exception {
        message: String,
        address: u32,
    },
    /// Programa terminou
    Terminated {
        exit_code: i32,
    },
    /// Logpoint triggered
    LogMessage {
        breakpoint_id: u32,
        message: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEBUGGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Estado do debugger
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerState {
    /// Parado (não iniciado ou terminado)
    Stopped,
    /// Rodando
    Running,
    /// Pausado em breakpoint/step
    Paused,
    /// Stepping
    Stepping,
}

/// Debugger VSP
pub struct Debugger {
    /// VM sendo debugada
    vm: Vsp,
    /// Breakpoints
    breakpoints: HashMap<u32, Breakpoint>,
    /// Watches
    watches: HashMap<u32, Watch>,
    /// Próximo ID de breakpoint
    next_bp_id: u32,
    /// Próximo ID de watch
    next_watch_id: u32,
    /// Estado do debugger
    state: DebuggerState,
    /// Eventos pendentes
    events: Vec<DebugEvent>,
    /// Histórico de instruções executadas
    instruction_history: Vec<(u32, Opcode)>,
    /// Máximo de histórico
    max_history: usize,
    /// Endereços de memória monitorados
    watched_memory: HashSet<u32>,
    /// Callback para eventos
    event_callback: Option<Box<dyn Fn(&DebugEvent) + Send + Sync>>,
}

impl Debugger {
    pub fn new(vm: Vsp) -> Self {
        Self {
            vm,
            breakpoints: HashMap::new(),
            watches: HashMap::new(),
            next_bp_id: 1,
            next_watch_id: 1,
            state: DebuggerState::Stopped,
            events: Vec::new(),
            instruction_history: Vec::new(),
            max_history: 1000,
            watched_memory: HashSet::new(),
            event_callback: None,
        }
    }
    
    pub fn with_config(config: VspConfig) -> Self {
        Self::new(Vsp::new(config).expect("Failed to create VSP"))
    }
    
    /// Define callback para eventos
    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(&DebugEvent) + Send + Sync + 'static,
    {
        self.event_callback = Some(Box::new(callback));
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // BREAKPOINTS
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Adiciona breakpoint
    pub fn add_breakpoint(&mut self, address: u32) -> u32 {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        
        let bp = Breakpoint::new(id, address);
        self.breakpoints.insert(address, bp);
        
        id
    }
    
    /// Adiciona breakpoint condicional
    pub fn add_conditional_breakpoint(
        &mut self,
        address: u32,
        condition: BreakCondition,
    ) -> u32 {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        
        let bp = Breakpoint::new(id, address).with_condition(condition);
        self.breakpoints.insert(address, bp);
        
        id
    }
    
    /// Adiciona logpoint
    pub fn add_logpoint(&mut self, address: u32, message: String) -> u32 {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        
        let bp = Breakpoint::new(id, address).with_log(message);
        self.breakpoints.insert(address, bp);
        
        id
    }
    
    /// Remove breakpoint por endereço
    pub fn remove_breakpoint(&mut self, address: u32) -> bool {
        self.breakpoints.remove(&address).is_some()
    }
    
    /// Remove breakpoint por ID
    pub fn remove_breakpoint_by_id(&mut self, id: u32) -> bool {
        let addr = self.breakpoints.iter()
            .find(|(_, bp)| bp.id == id)
            .map(|(addr, _)| *addr);
        
        if let Some(addr) = addr {
            self.breakpoints.remove(&addr);
            true
        } else {
            false
        }
    }
    
    /// Habilita/desabilita breakpoint
    pub fn toggle_breakpoint(&mut self, address: u32) -> Option<bool> {
        if let Some(bp) = self.breakpoints.get_mut(&address) {
            bp.enabled = !bp.enabled;
            Some(bp.enabled)
        } else {
            None
        }
    }
    
    /// Lista todos breakpoints
    pub fn list_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }
    
    /// Limpa todos breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // WATCHES
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Adiciona watch
    pub fn add_watch(&mut self, expression: WatchExpression) -> u32 {
        let id = self.next_watch_id;
        self.next_watch_id += 1;
        
        // Track memory watches
        if let WatchExpression::Memory(addr) = expression {
            self.watched_memory.insert(addr);
        } else if let WatchExpression::MemoryRange(start, end) = expression {
            for addr in start..end {
                self.watched_memory.insert(addr);
            }
        }
        
        let watch = Watch {
            id,
            expression,
            last_value: None,
        };
        self.watches.insert(id, watch);
        
        id
    }
    
    /// Remove watch
    pub fn remove_watch(&mut self, id: u32) -> bool {
        self.watches.remove(&id).is_some()
    }
    
    /// Avalia um watch
    pub fn evaluate_watch(&self, watch: &Watch) -> WatchValue {
        match &watch.expression {
            WatchExpression::Register(r) => {
                WatchValue::Byte(u8::from(self.vm.state().regs[*r as usize]))
            }
            WatchExpression::Memory(addr) => {
                let mem = self.vm.memory();
                if (*addr as usize) < mem.code().len() {
                    WatchValue::Byte(mem.code()[*addr as usize])
                } else {
                    WatchValue::Text("Invalid address".to_string())
                }
            }
            WatchExpression::MemoryRange(start, end) => {
                let mem = self.vm.memory();
                let bytes: Vec<u8> = (*start..*end)
                    .filter_map(|addr| mem.code().get(addr as usize).copied())
                    .collect();
                WatchValue::Bytes(bytes)
            }
            WatchExpression::State => {
                WatchValue::State(Box::new(self.vm.state().clone()))
            }
            WatchExpression::ProgramCounter => {
                WatchValue::Dword(self.vm.state().pc)
            }
            WatchExpression::StackPointer => {
                WatchValue::Dword(self.vm.state().sp)
            }
            WatchExpression::Flags => {
                let status = &self.vm.state().sr;
                WatchValue::Text(format!(
                    "Z={} N={} C={} O={}",
                    status.zero as u8,
                    status.negative as u8,
                    status.collapse as u8,
                    status.overflow as u8
                ))
            }
            WatchExpression::Custom(expr) => {
                WatchValue::Text(format!("Custom: {}", expr))
            }
        }
    }
    
    /// Atualiza todos watches
    pub fn update_watches(&mut self) {
        let watch_ids: Vec<u32> = self.watches.keys().copied().collect();
        
        for id in watch_ids {
            if let Some(watch) = self.watches.get(&id) {
                let new_value = self.evaluate_watch(watch);
                
                // Check for change
                if let Some(old_value) = &watch.last_value {
                    let changed = match (&new_value, old_value) {
                        (WatchValue::Byte(a), WatchValue::Byte(b)) => a != b,
                        (WatchValue::Word(a), WatchValue::Word(b)) => a != b,
                        (WatchValue::Dword(a), WatchValue::Dword(b)) => a != b,
                        (WatchValue::Qword(a), WatchValue::Qword(b)) => a != b,
                        _ => true,
                    };
                    
                    if changed {
                        self.emit_event(DebugEvent::WatchChanged {
                            watch_id: id,
                            old_value: old_value.clone(),
                            new_value: new_value.clone(),
                        });
                    }
                }
                
                // Update value
                if let Some(watch) = self.watches.get_mut(&id) {
                    watch.last_value = Some(new_value);
                }
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // EXECUTION CONTROL
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Carrega programa
    pub fn load(&mut self, bytecode: &[u8]) -> VspResult<()> {
        self.vm.load(bytecode)?;
        self.state = DebuggerState::Stopped;
        self.instruction_history.clear();
        Ok(())
    }
    
    /// Inicia/continua execução
    pub fn run(&mut self) -> VspResult<Option<DebugEvent>> {
        self.state = DebuggerState::Running;
        
        loop {
            // Check breakpoint
            let pc = self.vm.state().pc;
            if let Some(event) = self.check_breakpoint(pc) {
                self.state = DebuggerState::Paused;
                return Ok(Some(event));
            }
            
            // Execute step
            match self.vm.step() {
                Ok(true) => {
                    // Record history (using NOP as placeholder since we don't have the instruction)
                    self.record_instruction(pc, Opcode::Nop);
                    
                    // Update watches
                    self.update_watches();
                }
                Ok(false) => {
                    // Halted
                    self.state = DebuggerState::Stopped;
                    let event = DebugEvent::Terminated { exit_code: 0 };
                    self.emit_event(event.clone());
                    return Ok(Some(event));
                }
                Err(e) => {
                    self.state = DebuggerState::Paused;
                    let event = DebugEvent::Exception {
                        message: format!("{:?}", e),
                        address: pc,
                    };
                    self.emit_event(event.clone());
                    return Ok(Some(event));
                }
            }
        }
    }
    
    /// Executa um passo
    pub fn step(&mut self) -> VspResult<Option<DebugEvent>> {
        self.state = DebuggerState::Stepping;
        let pc = self.vm.state().pc;
        
        match self.vm.step() {
            Ok(true) => {
                self.record_instruction(pc, Opcode::Nop);
                self.update_watches();
                
                self.state = DebuggerState::Paused;
                let event = DebugEvent::StepComplete {
                    address: pc,
                    instruction: Opcode::Nop,
                };
                self.emit_event(event.clone());
                Ok(Some(event))
            }
            Ok(false) => {
                self.state = DebuggerState::Stopped;
                let event = DebugEvent::Terminated { exit_code: 0 };
                self.emit_event(event.clone());
                Ok(Some(event))
            }
            Err(e) => {
                self.state = DebuggerState::Paused;
                let event = DebugEvent::Exception {
                    message: format!("{:?}", e),
                    address: pc,
                };
                self.emit_event(event.clone());
                Ok(Some(event))
            }
        }
    }
    
    /// Step over (executa call como uma instrução)
    pub fn step_over(&mut self) -> VspResult<Option<DebugEvent>> {
        let pc = self.vm.state().pc;
        
        // Check if current instruction is CALL
        let mem = self.vm.memory();
        if let Some(&opcode_byte) = mem.code().get(pc as usize) {
            if opcode_byte == Opcode::Call as u8 {
                // Set temporary breakpoint at return address
                let return_addr = pc + 4; // CALL is format D (4 bytes)
                let _bp_id = self.add_breakpoint(return_addr);
                if let Some(bp) = self.breakpoints.get_mut(&return_addr) {
                    bp.temporary = true;
                }
                return self.run();
            }
        }
        
        // Otherwise, just step
        self.step()
    }
    
    /// Step out (executa até retornar da função atual)
    pub fn step_out(&mut self) -> VspResult<Option<DebugEvent>> {
        // Get return address from stack
        let sp = self.vm.state().sp;
        let mem = self.vm.memory();
        
        if sp >= 4 {
            let return_addr = u32::from_le_bytes([
                mem.code().get((sp - 4) as usize).copied().unwrap_or(0),
                mem.code().get((sp - 3) as usize).copied().unwrap_or(0),
                mem.code().get((sp - 2) as usize).copied().unwrap_or(0),
                mem.code().get((sp - 1) as usize).copied().unwrap_or(0),
            ]);
            
            // Set temporary breakpoint
            let _bp_id = self.add_breakpoint(return_addr);
            if let Some(bp) = self.breakpoints.get_mut(&return_addr) {
                bp.temporary = true;
            }
        }
        
        self.run()
    }
    
    /// Pausa execução
    pub fn pause(&mut self) {
        self.state = DebuggerState::Paused;
    }
    
    /// Para execução
    pub fn stop(&mut self) {
        self.state = DebuggerState::Stopped;
        self.emit_event(DebugEvent::Terminated { exit_code: -1 });
    }
    
    /// Reseta debugger
    pub fn reset(&mut self) {
        self.vm = Vsp::new(VspConfig::default()).expect("Failed to create VSP");
        self.state = DebuggerState::Stopped;
        self.instruction_history.clear();
        self.events.clear();
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // INSPECTION
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Retorna estado atual
    pub fn state(&self) -> &VspState {
        self.vm.state()
    }
    
    /// Retorna estado do debugger
    pub fn debugger_state(&self) -> DebuggerState {
        self.state
    }
    
    /// Retorna call stack
    pub fn call_stack(&self) -> Vec<StackFrame> {
        let mut frames = Vec::new();
        let state = self.vm.state();
        let mem = self.vm.memory();
        
        // Current frame
        frames.push(StackFrame {
            index: 0,
            return_address: state.pc,
            call_site: 0,
            function_name: None,
            arguments: Vec::new(),
            locals: Vec::new(),
        });
        
        // Walk stack
        let mut sp = state.sp;
        let mut index = 1u32;
        
        while sp >= 4 && index < 100 {
            let return_addr = u32::from_le_bytes([
                mem.code().get((sp - 4) as usize).copied().unwrap_or(0),
                mem.code().get((sp - 3) as usize).copied().unwrap_or(0),
                mem.code().get((sp - 2) as usize).copied().unwrap_or(0),
                mem.code().get((sp - 1) as usize).copied().unwrap_or(0),
            ]);
            
            if return_addr == 0 {
                break;
            }
            
            frames.push(StackFrame {
                index,
                return_address: return_addr,
                call_site: 0,
                function_name: None,
                arguments: Vec::new(),
                locals: Vec::new(),
            });
            
            sp -= 4;
            index += 1;
        }
        
        frames
    }
    
    /// Retorna histórico de instruções
    pub fn instruction_history(&self) -> &[(u32, Opcode)] {
        &self.instruction_history
    }
    
    /// Lê memória
    pub fn read_memory(&self, address: u32, length: u32) -> Vec<u8> {
        let mem = self.vm.memory();
        let start = address as usize;
        let end = (address + length) as usize;
        
        mem.code().get(start..end.min(mem.code().len()))
            .map(|s| s.to_vec())
            .unwrap_or_default()
    }
    
    /// Escreve memória
    pub fn write_memory(&mut self, address: u32, data: &[u8]) {
        let mem = self.vm.memory_mut();
        let start = address as usize;
        
        for (i, &byte) in data.iter().enumerate() {
            if start + i < mem.code_mut().len() {
                mem.code_mut()[start + i] = byte;
            }
        }
    }
    
    /// Lê registrador
    pub fn read_register(&self, reg: u8) -> u8 {
        self.vm.state().regs.get(reg as usize)
            .map(|b| u8::from(*b))
            .unwrap_or(0)
    }
    
    /// Escreve registrador
    pub fn write_register(&mut self, reg: u8, value: u8) {
        if (reg as usize) < self.vm.state().regs.len() {
            self.vm.state_mut().regs[reg as usize] = ByteSil::from_u8(value);
        }
    }
    
    /// Define PC
    pub fn set_pc(&mut self, address: u32) {
        self.vm.state_mut().pc = address;
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // INTERNAL
    // ═══════════════════════════════════════════════════════════════════════════
    
    fn check_breakpoint(&mut self, address: u32) -> Option<DebugEvent> {
        let bp = self.breakpoints.get_mut(&address)?;
        
        if !bp.enabled {
            return None;
        }
        
        // Check condition
        let should_break = match &bp.condition {
            BreakCondition::Always => true,
            BreakCondition::RegisterEquals(r, v) => {
                u8::from(self.vm.state().regs[*r as usize]) == *v
            }
            BreakCondition::RegisterNotEquals(r, v) => {
                u8::from(self.vm.state().regs[*r as usize]) != *v
            }
            BreakCondition::HitCount(count) => {
                bp.hit_count + 1 >= *count
            }
            BreakCondition::Expression(_) => true, // TODO: eval expression
        };
        
        if !should_break {
            return None;
        }
        
        bp.hit_count += 1;
        
        // Logpoint
        if let Some(msg) = &bp.log_message {
            let event = DebugEvent::LogMessage {
                breakpoint_id: bp.id,
                message: msg.clone(),
            };
            self.emit_event(event);
            return None; // Don't stop for logpoints
        }
        
        let event = DebugEvent::BreakpointHit {
            breakpoint_id: bp.id,
            address,
        };
        
        // Remove temporary breakpoint
        let temporary = bp.temporary;
        
        if temporary {
            self.breakpoints.remove(&address);
        }
        
        self.emit_event(event.clone());
        Some(event)
    }
    
    fn record_instruction(&mut self, address: u32, opcode: Opcode) {
        self.instruction_history.push((address, opcode));
        
        // Trim history
        if self.instruction_history.len() > self.max_history {
            self.instruction_history.remove(0);
        }
    }
    
    fn emit_event(&mut self, event: DebugEvent) {
        if let Some(callback) = &self.event_callback {
            callback(&event);
        }
        self.events.push(event);
    }
    
    /// Drena eventos pendentes
    pub fn drain_events(&mut self) -> Vec<DebugEvent> {
        std::mem::take(&mut self.events)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEBUG ADAPTER PROTOCOL (DAP) SUPPORT
// ═══════════════════════════════════════════════════════════════════════════════

/// Capabilities do debugger (para DAP)
#[derive(Debug, Clone)]
pub struct DebugCapabilities {
    pub supports_conditional_breakpoints: bool,
    pub supports_hit_conditional_breakpoints: bool,
    pub supports_evaluate_for_hovers: bool,
    pub supports_step_back: bool,
    pub supports_set_variable: bool,
    pub supports_restart_frame: bool,
    pub supports_goto_targets_request: bool,
    pub supports_stepping_granularity: bool,
    pub supports_instruction_breakpoints: bool,
    pub supports_read_memory_request: bool,
    pub supports_write_memory_request: bool,
    pub supports_disassemble_request: bool,
}

impl Default for DebugCapabilities {
    fn default() -> Self {
        Self {
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: true,
            supports_evaluate_for_hovers: true,
            supports_step_back: false, // TODO
            supports_set_variable: true,
            supports_restart_frame: false,
            supports_goto_targets_request: true,
            supports_stepping_granularity: true,
            supports_instruction_breakpoints: true,
            supports_read_memory_request: true,
            supports_write_memory_request: true,
            supports_disassemble_request: true,
        }
    }
}

impl Debugger {
    /// Retorna capabilities para DAP
    pub fn capabilities(&self) -> DebugCapabilities {
        DebugCapabilities::default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_breakpoint_creation() {
        let mut dbg = Debugger::with_config(VspConfig::default());
        
        let id = dbg.add_breakpoint(0x100);
        assert_eq!(id, 1);
        
        let bps = dbg.list_breakpoints();
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0].address, 0x100);
    }
    
    #[test]
    fn test_watch_creation() {
        let mut dbg = Debugger::with_config(VspConfig::default());
        
        let id = dbg.add_watch(WatchExpression::Register(0));
        assert_eq!(id, 1);
        
        let watch = dbg.watches.get(&id).unwrap();
        let value = dbg.evaluate_watch(watch);
        
        match value {
            WatchValue::Byte(_) => {}
            _ => panic!("Expected byte value"),
        }
    }
    
    #[test]
    fn test_conditional_breakpoint() {
        let mut dbg = Debugger::with_config(VspConfig::default());
        
        let _id = dbg.add_conditional_breakpoint(
            0x100,
            BreakCondition::RegisterEquals(0, 0xFF),
        );
        
        let bps = dbg.list_breakpoints();
        assert!(matches!(
            bps[0].condition,
            BreakCondition::RegisterEquals(0, 0xFF)
        ));
    }
    
    #[test]
    fn test_logpoint() {
        let mut dbg = Debugger::with_config(VspConfig::default());
        
        let _id = dbg.add_logpoint(0x100, "Hit at 0x100".to_string());
        
        let bps = dbg.list_breakpoints();
        assert_eq!(bps[0].log_message, Some("Hit at 0x100".to_string()));
    }
}
