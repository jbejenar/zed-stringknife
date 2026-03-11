//! `StringKnife` Language Server — LSP binary entry point.

use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, MessageType, Range, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

use stringknife_core::transforms::{base64, hex, html, misc, unicode, url};

/// Document store: maps document URIs to their full text content.
struct DocumentStore {
    documents: Mutex<HashMap<Url, String>>,
}

impl DocumentStore {
    fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }

    fn get_text(&self, uri: &Url) -> Option<String> {
        self.documents
            .lock()
            .ok()
            .and_then(|docs| docs.get(uri).cloned())
    }

    fn set_text(&self, uri: Url, text: String) {
        if let Ok(mut docs) = self.documents.lock() {
            docs.insert(uri, text);
        }
    }

    fn remove(&self, uri: &Url) {
        if let Ok(mut docs) = self.documents.lock() {
            docs.remove(uri);
        }
    }
}

/// The `StringKnife` LSP backend.
struct Backend {
    client: Client,
    store: DocumentStore,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "stringknife-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "stringknife-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.store
            .set_text(params.text_document.uri, params.text_document.text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Full sync: take the last content change (which is the full document).
        if let Some(change) = params.content_changes.into_iter().last() {
            self.store.set_text(params.text_document.uri, change.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.store.remove(&params.text_document.uri);
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        // No action if no text is selected (collapsed range).
        if range.start == range.end {
            return Ok(Some(Vec::new()));
        }

        let Some(text) = self.store.get_text(uri) else {
            return Ok(Some(Vec::new()));
        };

        let Some(selected) = extract_range(&text, range) else {
            return Ok(Some(Vec::new()));
        };

        let mut actions = Vec::new();

        // Helper: try a transform and add a code action if it produces a different result.
        let mut try_action =
            |title: &str,
             result: std::result::Result<String, stringknife_core::StringKnifeError>| {
                if let Ok(transformed) = result {
                    if transformed != selected {
                        actions.push(build_code_action(title, uri.clone(), range, &transformed));
                    }
                }
            };

        // Misc
        try_action(
            "StringKnife: Reverse String",
            misc::reverse_string(&selected),
        );

        // Base64
        try_action(
            "StringKnife: Base64 Encode",
            base64::base64_encode(&selected),
        );
        try_action(
            "StringKnife: Base64 Decode",
            base64::base64_decode(&selected),
        );
        try_action(
            "StringKnife: Base64URL Encode",
            base64::base64url_encode(&selected),
        );
        try_action(
            "StringKnife: Base64URL Decode",
            base64::base64url_decode(&selected),
        );

        // URL encoding
        try_action("StringKnife: URL Encode", url::url_encode(&selected));
        try_action("StringKnife: URL Decode", url::url_decode(&selected));
        try_action(
            "StringKnife: URL Encode (Component)",
            url::url_encode_component(&selected),
        );

        // HTML entities
        try_action("StringKnife: HTML Encode", html::html_encode(&selected));
        try_action("StringKnife: HTML Decode", html::html_decode(&selected));

        // Hex
        try_action("StringKnife: Hex Encode", hex::hex_encode(&selected));
        try_action("StringKnife: Hex Decode", hex::hex_decode(&selected));

        // Unicode
        try_action(
            "StringKnife: Unicode Escape",
            unicode::unicode_escape(&selected),
        );
        try_action(
            "StringKnife: Unicode Unescape",
            unicode::unicode_unescape(&selected),
        );
        try_action(
            "StringKnife: Show Unicode Codepoints",
            unicode::show_codepoints(&selected),
        );

        Ok(Some(actions))
    }
}

/// Extract the text within a given LSP range from the full document text.
fn extract_range(text: &str, range: Range) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();

    let start_line = range.start.line as usize;
    let start_char = range.start.character as usize;
    let end_line = range.end.line as usize;
    let end_char = range.end.character as usize;

    if start_line >= lines.len() {
        return None;
    }

    if start_line == end_line {
        let line = lines.get(start_line)?;
        let start_byte = char_offset_to_byte(line, start_char)?;
        let end_byte = char_offset_to_byte(line, end_char)?;
        Some(line[start_byte..end_byte].to_string())
    } else {
        let mut result = String::new();

        // First line: from start_char to end of line.
        let first_line = lines.get(start_line)?;
        let start_byte = char_offset_to_byte(first_line, start_char)?;
        result.push_str(&first_line[start_byte..]);

        // Middle lines: full lines.
        for line in lines.iter().take(end_line).skip(start_line + 1) {
            result.push('\n');
            result.push_str(line);
        }

        // Last line: from start to end_char.
        // When end_char is 0 and end_line is past the last line,
        // the selection ends at the very end of the previous line.
        if end_char == 0 && end_line >= lines.len() {
            // Nothing more to add — we already included up to the last line.
        } else {
            let last_line = lines.get(end_line)?;
            let end_byte = char_offset_to_byte(last_line, end_char)?;
            result.push('\n');
            result.push_str(&last_line[..end_byte]);
        }

        Some(result)
    }
}

/// Convert a UTF-16 character offset to a byte offset in a string.
fn char_offset_to_byte(s: &str, char_offset: usize) -> Option<usize> {
    let mut utf16_count = 0;
    for (byte_idx, ch) in s.char_indices() {
        if utf16_count == char_offset {
            return Some(byte_idx);
        }
        utf16_count += ch.len_utf16();
    }
    if utf16_count == char_offset {
        Some(s.len())
    } else {
        None
    }
}

/// Build a `CodeAction` that replaces the given range with new text.
fn build_code_action(title: &str, uri: Url, range: Range, new_text: &str) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                uri,
                vec![TextEdit {
                    range,
                    new_text: new_text.to_string(),
                }],
            )])),
            ..WorkspaceEdit::default()
        }),
        ..CodeAction::default()
    })
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        store: DocumentStore::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn extract_single_line_range() {
        let text = "hello world";
        let range = Range {
            start: Position {
                line: 0,
                character: 6,
            },
            end: Position {
                line: 0,
                character: 11,
            },
        };
        assert_eq!(extract_range(text, range).as_deref(), Some("world"));
    }

    #[test]
    fn extract_multi_line_range() {
        let text = "line one\nline two\nline three";
        let range = Range {
            start: Position {
                line: 0,
                character: 5,
            },
            end: Position {
                line: 1,
                character: 4,
            },
        };
        assert_eq!(extract_range(text, range).as_deref(), Some("one\nline"));
    }

    #[test]
    fn extract_empty_range() {
        let text = "hello";
        let range = Range {
            start: Position {
                line: 0,
                character: 2,
            },
            end: Position {
                line: 0,
                character: 2,
            },
        };
        assert_eq!(extract_range(text, range).as_deref(), Some(""));
    }

    #[test]
    fn char_offset_ascii() {
        assert_eq!(char_offset_to_byte("hello", 0), Some(0));
        assert_eq!(char_offset_to_byte("hello", 3), Some(3));
        assert_eq!(char_offset_to_byte("hello", 5), Some(5));
    }

    #[test]
    fn char_offset_unicode() {
        // "héllo" — é is 2 bytes in UTF-8, 1 code unit in UTF-16
        assert_eq!(char_offset_to_byte("héllo", 0), Some(0));
        assert_eq!(char_offset_to_byte("héllo", 1), Some(1)); // 'h'
        assert_eq!(char_offset_to_byte("héllo", 2), Some(3)); // after 'é' (2 bytes)
    }
}
