//! Servidor LSP principal

use std::sync::{Arc, RwLock, Mutex};

use super::{
    Document, DocumentStore, Position, Range,
    completion::CompletionProvider,
    hover::HoverProvider,
    diagnostics::DiagnosticsProvider,
    symbols::{SymbolProvider, LocationLink, SymbolInformation},
    semantic_tokens::{SemanticTokensProvider, SemanticTokensResult},
};

/// Configuração do LSP
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// Habilitar diagnósticos
    pub enable_diagnostics: bool,
    /// Habilitar completions
    pub enable_completions: bool,
    /// Habilitar hover
    pub enable_hover: bool,
    /// Habilitar semantic tokens
    pub enable_semantic_tokens: bool,
    /// Máximo de diagnósticos por arquivo
    pub max_diagnostics: usize,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enable_diagnostics: true,
            enable_completions: true,
            enable_hover: true,
            enable_semantic_tokens: true,
            max_diagnostics: 100,
        }
    }
}

/// Capabilities do servidor
#[derive(Debug, Clone)]
pub struct ServerCapabilities {
    pub completion_provider: bool,
    pub hover_provider: bool,
    pub definition_provider: bool,
    pub references_provider: bool,
    pub document_symbol_provider: bool,
    pub document_formatting_provider: bool,
    pub semantic_tokens_provider: bool,
    pub diagnostic_provider: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            completion_provider: true,
            hover_provider: true,
            definition_provider: true,
            references_provider: true,
            document_symbol_provider: true,
            document_formatting_provider: true,
            semantic_tokens_provider: true,
            diagnostic_provider: true,
        }
    }
}

/// Servidor LSP para SIL
pub struct SilLanguageServer {
    config: LspConfig,
    documents: Arc<RwLock<DocumentStore>>,
    completion: CompletionProvider,
    hover: HoverProvider,
    diagnostics: DiagnosticsProvider,
    symbols: Mutex<SymbolProvider>,
    semantic_tokens: SemanticTokensProvider,
    initialized: bool,
}

impl SilLanguageServer {
    pub fn new() -> Self {
        Self::with_config(LspConfig::default())
    }
    
    pub fn with_config(config: LspConfig) -> Self {
        Self {
            config,
            documents: Arc::new(RwLock::new(DocumentStore::new())),
            completion: CompletionProvider::new(),
            hover: HoverProvider::new(),
            diagnostics: DiagnosticsProvider::new(),
            symbols: Mutex::new(SymbolProvider::new()),
            semantic_tokens: SemanticTokensProvider::new(),
            initialized: false,
        }
    }
    
    /// Retorna capabilities do servidor
    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities::default()
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // LIFECYCLE
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Initialize
    pub fn initialize(&mut self, _params: InitializeParams) -> InitializeResult {
        self.initialized = true;
        
        InitializeResult {
            capabilities: self.capabilities(),
            server_info: ServerInfo {
                name: "sil-lsp".to_string(),
                version: Some("1.0.0".to_string()),
            },
        }
    }
    
    /// Shutdown
    pub fn shutdown(&mut self) {
        self.initialized = false;
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // DOCUMENT SYNC
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Document opened
    pub fn did_open(&mut self, params: DidOpenParams) -> Vec<Diagnostic> {
        let mut store = self.documents.write().unwrap();
        store.open(
            params.uri.clone(),
            params.version,
            params.text,
        );
        drop(store);
        
        // Run diagnostics
        if self.config.enable_diagnostics {
            self.get_diagnostics(&params.uri)
        } else {
            Vec::new()
        }
    }
    
    /// Document changed
    pub fn did_change(&mut self, params: DidChangeParams) -> Vec<Diagnostic> {
        let mut store = self.documents.write().unwrap();
        store.update(&params.uri, params.version, params.text);
        drop(store);
        
        if self.config.enable_diagnostics {
            self.get_diagnostics(&params.uri)
        } else {
            Vec::new()
        }
    }
    
    /// Document closed
    pub fn did_close(&mut self, uri: &str) {
        let mut store = self.documents.write().unwrap();
        store.close(uri);
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // FEATURES
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Completion
    pub fn completion(&self, uri: &str, position: Position) -> Vec<CompletionItem> {
        if !self.config.enable_completions {
            return Vec::new();
        }
        
        let store = self.documents.read().unwrap();
        if let Some(doc) = store.get(uri) {
            self.completion.provide(doc, position)
        } else {
            Vec::new()
        }
    }
    
    /// Hover
    pub fn hover(&self, uri: &str, position: Position) -> Option<HoverResult> {
        if !self.config.enable_hover {
            return None;
        }
        
        let store = self.documents.read().unwrap();
        let doc = store.get(uri)?;
        self.hover.provide(doc, position)
    }
    
    /// Go to definition
    pub fn definition(&self, uri: &str, position: Position) -> Option<LocationLink> {
        let store = self.documents.read().unwrap();
        let doc = store.get(uri)?;
        self.symbols.lock().unwrap().find_definition(doc, &position)
    }
    
    /// Find references
    pub fn references(&self, uri: &str, position: Position, include_declaration: bool) -> Vec<LocationLink> {
        let store = self.documents.read().unwrap();
        if let Some(doc) = store.get(uri) {
            self.symbols.lock().unwrap().find_references(doc, &position, include_declaration)
        } else {
            Vec::new()
        }
    }
    
    /// Document symbols
    pub fn document_symbols(&self, uri: &str) -> Vec<SymbolInformation> {
        let store = self.documents.read().unwrap();
        if let Some(doc) = store.get(uri) {
            self.symbols.lock().unwrap().document_symbols(doc)
        } else {
            Vec::new()
        }
    }
    
    /// Diagnostics
    pub fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        let store = self.documents.read().unwrap();
        if let Some(doc) = store.get(uri) {
            let mut diags = self.diagnostics.analyze(doc);
            diags.truncate(self.config.max_diagnostics);
            diags
        } else {
            Vec::new()
        }
    }
    
    /// Semantic tokens
    pub fn semantic_tokens(&self, uri: &str) -> SemanticTokensResult {
        if !self.config.enable_semantic_tokens {
            return SemanticTokensResult::default();
        }
        
        let store = self.documents.read().unwrap();
        if let Some(doc) = store.get(uri) {
            self.semantic_tokens.full(doc)
        } else {
            SemanticTokensResult::default()
        }
    }
    
    /// Format document
    pub fn format(&self, uri: &str) -> Vec<TextEdit> {
        let store = self.documents.read().unwrap();
        if let Some(doc) = store.get(uri) {
            format_document(doc)
        } else {
            Vec::new()
        }
    }
}

impl Default for SilLanguageServer {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LSP TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize params
#[derive(Debug, Clone, Default)]
pub struct InitializeParams {
    pub root_uri: Option<String>,
    pub capabilities: ClientCapabilities,
}

/// Client capabilities
#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    pub text_document: Option<TextDocumentClientCapabilities>,
}

/// Text document client capabilities
#[derive(Debug, Clone, Default)]
pub struct TextDocumentClientCapabilities {
    pub completion: Option<CompletionClientCapabilities>,
    pub hover: Option<HoverClientCapabilities>,
}

/// Completion client capabilities
#[derive(Debug, Clone, Default)]
pub struct CompletionClientCapabilities {
    pub snippet_support: bool,
}

/// Hover client capabilities
#[derive(Debug, Clone, Default)]
pub struct HoverClientCapabilities {
    pub content_format: Vec<String>,
}

/// Initialize result
#[derive(Debug, Clone)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

/// Server info
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Did open params
#[derive(Debug, Clone)]
pub struct DidOpenParams {
    pub uri: String,
    pub version: i32,
    pub text: String,
}

/// Did change params
#[derive(Debug, Clone)]
pub struct DidChangeParams {
    pub uri: String,
    pub version: i32,
    pub text: String,
}

/// Completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub insert_text_format: InsertTextFormat,
}

/// Completion item kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    Keyword,
    Function,
    Variable,
    Constant,
    Snippet,
    Field,
    Module,
}

/// Insert text format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertTextFormat {
    PlainText,
    Snippet,
}

impl Default for InsertTextFormat {
    fn default() -> Self {
        Self::PlainText
    }
}

/// Hover result
#[derive(Debug, Clone)]
pub struct HoverResult {
    pub contents: String,
    pub range: Option<Range>,
}

/// Location
#[derive(Debug, Clone)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// Document symbol
#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<DocumentSymbol>,
}

/// Symbol kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Variable,
    Constant,
    Module,
    Namespace,
}

/// Diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub source: String,
    pub message: String,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Semantic tokens
#[derive(Debug, Clone, Default)]
pub struct SemanticTokens {
    pub data: Vec<u32>,
}

/// Text edit
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

fn format_document(doc: &Document) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    
    for (line_num, line) in doc.lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        
        // Format instruction lines
        let formatted = if trimmed.ends_with(':') {
            // Label - no indent
            trimmed.to_string()
        } else if trimmed.starts_with('.') {
            // Directive - no indent
            trimmed.to_string()
        } else {
            // Instruction - 4 space indent
            format!("    {}", trimmed)
        };
        
        if line != &formatted {
            edits.push(TextEdit {
                range: Range::single_line(line_num as u32, 0, line.len() as u32),
                new_text: formatted,
            });
        }
    }
    
    edits
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_init() {
        let mut server = SilLanguageServer::new();
        let result = server.initialize(InitializeParams::default());
        
        assert_eq!(result.server_info.name, "sil-lsp");
        assert!(result.capabilities.completion_provider);
    }
    
    #[test]
    fn test_document_lifecycle() {
        let mut server = SilLanguageServer::new();
        
        server.did_open(DidOpenParams {
            uri: "file:///test.sil".to_string(),
            version: 1,
            text: "NOP\nHLT".to_string(),
        });
        
        let _symbols = server.document_symbols("file:///test.sil");
        // Should parse without error
        
        server.did_close("file:///test.sil");
    }
}
