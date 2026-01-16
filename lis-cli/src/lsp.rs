//! # lis-lsp — LIS Language Server
//!
//! Language Server Protocol implementation for .lis files.
//!
//! ```bash
//! lis-lsp              # Run on stdio
//! lis-lsp --debug      # With debug logging
//! ```

use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::ExitCode;

use lis_core::{Lexer, Parser};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");

    if debug {
        eprintln!("lis-lsp v1.0.0 starting in debug mode...");
    }

    let mut store = DocumentStore::new();

    // Run on stdio
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut reader = BufReader::new(stdin.lock());

    loop {
        // Read header
        let content_length = match read_header(&mut reader, debug) {
            Ok(Some(len)) => len,
            Ok(None) => return ExitCode::SUCCESS, // EOF
            Err(e) => {
                eprintln!("Header read error: {}", e);
                continue;
            }
        };

        if content_length == 0 {
            continue;
        }

        // Read content
        let mut content = vec![0u8; content_length];
        if let Err(e) = reader.read_exact(&mut content) {
            eprintln!("Content read error: {}", e);
            continue;
        }

        let request = match String::from_utf8(content) {
            Ok(s) => s,
            Err(_) => continue,
        };

        if debug {
            eprintln!("<-- {}", request);
        }

        // Handle request
        let response = handle_request(&mut store, &request, debug);

        if let Some(resp) = response {
            send_response(&mut stdout_lock, &resp, debug);
        }

        // Check for shutdown
        if request.contains("\"method\":\"shutdown\"") {
            continue;
        }

        if request.contains("\"method\":\"exit\"") {
            break;
        }
    }

    ExitCode::SUCCESS
}

// ═══════════════════════════════════════════════════════════════════════════════════
// DOCUMENT STORE
// ═══════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Position {
    line: u32,
    character: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Range {
    start: Position,
    end: Position,
}

impl Range {
    fn new(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Self {
        Self {
            start: Position { line: start_line, character: start_char },
            end: Position { line: end_line, character: end_char },
        }
    }
}

#[derive(Debug, Clone)]
struct Document {
    #[allow(dead_code)]
    uri: String,
    version: i32,
    content: String,
    lines: Vec<String>,
}

impl Document {
    fn new(uri: String, version: i32, content: String) -> Self {
        let lines = content.lines().map(String::from).collect();
        Self { uri, version, content, lines }
    }

    fn update(&mut self, version: i32, content: String) {
        self.version = version;
        self.lines = content.lines().map(String::from).collect();
        self.content = content;
    }

    fn get_line(&self, line: u32) -> Option<&str> {
        self.lines.get(line as usize).map(String::as_str)
    }
}

#[derive(Debug, Default)]
struct DocumentStore {
    documents: HashMap<String, Document>,
}

impl DocumentStore {
    fn new() -> Self {
        Self::default()
    }

    fn open(&mut self, uri: String, version: i32, content: String) {
        self.documents.insert(uri.clone(), Document::new(uri, version, content));
    }

    fn update(&mut self, uri: &str, version: i32, content: String) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.update(version, content);
        }
    }

    fn close(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    fn get(&self, uri: &str) -> Option<&Document> {
        self.documents.get(uri)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// IO HELPERS
// ═══════════════════════════════════════════════════════════════════════════════════

fn read_header<R: BufRead>(reader: &mut R, _debug: bool) -> io::Result<Option<usize>> {
    let mut content_length = 0;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let line = line.trim();

        if line.is_empty() {
            break;
        }

        if let Some(len) = line.strip_prefix("Content-Length: ") {
            content_length = len.parse().unwrap_or(0);
        }
    }

    Ok(Some(content_length))
}

fn send_response<W: Write>(writer: &mut W, response: &str, debug: bool) {
    let bytes = response.as_bytes();
    let header = format!("Content-Length: {}\r\n\r\n", bytes.len());

    if debug {
        eprintln!("--> {}", response);
    }

    let _ = writer.write_all(header.as_bytes());
    let _ = writer.write_all(bytes);
    let _ = writer.flush();
}

// ═══════════════════════════════════════════════════════════════════════════════════
// REQUEST HANDLER
// ═══════════════════════════════════════════════════════════════════════════════════

fn handle_request(store: &mut DocumentStore, request: &str, debug: bool) -> Option<String> {
    // Parse JSON
    let json: serde_json::Value = match serde_json::from_str(request) {
        Ok(v) => v,
        Err(e) => {
            if debug {
                eprintln!("JSON parse error: {}", e);
            }
            return None;
        }
    };

    let method = json.get("method")?.as_str()?;
    let id = json.get("id");
    let params = json.get("params");

    match method {
        // ═══════════════════════════════════════════════════════════════════════════
        // LIFECYCLE
        // ═══════════════════════════════════════════════════════════════════════════
        "initialize" => {
            let id = id?;
            let capabilities = build_server_capabilities();
            let result = serde_json::json!({
                "capabilities": capabilities,
                "serverInfo": {
                    "name": "lis-lsp",
                    "version": "1.0.0"
                }
            });

            Some(json_rpc_response(id, result))
        }

        "initialized" => None,

        "shutdown" => {
            let id = id?;
            Some(json_rpc_response(id, serde_json::Value::Null))
        }

        "exit" => None,

        // ═══════════════════════════════════════════════════════════════════════════
        // TEXT DOCUMENT SYNC
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/didOpen" => {
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?.to_string();
            let version = text_document.get("version")?.as_i64()? as i32;
            let text = text_document.get("text")?.as_str()?.to_string();

            store.open(uri.clone(), version, text.clone());
            let diagnostics = validate_lis(&text);

            publish_diagnostics(&uri, &diagnostics)
        }

        "textDocument/didChange" => {
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?.to_string();
            let version = text_document.get("version")?.as_i64()? as i32;

            let content_changes = params.get("contentChanges")?.as_array()?;
            let text = content_changes.first()?.get("text")?.as_str()?.to_string();

            store.update(&uri, version, text.clone());
            let diagnostics = validate_lis(&text);

            publish_diagnostics(&uri, &diagnostics)
        }

        "textDocument/didClose" => {
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            store.close(uri);

            Some(format!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"uri":"{}","diagnostics":[]}}}}"#,
                uri
            ))
        }

        "textDocument/didSave" => None,

        // ═══════════════════════════════════════════════════════════════════════════
        // COMPLETION
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/completion" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;
            let position = parse_position(params.get("position")?)?;

            let items = get_completions(store, uri, position);

            Some(json_rpc_response(id, serde_json::json!({
                "isIncomplete": false,
                "items": items
            })))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // HOVER
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/hover" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;
            let position = parse_position(params.get("position")?)?;

            let result = get_hover(store, uri, position);

            Some(json_rpc_response(id, result))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // DOCUMENT SYMBOLS
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/documentSymbol" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            let symbols = get_document_symbols(store, uri);

            Some(json_rpc_response(id, serde_json::json!(symbols)))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // SEMANTIC TOKENS
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/semanticTokens/full" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            let tokens = get_semantic_tokens(store, uri);

            Some(json_rpc_response(id, serde_json::json!({
                "data": tokens
            })))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // PULL-BASED DIAGNOSTICS (LSP 3.17+)
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/diagnostic" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            // Get document text from store
            let text = match store.get(uri) {
                Some(doc) => &doc.content,
                None => return Some(json_rpc_response(id, serde_json::json!({
                    "kind": "full",
                    "items": []
                })))
            };

            let diagnostics = validate_lis(text);
            let items: Vec<serde_json::Value> = diagnostics.into_iter().map(|d| {
                serde_json::json!({
                    "range": {
                        "start": { "line": d.range.start.line, "character": d.range.start.character },
                        "end": { "line": d.range.end.line, "character": d.range.end.character }
                    },
                    "severity": d.severity,
                    "message": d.message,
                    "code": d.code
                })
            }).collect();

            Some(json_rpc_response(id, serde_json::json!({
                "kind": "full",
                "items": items
            })))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // UNKNOWN
        // ═══════════════════════════════════════════════════════════════════════════
        _ => {
            if debug {
                eprintln!("Unknown method: {}", method);
            }

            if let Some(id) = id {
                Some(json_rpc_error(id, -32601, &format!("Method not found: {}", method)))
            } else {
                None
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LIS VALIDATION
// ═══════════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct Diagnostic {
    range: Range,
    severity: u32, // 1=Error, 2=Warning, 3=Info, 4=Hint
    message: String,
    code: Option<String>,
}

fn validate_lis(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Tokenize
    let tokens = match Lexer::new(source).tokenize_with_spans() {
        Ok(t) => t,
        Err(e) => {
            diagnostics.push(Diagnostic {
                range: Range::new(0, 0, 0, 0),
                severity: 1,
                message: format!("Lexer error: {}", e),
                code: Some("E0001".to_string()),
            });
            return diagnostics;
        }
    };

    // Parse
    match Parser::new(tokens).parse() {
        Ok(_ast) => {
            // TODO: Run type checker for more diagnostics
        }
        Err(e) => {
            diagnostics.push(Diagnostic {
                range: Range::new(0, 0, 0, 10),
                severity: 1,
                message: format!("{}", e),
                code: Some("E0002".to_string()),
            });
        }
    }

    diagnostics
}

// ═══════════════════════════════════════════════════════════════════════════════════
// COMPLETIONS
// ═══════════════════════════════════════════════════════════════════════════════════

fn get_completions(store: &DocumentStore, uri: &str, position: Position) -> Vec<serde_json::Value> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };

    let line = match doc.get_line(position.line) {
        Some(l) => l,
        None => return vec![],
    };

    let col = position.character as usize;
    let prefix = if col <= line.len() {
        line[..col].trim_start()
    } else {
        ""
    };

    let mut items = Vec::new();

    // Keywords
    let keywords = [
        ("fn", "Define a function"),
        ("transform", "Define a transform"),
        ("type", "Define a type alias"),
        ("let", "Variable binding"),
        ("return", "Return from function"),
        ("if", "Conditional"),
        ("else", "Else branch"),
        ("loop", "Infinite loop"),
        ("break", "Break from loop"),
        ("continue", "Continue to next iteration"),
        ("use", "Import a module"),
        ("mod", "Declare a module"),
        ("pub", "Make item public"),
        ("as", "Alias for import"),
        ("extern", "External function declaration"),
        ("feedback", "Feedback loop"),
        ("emerge", "Emergence detection"),
    ];

    for (kw, desc) in keywords {
        if kw.starts_with(prefix) || prefix.is_empty() {
            items.push(serde_json::json!({
                "label": kw,
                "kind": 14, // Keyword
                "detail": desc,
                "insertText": kw
            }));
        }
    }

    // Types
    let types = [
        ("Int", "Integer type"),
        ("Float", "Floating-point type"),
        ("Bool", "Boolean type"),
        ("String", "String type"),
        ("ByteSil", "8-bit log-polar complex"),
        ("State", "16-layer state"),
        ("Complex", "Complex number"),
    ];

    for (ty, desc) in types {
        if ty.to_lowercase().starts_with(&prefix.to_lowercase()) || prefix.is_empty() {
            items.push(serde_json::json!({
                "label": ty,
                "kind": 7, // Class/Type
                "detail": desc,
                "insertText": ty
            }));
        }
    }

    // Intrinsic functions
    let intrinsics = [
        ("print_int", "Print integer to stdout"),
        ("print_float", "Print float to stdout"),
        ("print_string", "Print string to stdout"),
        ("state_vacuum", "Create zero state"),
        ("state_neutral", "Create neutral state"),
        ("state_get_layer", "Get layer from state"),
        ("state_set_layer", "Set layer in state"),
        ("bytesil_new", "Create ByteSil from rho and theta"),
        ("sin", "Sine function"),
        ("cos", "Cosine function"),
        ("sqrt", "Square root"),
        ("abs", "Absolute value"),
    ];

    for (func, desc) in intrinsics {
        if func.starts_with(prefix) || prefix.is_empty() {
            items.push(serde_json::json!({
                "label": func,
                "kind": 3, // Function
                "detail": desc,
                "insertText": format!("{}($1)", func),
                "insertTextFormat": 2 // Snippet
            }));
        }
    }

    items
}

// ═══════════════════════════════════════════════════════════════════════════════════
// HOVER
// ═══════════════════════════════════════════════════════════════════════════════════

fn get_hover(store: &DocumentStore, uri: &str, position: Position) -> serde_json::Value {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return serde_json::Value::Null,
    };

    let line = match doc.get_line(position.line) {
        Some(l) => l,
        None => return serde_json::Value::Null,
    };

    // Find word at position
    let col = position.character as usize;
    let chars: Vec<char> = line.chars().collect();

    let mut start = col;
    let mut end = col;

    while start > 0 && chars.get(start - 1).map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
        start -= 1;
    }
    while end < chars.len() && chars.get(end).map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
        end += 1;
    }

    if start >= end {
        return serde_json::Value::Null;
    }

    let word: String = chars[start..end].iter().collect();

    // Check if it's a known keyword/type/function
    let hover_info = get_hover_info(&word);

    match hover_info {
        Some(info) => serde_json::json!({
            "contents": {
                "kind": "markdown",
                "value": info
            }
        }),
        None => serde_json::Value::Null,
    }
}

fn get_hover_info(word: &str) -> Option<String> {
    match word {
        // Keywords
        "fn" => Some("```lis\nfn name(args) -> ReturnType { ... }\n```\nDefine a function.".to_string()),
        "transform" => Some("```lis\ntransform name(input: State) -> State { ... }\n```\nDefine a state transform with implicit feedback support.".to_string()),
        "type" => Some("```lis\ntype Name = BaseType;\n```\nDefine a type alias.".to_string()),
        "use" => Some("```lis\nuse module::path;\nuse module::path as alias;\n```\nImport a module or symbol.".to_string()),
        "mod" => Some("```lis\nmod name;\npub mod name;\n```\nDeclare a module.".to_string()),
        "pub" => Some("`pub` makes an item visible outside the current module.".to_string()),
        "extern" => Some("```lis\nextern fn name(args) -> ReturnType;\n```\nDeclare an external Rust function (FFI).".to_string()),
        "as" => Some("Rename an imported symbol.\n```lis\nuse module::item as alias;\n```".to_string()),

        // Types
        "Int" => Some("**Int** - 64-bit signed integer\n\nMapped to `MOVI` instruction in VSP.".to_string()),
        "Float" => Some("**Float** - 64-bit floating point\n\nMapped to `COMPLEX` instruction with scaled rho/theta.".to_string()),
        "Bool" => Some("**Bool** - Boolean value\n\n`true` = ByteSil ONE, `false` = ByteSil NULL".to_string()),
        "String" => Some("**String** - UTF-8 string\n\nStored in string pool, referenced by index.".to_string()),
        "ByteSil" => Some("**ByteSil** - 8-bit log-polar complex number\n\n- 4 bits signed rho (-8 to +7)\n- 4 bits unsigned theta (0 to 15)\n\nEnables O(1) complex multiplication.".to_string()),
        "State" => Some("**State** - 16-layer container\n\nEach layer (L0-LF) contains a ByteSil:\n- L0-L3: Perception\n- L4: Boundary\n- L5-L7: Processing\n- L8-LA: Interaction\n- LB-LC: Emergence\n- LD-LF: Meta".to_string()),
        "Complex" => Some("**Complex** - Complex number\n\nAliased to ByteSil internally.".to_string()),

        // Intrinsics
        "print_int" => Some("```lis\nprint_int(value: Int) -> ()\n```\nPrint an integer to stdout.".to_string()),
        "print_float" => Some("```lis\nprint_float(value: Float) -> ()\n```\nPrint a float to stdout.".to_string()),
        "print_string" => Some("```lis\nprint_string(value: String) -> ()\n```\nPrint a string to stdout.".to_string()),
        "state_vacuum" => Some("```lis\nstate_vacuum() -> State\n```\nCreate a State with all layers zeroed.".to_string()),
        "state_neutral" => Some("```lis\nstate_neutral() -> State\n```\nCreate a neutral State (multiplicative identity).".to_string()),
        "state_get_layer" => Some("```lis\nstate_get_layer(s: State, layer: Int) -> ByteSil\n```\nGet a layer (0-15) from a State.".to_string()),
        "state_set_layer" => Some("```lis\nstate_set_layer(s: State, layer: Int, value: ByteSil) -> State\n```\nReturn a new State with the specified layer updated.".to_string()),
        "bytesil_new" => Some("```lis\nbytesil_new(rho: Int, theta: Int) -> ByteSil\n```\nCreate a ByteSil from raw rho (-8..7) and theta (0..15).".to_string()),
        "sin" | "cos" | "tan" | "sqrt" | "exp" | "ln" | "abs" =>
            Some(format!("```lis\n{}(x: Float) -> Float\n```\nMath function.", word)),

        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// DOCUMENT SYMBOLS
// ═══════════════════════════════════════════════════════════════════════════════════

fn get_document_symbols(store: &DocumentStore, uri: &str) -> Vec<serde_json::Value> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };

    let mut symbols = Vec::new();

    for (line_num, line) in doc.lines.iter().enumerate() {
        let trimmed = line.trim();

        // Function: fn name(...) or pub fn name(...)
        if let Some(fn_match) = extract_function_name(trimmed) {
            symbols.push(serde_json::json!({
                "name": fn_match.0,
                "kind": 12, // Function
                "range": {
                    "start": { "line": line_num, "character": 0 },
                    "end": { "line": line_num, "character": line.len() }
                },
                "selectionRange": {
                    "start": { "line": line_num, "character": fn_match.1 },
                    "end": { "line": line_num, "character": fn_match.1 + fn_match.0.len() }
                }
            }));
        }

        // Transform: transform name(...) or pub transform name(...)
        if let Some(t_match) = extract_transform_name(trimmed) {
            symbols.push(serde_json::json!({
                "name": t_match.0,
                "kind": 12, // Function
                "detail": "transform",
                "range": {
                    "start": { "line": line_num, "character": 0 },
                    "end": { "line": line_num, "character": line.len() }
                },
                "selectionRange": {
                    "start": { "line": line_num, "character": t_match.1 },
                    "end": { "line": line_num, "character": t_match.1 + t_match.0.len() }
                }
            }));
        }

        // Type alias: type Name = ...
        if let Some(type_match) = extract_type_name(trimmed) {
            symbols.push(serde_json::json!({
                "name": type_match.0,
                "kind": 5, // Class (used for type)
                "range": {
                    "start": { "line": line_num, "character": 0 },
                    "end": { "line": line_num, "character": line.len() }
                },
                "selectionRange": {
                    "start": { "line": line_num, "character": type_match.1 },
                    "end": { "line": line_num, "character": type_match.1 + type_match.0.len() }
                }
            }));
        }

        // Module: mod name
        if let Some(mod_match) = extract_mod_name(trimmed) {
            symbols.push(serde_json::json!({
                "name": mod_match.0,
                "kind": 2, // Module
                "range": {
                    "start": { "line": line_num, "character": 0 },
                    "end": { "line": line_num, "character": line.len() }
                },
                "selectionRange": {
                    "start": { "line": line_num, "character": mod_match.1 },
                    "end": { "line": line_num, "character": mod_match.1 + mod_match.0.len() }
                }
            }));
        }
    }

    symbols
}

fn extract_function_name(line: &str) -> Option<(String, usize)> {
    let stripped = line.trim_start_matches("pub").trim();
    if !stripped.starts_with("fn ") {
        return None;
    }
    let after_fn = &stripped[3..];
    let name_end = after_fn.find('(')?;
    let name = after_fn[..name_end].trim();
    let offset = line.find(name)?;
    Some((name.to_string(), offset))
}

fn extract_transform_name(line: &str) -> Option<(String, usize)> {
    let stripped = line.trim_start_matches("pub").trim();
    if !stripped.starts_with("transform ") {
        return None;
    }
    let after_transform = &stripped[10..];
    let name_end = after_transform.find('(')?;
    let name = after_transform[..name_end].trim();
    let offset = line.find(name)?;
    Some((name.to_string(), offset))
}

fn extract_type_name(line: &str) -> Option<(String, usize)> {
    let stripped = line.trim_start_matches("pub").trim();
    if !stripped.starts_with("type ") {
        return None;
    }
    let after_type = &stripped[5..];
    let name_end = after_type.find(|c| c == '=' || c == ' ')?;
    let name = after_type[..name_end].trim();
    let offset = line.find(name)?;
    Some((name.to_string(), offset))
}

fn extract_mod_name(line: &str) -> Option<(String, usize)> {
    let stripped = line.trim_start_matches("pub").trim();
    if !stripped.starts_with("mod ") {
        return None;
    }
    let after_mod = &stripped[4..];
    let name_end = after_mod.find(|c| c == ';' || c == ' ' || c == '{')?;
    let name = after_mod[..name_end].trim();
    let offset = line.find(name)?;
    Some((name.to_string(), offset))
}

// ═══════════════════════════════════════════════════════════════════════════════════
// SEMANTIC TOKENS
// ═══════════════════════════════════════════════════════════════════════════════════

fn get_semantic_tokens(store: &DocumentStore, uri: &str) -> Vec<u32> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };

    let mut tokens = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;

    for (line_num, line) in doc.lines.iter().enumerate() {
        let line_num = line_num as u32;

        if line_num != prev_line {
            prev_char = 0;
        }

        let keywords = ["fn", "transform", "type", "let", "return", "if", "else", "loop",
                       "break", "continue", "use", "mod", "pub", "as", "extern", "true", "false"];

        for kw in keywords {
            if let Some(start) = line.find(kw) {
                let before_ok = start == 0 || !line.chars().nth(start - 1).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                let after_ok = start + kw.len() >= line.len() || !line.chars().nth(start + kw.len()).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);

                if before_ok && after_ok {
                    let delta_line = line_num - prev_line;
                    let delta_char = if delta_line == 0 {
                        start as u32 - prev_char
                    } else {
                        start as u32
                    };

                    tokens.extend_from_slice(&[
                        delta_line,
                        delta_char,
                        kw.len() as u32,
                        15, // keyword
                        0,
                    ]);

                    prev_line = line_num;
                    prev_char = start as u32;
                }
            }
        }
    }

    tokens
}

// ═══════════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════════

fn build_server_capabilities() -> serde_json::Value {
    serde_json::json!({
        "textDocumentSync": {
            "openClose": true,
            "change": 1,
            "save": { "includeText": false }
        },
        "completionProvider": {
            "triggerCharacters": [".", ":", " "],
            "resolveProvider": false
        },
        "hoverProvider": true,
        "documentSymbolProvider": true,
        "semanticTokensProvider": {
            "legend": {
                "tokenTypes": [
                    "namespace", "type", "class", "enum", "interface", "struct",
                    "typeParameter", "parameter", "variable", "property", "enumMember",
                    "event", "function", "method", "macro", "keyword", "modifier",
                    "comment", "string", "number"
                ],
                "tokenModifiers": ["declaration", "definition", "readonly", "static", "deprecated"]
            },
            "full": true,
            "range": false
        },
        "diagnosticProvider": {
            "identifier": "lis",
            "interFileDependencies": false,
            "workspaceDiagnostics": false
        }
    })
}

fn json_rpc_response(id: &serde_json::Value, result: serde_json::Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }).to_string()
}

fn json_rpc_error(id: &serde_json::Value, code: i32, message: &str) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    }).to_string()
}

fn parse_position(json: &serde_json::Value) -> Option<Position> {
    Some(Position {
        line: json.get("line")?.as_u64()? as u32,
        character: json.get("character")?.as_u64()? as u32,
    })
}

fn range_to_json(range: &Range) -> serde_json::Value {
    serde_json::json!({
        "start": {
            "line": range.start.line,
            "character": range.start.character
        },
        "end": {
            "line": range.end.line,
            "character": range.end.character
        }
    })
}

fn publish_diagnostics(uri: &str, diagnostics: &[Diagnostic]) -> Option<String> {
    let diags_json: Vec<serde_json::Value> = diagnostics.iter().map(|d| {
        let mut obj = serde_json::json!({
            "range": range_to_json(&d.range),
            "severity": d.severity,
            "message": d.message,
            "source": "lis"
        });
        if let Some(code) = &d.code {
            obj["code"] = serde_json::json!(code);
        }
        obj
    }).collect();

    Some(serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diags_json
        }
    }).to_string())
}
