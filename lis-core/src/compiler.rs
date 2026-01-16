//! Compiler for LIS language
//!
//! Compiles LIS AST to VSP assembly code.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::types::checker::TypeChecker;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "jsil")]
use sil_core::io::jsonl::{JsonlRecord, InstructionArgs};

/// Loop context for tracking break/continue targets
#[derive(Debug, Clone)]
struct LoopContext {
    /// Label for loop start (continue target)
    start_label: String,
    /// Label for loop end (break target)
    end_label: String,
}

pub struct Compiler {
    /// Register allocator (simple counter for now)
    next_reg: u8,

    /// Variable to register mapping
    vars: HashMap<String, u8>,

    /// Label counter for control flow
    next_label: usize,

    /// Generated assembly code
    output: Vec<String>,

    /// Stack of loop contexts for break/continue
    loop_stack: Vec<LoopContext>,

    /// Current function name (to check if we're in main)
    current_function: Option<String>,

    /// Set of registered intrinsic functions from stdlib
    intrinsics: HashSet<String>,

    /// Set of extern functions declared in the program (FFI to Rust)
    extern_fns: HashSet<String>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut compiler = Self {
            next_reg: 0,
            vars: HashMap::new(),
            next_label: 0,
            output: Vec::new(),
            loop_stack: Vec::new(),
            current_function: None,
            intrinsics: HashSet::new(),
            extern_fns: HashSet::new(),
        };
        compiler.register_stdlib_intrinsics();
        compiler
    }

    /// Registers all stdlib intrinsic functions
    fn register_stdlib_intrinsics(&mut self) {
        // ByteSil operations
        let bytesil_fns = vec![
            "bytesil_new", "bytesil_from_complex", "bytesil_to_complex",
            "bytesil_null", "bytesil_one", "bytesil_i", "bytesil_neg_one",
            "bytesil_neg_i", "bytesil_max", "bytesil_mul", "bytesil_div",
            "bytesil_pow", "bytesil_root", "bytesil_inv", "bytesil_conj",
            "bytesil_xor", "bytesil_mix", "bytesil_rho", "bytesil_theta",
            "bytesil_magnitude", "bytesil_phase_degrees", "bytesil_phase_radians",
            "bytesil_is_null", "bytesil_is_real", "bytesil_is_imaginary",
            "bytesil_norm", "bytesil_from_u8", "bytesil_to_u8",
        ];

        // Math operations
        let math_fns = vec![
            "complex_add", "complex_sub", "complex_scale", "complex_rotate",
            "complex_lerp", "sin", "cos", "tan", "asin", "acos", "atan",
            "atan2", "pi", "tau", "e", "phi", "abs_int", "abs_float",
            "min_int", "max_int", "min_float", "max_float", "clamp_int",
            "clamp_float", "sqrt", "pow_float", "exp", "ln", "log10",
            "log2", "floor", "ceil", "round", "sign_float", "sign_int",
            "degrees_to_radians", "radians_to_degrees",
        ];

        // Console I/O
        let console_fns = vec![
            "print_int", "print_float", "print_string", "print_bool",
            "print_bytesil", "print_state", "println", "read_line",
            "read_int", "read_float",
        ];

        // State operations
        let state_fns = vec![
            "state_vacuum", "state_neutral", "state_maximum", "state_from_bytes",
            "state_to_bytes", "state_from_layers", "state_get_layer", "state_set_layer",
            "state_get_perception", "state_get_processing", "state_get_interaction",
            "state_get_emergence", "state_get_meta", "state_tensor", "state_xor",
            "state_project", "state_collapse_xor", "state_collapse_sum",
            "state_collapse_first", "state_collapse_last", "state_hash",
            "perception_mask", "processing_mask", "interaction_mask",
            "emergence_mask", "meta_mask", "state_equals", "state_is_vacuum",
            "state_is_neutral", "state_count_null_layers", "state_count_active_layers",
        ];

        // Layer operations
        let layer_fns = vec![
            "fuse_vision_audio", "fuse_multimodal", "normalize_perception",
            "shift_layers_up", "shift_layers_down", "rotate_layers",
            "spread_to_group",
        ];

        // Transform operations
        let transform_fns = vec![
            "transform_phase_shift", "transform_magnitude_scale", "transform_layer_swap",
            "transform_xor_layers", "transform_identity", "apply_feedback",
            "detect_emergence", "emergence_pattern", "autopoietic_loop",
        ];

        // Debug utilities
        let debug_fns = vec![
            "assert", "assert_eq_int", "assert_eq_bytesil", "assert_eq_state",
            "debug_print", "trace_state", "timestamp_millis", "timestamp_micros",
            "sleep_millis", "memory_used",
        ];

        // String operations
        let string_fns = vec![
            "string_length", "string_concat", "string_slice", "string_to_upper",
            "string_to_lower", "string_contains", "string_starts_with",
            "string_ends_with", "string_equals", "int_to_string", "float_to_string",
            "bool_to_string", "bytesil_to_string", "state_to_string",
            "string_to_int", "string_to_float", "string_trim", "string_replace",
            "string_index_of",
        ];

        // HTTP operations
        let http_fns = vec![
            "http_get", "http_get_with_status", "http_post", "http_post_json",
            "http_put", "http_put_json", "http_delete", "http_patch", "http_patch_json",
            "http_head", "http_get_with_header", "http_get_auth", "http_post_auth",
            "url_encode", "url_decode", "http_status_ok", "http_status_text",
        ];

        // Register all intrinsics
        for name in bytesil_fns.iter()
            .chain(math_fns.iter())
            .chain(console_fns.iter())
            .chain(state_fns.iter())
            .chain(layer_fns.iter())
            .chain(transform_fns.iter())
            .chain(debug_fns.iter())
            .chain(string_fns.iter())
            .chain(http_fns.iter())
        {
            self.intrinsics.insert(name.to_string());
        }
    }

    /// Checks if a function name is a stdlib intrinsic
    pub fn is_intrinsic(&self, name: &str) -> bool {
        self.intrinsics.contains(name)
    }

    pub fn compile(&mut self, program: &Program) -> Result<String> {
        // Type check the program before code generation
        let mut type_checker = TypeChecker::new();
        let _typed_program = type_checker.check_program(program)?;

        // If type checking succeeds, proceed with code generation
        self.emit(".mode SIL-128");
        self.emit("");
        self.emit(".code");

        for item in &program.items {
            self.compile_item(item)?;
        }

        Ok(self.output.join("\n"))
    }

    // ===== Item Compilation =====

    fn compile_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function { name, params, body, .. } => {
                self.compile_function(name, params, body)
            }
            Item::Transform { name, params, body, .. } => {
                self.compile_transform(name, params, body)
            }
            Item::TypeAlias { .. } => {
                // Type aliases are purely compile-time, no code generation
                Ok(())
            }
            Item::Use(_) => {
                // Use statements are handled by the module resolver, no code generation
                Ok(())
            }
            Item::Module(_) => {
                // Module declarations are handled by the module resolver, no code generation
                Ok(())
            }
            Item::ExternFunction(extern_fn) => {
                // Register extern function for later lookup during CALL compilation
                // The actual implementation is provided by the Rust host via SYSCALL
                self.extern_fns.insert(extern_fn.name.clone());
                Ok(())
            }
        }
    }

    fn compile_function(&mut self, name: &str, params: &[Param], body: &[Stmt]) -> Result<()> {
        self.emit("");
        self.emit(&format!("{}:", name));

        // Reset register allocator for new function
        self.next_reg = 0;
        self.vars.clear();
        self.current_function = Some(name.to_string());

        // Allocate registers for parameters
        for param in params {
            let reg = self.alloc_reg();
            self.vars.insert(param.name.clone(), reg);
        }

        // Compile function body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // For main function, use HLT instead of RET
        if name == "main" {
            self.emit("    HLT");
        } else {
            // Implicit return if no explicit return
            if !matches!(body.last(), Some(Stmt::Return(_, _))) {
                self.emit("    RET");
            }
        }

        self.current_function = None;
        Ok(())
    }

    fn compile_transform(&mut self, name: &str, params: &[Param], body: &[Stmt]) -> Result<()> {
        // For now, transforms compile the same as functions
        // In the future, they could be optimized differently
        self.compile_function(name, params, body)
    }

    // ===== Statement Compilation =====

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let reg = self.compile_expr(value)?;
                self.vars.insert(name.clone(), reg);
                Ok(())
            }

            Stmt::Assign { name, value, .. } => {
                let value_reg = self.compile_expr(value)?;
                if let Some(&target_reg) = self.vars.get(name) {
                    if value_reg != target_reg {
                        self.emit(&format!("    MOV R{:X}, R{:X}", target_reg, value_reg));
                    }
                    Ok(())
                } else {
                    Err(Error::SemanticError {
                        message: format!("Undefined variable: {}", name),
                    })
                }
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                Ok(())
            }

            Stmt::Return(value, _) => {
                if let Some(expr) = value {
                    let reg = self.compile_expr(expr)?;
                    if reg != 0 {
                        self.emit(&format!("    MOV R0, R{:X}", reg));
                    }
                }
                // In main function, use HLT instead of RET
                if self.current_function.as_deref() == Some("main") {
                    self.emit("    HLT");
                } else {
                    self.emit("    RET");
                }
                Ok(())
            }

            Stmt::Loop { body, .. } => {
                let loop_label = self.fresh_label("loop");
                let end_label = self.fresh_label("loop_end");

                // Push loop context for break/continue
                self.loop_stack.push(LoopContext {
                    start_label: loop_label.clone(),
                    end_label: end_label.clone(),
                });

                self.emit(&format!("{}:", loop_label));
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }
                self.emit(&format!("    JMP {}", loop_label));
                self.emit(&format!("{}:", end_label));

                // Pop loop context
                self.loop_stack.pop();

                Ok(())
            }

            Stmt::Break(_) => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.emit(&format!("    JMP {}", loop_ctx.end_label));
                    Ok(())
                } else {
                    Err(Error::CodeGenError {
                        message: "Break statement outside of loop".to_string(),
                    })
                }
            }

            Stmt::Continue(_) => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.emit(&format!("    JMP {}", loop_ctx.start_label));
                    Ok(())
                } else {
                    Err(Error::CodeGenError {
                        message: "Continue statement outside of loop".to_string(),
                    })
                }
            }

            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond_reg = self.compile_expr(condition)?;
                let else_label = self.fresh_label("else");
                let end_label = self.fresh_label("if_end");

                // Jump to else if condition is false (zero)
                self.emit(&format!("    JZ R{:X}, {}", cond_reg, else_label));

                // Then branch
                for stmt in then_body {
                    self.compile_stmt(stmt)?;
                }
                self.emit(&format!("    JMP {}", end_label));

                // Else branch
                self.emit(&format!("{}:", else_label));
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.compile_stmt(stmt)?;
                    }
                }

                self.emit(&format!("{}:", end_label));
                Ok(())
            }
        }
    }

    // ===== Expression Compilation =====

    fn compile_expr(&mut self, expr: &Expr) -> Result<u8> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),

            ExprKind::Ident(name) => {
                if let Some(&reg) = self.vars.get(name) {
                    Ok(reg)
                } else {
                    Err(Error::SemanticError {
                        message: format!("Undefined variable: {}", name),
                    })
                }
            }

            ExprKind::Binary { left, op, right } => {
                let left_reg = self.compile_expr(left)?;
                let right_reg = self.compile_expr(right)?;
                let dest_reg = self.alloc_reg();

                let instr = match op {
                    BinOp::Add => "ADD",
                    BinOp::Sub => "SUB",
                    BinOp::Mul => "MUL",
                    BinOp::Div => "DIV",
                    BinOp::Pow => "POW",
                    BinOp::Xor => "XOR",
                    BinOp::Eq => "CMP",   // TODO: proper comparison
                    BinOp::Ne => "CMP",
                    BinOp::Lt => "CMP",
                    BinOp::Le => "CMP",
                    BinOp::Gt => "CMP",
                    BinOp::Ge => "CMP",
                    BinOp::And => "AND",
                    BinOp::Or => "OR",
                    BinOp::BitAnd => "ANDL",
                    BinOp::BitOr => "ORL",
                };

                self.emit(&format!(
                    "    {} R{:X}, R{:X}, R{:X}",
                    instr, dest_reg, left_reg, right_reg
                ));

                Ok(dest_reg)
            }

            ExprKind::Unary { op, expr } => {
                let expr_reg = self.compile_expr(expr)?;
                let dest_reg = self.alloc_reg();

                let instr = match op {
                    UnOp::Neg => "NEG",
                    UnOp::Not => "NOT",
                    UnOp::Conj => "CONJ",
                    UnOp::Mag => "MAG",
                };

                self.emit(&format!("    {} R{:X}, R{:X}", instr, dest_reg, expr_reg));
                Ok(dest_reg)
            }

            ExprKind::Call { name, args } => {
                // Compile arguments into registers
                let arg_regs: Result<Vec<u8>> = args.iter().map(|arg| self.compile_expr(arg)).collect();
                let _arg_regs = arg_regs?;

                // Check if this is a stdlib intrinsic
                if self.is_intrinsic(name) {
                    self.emit(&format!("    ; stdlib intrinsic: {}", name));
                }

                // TODO: proper calling convention
                self.emit(&format!("    CALL {}", name));

                // Result is in R0
                Ok(0)
            }

            ExprKind::LayerAccess { expr, layer } => {
                let state_reg = self.compile_expr(expr)?;
                let dest_reg = self.alloc_reg();

                // Extract specific layer from state
                self.emit(&format!(
                    "    LOADL R{:X}, R{:X}, L{:X}",
                    dest_reg, state_reg, layer
                ));

                Ok(dest_reg)
            }

            ExprKind::StateConstruct { layers } => {
                let dest_reg = self.alloc_reg();

                // Initialize state to zero
                self.emit(&format!("    MOVI R{:X}, 0", dest_reg));

                // Set each layer
                for (layer, layer_expr) in layers {
                    let value_reg = self.compile_expr(layer_expr)?;
                    self.emit(&format!(
                        "    STOREL R{:X}, L{:X}, R{:X}",
                        dest_reg, layer, value_reg
                    ));
                }

                Ok(dest_reg)
            }

            ExprKind::Complex { rho, theta } => {
                let rho_reg = self.compile_expr(rho)?;
                let theta_reg = self.compile_expr(theta)?;
                let dest_reg = self.alloc_reg();

                // Construct complex value (rho, theta)
                self.emit(&format!(
                    "    COMPLEX R{:X}, R{:X}, R{:X}",
                    dest_reg, rho_reg, theta_reg
                ));

                Ok(dest_reg)
            }

            ExprKind::Tuple { elements } => {
                // Compile tuple elements into consecutive registers
                let dest_reg = self.alloc_reg();

                // For now, just compile all elements and return the first one's register
                // TODO: proper tuple representation
                for elem in elements {
                    self.compile_expr(elem)?;
                }

                self.emit(&format!("    ; tuple with {} elements", elements.len()));
                Ok(dest_reg)
            }

            ExprKind::Pipe { expr, transform } => {
                let input_reg = self.compile_expr(expr)?;
                let dest_reg = self.alloc_reg();

                // Apply transform
                self.emit(&format!("    MOV R0, R{:X}", input_reg));
                self.emit(&format!("    CALL {}", transform));
                self.emit(&format!("    MOV R{:X}, R0", dest_reg));

                Ok(dest_reg)
            }

            ExprKind::Feedback { expr } => {
                let expr_reg = self.compile_expr(expr)?;

                // Feedback: connect L(F) -> L(0)
                self.emit(&format!("    FEEDBACK R{:X}", expr_reg));

                Ok(expr_reg)
            }

            ExprKind::Emerge { expr } => {
                let expr_reg = self.compile_expr(expr)?;

                // Emergence detection
                self.emit(&format!("    EMERGE R{:X}", expr_reg));

                Ok(expr_reg)
            }
        }
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<u8> {
        let reg = self.alloc_reg();

        match lit {
            Literal::Int(n) => {
                self.emit(&format!("    MOVI R{:X}, {}", reg, n));
            }
            Literal::Float(f) => {
                // Convert float to ByteSil (rho, theta) format
                // For now, use (magnitude, 0) representation
                let rho_reg = self.alloc_reg();
                let theta_reg = self.alloc_reg();

                // Load magnitude as integer scaled by 1000 for precision
                let scaled = (f * 1000.0) as i64;
                self.emit(&format!("    MOVI R{:X}, {}", rho_reg, scaled));
                self.emit(&format!("    MOVI R{:X}, 0", theta_reg));

                // Create complex number (rho, theta)
                self.emit(&format!("    COMPLEX R{:X}, R{:X}, R{:X}", reg, rho_reg, theta_reg));
            }
            Literal::Bool(b) => {
                // Bool uses ByteSil log-polar representation:
                // - false = NULL (rho=-8, theta=0) = vacuum (magnitude ~0)
                // - true  = ONE  (rho=0, theta=0)  = neutral (magnitude 1)
                // JZ/JNZ test magnitude via is_null(), so this works correctly
                if *b {
                    // true = ByteSil::ONE (rho=0, theta=0)
                    let rho_reg = self.alloc_reg();
                    let theta_reg = self.alloc_reg();
                    self.emit(&format!("    MOVI R{:X}, 0", rho_reg));
                    self.emit(&format!("    MOVI R{:X}, 0", theta_reg));
                    self.emit(&format!("    COMPLEX R{:X}, R{:X}, R{:X}", reg, rho_reg, theta_reg));
                } else {
                    // false = ByteSil::NULL via MOVI 0 (from_u8(0) → rho=-8)
                    self.emit(&format!("    MOVI R{:X}, 0", reg));
                }
            }
            Literal::String(s) => {
                // TODO: proper string handling
                self.emit(&format!("    ; String literal: {}", s));
                self.emit(&format!("    MOVI R{:X}, 0", reg));
            }
        }

        Ok(reg)
    }

    // ===== Helper Methods =====

    fn emit(&mut self, line: &str) {
        self.output.push(line.to_string());
    }

    fn alloc_reg(&mut self) -> u8 {
        let reg = self.next_reg;
        self.next_reg = (self.next_reg + 1) % 16; // Wrap around at 16 registers
        reg
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{}{}", prefix, self.next_label);
        self.next_label += 1;
        label
    }

    // ===== JSIL Compilation =====

    #[cfg(feature = "jsil")]
    /// Compiles program directly to JSONL records for JSIL output
    pub fn compile_to_jsonl(&mut self, program: &Program) -> Result<Vec<JsonlRecord>> {
        // Type check first
        let mut type_checker = TypeChecker::new();
        let _typed_program = type_checker.check_program(program)?;

        let mut records = Vec::new();
        let mut current_addr: u32 = 0;

        // 1. Metadata record
        records.push(JsonlRecord::Metadata {
            version: "1.0".into(),
            mode: "Sil128".into(),
            entry_point: 0,
            code_size: 0, // Will be updated later
            data_size: 0,
            symbol_count: program.items.len() as u32,
            checksum: String::new(), // Will be computed by JsilWriter
            source_file: None,
        });

        // 2. Symbol records
        for item in &program.items {
            match item {
                Item::Function { name, .. } => {
                    records.push(JsonlRecord::Symbol {
                        name: name.clone(),
                        addr: current_addr,
                        kind: "function".into(),
                    });
                }
                Item::Transform { name, .. } => {
                    records.push(JsonlRecord::Symbol {
                        name: name.clone(),
                        addr: current_addr,
                        kind: "transform".into(),
                    });
                }
                _ => {}
            }
        }

        // 3. Compile to assembly first, then convert to instruction records
        // Reset compiler state
        self.output.clear();
        self.next_reg = 0;
        self.vars.clear();
        self.next_label = 0;

        for item in &program.items {
            self.compile_item(item)?;
        }

        // 4. Parse assembly output into instruction records
        for line in &self.output {
            let trimmed = line.trim();

            // Skip empty lines and directives
            if trimmed.is_empty() || trimmed.starts_with('.') {
                continue;
            }

            // Skip labels (they're in symbol table already)
            if trimmed.ends_with(':') {
                continue;
            }

            // Parse instruction
            if let Some((op, args_str)) = trimmed.split_once(char::is_whitespace) {
                let args = self.parse_instruction_args(args_str);

                records.push(JsonlRecord::Instruction {
                    addr: current_addr,
                    op: op.to_string(),
                    bytes: self.instruction_to_bytes(op, &args),
                    args: Some(args),
                    line: None,
                });

                current_addr += 4; // Assume 4 bytes per instruction
            } else if !trimmed.is_empty() {
                // Single instruction with no args
                records.push(JsonlRecord::Instruction {
                    addr: current_addr,
                    op: trimmed.to_string(),
                    bytes: self.instruction_to_bytes(trimmed, &InstructionArgs {
                        reg: None,
                        reg_a: None,
                        reg_b: None,
                        imm: None,
                        addr: None,
                        label: None,
                    }),
                    args: None,
                    line: None,
                });

                current_addr += 4;
            }
        }

        // 5. Update metadata with final code size
        if let Some(JsonlRecord::Metadata { code_size, .. }) = records.get_mut(0) {
            *code_size = current_addr;
        }

        Ok(records)
    }

    #[cfg(feature = "jsil")]
    fn parse_instruction_args(&self, args_str: &str) -> InstructionArgs {
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        let mut args = InstructionArgs {
            reg: None,
            reg_a: None,
            reg_b: None,
            imm: None,
            addr: None,
            label: None,
        };

        for (i, part) in parts.iter().enumerate() {
            if part.starts_with('R') || part.starts_with('r') {
                // Register
                let reg_name = part.to_uppercase();
                match i {
                    0 => args.reg = Some(reg_name.clone()),
                    1 => args.reg_a = Some(reg_name.clone()),
                    2 => args.reg_b = Some(reg_name),
                    _ => {}
                }
            } else if let Ok(num) = part.parse::<u32>() {
                // Immediate value
                if args.imm.is_none() {
                    args.imm = Some(num);
                } else {
                    args.addr = Some(num);
                }
            } else if part.starts_with("0x") {
                // Hex immediate
                if let Ok(num) = u32::from_str_radix(&part[2..], 16) {
                    args.imm = Some(num);
                }
            } else {
                // Label
                args.label = Some(part.to_string());
            }
        }

        args
    }

    #[cfg(feature = "jsil")]
    fn instruction_to_bytes(&self, op: &str, args: &InstructionArgs) -> String {
        // Simplified bytecode generation
        // In a real implementation, this would use the VSP instruction encoding
        let mut bytes = Vec::new();

        // Opcode (simplified mapping)
        let opcode = match op {
            "NOP" => 0x00,
            "MOVI" => 0x10,
            "MOV" => 0x11,
            "ADD" => 0x20,
            "SUB" => 0x21,
            "MUL" => 0x22,
            "DIV" => 0x23,
            "CALL" => 0x30,
            "RET" => 0x31,
            "JMP" => 0x40,
            "JZ" => 0x41,
            _ => 0xFF, // Unknown
        };
        bytes.push(opcode);

        // Encode registers (simplified)
        if let Some(ref reg) = args.reg {
            if let Some(hex) = reg.strip_prefix('R') {
                if let Ok(num) = u8::from_str_radix(hex, 16) {
                    bytes.push(num);
                }
            }
        }

        if let Some(ref reg_a) = args.reg_a {
            if let Some(hex) = reg_a.strip_prefix('R') {
                if let Ok(num) = u8::from_str_radix(hex, 16) {
                    bytes.push(num);
                }
            }
        }

        if let Some(ref reg_b) = args.reg_b {
            if let Some(hex) = reg_b.strip_prefix('R') {
                if let Ok(num) = u8::from_str_radix(hex, 16) {
                    bytes.push(num);
                }
            }
        }

        // Encode immediate (simplified, little-endian u32)
        if let Some(imm) = args.imm {
            bytes.extend_from_slice(&imm.to_le_bytes());
        }

        // Convert to hex string
        bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>()
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile(source: &str) -> Result<String> {
        let tokens = Lexer::new(source).tokenize_with_spans()?;
        let program = Parser::new(tokens).parse()?;
        let mut compiler = Compiler::new();
        compiler.compile(&program)
    }

    #[test]
    fn test_compile_empty_function() {
        let source = "fn main() {}";
        let asm = compile(source).unwrap();
        assert!(asm.contains("main:"));
        // main function uses HLT (halt) instead of RET (return)
        assert!(asm.contains("HLT"), "main should end with HLT");
    }

    #[test]
    fn test_compile_let_statement() {
        let source = "fn main() { let x = 42; }";
        let asm = compile(source).unwrap();
        assert!(asm.contains("MOVI"));
    }

    #[test]
    fn test_compile_arithmetic() {
        let source = "fn main() { let x = 1 + 2 * 3; }";
        let asm = compile(source).unwrap();
        assert!(asm.contains("MUL"));
        assert!(asm.contains("ADD"));
    }

    #[test]
    fn test_compile_function_call() {
        let source = r#"
            fn helper() {
                return 42;
            }
            fn main() {
                let x = helper();
            }
        "#;
        let asm = compile(source).unwrap();
        assert!(asm.contains("CALL helper"));
    }

    #[test]
    fn test_compile_if_statement() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else {
                    let x = 2;
                }
            }
        "#;
        let asm = compile(source).unwrap();
        assert!(asm.contains("JZ"));
        assert!(asm.contains("JMP"));
    }

    #[test]
    fn test_compile_loop_with_break() {
        let source = r#"
            fn main() {
                let x = 0;
                loop {
                    x = x + 1;
                    if x == 5 {
                        break;
                    }
                }
            }
        "#;
        let asm = compile(source).unwrap();
        assert!(asm.contains("loop"));
        assert!(asm.contains("loop_end"));
        assert!(asm.contains("JMP loop_end"));
    }

    #[test]
    fn test_compile_loop_with_continue() {
        let source = r#"
            fn main() {
                let x = 0;
                loop {
                    x = x + 1;
                    if x < 10 {
                        continue;
                    }
                    break;
                }
            }
        "#;
        let asm = compile(source).unwrap();
        assert!(asm.contains("loop"));
        assert!(asm.contains("JMP loop0")); // continue jumps to loop start
    }

    #[test]
    fn test_break_outside_loop_fails() {
        let source = r#"
            fn main() {
                break;
            }
        "#;
        let result = compile(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside"));
    }

    #[test]
    fn test_continue_outside_loop_fails() {
        let source = r#"
            fn main() {
                continue;
            }
        "#;
        let result = compile(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside"));
    }
}
