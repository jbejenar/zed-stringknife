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

use stringknife_core::detect::{detect_encodings, DetectedEncoding};
use stringknife_core::transforms::{
    base64, case, csv, escape, hash, hex, html, inspect, json, jwt, misc, unicode, url, whitespace,
    xml,
};

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

        // T-154: No action if no text is selected (collapsed range).
        if range.start == range.end {
            return Ok(Some(Vec::new()));
        }

        let Some(text) = self.store.get_text(uri) else {
            return Ok(Some(Vec::new()));
        };

        // T-153: extract_range handles multi-line selections correctly.
        let Some(selected) = extract_range(&text, range) else {
            return Ok(Some(Vec::new()));
        };

        Ok(Some(build_actions(uri, range, &selected)))
    }
}

/// Build the full list of code actions for a given selection.
///
/// T-151: Detected decode actions appear first; encode/hash actions always appear.
/// T-152: Actions are ordered by relevance (detected decodes, then encodes, then hashes).
#[allow(clippy::too_many_lines)] // flat action registration list, splitting adds no clarity
fn build_actions(uri: &Url, range: Range, selected: &str) -> Vec<CodeActionOrCommand> {
    let detected = detect_encodings(selected);

    let mut detected_actions = Vec::new();
    let mut encode_actions = Vec::new();

    let mut try_decode =
        |title: &str,
         encoding: DetectedEncoding,
         result: std::result::Result<String, stringknife_core::StringKnifeError>| {
            if let Ok(ref transformed) = result {
                if *transformed != *selected {
                    let action = build_code_action(title, uri.clone(), range, transformed);
                    if detected.contains(&encoding) {
                        detected_actions.push(action);
                    }
                }
            }
        };

    let mut try_encode =
        |title: &str, result: std::result::Result<String, stringknife_core::StringKnifeError>| {
            if let Ok(ref transformed) = result {
                if *transformed != *selected {
                    encode_actions.push(build_code_action(title, uri.clone(), range, transformed));
                }
            }
        };

    // --- Decode actions (only shown if detected) ---
    try_decode(
        "StringKnife: Base64 Decode",
        DetectedEncoding::Base64,
        base64::base64_decode(selected),
    );
    try_decode(
        "StringKnife: Base64URL Decode",
        DetectedEncoding::Base64,
        base64::base64url_decode(selected),
    );
    try_decode(
        "StringKnife: URL Decode",
        DetectedEncoding::UrlEncoded,
        url::url_decode(selected),
    );
    try_decode(
        "StringKnife: HTML Decode",
        DetectedEncoding::HtmlEntity,
        html::html_decode(selected),
    );
    try_decode(
        "StringKnife: Hex Decode",
        DetectedEncoding::Hex,
        hex::hex_decode(selected),
    );
    try_decode(
        "StringKnife: Unicode Unescape",
        DetectedEncoding::UnicodeEscape,
        unicode::unicode_unescape(selected),
    );

    // JWT decode (shown if JWT detected)
    try_decode(
        "StringKnife: JWT Decode Header",
        DetectedEncoding::Jwt,
        jwt::jwt_decode_header(selected),
    );
    try_decode(
        "StringKnife: JWT Decode Payload",
        DetectedEncoding::Jwt,
        jwt::jwt_decode_payload(selected),
    );
    try_decode(
        "StringKnife: JWT Decode (Full)",
        DetectedEncoding::Jwt,
        jwt::jwt_decode_full(selected),
    );

    // --- Encode actions (always shown) ---
    try_encode(
        "StringKnife: Base64 Encode",
        base64::base64_encode(selected),
    );
    try_encode(
        "StringKnife: Base64URL Encode",
        base64::base64url_encode(selected),
    );
    try_encode("StringKnife: URL Encode", url::url_encode(selected));
    try_encode(
        "StringKnife: URL Encode (Component)",
        url::url_encode_component(selected),
    );
    try_encode("StringKnife: HTML Encode", html::html_encode(selected));
    try_encode("StringKnife: Hex Encode", hex::hex_encode(selected));
    try_encode(
        "StringKnife: Unicode Escape",
        unicode::unicode_escape(selected),
    );
    try_encode(
        "StringKnife: Show Unicode Codepoints",
        unicode::show_codepoints(selected),
    );
    try_encode(
        "StringKnife: Reverse String",
        misc::reverse_string(selected),
    );

    // --- Case conversion actions (always shown) ---
    try_encode("StringKnife: To UPPERCASE", case::to_upper(selected));
    try_encode("StringKnife: To lowercase", case::to_lower(selected));
    try_encode("StringKnife: To Title Case", case::to_title_case(selected));
    try_encode(
        "StringKnife: To Sentence Case",
        case::to_sentence_case(selected),
    );
    try_encode("StringKnife: To camelCase", case::to_camel_case(selected));
    try_encode("StringKnife: To PascalCase", case::to_pascal_case(selected));
    try_encode("StringKnife: To snake_case", case::to_snake_case(selected));
    try_encode(
        "StringKnife: To SCREAMING_SNAKE_CASE",
        case::to_screaming_snake_case(selected),
    );
    try_encode("StringKnife: To kebab-case", case::to_kebab_case(selected));
    try_encode("StringKnife: To dot.case", case::to_dot_case(selected));
    try_encode("StringKnife: To path/case", case::to_path_case(selected));
    try_encode(
        "StringKnife: To CONSTANT_CASE",
        case::to_constant_case(selected),
    );
    try_encode("StringKnife: Toggle Case", case::toggle_case(selected));

    // --- JSON actions (always shown) ---
    try_encode(
        "StringKnife: JSON Pretty Print",
        json::json_pretty_print(selected),
    );
    try_encode("StringKnife: JSON Minify", json::json_minify(selected));
    try_encode(
        "StringKnife: JSON Escape String",
        json::json_escape(selected),
    );
    try_encode(
        "StringKnife: JSON Unescape String",
        json::json_unescape(selected),
    );

    // --- XML actions (always shown) ---
    try_encode(
        "StringKnife: XML Pretty Print",
        xml::xml_pretty_print(selected),
    );
    try_encode("StringKnife: XML Minify", xml::xml_minify(selected));

    // --- CSV actions (always shown) ---
    try_encode("StringKnife: CSV → JSON Array", csv::csv_to_json(selected));

    // --- Whitespace & line actions (always shown) ---
    try_encode(
        "StringKnife: Trim Whitespace",
        whitespace::trim_whitespace(selected),
    );
    try_encode(
        "StringKnife: Trim Leading",
        whitespace::trim_leading(selected),
    );
    try_encode(
        "StringKnife: Trim Trailing",
        whitespace::trim_trailing(selected),
    );
    try_encode(
        "StringKnife: Collapse Whitespace",
        whitespace::collapse_whitespace(selected),
    );
    try_encode(
        "StringKnife: Remove Blank Lines",
        whitespace::remove_blank_lines(selected),
    );
    try_encode(
        "StringKnife: Remove Duplicate Lines",
        whitespace::remove_duplicate_lines(selected),
    );
    try_encode(
        "StringKnife: Sort Lines (A→Z)",
        whitespace::sort_lines_asc(selected),
    );
    try_encode(
        "StringKnife: Sort Lines (Z→A)",
        whitespace::sort_lines_desc(selected),
    );
    try_encode(
        "StringKnife: Sort Lines (by length)",
        whitespace::sort_lines_by_length(selected),
    );
    try_encode(
        "StringKnife: Reverse Lines",
        whitespace::reverse_lines(selected),
    );
    try_encode(
        "StringKnife: Shuffle Lines",
        whitespace::shuffle_lines(selected),
    );
    try_encode(
        "StringKnife: Number Lines",
        whitespace::number_lines(selected),
    );

    // --- Hash actions (one-way, always shown) ---
    try_encode("StringKnife: MD5 Hash", hash::md5(selected));
    try_encode("StringKnife: SHA-1 Hash", hash::sha1(selected));
    try_encode("StringKnife: SHA-256 Hash", hash::sha256(selected));
    try_encode("StringKnife: SHA-512 Hash", hash::sha512(selected));
    try_encode("StringKnife: CRC32 Checksum", hash::crc32(selected));

    // --- Escape actions (always shown) ---
    try_encode(
        "StringKnife: Escape Backslashes",
        escape::escape_backslashes(selected),
    );
    try_encode(
        "StringKnife: Unescape Backslashes",
        escape::unescape_backslashes(selected),
    );
    try_encode("StringKnife: Escape Regex", escape::escape_regex(selected));
    try_encode(
        "StringKnife: Escape SQL String",
        escape::escape_sql(selected),
    );
    try_encode(
        "StringKnife: Escape Shell String",
        escape::escape_shell(selected),
    );
    try_encode(
        "StringKnife: Escape CSV Field",
        escape::escape_csv(selected),
    );

    // --- Inspection actions (always shown) ---
    try_encode(
        "StringKnife: Count Characters",
        inspect::count_chars(selected),
    );
    try_encode(
        "StringKnife: String Length (bytes)",
        inspect::byte_length(selected),
    );
    try_encode(
        "StringKnife: Detect Encoding",
        inspect::detect_encoding(selected),
    );

    // T-152: Detected decodes first, then all encode/misc/hash actions.
    let mut actions = detected_actions;
    actions.extend(encode_actions);
    actions
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
