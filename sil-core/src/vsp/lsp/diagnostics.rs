//! Provider de diagnósticos

use super::{Document, Range};
use super::server::{Diagnostic, DiagnosticSeverity};

/// Provider de diagnósticos
pub struct DiagnosticsProvider {
    /// Opcodes válidos
    valid_opcodes: Vec<String>,
}

impl DiagnosticsProvider {
    pub fn new() -> Self {
        Self {
            valid_opcodes: build_valid_opcodes(),
        }
    }
    
    /// Analisa documento e retorna diagnósticos
    pub fn analyze(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut in_code_section = false;
        let mut labels: Vec<String> = Vec::new();
        let mut label_refs: Vec<(String, u32)> = Vec::new();
        
        for (line_num, line) in doc.lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_num = line_num as u32;
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }
            
            // Track sections
            if trimmed == ".code" || trimmed == ".text" {
                in_code_section = true;
                continue;
            }
            if trimmed == ".data" {
                in_code_section = false;
                continue;
            }
            
            // Check directives
            if trimmed.starts_with('.') {
                if let Some(diag) = self.check_directive(trimmed, line_num, line) {
                    diagnostics.push(diag);
                }
                continue;
            }
            
            // Check labels
            if trimmed.ends_with(':') {
                let label_name = &trimmed[..trimmed.len()-1];
                if labels.contains(&label_name.to_string()) {
                    diagnostics.push(Diagnostic {
                        range: Range::single_line(line_num, 0, trimmed.len() as u32),
                        severity: DiagnosticSeverity::Error,
                        code: Some("E002".to_string()),
                        source: "sil".to_string(),
                        message: format!("Duplicate label: '{}'", label_name),
                    });
                }
                labels.push(label_name.to_string());
                continue;
            }
            
            // Check instructions in code section
            if in_code_section {
                let diags = self.check_instruction(trimmed, line_num, line, &mut label_refs);
                diagnostics.extend(diags);
            }
        }
        
        // Check undefined labels
        for (label_ref, line_num) in label_refs {
            if !labels.contains(&label_ref) {
                diagnostics.push(Diagnostic {
                    range: Range::single_line(line_num, 0, 10),
                    severity: DiagnosticSeverity::Error,
                    code: Some("E003".to_string()),
                    source: "sil".to_string(),
                    message: format!("Undefined label: '{}'", label_ref),
                });
            }
        }
        
        diagnostics
    }
    
    fn check_directive(&self, trimmed: &str, line_num: u32, full_line: &str) -> Option<Diagnostic> {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let directive = parts.first()?;
        
        let valid_directives = [
            ".mode", ".code", ".text", ".data", ".global", ".globl",
            ".extern", ".align", ".byte", ".state",
        ];
        
        if !valid_directives.contains(&directive.to_lowercase().as_str()) {
            return Some(Diagnostic {
                range: Range::single_line(line_num, 0, directive.len() as u32),
                severity: DiagnosticSeverity::Error,
                code: Some("E004".to_string()),
                source: "sil".to_string(),
                message: format!("Unknown directive: '{}'", directive),
            });
        }
        
        // Check .mode argument
        if *directive == ".mode" {
            if let Some(mode) = parts.get(1) {
                let valid_modes = ["SIL-8", "SIL-16", "SIL-32", "SIL-64", "SIL-128"];
                if !valid_modes.contains(&mode.to_uppercase().as_str()) {
                    return Some(Diagnostic {
                        range: Range::single_line(line_num, 0, full_line.len() as u32),
                        severity: DiagnosticSeverity::Warning,
                        code: Some("W001".to_string()),
                        source: "sil".to_string(),
                        message: format!("Unknown mode '{}'. Valid modes: SIL-8, SIL-16, SIL-32, SIL-64, SIL-128", mode),
                    });
                }
            }
        }
        
        None
    }
    
    fn check_instruction(
        &self,
        trimmed: &str,
        line_num: u32,
        full_line: &str,
        label_refs: &mut Vec<(String, u32)>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Parse instruction
        let parts: Vec<&str> = trimmed
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.is_empty() {
            return diagnostics;
        }
        
        let mnemonic = parts[0].to_uppercase();
        let operands = &parts[1..];
        
        // Check valid opcode
        if !self.valid_opcodes.contains(&mnemonic) {
            let start = full_line.find(parts[0]).unwrap_or(0) as u32;
            diagnostics.push(Diagnostic {
                range: Range::single_line(line_num, start, start + mnemonic.len() as u32),
                severity: DiagnosticSeverity::Error,
                code: Some("E001".to_string()),
                source: "sil".to_string(),
                message: format!("Unknown opcode: '{}'", mnemonic),
            });
            return diagnostics;
        }
        
        // Check operands
        for op in operands {
            // Register validation
            if op.starts_with('R') || op.starts_with('r') {
                if let Some(reg_num) = op[1..].parse::<u32>().ok() {
                    if reg_num > 15 {
                        let start = full_line.find(op).unwrap_or(0) as u32;
                        diagnostics.push(Diagnostic {
                            range: Range::single_line(line_num, start, start + op.len() as u32),
                            severity: DiagnosticSeverity::Error,
                            code: Some("E005".to_string()),
                            source: "sil".to_string(),
                            message: format!("Invalid register '{}'. Valid: R0-R15 (or RA-RF)", op),
                        });
                    }
                } else if !["RA", "RB", "RC", "RD", "RE", "RF", "ra", "rb", "rc", "rd", "re", "rf"]
                    .contains(&op.to_uppercase().as_str())
                {
                    // Not a valid hex register
                    // Might be a label reference
                    if op.chars().skip(1).all(|c| c.is_alphabetic() || c == '_') {
                        label_refs.push((op.to_string(), line_num));
                    }
                }
            }
            // Label reference (for jumps)
            else if op.chars().all(|c| c.is_alphanumeric() || c == '_') {
                // Could be a label or immediate
                if !op.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                    label_refs.push((op.to_string(), line_num));
                }
            }
        }
        
        // Check operand count
        let expected_operands = get_expected_operands(&mnemonic);
        if let Some((min, max)) = expected_operands {
            if operands.len() < min || operands.len() > max {
                diagnostics.push(Diagnostic {
                    range: Range::single_line(line_num, 0, full_line.len() as u32),
                    severity: DiagnosticSeverity::Error,
                    code: Some("E006".to_string()),
                    source: "sil".to_string(),
                    message: if min == max {
                        format!("{} expects {} operand(s), found {}", mnemonic, min, operands.len())
                    } else {
                        format!("{} expects {}-{} operand(s), found {}", mnemonic, min, max, operands.len())
                    },
                });
            }
        }
        
        diagnostics
    }
}

impl Default for DiagnosticsProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn build_valid_opcodes() -> Vec<String> {
    vec![
        // Control
        "NOP", "HLT", "RET", "YIELD", "JMP", "JZ", "JN", "JC", "JO", "CALL", "LOOP",
        // Data
        "MOV", "MOVI", "LOAD", "STORE", "PUSH", "POP", "XCHG", "LSTATE", "SSTATE",
        // Arithmetic
        "MUL", "DIV", "POW", "ROOT", "INV", "CONJ", "ADD", "SUB", "MAG", "PHASE", "SCALE", "ROTATE",
        // Layer
        "XORL", "ANDL", "ORL", "NOTL", "SHIFTL", "ROTATL", "FOLD", "SPREAD", "GATHER",
        // Transform
        "TRANS", "PIPE", "LERP", "SLERP", "GRAD", "DESCENT", "EMERGE", "COLLAPSE",
        // Compat
        "SETMODE", "PROMOTE", "DEMOTE", "TRUNCATE", "XORDEM", "AVGDEM", "MAXDEM", "COMPAT",
        // System
        "IN", "OUT", "SENSE", "ACT", "SYNC", "BROADCAST", "RECEIVE", "ENTANGLE",
        // Hints
        "HINT.CPU", "HINT.GPU", "HINT.NPU", "HINT.ANY", "BATCH", "UNBATCH", "PREFETCH", "FENCE", "SYSCALL",
    ].into_iter().map(String::from).collect()
}

fn get_expected_operands(mnemonic: &str) -> Option<(usize, usize)> {
    Some(match mnemonic {
        // No operands
        "NOP" | "HLT" | "RET" | "YIELD" | "SHIFTL" | "ROTATL" | "FOLD" |
        "HINT.CPU" | "HINT.GPU" | "HINT.NPU" | "HINT.ANY" | "UNBATCH" | "FENCE" => (0, 0),
        
        // 1 operand
        "PUSH" | "POP" | "INV" | "CONJ" | "MAG" | "PHASE" | "NOTL" |
        "SPREAD" | "GATHER" | "GRAD" | "EMERGE" | "COLLAPSE" |
        "SETMODE" | "SENSE" | "ACT" | "SYNC" => (1, 1),
        
        // 1-2 operands (jumps can have register or address)
        "JMP" | "JZ" | "JN" | "JC" | "JO" | "CALL" | "LOOP" => (1, 1),
        
        // 2 operands
        "MOV" | "MOVI" | "LOAD" | "STORE" | "XCHG" | "LSTATE" | "SSTATE" |
        "MUL" | "DIV" | "POW" | "ROOT" | "ADD" | "SUB" | "SCALE" | "ROTATE" |
        "XORL" | "ANDL" | "ORL" | "TRANS" | "PIPE" |
        "PROMOTE" | "DEMOTE" | "TRUNCATE" | "XORDEM" | "AVGDEM" | "MAXDEM" | "COMPAT" |
        "IN" | "OUT" | "ENTANGLE" => (2, 2),
        
        // 3 operands
        "LERP" | "SLERP" | "DESCENT" => (3, 3),
        
        // Variable operands
        "BROADCAST" | "RECEIVE" => (1, 2),
        "BATCH" | "PREFETCH" | "SYSCALL" => (1, 1),
        
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_program() {
        let provider = DiagnosticsProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".code\nstart:\n    NOP\n    HLT".to_string(),
        );
        
        let diags = provider.analyze(&doc);
        assert!(diags.is_empty(), "Expected no diagnostics: {:?}", diags);
    }
    
    #[test]
    fn test_unknown_opcode() {
        let provider = DiagnosticsProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".code\n    INVALID".to_string(),
        );
        
        let diags = provider.analyze(&doc);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Unknown opcode"));
    }
    
    #[test]
    fn test_undefined_label() {
        let provider = DiagnosticsProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".code\n    JMP undefined_label".to_string(),
        );
        
        let diags = provider.analyze(&doc);
        assert!(diags.iter().any(|d| d.message.contains("Undefined label")));
    }
    
    #[test]
    fn test_invalid_register() {
        let provider = DiagnosticsProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".code\n    MOV R99, R0".to_string(),
        );
        
        let diags = provider.analyze(&doc);
        assert!(diags.iter().any(|d| d.message.contains("Invalid register")));
    }
}
