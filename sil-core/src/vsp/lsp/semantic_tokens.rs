//! Provider de tokens semânticos para syntax highlighting avançado

use super::Document;

/// Tipos de token semântico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SemanticTokenType {
    Namespace = 0,      // Seções (.code, .data)
    Type = 1,           // Modos (SIL-128)
    Class = 2,          // Não usado
    Enum = 3,           // Não usado
    Interface = 4,      // Não usado
    Struct = 5,         // Não usado
    TypeParameter = 6,  // Não usado
    Parameter = 7,      // Operandos
    Variable = 8,       // Registradores
    Property = 9,       // Campos de dados
    EnumMember = 10,    // Não usado
    Event = 11,         // Não usado
    Function = 12,      // Labels
    Method = 13,        // Não usado
    Macro = 14,         // Diretivas
    Keyword = 15,       // Opcodes
    Modifier = 16,      // Modificadores (hints)
    Comment = 17,       // Comentários
    String = 18,        // Strings
    Number = 19,        // Números
    Regexp = 20,        // Não usado
    Operator = 21,      // Operadores
    Decorator = 22,     // Prefixos especiais
    Label = 23,         // Labels (definição)
}

impl SemanticTokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Namespace => "namespace",
            Self::Type => "type",
            Self::Class => "class",
            Self::Enum => "enum",
            Self::Interface => "interface",
            Self::Struct => "struct",
            Self::TypeParameter => "typeParameter",
            Self::Parameter => "parameter",
            Self::Variable => "variable",
            Self::Property => "property",
            Self::EnumMember => "enumMember",
            Self::Event => "event",
            Self::Function => "function",
            Self::Method => "method",
            Self::Macro => "macro",
            Self::Keyword => "keyword",
            Self::Modifier => "modifier",
            Self::Comment => "comment",
            Self::String => "string",
            Self::Number => "number",
            Self::Regexp => "regexp",
            Self::Operator => "operator",
            Self::Decorator => "decorator",
            Self::Label => "label",
        }
    }
    
    /// Lista de todos os tipos para capabilities
    pub fn all() -> Vec<&'static str> {
        vec![
            "namespace", "type", "class", "enum", "interface", "struct",
            "typeParameter", "parameter", "variable", "property", "enumMember",
            "event", "function", "method", "macro", "keyword", "modifier",
            "comment", "string", "number", "regexp", "operator", "decorator", "label",
        ]
    }
}

/// Modificadores de token semântico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SemanticTokenModifier {
    Declaration = 0,
    Definition = 1,
    Readonly = 2,
    Static = 3,
    Deprecated = 4,
    Abstract = 5,
    Async = 6,
    Modification = 7,
    Documentation = 8,
    DefaultLibrary = 9,
}

impl SemanticTokenModifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Declaration => "declaration",
            Self::Definition => "definition",
            Self::Readonly => "readonly",
            Self::Static => "static",
            Self::Deprecated => "deprecated",
            Self::Abstract => "abstract",
            Self::Async => "async",
            Self::Modification => "modification",
            Self::Documentation => "documentation",
            Self::DefaultLibrary => "defaultLibrary",
        }
    }
    
    /// Lista de todos os modificadores para capabilities
    pub fn all() -> Vec<&'static str> {
        vec![
            "declaration", "definition", "readonly", "static", "deprecated",
            "abstract", "async", "modification", "documentation", "defaultLibrary",
        ]
    }
}

/// Token semântico individual
#[derive(Debug, Clone)]
pub struct SemanticToken {
    pub line: u32,
    pub start_char: u32,
    pub length: u32,
    pub token_type: SemanticTokenType,
    pub modifiers: u32,  // Bitmask de modificadores
}

/// Provider de tokens semânticos
pub struct SemanticTokensProvider {
    /// Opcodes conhecidos
    opcodes: Vec<&'static str>,
}

impl SemanticTokensProvider {
    pub fn new() -> Self {
        Self {
            opcodes: vec![
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
            ],
        }
    }
    
    /// Tokeniza documento completo
    pub fn full(&self, doc: &Document) -> SemanticTokensResult {
        let mut tokens = Vec::new();
        
        for (line_num, line) in doc.lines.iter().enumerate() {
            let line_tokens = self.tokenize_line(line, line_num as u32);
            tokens.extend(line_tokens);
        }
        
        SemanticTokensResult {
            result_id: Some(format!("{}:{}", doc.uri, doc.version)),
            data: self.encode_tokens(&tokens),
        }
    }
    
    /// Tokeniza range específico
    pub fn range(&self, doc: &Document, start_line: u32, end_line: u32) -> SemanticTokensResult {
        let mut tokens = Vec::new();
        
        for line_num in start_line..=end_line.min(doc.lines.len() as u32 - 1) {
            if let Some(line) = doc.lines.get(line_num as usize) {
                let line_tokens = self.tokenize_line(line, line_num);
                tokens.extend(line_tokens);
            }
        }
        
        SemanticTokensResult {
            result_id: None,
            data: self.encode_tokens(&tokens),
        }
    }
    
    fn tokenize_line(&self, line: &str, line_num: u32) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let trimmed = line.trim();
        
        // Empty line
        if trimmed.is_empty() {
            return tokens;
        }
        
        // Comment
        if let Some(comment_start) = line.find(';') {
            tokens.push(SemanticToken {
                line: line_num,
                start_char: comment_start as u32,
                length: (line.len() - comment_start) as u32,
                token_type: SemanticTokenType::Comment,
                modifiers: 0,
            });
            
            // If whole line is comment, return
            if line.trim().starts_with(';') {
                return tokens;
            }
        }
        
        // Directive
        if trimmed.starts_with('.') {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let directive = parts.first().unwrap();
            let directive_start = line.find(directive).unwrap_or(0);
            
            tokens.push(SemanticToken {
                line: line_num,
                start_char: directive_start as u32,
                length: directive.len() as u32,
                token_type: SemanticTokenType::Macro,
                modifiers: 0,
            });
            
            // Mode argument
            if *directive == ".mode" && parts.len() > 1 {
                let mode = parts[1];
                if let Some(mode_start) = line.find(mode) {
                    tokens.push(SemanticToken {
                        line: line_num,
                        start_char: mode_start as u32,
                        length: mode.len() as u32,
                        token_type: SemanticTokenType::Type,
                        modifiers: 0,
                    });
                }
            }
            
            // .global/.extern symbols
            if (*directive == ".global" || *directive == ".globl" || *directive == ".extern") && parts.len() > 1 {
                for symbol in &parts[1..] {
                    if let Some(sym_start) = line.rfind(symbol) {
                        tokens.push(SemanticToken {
                            line: line_num,
                            start_char: sym_start as u32,
                            length: symbol.len() as u32,
                            token_type: SemanticTokenType::Function,
                            modifiers: 1 << SemanticTokenModifier::Declaration as u32,
                        });
                    }
                }
            }
            
            return tokens;
        }
        
        // Label definition
        if trimmed.ends_with(':') {
            let label_name = &trimmed[..trimmed.len()-1];
            let label_start = line.find(label_name).unwrap_or(0);
            
            tokens.push(SemanticToken {
                line: line_num,
                start_char: label_start as u32,
                length: label_name.len() as u32,
                token_type: SemanticTokenType::Label,
                modifiers: 1 << SemanticTokenModifier::Definition as u32,
            });
            
            // Colon
            tokens.push(SemanticToken {
                line: line_num,
                start_char: (label_start + label_name.len()) as u32,
                length: 1,
                token_type: SemanticTokenType::Operator,
                modifiers: 0,
            });
            
            return tokens;
        }
        
        // Instruction
        self.tokenize_instruction(line, line_num, &mut tokens);
        
        tokens
    }
    
    fn tokenize_instruction(&self, line: &str, line_num: u32, tokens: &mut Vec<SemanticToken>) {
        let parts: Vec<&str> = line
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.is_empty() {
            return;
        }
        
        let mnemonic = parts[0];
        let mnemonic_upper = mnemonic.to_uppercase();
        
        // Check if valid opcode
        if self.opcodes.iter().any(|op| *op == mnemonic_upper) {
            let start = line.find(mnemonic).unwrap_or(0);
            
            // Hints get modifier treatment
            let modifiers = if mnemonic_upper.starts_with("HINT.") {
                1 << SemanticTokenModifier::Modification as u32
            } else {
                0
            };
            
            tokens.push(SemanticToken {
                line: line_num,
                start_char: start as u32,
                length: mnemonic.len() as u32,
                token_type: SemanticTokenType::Keyword,
                modifiers,
            });
        }
        
        // Process operands
        let mut search_start = line.find(mnemonic).map(|i| i + mnemonic.len()).unwrap_or(0);
        
        for operand in parts.iter().skip(1) {
            if operand.starts_with(';') {
                break; // Rest is comment
            }
            
            // Find operand position
            let op_start = match line[search_start..].find(operand) {
                Some(i) => search_start + i,
                None => continue,
            };
            search_start = op_start + operand.len();
            
            let (token_type, modifiers) = self.classify_operand(operand);
            
            tokens.push(SemanticToken {
                line: line_num,
                start_char: op_start as u32,
                length: operand.len() as u32,
                token_type,
                modifiers,
            });
        }
    }
    
    fn classify_operand(&self, operand: &str) -> (SemanticTokenType, u32) {
        let op = operand.trim_matches(|c| c == ',' || c == ' ');
        
        // Register
        if op.starts_with('R') || op.starts_with('r') {
            return (SemanticTokenType::Variable, 1 << SemanticTokenModifier::Readonly as u32);
        }
        
        // Immediate number (hex or decimal)
        if op.starts_with("0x") || op.starts_with("0X") || op.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return (SemanticTokenType::Number, 0);
        }
        
        // Layer constant (L0-LF)
        if op.starts_with('L') && op.len() <= 2 {
            let layer_part = &op[1..];
            if layer_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return (SemanticTokenType::Type, 1 << SemanticTokenModifier::Static as u32);
            }
        }
        
        // String literal
        if op.starts_with('"') && op.ends_with('"') {
            return (SemanticTokenType::String, 0);
        }
        
        // Label reference
        if op.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return (SemanticTokenType::Function, 0);
        }
        
        // Default to parameter
        (SemanticTokenType::Parameter, 0)
    }
    
    /// Encode tokens to LSP format (delta encoding)
    fn encode_tokens(&self, tokens: &[SemanticToken]) -> Vec<u32> {
        let mut data = Vec::with_capacity(tokens.len() * 5);
        let mut prev_line = 0u32;
        let mut prev_char = 0u32;
        
        for token in tokens {
            let delta_line = token.line - prev_line;
            let delta_char = if delta_line == 0 {
                token.start_char - prev_char
            } else {
                token.start_char
            };
            
            data.push(delta_line);
            data.push(delta_char);
            data.push(token.length);
            data.push(token.token_type as u32);
            data.push(token.modifiers);
            
            prev_line = token.line;
            prev_char = token.start_char;
        }
        
        data
    }
}

impl Default for SemanticTokensProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Resultado de tokens semânticos
#[derive(Debug, Clone, Default)]
pub struct SemanticTokensResult {
    pub result_id: Option<String>,
    pub data: Vec<u32>,
}

/// Legend para capabilities
#[derive(Debug, Clone)]
pub struct SemanticTokensLegend {
    pub token_types: Vec<String>,
    pub token_modifiers: Vec<String>,
}

impl Default for SemanticTokensLegend {
    fn default() -> Self {
        Self {
            token_types: SemanticTokenType::all().into_iter().map(String::from).collect(),
            token_modifiers: SemanticTokenModifier::all().into_iter().map(String::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_directive() {
        let provider = SemanticTokensProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".mode SIL-128".to_string(),
        );
        
        let result = provider.full(&doc);
        assert!(!result.data.is_empty());
    }
    
    #[test]
    fn test_tokenize_label() {
        let provider = SemanticTokensProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "start:".to_string(),
        );
        
        let result = provider.full(&doc);
        // Should have label + colon
        assert!(result.data.len() >= 10);
    }
    
    #[test]
    fn test_tokenize_instruction() {
        let provider = SemanticTokensProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "    MOV R0, R1".to_string(),
        );
        
        let result = provider.full(&doc);
        // Should have opcode + 2 registers
        assert!(result.data.len() >= 15);
    }
    
    #[test]
    fn test_tokenize_comment() {
        let provider = SemanticTokensProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "; This is a comment".to_string(),
        );
        
        let result = provider.full(&doc);
        assert!(!result.data.is_empty());
        // Token type should be Comment
        assert_eq!(result.data[3], SemanticTokenType::Comment as u32);
    }
    
    #[test]
    fn test_encode_tokens() {
        let provider = SemanticTokensProvider::new();
        let tokens = vec![
            SemanticToken {
                line: 0,
                start_char: 0,
                length: 3,
                token_type: SemanticTokenType::Keyword,
                modifiers: 0,
            },
            SemanticToken {
                line: 0,
                start_char: 4,
                length: 2,
                token_type: SemanticTokenType::Variable,
                modifiers: 0,
            },
        ];
        
        let encoded = provider.encode_tokens(&tokens);
        // First token: delta_line=0, delta_char=0, length=3, type=15, modifiers=0
        assert_eq!(encoded[0..5], [0, 0, 3, SemanticTokenType::Keyword as u32, 0]);
        // Second token: delta_line=0, delta_char=4, length=2, type=8, modifiers=0
        assert_eq!(encoded[5..10], [0, 4, 2, SemanticTokenType::Variable as u32, 0]);
    }
}
