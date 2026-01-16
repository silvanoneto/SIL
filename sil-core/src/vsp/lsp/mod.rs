//! # 📝 SIL Language Server Protocol
//!
//! Servidor LSP para arquivos .sil com suporte completo a IDE.
//!
//! ## Funcionalidades
//!
//! - **Completions**: Opcodes, registradores, diretivas
//! - **Hover**: Documentação inline
//! - **Diagnostics**: Erros de sintaxe e semântica
//! - **Go to Definition**: Labels e símbolos
//! - **Document Symbols**: Outline de labels
//! - **Formatting**: Formatação de código
//! - **Semantic Tokens**: Syntax highlighting semântico
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::vsp::lsp::SilLanguageServer;
//!
//! let server = SilLanguageServer::new();
//! server.run_stdio();
//! ```

pub mod server;
pub mod completion;
pub mod hover;
pub mod diagnostics;
pub mod symbols;
pub mod semantic_tokens;

pub use server::{SilLanguageServer, LspConfig};
pub use completion::CompletionProvider;
pub use hover::HoverProvider;
pub use diagnostics::DiagnosticsProvider;
pub use symbols::SymbolProvider;
pub use semantic_tokens::SemanticTokensProvider;

use std::collections::HashMap;

/// Posição no documento
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range no documento
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Self {
        Self {
            start: Position { line: start_line, character: start_char },
            end: Position { line: end_line, character: end_char },
        }
    }
    
    pub fn single_line(line: u32, start: u32, end: u32) -> Self {
        Self::new(line, start, line, end)
    }
}

/// Documento aberto
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: String,
    pub version: i32,
    pub content: String,
    pub lines: Vec<String>,
}

impl Document {
    pub fn new(uri: String, version: i32, content: String) -> Self {
        let lines = content.lines().map(String::from).collect();
        Self { uri, version, content, lines }
    }
    
    pub fn update(&mut self, version: i32, content: String) {
        self.version = version;
        self.lines = content.lines().map(String::from).collect();
        self.content = content;
    }
    
    pub fn get_line(&self, line: u32) -> Option<&str> {
        self.lines.get(line as usize).map(String::as_str)
    }
    
    pub fn offset_at(&self, pos: Position) -> usize {
        let mut offset = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if i == pos.line as usize {
                return offset + pos.character as usize;
            }
            offset += line.len() + 1; // +1 for newline
        }
        offset
    }
    
    pub fn position_at(&self, offset: usize) -> Position {
        let mut current = 0;
        for (line, text) in self.lines.iter().enumerate() {
            if current + text.len() + 1 > offset {
                return Position {
                    line: line as u32,
                    character: (offset - current) as u32,
                };
            }
            current += text.len() + 1;
        }
        Position {
            line: self.lines.len().saturating_sub(1) as u32,
            character: self.lines.last().map(|l| l.len()).unwrap_or(0) as u32,
        }
    }
}

/// Store de documentos
#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<String, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn open(&mut self, uri: String, version: i32, content: String) {
        self.documents.insert(uri.clone(), Document::new(uri, version, content));
    }
    
    pub fn update(&mut self, uri: &str, version: i32, content: String) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.update(version, content);
        }
    }
    
    pub fn close(&mut self, uri: &str) {
        self.documents.remove(uri);
    }
    
    pub fn get(&self, uri: &str) -> Option<&Document> {
        self.documents.get(uri)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document() {
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "NOP\nMOV R0, R1\nHLT".to_string(),
        );
        
        assert_eq!(doc.lines.len(), 3);
        assert_eq!(doc.get_line(0), Some("NOP"));
        assert_eq!(doc.get_line(1), Some("MOV R0, R1"));
    }
    
    #[test]
    fn test_position_conversion() {
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            "NOP\nMOV R0, R1\nHLT".to_string(),
        );
        
        let pos = Position { line: 1, character: 4 };
        let offset = doc.offset_at(pos);
        
        let pos2 = doc.position_at(offset);
        assert_eq!(pos, pos2);
    }
}
