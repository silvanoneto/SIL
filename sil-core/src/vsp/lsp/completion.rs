//! Provider de completions

use super::{Document, Position};
use super::server::{CompletionItem, CompletionItemKind, InsertTextFormat};

/// Provider de completions
pub struct CompletionProvider {
    /// Completions de opcodes
    opcodes: Vec<CompletionItem>,
    /// Completions de registradores
    registers: Vec<CompletionItem>,
    /// Completions de diretivas
    directives: Vec<CompletionItem>,
    /// Snippets
    snippets: Vec<CompletionItem>,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            opcodes: build_opcode_completions(),
            registers: build_register_completions(),
            directives: build_directive_completions(),
            snippets: build_snippet_completions(),
        }
    }
    
    /// Fornece completions para posição
    pub fn provide(&self, doc: &Document, pos: Position) -> Vec<CompletionItem> {
        let line = match doc.get_line(pos.line) {
            Some(l) => l,
            None => return Vec::new(),
        };
        
        let prefix = &line[..pos.character as usize];
        let trimmed = prefix.trim_start();
        
        // Contexto
        if trimmed.is_empty() || is_at_instruction_start(prefix) {
            // Início de linha - sugerir opcodes, diretivas, snippets
            let mut items = Vec::new();
            items.extend(self.opcodes.iter().cloned());
            items.extend(self.directives.iter().cloned());
            items.extend(self.snippets.iter().cloned());
            return filter_completions(items, trimmed);
        }
        
        if trimmed.starts_with('.') {
            // Diretiva
            return filter_completions(self.directives.clone(), trimmed);
        }
        
        if prefix.contains(',') || prefix.contains(' ') {
            // Após vírgula ou espaço - sugerir registradores
            let last_word = get_last_word(prefix);
            return filter_completions(self.registers.clone(), last_word);
        }
        
        // Default - sugerir tudo
        let mut items = Vec::new();
        items.extend(self.opcodes.iter().cloned());
        items.extend(self.registers.iter().cloned());
        filter_completions(items, trimmed)
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn is_at_instruction_start(prefix: &str) -> bool {
    prefix.trim().is_empty() || prefix.ends_with(':')
}

fn get_last_word(s: &str) -> &str {
    s.split(|c: char| c.is_whitespace() || c == ',')
        .last()
        .unwrap_or("")
}

fn filter_completions(items: Vec<CompletionItem>, prefix: &str) -> Vec<CompletionItem> {
    let prefix_upper = prefix.to_uppercase();
    items.into_iter()
        .filter(|item| {
            prefix.is_empty() || 
            item.label.to_uppercase().starts_with(&prefix_upper)
        })
        .collect()
}

fn build_opcode_completions() -> Vec<CompletionItem> {
    vec![
        // Control
        opcode_completion("NOP", "No operation", "NOP"),
        opcode_completion("HLT", "Halt execution", "HLT"),
        opcode_completion("RET", "Return from subroutine", "RET"),
        opcode_completion("YIELD", "Yield to scheduler", "YIELD"),
        opcode_completion("JMP", "Jump to address", "JMP ${1:label}"),
        opcode_completion("JZ", "Jump if zero", "JZ ${1:label}"),
        opcode_completion("JN", "Jump if negative", "JN ${1:label}"),
        opcode_completion("JC", "Jump if collapse", "JC ${1:label}"),
        opcode_completion("JO", "Jump if overflow", "JO ${1:label}"),
        opcode_completion("CALL", "Call subroutine", "CALL ${1:label}"),
        opcode_completion("LOOP", "Decrement and jump if not zero", "LOOP ${1:label}"),
        
        // Data
        opcode_completion("MOV", "Move register to register", "MOV ${1:Rd}, ${2:Rs}"),
        opcode_completion("MOVI", "Move immediate to register", "MOVI ${1:Rd}, ${2:imm}"),
        opcode_completion("LOAD", "Load from memory", "LOAD ${1:Rd}, ${2:addr}"),
        opcode_completion("STORE", "Store to memory", "STORE ${1:Rs}, ${2:addr}"),
        opcode_completion("PUSH", "Push to stack", "PUSH ${1:Rs}"),
        opcode_completion("POP", "Pop from stack", "POP ${1:Rd}"),
        opcode_completion("XCHG", "Exchange registers", "XCHG ${1:R1}, ${2:R2}"),
        opcode_completion("LSTATE", "Load full state (128 bits)", "LSTATE ${1:Rd}, ${2:addr}"),
        opcode_completion("SSTATE", "Store full state (128 bits)", "SSTATE ${1:Rs}, ${2:addr}"),
        
        // Arithmetic
        opcode_completion("MUL", "Multiply (ρ+, θ+)", "MUL ${1:Rd}, ${2:Rs}"),
        opcode_completion("DIV", "Divide (ρ-, θ-)", "DIV ${1:Rd}, ${2:Rs}"),
        opcode_completion("POW", "Power (ρ×n, θ×n)", "POW ${1:Rd}, ${2:n}"),
        opcode_completion("ROOT", "Root (ρ÷n, θ÷n)", "ROOT ${1:Rd}, ${2:n}"),
        opcode_completion("INV", "Inverse (-ρ, -θ)", "INV ${1:Rd}"),
        opcode_completion("CONJ", "Conjugate (-θ)", "CONJ ${1:Rd}"),
        opcode_completion("ADD", "Add (cartesian)", "ADD ${1:Rd}, ${2:Rs}"),
        opcode_completion("SUB", "Subtract (cartesian)", "SUB ${1:Rd}, ${2:Rs}"),
        opcode_completion("MAG", "Extract magnitude", "MAG ${1:Rd}"),
        opcode_completion("PHASE", "Extract phase", "PHASE ${1:Rd}"),
        opcode_completion("SCALE", "Scale magnitude", "SCALE ${1:Rd}, ${2:factor}"),
        opcode_completion("ROTATE", "Rotate phase", "ROTATE ${1:Rd}, ${2:angle}"),
        
        // Layer
        opcode_completion("XORL", "XOR layers", "XORL ${1:Rd}, ${2:Rs}"),
        opcode_completion("ANDL", "AND layers", "ANDL ${1:Rd}, ${2:Rs}"),
        opcode_completion("ORL", "OR layers", "ORL ${1:Rd}, ${2:Rs}"),
        opcode_completion("NOTL", "NOT layer", "NOTL ${1:Rd}"),
        opcode_completion("SHIFTL", "Shift layers", "SHIFTL"),
        opcode_completion("ROTATL", "Rotate layers circular", "ROTATL"),
        opcode_completion("FOLD", "Fold layers (R[i] ⊕ R[i+8])", "FOLD"),
        opcode_completion("SPREAD", "Spread to group", "SPREAD ${1:Rd}"),
        opcode_completion("GATHER", "Gather from group", "GATHER ${1:Rd}"),
        
        // Transform
        opcode_completion("TRANS", "Apply transformation", "TRANS ${1:Rd}, ${2:transform_id}"),
        opcode_completion("PIPE", "Pipeline transformations", "PIPE ${1:Rd}, ${2:pipeline_id}"),
        opcode_completion("LERP", "Linear interpolation", "LERP ${1:Rd}, ${2:Rs}, ${3:t}"),
        opcode_completion("SLERP", "Spherical interpolation", "SLERP ${1:Rd}, ${2:Rs}, ${3:t}"),
        opcode_completion("GRAD", "Calculate gradient", "GRAD ${1:Rd}"),
        opcode_completion("DESCENT", "Gradient descent", "DESCENT ${1:Rd}, ${2:Rs}, ${3:lr}"),
        opcode_completion("EMERGE", "Emergence (NPU)", "EMERGE ${1:Rd}"),
        opcode_completion("COLLAPSE", "Collapse state", "COLLAPSE ${1:Rd}"),
        
        // Compat
        opcode_completion("SETMODE", "Set SIL mode", "SETMODE ${1:mode}"),
        opcode_completion("PROMOTE", "Promote to larger mode", "PROMOTE ${1:Rd}, ${2:Rs}"),
        opcode_completion("DEMOTE", "Demote to smaller mode", "DEMOTE ${1:Rd}, ${2:Rs}"),
        opcode_completion("TRUNCATE", "Demote by truncation", "TRUNCATE ${1:Rd}, ${2:Rs}"),
        opcode_completion("XORDEM", "Demote by XOR", "XORDEM ${1:Rd}, ${2:Rs}"),
        opcode_completion("AVGDEM", "Demote by average", "AVGDEM ${1:Rd}, ${2:Rs}"),
        opcode_completion("MAXDEM", "Demote by maximum", "MAXDEM ${1:Rd}, ${2:Rs}"),
        opcode_completion("COMPAT", "Negotiate compatibility", "COMPAT ${1:Rd}, ${2:Rs}"),
        
        // System
        opcode_completion("IN", "Input from port", "IN ${1:Rd}, ${2:port}"),
        opcode_completion("OUT", "Output to port", "OUT ${1:Rs}, ${2:port}"),
        opcode_completion("SENSE", "Read sensor", "SENSE ${1:Rd}"),
        opcode_completion("ACT", "Write actuator", "ACT ${1:Rs}"),
        opcode_completion("SYNC", "Synchronize with node", "SYNC ${1:node}"),
        opcode_completion("BROADCAST", "Broadcast state", "BROADCAST ${1:Rs}"),
        opcode_completion("RECEIVE", "Receive state", "RECEIVE ${1:Rd}"),
        opcode_completion("ENTANGLE", "Entangle with remote node", "ENTANGLE ${1:Rd}, ${2:node}"),
        
        // Hints
        opcode_completion("HINT.CPU", "Prefer CPU backend", "HINT.CPU"),
        opcode_completion("HINT.GPU", "Prefer GPU backend", "HINT.GPU"),
        opcode_completion("HINT.NPU", "Prefer NPU backend", "HINT.NPU"),
        opcode_completion("HINT.ANY", "Any backend", "HINT.ANY"),
        opcode_completion("BATCH", "Start batch", "BATCH ${1:size}"),
        opcode_completion("UNBATCH", "End batch", "UNBATCH"),
        opcode_completion("PREFETCH", "Prefetch memory", "PREFETCH ${1:addr}"),
        opcode_completion("FENCE", "Memory fence", "FENCE"),
        opcode_completion("SYSCALL", "System call", "SYSCALL ${1:num}"),
    ]
}

fn opcode_completion(label: &str, detail: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: CompletionItemKind::Function,
        detail: Some(detail.to_string()),
        documentation: None,
        insert_text: Some(snippet.to_string()),
        insert_text_format: if snippet.contains('$') {
            InsertTextFormat::Snippet
        } else {
            InsertTextFormat::PlainText
        },
    }
}

fn build_register_completions() -> Vec<CompletionItem> {
    let descriptions = [
        ("R0", "L(0) Fotônico - Percepção visual"),
        ("R1", "L(1) Acústico - Percepção auditiva"),
        ("R2", "L(2) Olfativo - Percepção olfativa"),
        ("R3", "L(3) Gustativo - Percepção gustativa"),
        ("R4", "L(4) Dérmico - Percepção tátil"),
        ("R5", "L(5) Eletrônico - Processamento elétrico"),
        ("R6", "L(6) Psicomotor - Controle motor"),
        ("R7", "L(7) Ambiental - Contexto ambiental"),
        ("R8", "L(8) Cibernético - Interação digital"),
        ("R9", "L(9) Geopolítico - Contexto geográfico"),
        ("RA", "L(A) Cosmopolítico - Contexto global"),
        ("RB", "L(B) Sinérgico - Emergência sinérgica"),
        ("RC", "L(C) Quântico - Estado quântico"),
        ("RD", "L(D) Superposição - Meta-estado"),
        ("RE", "L(E) Entanglement - Correlação remota"),
        ("RF", "L(F) Colapso - Estado colapsado"),
    ];
    
    descriptions.iter().map(|(reg, desc)| {
        CompletionItem {
            label: reg.to_string(),
            kind: CompletionItemKind::Variable,
            detail: Some(desc.to_string()),
            documentation: None,
            insert_text: Some(reg.to_string()),
            insert_text_format: InsertTextFormat::PlainText,
        }
    }).collect()
}

fn build_directive_completions() -> Vec<CompletionItem> {
    vec![
        directive_completion(".mode", "Set SIL mode", ".mode ${1|SIL-8,SIL-16,SIL-32,SIL-64,SIL-128|}"),
        directive_completion(".code", "Start code section", ".code"),
        directive_completion(".text", "Start code section (alias)", ".text"),
        directive_completion(".data", "Start data section", ".data"),
        directive_completion(".global", "Export symbol globally", ".global ${1:symbol}"),
        directive_completion(".globl", "Export symbol (alias)", ".globl ${1:symbol}"),
        directive_completion(".extern", "Import external symbol", ".extern ${1:symbol}"),
        directive_completion(".align", "Align to boundary", ".align ${1:4}"),
        directive_completion(".byte", "Define byte data", ".byte ${1:0x00}"),
        directive_completion(".state", "Define SilState constant", ".state ${1:name}, [${2:bytes}]"),
    ]
}

fn directive_completion(label: &str, detail: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: CompletionItemKind::Keyword,
        detail: Some(detail.to_string()),
        documentation: None,
        insert_text: Some(snippet.to_string()),
        insert_text_format: if snippet.contains('$') {
            InsertTextFormat::Snippet
        } else {
            InsertTextFormat::PlainText
        },
    }
}

fn build_snippet_completions() -> Vec<CompletionItem> {
    vec![
        snippet_completion(
            "sil-program",
            "Complete SIL program template",
            r#"; SIL Program
.mode SIL-128

.data
    ; Data declarations here

.code
.global _start

_start:
    $0
    HLT
"#
        ),
        snippet_completion(
            "sil-loop",
            "Loop construct",
            r#"${1:loop_name}:
    $0
    LOOP ${1:loop_name}
"#
        ),
        snippet_completion(
            "sil-function",
            "Function definition",
            r#"${1:func_name}:
    PUSH R0
    $0
    POP R0
    RET
"#
        ),
        snippet_completion(
            "sil-gradient-descent",
            "Gradient descent iteration",
            r#"; Gradient descent
    GRAD R0
    DESCENT R0, R1, ${1:0x10}  ; learning rate
"#
        ),
        snippet_completion(
            "sil-entangle",
            "Entanglement setup",
            r#"; Entangle with remote
    ENTANGLE R0, ${1:node_id}
    SYNC R0
"#
        ),
    ]
}

fn snippet_completion(label: &str, detail: &str, body: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: CompletionItemKind::Snippet,
        detail: Some(detail.to_string()),
        documentation: None,
        insert_text: Some(body.to_string()),
        insert_text_format: InsertTextFormat::Snippet,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_completion_at_line_start() {
        let provider = CompletionProvider::new();
        // Documento com uma linha vazia (não documento totalmente vazio)
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "\n".to_string(),
        );
        
        let completions = provider.provide(&doc, Position { line: 0, character: 0 });
        assert!(!completions.is_empty());
    }
    
    #[test]
    fn test_filter_completions() {
        let provider = CompletionProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "MO".to_string(),
        );
        
        let completions = provider.provide(&doc, Position { line: 0, character: 2 });
        assert!(completions.iter().any(|c| c.label == "MOV"));
        assert!(completions.iter().any(|c| c.label == "MOVI"));
    }
}
