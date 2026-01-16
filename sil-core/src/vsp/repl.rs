//! REPL interativo para VSP
//!
//! # Uso
//!
//! ```text
//! $ silrepl
//! SIL REPL v1.0 (SIL-128 mode)
//! > NOP
//! > MOVI R0, 0xFF
//! > :state
//! R0 = FF (ρ=7, θ=15)
//! > :run
//! Executed 2 instructions
//! > :help
//! ```

use std::io::{self, BufRead, Write};
use std::collections::HashMap;

use super::{
    Vsp, VspConfig, VspResult,
    assembler::{Assembler, disassemble},
    state::SilMode,
};

// ═══════════════════════════════════════════════════════════════════════════════
// REPL STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Estado do REPL
pub struct Repl {
    /// VM atual
    vm: Vsp,
    /// Assembler acumulativo
    assembler: Assembler,
    /// Código acumulado
    code_buffer: Vec<String>,
    /// Histórico de comandos
    history: Vec<String>,
    /// Breakpoints ativos
    breakpoints: HashMap<u32, Breakpoint>,
    /// Variáveis do REPL
    variables: HashMap<String, u64>,
    /// Modo verboso
    verbose: bool,
    /// Continuar executando
    running: bool,
}

/// Breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub address: u32,
    pub condition: Option<String>,
    pub hit_count: u32,
    pub enabled: bool,
}

/// Resultado de comando REPL
#[derive(Debug)]
pub enum ReplResult {
    Continue,
    Output(String),
    Error(String),
    Exit,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    pub fn new() -> Self {
        let config = VspConfig::default();
        Self {
            vm: Vsp::new(config).expect("Failed to create VSP"),
            assembler: Assembler::new(),
            code_buffer: Vec::new(),
            history: Vec::new(),
            breakpoints: HashMap::new(),
            variables: HashMap::new(),
            verbose: false,
            running: true,
        }
    }
    
    /// Configura o modo SIL
    pub fn with_mode(mut self, mode: SilMode) -> Self {
        let config = VspConfig::default().with_mode(mode);
        self.vm = Vsp::new(config).expect("Failed to create VSP");
        self
    }
    
    /// Executa o REPL interativo
    pub fn run(&mut self) -> VspResult<()> {
        self.print_banner();
        
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        while self.running {
            // Prompt
            print!("> ");
            stdout.flush().ok();
            
            // Ler linha
            let mut line = String::new();
            if stdin.lock().read_line(&mut line).is_err() {
                break;
            }
            
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // Adicionar ao histórico
            self.history.push(line.to_string());
            
            // Processar
            match self.process_line(line) {
                ReplResult::Continue => {}
                ReplResult::Output(s) => println!("{}", s),
                ReplResult::Error(e) => eprintln!("Error: {}", e),
                ReplResult::Exit => break,
            }
        }
        
        Ok(())
    }
    
    fn print_banner(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    SIL REPL v1.0                             ║");
        println!("║              Virtual Sil Processor Console                   ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  Mode: SIL-128  │  Type :help for commands                   ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }
    
    /// Processa uma linha de input
    pub fn process_line(&mut self, line: &str) -> ReplResult {
        let line = line.trim();
        
        // Comando especial
        if line.starts_with(':') {
            return self.process_command(&line[1..]);
        }
        
        // Comentário
        if line.starts_with(';') {
            return ReplResult::Continue;
        }
        
        // Assembly
        self.process_assembly(line)
    }
    
    fn process_command(&mut self, cmd: &str) -> ReplResult {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let cmd_name = parts.first().map(|s| *s).unwrap_or("");
        let args = &parts[1..];
        
        match cmd_name {
            "help" | "h" | "?" => self.cmd_help(),
            "quit" | "q" | "exit" => {
                self.running = false;
                ReplResult::Exit
            }
            "state" | "s" => self.cmd_state(),
            "regs" | "r" => self.cmd_registers(),
            "mem" | "m" => self.cmd_memory(args),
            "run" | "go" => self.cmd_run(),
            "step" | "n" => self.cmd_step(args),
            "reset" => self.cmd_reset(),
            "clear" | "cls" => self.cmd_clear(),
            "load" => self.cmd_load(args),
            "save" => self.cmd_save(args),
            "dis" | "disasm" => self.cmd_disassemble(args),
            "bp" | "break" => self.cmd_breakpoint(args),
            "del" | "delete" => self.cmd_delete(args),
            "list" | "l" => self.cmd_list(),
            "mode" => self.cmd_mode(args),
            "verbose" | "v" => self.cmd_verbose(),
            "history" => self.cmd_history(),
            "set" => self.cmd_set(args),
            "eval" | "e" => self.cmd_eval(args),
            _ => ReplResult::Error(format!("Unknown command: {}", cmd_name)),
        }
    }
    
    fn cmd_help(&self) -> ReplResult {
        ReplResult::Output(r#"
╔══════════════════════════════════════════════════════════════════════╗
║                           REPL Commands                              ║
╠══════════════════════════════════════════════════════════════════════╣
║ EXECUTION                                                            ║
║   :run, :go           Run until halt or breakpoint                   ║
║   :step [N], :n       Step N instructions (default 1)                ║
║   :reset              Reset VM to initial state                      ║
║                                                                      ║
║ INSPECTION                                                           ║
║   :state, :s          Show full SilState                             ║
║   :regs, :r           Show registers                                 ║
║   :mem [ADDR] [LEN]   Show memory at address                         ║
║   :dis [ADDR] [LEN]   Disassemble code                               ║
║                                                                      ║
║ DEBUGGING                                                            ║
║   :bp ADDR            Set breakpoint at address                      ║
║   :del [N|all]        Delete breakpoint(s)                           ║
║   :list, :l           List breakpoints                               ║
║                                                                      ║
║ FILE I/O                                                             ║
║   :load FILE          Load .sil or .silc file                        ║
║   :save FILE          Save assembled code to .silc                   ║
║                                                                      ║
║ SETTINGS                                                             ║
║   :mode MODE          Set SIL mode (8/16/32/64/128)                  ║
║   :verbose, :v        Toggle verbose output                          ║
║   :set VAR VALUE      Set REPL variable                              ║
║   :eval EXPR          Evaluate expression                            ║
║                                                                      ║
║ OTHER                                                                ║
║   :clear, :cls        Clear code buffer                              ║
║   :history            Show command history                           ║
║   :quit, :q           Exit REPL                                      ║
║                                                                      ║
║ Type assembly directly: MOV R0, R1                                   ║
╚══════════════════════════════════════════════════════════════════════╝
"#.to_string())
    }
    
    fn cmd_state(&self) -> ReplResult {
        let state = self.vm.state();
        let mut output = String::new();
        
        output.push_str("┌─────────────────────────────────────────────────────────────┐\n");
        output.push_str("│                        SIL STATE                            │\n");
        output.push_str("├─────────────────────────────────────────────────────────────┤\n");
        
        // Registradores principais (SilState layers)
        output.push_str("│ Layers (R0-R15):                                            │\n");
        for i in 0..16 {
            let byte = state.regs[i];
            output.push_str(&format!(
                "│   R{:X}: 0x{:02X}  (ρ={:+2}, θ={:2})                              │\n",
                i, u8::from(byte), byte.rho, byte.theta
            ));
        }
        
        output.push_str("├─────────────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("│ PC: 0x{:08X}  │  SP: 0x{:08X}                      │\n",
            state.pc, state.sp));
        output.push_str(&format!("│ Flags: Z={} N={} C={} O={}                                    │\n",
            state.sr.zero as u8,
            state.sr.negative as u8,
            state.sr.collapse as u8,
            state.sr.overflow as u8));
        output.push_str(&format!("│ Mode: {:?}                                               │\n",
            state.mode));
        output.push_str("└─────────────────────────────────────────────────────────────┘\n");
        
        ReplResult::Output(output)
    }
    
    fn cmd_registers(&self) -> ReplResult {
        let state = self.vm.state();
        let mut output = String::new();
        
        output.push_str("Registers:\n");
        for i in 0..16 {
            let byte = state.regs[i];
            output.push_str(&format!(
                "  R{:X} = 0x{:02X} (ρ={:+2}, θ={:2})\n",
                i, u8::from(byte), byte.rho, byte.theta
            ));
        }
        output.push_str(&format!("\nPC = 0x{:08X}\n", state.pc));
        output.push_str(&format!("SP = 0x{:08X}\n", state.sp));
        
        ReplResult::Output(output)
    }
    
    fn cmd_memory(&self, args: &[&str]) -> ReplResult {
        let addr = args.first()
            .and_then(|s| parse_number(s))
            .unwrap_or(0) as usize;
        let len = args.get(1)
            .and_then(|s| parse_number(s))
            .unwrap_or(64) as usize;
        
        let memory = self.vm.memory();
        let code = memory.code();
        let end = (addr + len).min(code.len());
        
        let mut output = String::new();
        output.push_str(&format!("Memory at 0x{:08X}:\n", addr));
        
        for (i, chunk) in code[addr..end].chunks(16).enumerate() {
            let line_addr = addr + i * 16;
            output.push_str(&format!("{:08X}: ", line_addr));
            
            // Hex
            for byte in chunk {
                output.push_str(&format!("{:02X} ", byte));
            }
            
            // Padding
            let chunk_len = chunk.len();
            for _ in chunk_len..16 {
                output.push_str("   ");
            }
            
            // ASCII
            output.push_str(" |");
            for byte in chunk {
                let c = if *byte >= 0x20 && *byte < 0x7F {
                    *byte as char
                } else {
                    '.'
                };
                output.push(c);
            }
            output.push_str("|\n");
        }
        
        ReplResult::Output(output)
    }
    
    fn cmd_run(&mut self) -> ReplResult {
        match self.vm.run() {
            Ok(cycles) => ReplResult::Output(format!("Executed {} cycles", cycles)),
            Err(e) => ReplResult::Error(format!("{:?}", e)),
        }
    }
    
    fn cmd_step(&mut self, args: &[&str]) -> ReplResult {
        let count = args.first()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        
        let mut output = String::new();
        let mut stepped = 0;
        for _ in 0..count {
            match self.vm.step() {
                Ok(true) => {
                    stepped += 1;
                    if self.verbose {
                        output.push_str(&format!(
                            "{:04X}: stepped\n",
                            self.vm.state().pc
                        ));
                    }
                }
                Ok(false) => {
                    output.push_str("Halted\n");
                    break;
                }
                Err(e) => {
                    return ReplResult::Error(format!("{:?}", e));
                }
            }
        }
        
        if output.is_empty() {
            output = format!("Stepped {} instruction(s)", stepped);
        }
        
        ReplResult::Output(output)
    }
    
    fn cmd_reset(&mut self) -> ReplResult {
        self.vm = Vsp::new(VspConfig::default()).expect("Failed to create VSP");
        ReplResult::Output("VM reset".to_string())
    }
    
    fn cmd_clear(&mut self) -> ReplResult {
        self.code_buffer.clear();
        self.assembler = Assembler::new();
        ReplResult::Output("Buffer cleared".to_string())
    }
    
    fn cmd_load(&mut self, args: &[&str]) -> ReplResult {
        let path = match args.first() {
            Some(p) => *p,
            None => return ReplResult::Error("Usage: :load <file>".to_string()),
        };
        
        match std::fs::read(path) {
            Ok(data) => {
                if path.ends_with(".silc") {
                    // Load bytecode directly
                    match self.vm.load(&data) {
                        Ok(()) => ReplResult::Output(format!("Loaded {} bytes from {}", data.len(), path)),
                        Err(e) => ReplResult::Error(format!("Load error: {:?}", e)),
                    }
                } else {
                    // Assemble .sil file
                    match String::from_utf8(data) {
                        Ok(source) => {
                            match self.assembler.assemble(&source) {
                                Ok(silc) => {
                                    match self.vm.load(silc.to_bytes().as_slice()) {
                                        Ok(()) => ReplResult::Output(format!("Assembled and loaded {}", path)),
                                        Err(e) => ReplResult::Error(format!("Load error: {:?}", e)),
                                    }
                                }
                                Err(e) => ReplResult::Error(format!("Assembly error: {:?}", e)),
                            }
                        }
                        Err(e) => ReplResult::Error(format!("UTF-8 error: {}", e)),
                    }
                }
            }
            Err(e) => ReplResult::Error(format!("Read error: {}", e)),
        }
    }
    
    fn cmd_save(&mut self, args: &[&str]) -> ReplResult {
        let path = match args.first() {
            Some(p) => *p,
            None => return ReplResult::Error("Usage: :save <file>".to_string()),
        };
        
        // Assemble current buffer
        let source = self.code_buffer.join("\n");
        match self.assembler.assemble(&source) {
            Ok(silc) => {
                match std::fs::write(path, silc.to_bytes()) {
                    Ok(()) => ReplResult::Output(format!("Saved to {}", path)),
                    Err(e) => ReplResult::Error(format!("Write error: {}", e)),
                }
            }
            Err(e) => ReplResult::Error(format!("Assembly error: {:?}", e)),
        }
    }
    
    fn cmd_disassemble(&self, args: &[&str]) -> ReplResult {
        let addr = args.first()
            .and_then(|s| parse_number(s))
            .unwrap_or(0) as usize;
        let len = args.get(1)
            .and_then(|s| parse_number(s))
            .unwrap_or(32) as usize;
        
        let memory = self.vm.memory();
        let code = memory.code();
        let end = (addr + len).min(code.len());
        let code_slice = &code[addr..end];
        
        ReplResult::Output(disassemble(code_slice))
    }
    
    fn cmd_breakpoint(&mut self, args: &[&str]) -> ReplResult {
        let addr = match args.first().and_then(|s| parse_number(s)) {
            Some(a) => a as u32,
            None => return ReplResult::Error("Usage: :bp <address>".to_string()),
        };
        
        let bp = Breakpoint {
            address: addr,
            condition: None,
            hit_count: 0,
            enabled: true,
        };
        
        self.breakpoints.insert(addr, bp);
        ReplResult::Output(format!("Breakpoint set at 0x{:08X}", addr))
    }
    
    fn cmd_delete(&mut self, args: &[&str]) -> ReplResult {
        match args.first() {
            Some(&"all") => {
                self.breakpoints.clear();
                ReplResult::Output("All breakpoints deleted".to_string())
            }
            Some(s) => {
                if let Some(addr) = parse_number(s) {
                    self.breakpoints.remove(&(addr as u32));
                    ReplResult::Output(format!("Breakpoint at 0x{:08X} deleted", addr))
                } else {
                    ReplResult::Error("Invalid address".to_string())
                }
            }
            None => ReplResult::Error("Usage: :del <address|all>".to_string()),
        }
    }
    
    fn cmd_list(&self) -> ReplResult {
        if self.breakpoints.is_empty() {
            return ReplResult::Output("No breakpoints set".to_string());
        }
        
        let mut output = String::from("Breakpoints:\n");
        for (addr, bp) in &self.breakpoints {
            output.push_str(&format!(
                "  0x{:08X} [{}] hits={}\n",
                addr,
                if bp.enabled { "enabled" } else { "disabled" },
                bp.hit_count
            ));
        }
        ReplResult::Output(output)
    }
    
    fn cmd_mode(&mut self, args: &[&str]) -> ReplResult {
        let mode = match args.first() {
            Some(&"8") => SilMode::Sil8,
            Some(&"16") => SilMode::Sil16,
            Some(&"32") => SilMode::Sil32,
            Some(&"64") => SilMode::Sil64,
            Some(&"128") | None => SilMode::Sil128,
            Some(m) => return ReplResult::Error(format!("Unknown mode: {}", m)),
        };
        
        let config = VspConfig::default().with_mode(mode);
        self.vm = Vsp::new(config).expect("Failed to create VSP");
        ReplResult::Output(format!("Mode set to {:?}", mode))
    }
    
    fn cmd_verbose(&mut self) -> ReplResult {
        self.verbose = !self.verbose;
        ReplResult::Output(format!("Verbose: {}", self.verbose))
    }
    
    fn cmd_history(&self) -> ReplResult {
        let mut output = String::from("History:\n");
        for (i, cmd) in self.history.iter().enumerate() {
            output.push_str(&format!("  {:4}: {}\n", i, cmd));
        }
        ReplResult::Output(output)
    }
    
    fn cmd_set(&mut self, args: &[&str]) -> ReplResult {
        if args.len() < 2 {
            return ReplResult::Error("Usage: :set <var> <value>".to_string());
        }
        
        let name = args[0].to_string();
        let value = match parse_number(args[1]) {
            Some(v) => v,
            None => return ReplResult::Error("Invalid value".to_string()),
        };
        
        self.variables.insert(name.clone(), value);
        ReplResult::Output(format!("{} = {}", name, value))
    }
    
    fn cmd_eval(&mut self, args: &[&str]) -> ReplResult {
        if args.is_empty() {
            return ReplResult::Error("Usage: :eval <expression>".to_string());
        }
        
        let expr = args.join(" ");
        
        // Simple expression evaluator
        // Supports: R0, $var, hex, dec, +, -, *, /
        match self.eval_expr(&expr) {
            Ok(value) => ReplResult::Output(format!(
                "{} = {} (0x{:X})",
                expr, value, value
            )),
            Err(e) => ReplResult::Error(e),
        }
    }
    
    fn eval_expr(&self, expr: &str) -> Result<u64, String> {
        let expr = expr.trim();
        
        // Register
        if expr.starts_with('R') || expr.starts_with('r') {
            if let Ok(n) = expr[1..].parse::<usize>() {
                if n < 16 {
                    return Ok(u8::from(self.vm.state().regs[n]) as u64);
                }
            }
        }
        
        // Variable
        if expr.starts_with('$') {
            if let Some(&v) = self.variables.get(&expr[1..]) {
                return Ok(v);
            }
            return Err(format!("Unknown variable: {}", expr));
        }
        
        // Number
        if let Some(n) = parse_number(expr) {
            return Ok(n);
        }
        
        Err(format!("Cannot evaluate: {}", expr))
    }
    
    fn process_assembly(&mut self, line: &str) -> ReplResult {
        // Adicionar ao buffer
        self.code_buffer.push(line.to_string());
        
        // Tentar montar
        let source = self.code_buffer.join("\n");
        match self.assembler.assemble(&source) {
            Ok(silc) => {
                // Carregar na VM
                let bytecode = silc.to_bytes();
                match self.vm.load(&bytecode) {
                    Ok(()) => {
                        if self.verbose {
                            ReplResult::Output(format!(
                                "Assembled {} bytes",
                                silc.code.len()
                            ))
                        } else {
                            ReplResult::Continue
                        }
                    }
                    Err(e) => ReplResult::Error(format!("Load error: {:?}", e)),
                }
            }
            Err(e) => {
                // Remove linha com erro
                self.code_buffer.pop();
                ReplResult::Error(format!("{:?}", e))
            }
        }
    }
}

/// Parse número (decimal ou hex)
fn parse_number(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse().ok()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repl_help() {
        let mut repl = Repl::new();
        match repl.process_command("help") {
            ReplResult::Output(s) => assert!(s.contains("REPL Commands")),
            _ => panic!("Expected output"),
        }
    }
    
    #[test]
    fn test_repl_state() {
        let repl = Repl::new();
        match repl.cmd_state() {
            ReplResult::Output(s) => assert!(s.contains("SIL STATE")),
            _ => panic!("Expected output"),
        }
    }
    
    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42"), Some(42));
        assert_eq!(parse_number("0xFF"), Some(255));
        assert_eq!(parse_number("0x10"), Some(16));
    }
}
