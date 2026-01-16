//! Provider de hover

use std::collections::HashMap;

use super::{Document, Position, Range};
use super::server::HoverResult;

/// Provider de hover
pub struct HoverProvider {
    /// Documentação de opcodes
    opcode_docs: HashMap<String, OpcodeDoc>,
    /// Documentação de registradores
    register_docs: HashMap<String, RegisterDoc>,
    /// Documentação de diretivas
    directive_docs: HashMap<String, DirectiveDoc>,
}

struct OpcodeDoc {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    format: &'static str,
    example: &'static str,
    encoding: &'static str,
}

struct RegisterDoc {
    name: &'static str,
    layer: &'static str,
    description: &'static str,
    semantic: &'static str,
}

struct DirectiveDoc {
    name: &'static str,
    description: &'static str,
    syntax: &'static str,
    example: &'static str,
}

impl HoverProvider {
    pub fn new() -> Self {
        Self {
            opcode_docs: build_opcode_docs(),
            register_docs: build_register_docs(),
            directive_docs: build_directive_docs(),
        }
    }
    
    /// Fornece hover para posição
    pub fn provide(&self, doc: &Document, pos: Position) -> Option<HoverResult> {
        let line = doc.get_line(pos.line)?;
        let (word, range) = get_word_at(line, pos.character, pos.line)?;
        let word_upper = word.to_uppercase();
        
        // Verificar opcode
        if let Some(doc) = self.opcode_docs.get(&word_upper) {
            return Some(HoverResult {
                contents: format_opcode_hover(doc),
                range: Some(range),
            });
        }
        
        // Verificar registrador
        if let Some(doc) = self.register_docs.get(&word_upper) {
            return Some(HoverResult {
                contents: format_register_hover(doc),
                range: Some(range),
            });
        }
        
        // Verificar diretiva
        let word_with_dot = if word.starts_with('.') {
            word.to_lowercase()
        } else {
            format!(".{}", word.to_lowercase())
        };
        
        if let Some(doc) = self.directive_docs.get(&word_with_dot) {
            return Some(HoverResult {
                contents: format_directive_hover(doc),
                range: Some(range),
            });
        }
        
        None
    }
}

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn get_word_at(line: &str, col: u32, line_num: u32) -> Option<(String, Range)> {
    let chars: Vec<char> = line.chars().collect();
    let col = col as usize;
    
    if col >= chars.len() {
        return None;
    }
    
    // Find word boundaries
    let mut start = col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    
    let mut end = col;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    
    if start == end {
        return None;
    }
    
    let word: String = chars[start..end].iter().collect();
    let range = Range::single_line(line_num, start as u32, end as u32);
    
    Some((word, range))
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.'
}

fn format_opcode_hover(doc: &OpcodeDoc) -> String {
    format!(
        r#"## {} ({})

{}

**Format:** `{}`

**Encoding:** `{}`

### Example
```sil
{}
```
"#,
        doc.name,
        doc.category,
        doc.description,
        doc.format,
        doc.encoding,
        doc.example
    )
}

fn format_register_hover(doc: &RegisterDoc) -> String {
    format!(
        r#"## {} — {}

**Layer:** {}

{}

**Semantic:** {}
"#,
        doc.name,
        doc.layer,
        doc.layer,
        doc.description,
        doc.semantic
    )
}

fn format_directive_hover(doc: &DirectiveDoc) -> String {
    format!(
        r#"## {}

{}

**Syntax:** `{}`

### Example
```sil
{}
```
"#,
        doc.name,
        doc.description,
        doc.syntax,
        doc.example
    )
}

fn build_opcode_docs() -> HashMap<String, OpcodeDoc> {
    let mut docs = HashMap::new();
    
    // Control
    docs.insert("NOP".to_string(), OpcodeDoc {
        name: "NOP",
        category: "Control",
        description: "No operation. Does nothing, advances PC.",
        format: "NOP",
        encoding: "0x00",
        example: "NOP  ; Delay cycle",
    });
    
    docs.insert("HLT".to_string(), OpcodeDoc {
        name: "HLT",
        category: "Control",
        description: "Halt execution. Stops the VM.",
        format: "HLT",
        encoding: "0x01",
        example: "HLT  ; End program",
    });
    
    docs.insert("JMP".to_string(), OpcodeDoc {
        name: "JMP",
        category: "Control",
        description: "Unconditional jump to address or label.",
        format: "JMP addr",
        encoding: "0x10 [addr:24]",
        example: "JMP loop  ; Jump to label 'loop'",
    });
    
    docs.insert("CALL".to_string(), OpcodeDoc {
        name: "CALL",
        category: "Control",
        description: "Call subroutine. Pushes return address and jumps.",
        format: "CALL addr",
        encoding: "0x15 [addr:24]",
        example: "CALL my_func  ; Call function",
    });
    
    docs.insert("RET".to_string(), OpcodeDoc {
        name: "RET",
        category: "Control",
        description: "Return from subroutine. Pops and jumps to return address.",
        format: "RET",
        encoding: "0x02",
        example: "RET  ; Return to caller",
    });
    
    // Data
    docs.insert("MOV".to_string(), OpcodeDoc {
        name: "MOV",
        category: "Data",
        description: "Move value from source register to destination register.",
        format: "MOV Rd, Rs",
        encoding: "0x20 [Rd:4][Rs:4] [ext:8]",
        example: "MOV R0, R1  ; Copy R1 to R0",
    });
    
    docs.insert("MOVI".to_string(), OpcodeDoc {
        name: "MOVI",
        category: "Data",
        description: "Move immediate value to register.",
        format: "MOVI Rd, imm",
        encoding: "0x21 [Rd:4][imm:12]",
        example: "MOVI R0, 0xFF  ; Load 0xFF into R0",
    });
    
    docs.insert("LOAD".to_string(), OpcodeDoc {
        name: "LOAD",
        category: "Data",
        description: "Load byte from memory address into register.",
        format: "LOAD Rd, addr",
        encoding: "0x22 [Rd:4][addr:20]",
        example: "LOAD R0, 0x1000  ; Load from address",
    });
    
    docs.insert("STORE".to_string(), OpcodeDoc {
        name: "STORE",
        category: "Data",
        description: "Store register value to memory address.",
        format: "STORE Rs, addr",
        encoding: "0x23 [Rs:4][addr:20]",
        example: "STORE R0, 0x1000  ; Store to address",
    });
    
    // Arithmetic
    docs.insert("MUL".to_string(), OpcodeDoc {
        name: "MUL",
        category: "Arithmetic",
        description: "Multiply two ByteSil values: ρ_result = ρ₁ + ρ₂, θ_result = θ₁ + θ₂",
        format: "MUL Rd, Rs",
        encoding: "0x40 [Rd:4][Rs:4] [ext:8]",
        example: "MUL R0, R1  ; R0 = R0 * R1",
    });
    
    docs.insert("XOR".to_string(), OpcodeDoc {
        name: "XORL",
        category: "Layer",
        description: "XOR two ByteSil values bit-by-bit.",
        format: "XORL Rd, Rs",
        encoding: "0x60 [Rd:4][Rs:4] [ext:8]",
        example: "XORL R0, R1  ; R0 = R0 XOR R1",
    });
    
    docs.insert("COLLAPSE".to_string(), OpcodeDoc {
        name: "COLLAPSE",
        category: "Transform",
        description: "Collapse state to single layer via XOR fold.",
        format: "COLLAPSE Rd",
        encoding: "0x87 [Rd:4][0:12]",
        example: "COLLAPSE R0  ; Collapse R0",
    });
    
    // Entanglement
    docs.insert("ENTANGLE".to_string(), OpcodeDoc {
        name: "ENTANGLE",
        category: "System",
        description: "Establish entanglement with remote node for distributed sync.",
        format: "ENTANGLE Rd, node_id",
        encoding: "0xC7 [Rd:4][node:12]",
        example: "ENTANGLE R0, 0x001  ; Entangle with node 1",
    });
    
    docs.insert("SYNC".to_string(), OpcodeDoc {
        name: "SYNC",
        category: "System",
        description: "Synchronize entangled state with remote node.",
        format: "SYNC Rd",
        encoding: "0xC4 [Rd:4][0:12]",
        example: "SYNC R0  ; Sync entangled pair",
    });
    
    docs
}

fn build_register_docs() -> HashMap<String, RegisterDoc> {
    let mut docs = HashMap::new();
    
    let registers = [
        ("R0", "L(0) Fotônico", "Camada de percepção visual", "Frequência luminosa, intensidade"),
        ("R1", "L(1) Acústico", "Camada de percepção auditiva", "Frequência sonora, amplitude"),
        ("R2", "L(2) Olfativo", "Camada de percepção olfativa", "Identificador químico, concentração"),
        ("R3", "L(3) Gustativo", "Camada de percepção gustativa", "Sabor, intensidade"),
        ("R4", "L(4) Dérmico", "Camada de percepção tátil", "Pressão, temperatura"),
        ("R5", "L(5) Eletrônico", "Camada de processamento elétrico", "Tensão, corrente"),
        ("R6", "L(6) Psicomotor", "Camada de controle motor", "Velocidade angular, torque"),
        ("R7", "L(7) Ambiental", "Camada de contexto ambiental", "Clima, iluminação"),
        ("R8", "L(8) Cibernético", "Camada de interação digital", "Protocolo, estado"),
        ("R9", "L(9) Geopolítico", "Camada de contexto geográfico", "Coordenadas, jurisdição"),
        ("RA", "L(A) Cosmopolítico", "Camada de contexto global", "Cultura, idioma"),
        ("RB", "L(B) Sinérgico", "Camada de emergência sinérgica", "Padrão emergente"),
        ("RC", "L(C) Quântico", "Camada de estado quântico", "Superposição, coerência"),
        ("RD", "L(D) Superposição", "Camada de meta-estado", "Probabilidade, fase"),
        ("RE", "L(E) Entanglement", "Camada de correlação remota", "Par entangled, correlação"),
        ("RF", "L(F) Colapso", "Camada de estado colapsado", "Estado final, medição"),
    ];
    
    for (name, layer, desc, semantic) in registers {
        docs.insert(name.to_string(), RegisterDoc {
            name,
            layer,
            description: desc,
            semantic,
        });
    }
    
    docs
}

fn build_directive_docs() -> HashMap<String, DirectiveDoc> {
    let mut docs = HashMap::new();
    
    docs.insert(".mode".to_string(), DirectiveDoc {
        name: ".mode",
        description: "Set the SIL compatibility mode (8, 16, 32, 64, or 128 bits).",
        syntax: ".mode SIL-{8|16|32|64|128}",
        example: ".mode SIL-128  ; Full 16-layer mode",
    });
    
    docs.insert(".code".to_string(), DirectiveDoc {
        name: ".code",
        description: "Start the code section. Instructions follow this directive.",
        syntax: ".code",
        example: ".code\n_start:\n    NOP",
    });
    
    docs.insert(".data".to_string(), DirectiveDoc {
        name: ".data",
        description: "Start the data section. Constants and states follow.",
        syntax: ".data",
        example: ".data\n    initial_state: .byte 0xFF, 0x00, ...",
    });
    
    docs.insert(".global".to_string(), DirectiveDoc {
        name: ".global",
        description: "Export a symbol for linking with other modules.",
        syntax: ".global symbol_name",
        example: ".global _start",
    });
    
    docs.insert(".extern".to_string(), DirectiveDoc {
        name: ".extern",
        description: "Import an external symbol defined in another module.",
        syntax: ".extern symbol_name",
        example: ".extern printf",
    });
    
    docs.insert(".align".to_string(), DirectiveDoc {
        name: ".align",
        description: "Align next data/code to N-byte boundary.",
        syntax: ".align N",
        example: ".align 4  ; Align to 4 bytes",
    });
    
    docs.insert(".byte".to_string(), DirectiveDoc {
        name: ".byte",
        description: "Define one or more byte values.",
        syntax: ".byte val1, val2, ...",
        example: ".byte 0xFF, 0x00, 0x42",
    });
    
    docs
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hover_opcode() {
        let provider = HoverProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "NOP".to_string(),
        );
        
        let result = provider.provide(&doc, Position { line: 0, character: 1 });
        assert!(result.is_some());
        assert!(result.unwrap().contents.contains("No operation"));
    }
    
    #[test]
    fn test_hover_register() {
        let provider = HoverProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "MOV R0, R1".to_string(),
        );
        
        let result = provider.provide(&doc, Position { line: 0, character: 4 });
        assert!(result.is_some());
        assert!(result.unwrap().contents.contains("Fotônico"));
    }
}
