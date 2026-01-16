//! Provider de símbolos para navegação

use super::{Document, Range, Position};
use std::collections::HashMap;

/// Tipo de símbolo
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,   // Labels
    Variable,   // Registradores ou variáveis .data
    Constant,   // Constantes .byte
    Module,     // Seções
    String,     // Strings em .data
}

/// Informação de símbolo
#[derive(Debug, Clone)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub detail: Option<String>,
    pub children: Vec<SymbolInformation>,
}

/// Localização de definição
#[derive(Debug, Clone)]
pub struct LocationLink {
    pub target_uri: String,
    pub target_range: Range,
    pub target_selection_range: Range,
}

/// Provider de símbolos
pub struct SymbolProvider {
    /// Cache de símbolos por documento
    cache: HashMap<String, Vec<SymbolInformation>>,
}

impl SymbolProvider {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    /// Retorna símbolos do documento (outline)
    pub fn document_symbols(&mut self, doc: &Document) -> Vec<SymbolInformation> {
        let symbols = self.parse_symbols(doc);
        self.cache.insert(doc.uri.clone(), symbols.clone());
        symbols
    }
    
    /// Encontra definição de símbolo
    pub fn find_definition(
        &mut self,
        doc: &Document,
        position: &Position,
    ) -> Option<LocationLink> {
        // Get word at position
        let word = self.get_word_at_position(doc, position)?;
        
        // Search for label definition
        for (line_num, line) in doc.lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with(':') {
                let label_name = &trimmed[..trimmed.len()-1];
                if label_name == word {
                    let col_start = line.find(label_name).unwrap_or(0) as u32;
                    return Some(LocationLink {
                        target_uri: doc.uri.clone(),
                        target_range: Range::single_line(
                            line_num as u32,
                            col_start,
                            col_start + label_name.len() as u32,
                        ),
                        target_selection_range: Range::single_line(
                            line_num as u32,
                            col_start,
                            col_start + label_name.len() as u32,
                        ),
                    });
                }
            }
        }
        
        None
    }
    
    /// Encontra todas as referências
    pub fn find_references(
        &mut self,
        doc: &Document,
        position: &Position,
        include_declaration: bool,
    ) -> Vec<LocationLink> {
        let mut refs = Vec::new();
        
        let word = match self.get_word_at_position(doc, position) {
            Some(w) => w,
            None => return refs,
        };
        
        for (line_num, line) in doc.lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Check label definition
            if trimmed.ends_with(':') {
                let label_name = &trimmed[..trimmed.len()-1];
                if label_name == word && include_declaration {
                    let col_start = line.find(&word).unwrap_or(0) as u32;
                    refs.push(LocationLink {
                        target_uri: doc.uri.clone(),
                        target_range: Range::single_line(line_num as u32, col_start, col_start + word.len() as u32),
                        target_selection_range: Range::single_line(line_num as u32, col_start, col_start + word.len() as u32),
                    });
                }
                continue;
            }
            
            // Check references in instructions
            if line.contains(&word) {
                // Find all occurrences
                let mut start_idx = 0;
                while let Some(idx) = line[start_idx..].find(&word) {
                    let abs_idx = start_idx + idx;
                    
                    // Check if it's a whole word
                    let before_ok = abs_idx == 0 || !line.chars().nth(abs_idx - 1)
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false);
                    let after_ok = abs_idx + word.len() >= line.len() || !line.chars().nth(abs_idx + word.len())
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false);
                    
                    if before_ok && after_ok {
                        refs.push(LocationLink {
                            target_uri: doc.uri.clone(),
                            target_range: Range::single_line(
                                line_num as u32,
                                abs_idx as u32,
                                (abs_idx + word.len()) as u32,
                            ),
                            target_selection_range: Range::single_line(
                                line_num as u32,
                                abs_idx as u32,
                                (abs_idx + word.len()) as u32,
                            ),
                        });
                    }
                    
                    start_idx = abs_idx + word.len();
                }
            }
        }
        
        refs
    }
    
    /// Rename symbol
    pub fn prepare_rename(
        &self,
        doc: &Document,
        position: &Position,
    ) -> Option<Range> {
        let _word = self.get_word_at_position(doc, position)?;
        let line = doc.lines.get(position.line as usize)?;
        
        // Find word position
        let col = position.character as usize;
        let line_chars: Vec<char> = line.chars().collect();
        
        // Find word boundaries
        let mut start = col;
        while start > 0 && (line_chars[start - 1].is_alphanumeric() || line_chars[start - 1] == '_') {
            start -= 1;
        }
        
        let mut end = col;
        while end < line_chars.len() && (line_chars[end].is_alphanumeric() || line_chars[end] == '_') {
            end += 1;
        }
        
        Some(Range::single_line(position.line, start as u32, end as u32))
    }
    
    /// Apply rename
    pub fn rename(
        &mut self,
        doc: &Document,
        position: &Position,
        new_name: &str,
    ) -> Vec<TextEdit> {
        let refs = self.find_references(doc, position, true);
        refs.into_iter()
            .map(|loc| TextEdit {
                range: loc.target_range,
                new_text: new_name.to_string(),
            })
            .collect()
    }
    
    fn parse_symbols(&self, doc: &Document) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let mut current_section: Option<SymbolInformation> = None;
        let mut section_children = Vec::new();
        
        for (line_num, line) in doc.lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_num = line_num as u32;
            
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }
            
            // Section detection
            if trimmed.starts_with('.') {
                // Save previous section
                if let Some(mut section) = current_section.take() {
                    section.children = section_children.clone();
                    symbols.push(section);
                    section_children.clear();
                }
                
                let name = trimmed.split_whitespace().next().unwrap_or(trimmed);
                match name {
                    ".code" | ".text" => {
                        current_section = Some(SymbolInformation {
                            name: "Code Section".to_string(),
                            kind: SymbolKind::Module,
                            range: Range::single_line(line_num, 0, trimmed.len() as u32),
                            selection_range: Range::single_line(line_num, 0, name.len() as u32),
                            detail: Some("Executable code".to_string()),
                            children: Vec::new(),
                        });
                    }
                    ".data" => {
                        current_section = Some(SymbolInformation {
                            name: "Data Section".to_string(),
                            kind: SymbolKind::Module,
                            range: Range::single_line(line_num, 0, trimmed.len() as u32),
                            selection_range: Range::single_line(line_num, 0, name.len() as u32),
                            detail: Some("Static data".to_string()),
                            children: Vec::new(),
                        });
                    }
                    ".mode" => {
                        let mode = trimmed.split_whitespace().nth(1).unwrap_or("unknown");
                        symbols.push(SymbolInformation {
                            name: format!("Mode: {}", mode),
                            kind: SymbolKind::Constant,
                            range: Range::single_line(line_num, 0, trimmed.len() as u32),
                            selection_range: Range::single_line(line_num, 0, name.len() as u32),
                            detail: Some("Compatibility mode".to_string()),
                            children: Vec::new(),
                        });
                    }
                    _ => {}
                }
                continue;
            }
            
            // Label detection
            if trimmed.ends_with(':') {
                let label_name = &trimmed[..trimmed.len()-1];
                let col_start = line.find(label_name).unwrap_or(0) as u32;
                
                section_children.push(SymbolInformation {
                    name: label_name.to_string(),
                    kind: SymbolKind::Function,
                    range: Range::single_line(line_num, col_start, col_start + label_name.len() as u32 + 1),
                    selection_range: Range::single_line(line_num, col_start, col_start + label_name.len() as u32),
                    detail: Some("Label".to_string()),
                    children: Vec::new(),
                });
                continue;
            }
            
            // Data definition detection
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 && !parts[0].starts_with('.') {
                if let Some(directive) = parts.get(1) {
                    if [".byte", ".state", ".string"].contains(&directive.to_lowercase().as_str()) {
                        let col_start = line.find(parts[0]).unwrap_or(0) as u32;
                        section_children.push(SymbolInformation {
                            name: parts[0].to_string(),
                            kind: if *directive == ".string" { SymbolKind::String } else { SymbolKind::Variable },
                            range: Range::single_line(line_num, 0, trimmed.len() as u32),
                            selection_range: Range::single_line(line_num, col_start, col_start + parts[0].len() as u32),
                            detail: Some(format!("{} definition", directive)),
                            children: Vec::new(),
                        });
                    }
                }
            }
        }
        
        // Save last section
        if let Some(mut section) = current_section {
            section.children = section_children;
            symbols.push(section);
        }
        
        symbols
    }
    
    fn get_word_at_position(&self, doc: &Document, position: &Position) -> Option<String> {
        let line = doc.lines.get(position.line as usize)?;
        let col = position.character as usize;
        
        if col >= line.len() {
            return None;
        }
        
        let line_chars: Vec<char> = line.chars().collect();
        
        // Find word boundaries
        let mut start = col;
        while start > 0 && (line_chars[start - 1].is_alphanumeric() || line_chars[start - 1] == '_') {
            start -= 1;
        }
        
        let mut end = col;
        while end < line_chars.len() && (line_chars[end].is_alphanumeric() || line_chars[end] == '_') {
            end += 1;
        }
        
        if start == end {
            return None;
        }
        
        Some(line_chars[start..end].iter().collect())
    }
}

impl Default for SymbolProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Edit de texto
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_symbols() {
        let mut provider = SymbolProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".mode SIL-128\n.code\nstart:\n    NOP\nloop:\n    JMP loop\n.data\nvalue .byte 42".to_string(),
        );
        
        let symbols = provider.document_symbols(&doc);
        assert!(!symbols.is_empty());
        
        // Should have mode, code section, data section
        let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("Mode")));
        assert!(names.iter().any(|n| n.contains("Code")));
        assert!(names.iter().any(|n| n.contains("Data")));
    }
    
    #[test]
    fn test_find_definition() {
        let mut provider = SymbolProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".code\nstart:\n    NOP\n    JMP start".to_string(),
        );
        
        // Position on "start" in JMP instruction
        let pos = Position { line: 3, character: 8 };
        let def = provider.find_definition(&doc, &pos);
        
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.target_range.start.line, 1); // Label on line 1
    }
    
    #[test]
    fn test_find_references() {
        let mut provider = SymbolProvider::new();
        let doc = Document::new(
            "file:///test.sil".to_string(),
            1,
            ".code\nloop:\n    NOP\n    JMP loop\n    JZ loop".to_string(),
        );
        
        // Position on "loop" label
        let pos = Position { line: 1, character: 0 };
        let refs = provider.find_references(&doc, &pos, true);
        
        assert_eq!(refs.len(), 3); // Definition + 2 references
    }
}
