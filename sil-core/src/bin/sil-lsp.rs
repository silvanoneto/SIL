//! # sil-lsp — SIL Language Server
//!
//! Language Server Protocol implementation for .sil files.
//!
//! ```bash
//! sil-lsp              # Run on stdio
//! sil-lsp --debug      # With debug logging
//! ```

use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::ExitCode;

use sil_core::vsp::lsp::{
    SilLanguageServer, LspConfig, Position, Range,
    semantic_tokens::{SemanticTokenType, SemanticTokenModifier},
};
use sil_core::vsp::lsp::server::{
    DidOpenParams, DidChangeParams, CompletionItemKind, InsertTextFormat,
    DiagnosticSeverity,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");

    if debug {
        eprintln!("sil-lsp v1.0.0 starting in debug mode...");
    }

    let config = LspConfig::default();
    let mut server = SilLanguageServer::with_config(config);

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
        let response = handle_request(&mut server, &request, debug);

        if let Some(resp) = response {
            send_response(&mut stdout_lock, &resp, debug);
        }

        // Check for shutdown
        if request.contains("\"method\":\"shutdown\"") {
            // Wait for exit notification
            continue;
        }

        if request.contains("\"method\":\"exit\"") {
            break;
        }
    }

    ExitCode::SUCCESS
}

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

fn handle_request(server: &mut SilLanguageServer, request: &str, debug: bool) -> Option<String> {
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
            server.initialize(Default::default());

            let capabilities = build_server_capabilities();
            let result = serde_json::json!({
                "capabilities": capabilities,
                "serverInfo": {
                    "name": "sil-lsp",
                    "version": "1.0.0"
                }
            });

            Some(json_rpc_response(id, result))
        }

        "initialized" => None, // Notification, no response

        "shutdown" => {
            let id = id?;
            server.shutdown();
            Some(json_rpc_response(id, serde_json::Value::Null))
        }

        "exit" => None, // Handled in main loop

        // ═══════════════════════════════════════════════════════════════════════════
        // TEXT DOCUMENT SYNC
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/didOpen" => {
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?.to_string();
            let version = text_document.get("version")?.as_i64()? as i32;
            let text = text_document.get("text")?.as_str()?.to_string();

            let diagnostics = server.did_open(DidOpenParams { uri: uri.clone(), version, text });

            // Publish diagnostics
            publish_diagnostics(&uri, &diagnostics)
        }

        "textDocument/didChange" => {
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?.to_string();
            let version = text_document.get("version")?.as_i64()? as i32;

            // Get full content from contentChanges
            let content_changes = params.get("contentChanges")?.as_array()?;
            let text = content_changes.first()?.get("text")?.as_str()?.to_string();

            let diagnostics = server.did_change(DidChangeParams { uri: uri.clone(), version, text });

            publish_diagnostics(&uri, &diagnostics)
        }

        "textDocument/didClose" => {
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            server.did_close(uri);

            // Clear diagnostics
            Some(format!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"uri":"{}","diagnostics":[]}}}}"#,
                uri
            ))
        }

        "textDocument/didSave" => None, // We don't need special handling for save

        // ═══════════════════════════════════════════════════════════════════════════
        // COMPLETION
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/completion" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;
            let position = parse_position(params.get("position")?)?;

            let items = server.completion(uri, position);

            let items_json: Vec<serde_json::Value> = items.into_iter().map(|item| {
                serde_json::json!({
                    "label": item.label,
                    "kind": completion_kind_to_lsp(item.kind),
                    "detail": item.detail,
                    "documentation": item.documentation.map(|d| {
                        serde_json::json!({
                            "kind": "markdown",
                            "value": d
                        })
                    }),
                    "insertText": item.insert_text,
                    "insertTextFormat": if item.insert_text_format == InsertTextFormat::Snippet { 2 } else { 1 }
                })
            }).collect();

            Some(json_rpc_response(id, serde_json::json!({
                "isIncomplete": false,
                "items": items_json
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

            let result = server.hover(uri, position);

            let response = match result {
                Some(hover) => serde_json::json!({
                    "contents": {
                        "kind": "markdown",
                        "value": hover.contents
                    },
                    "range": hover.range.map(|r| range_to_json(&r))
                }),
                None => serde_json::Value::Null,
            };

            Some(json_rpc_response(id, response))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // GO TO DEFINITION
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/definition" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;
            let position = parse_position(params.get("position")?)?;

            let result = server.definition(uri, position);

            let response = match result {
                Some(loc) => serde_json::json!({
                    "uri": loc.target_uri,
                    "range": range_to_json(&loc.target_range)
                }),
                None => serde_json::Value::Null,
            };

            Some(json_rpc_response(id, response))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // FIND REFERENCES
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/references" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;
            let position = parse_position(params.get("position")?)?;
            let include_declaration = params.get("context")
                .and_then(|c| c.get("includeDeclaration"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let refs = server.references(uri, position, include_declaration);

            let locations: Vec<serde_json::Value> = refs.into_iter().map(|loc| {
                serde_json::json!({
                    "uri": loc.target_uri,
                    "range": range_to_json(&loc.target_range)
                })
            }).collect();

            Some(json_rpc_response(id, serde_json::json!(locations)))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // DOCUMENT SYMBOLS
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/documentSymbol" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            let symbols = server.document_symbols(uri);

            let symbols_json: Vec<serde_json::Value> = symbols.into_iter().map(|sym| {
                symbol_to_json(&sym)
            }).collect();

            Some(json_rpc_response(id, serde_json::json!(symbols_json)))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // FORMATTING
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/formatting" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            let edits = server.format(uri);

            let edits_json: Vec<serde_json::Value> = edits.into_iter().map(|edit| {
                serde_json::json!({
                    "range": range_to_json(&edit.range),
                    "newText": edit.new_text
                })
            }).collect();

            Some(json_rpc_response(id, serde_json::json!(edits_json)))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // SEMANTIC TOKENS
        // ═══════════════════════════════════════════════════════════════════════════
        "textDocument/semanticTokens/full" => {
            let id = id?;
            let params = params?;
            let text_document = params.get("textDocument")?;
            let uri = text_document.get("uri")?.as_str()?;

            let result = server.semantic_tokens(uri);

            Some(json_rpc_response(id, serde_json::json!({
                "resultId": result.result_id,
                "data": result.data
            })))
        }

        // ═══════════════════════════════════════════════════════════════════════════
        // UNKNOWN
        // ═══════════════════════════════════════════════════════════════════════════
        _ => {
            if debug {
                eprintln!("Unknown method: {}", method);
            }

            // If it has an id, respond with method not found
            if let Some(id) = id {
                Some(json_rpc_error(id, -32601, &format!("Method not found: {}", method)))
            } else {
                None
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════════

fn build_server_capabilities() -> serde_json::Value {
    serde_json::json!({
        "textDocumentSync": {
            "openClose": true,
            "change": 1, // Full sync
            "save": { "includeText": false }
        },
        "completionProvider": {
            "triggerCharacters": [".", " ", ","],
            "resolveProvider": false
        },
        "hoverProvider": true,
        "definitionProvider": true,
        "referencesProvider": true,
        "documentSymbolProvider": true,
        "documentFormattingProvider": true,
        "semanticTokensProvider": {
            "legend": {
                "tokenTypes": SemanticTokenType::all(),
                "tokenModifiers": SemanticTokenModifier::all()
            },
            "full": true,
            "range": false
        },
        "diagnosticProvider": {
            "identifier": "sil",
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

fn publish_diagnostics(uri: &str, diagnostics: &[sil_core::vsp::lsp::server::Diagnostic]) -> Option<String> {
    let diags_json: Vec<serde_json::Value> = diagnostics.iter().map(|d| {
        serde_json::json!({
            "range": range_to_json(&d.range),
            "severity": diagnostic_severity_to_lsp(d.severity),
            "code": d.code,
            "source": d.source,
            "message": d.message
        })
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

fn completion_kind_to_lsp(kind: CompletionItemKind) -> u32 {
    match kind {
        CompletionItemKind::Keyword => 14,
        CompletionItemKind::Function => 3,
        CompletionItemKind::Variable => 6,
        CompletionItemKind::Constant => 21,
        CompletionItemKind::Snippet => 15,
        CompletionItemKind::Field => 5,
        CompletionItemKind::Module => 9,
    }
}

fn diagnostic_severity_to_lsp(severity: DiagnosticSeverity) -> u32 {
    match severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Information => 3,
        DiagnosticSeverity::Hint => 4,
    }
}

fn symbol_kind_to_lsp(kind: &sil_core::vsp::lsp::symbols::SymbolKind) -> u32 {
    match kind {
        sil_core::vsp::lsp::symbols::SymbolKind::Function => 12,
        sil_core::vsp::lsp::symbols::SymbolKind::Variable => 13,
        sil_core::vsp::lsp::symbols::SymbolKind::Constant => 14,
        sil_core::vsp::lsp::symbols::SymbolKind::Module => 2,
        sil_core::vsp::lsp::symbols::SymbolKind::String => 15,
    }
}

fn symbol_to_json(sym: &sil_core::vsp::lsp::symbols::SymbolInformation) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "name": sym.name,
        "kind": symbol_kind_to_lsp(&sym.kind),
        "range": range_to_json(&sym.range),
        "selectionRange": range_to_json(&sym.selection_range)
    });

    if let Some(detail) = &sym.detail {
        obj["detail"] = serde_json::json!(detail);
    }

    if !sym.children.is_empty() {
        obj["children"] = serde_json::json!(
            sym.children.iter().map(symbol_to_json).collect::<Vec<_>>()
        );
    }

    obj
}
