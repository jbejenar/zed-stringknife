//! `StringKnife` Language Server — LSP binary entry point.

mod config;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

use stringknife_core::detect::{detect_encodings, DetectedEncoding};
use stringknife_core::transforms::{
    base64, case, csv, escape, hash, hex, html, inspect, json, jwt, misc, unicode, url, whitespace,
    xml,
};
use stringknife_core::MAX_INPUT_BYTES;

use crate::config::{Config, HashFormat};

/// Document store: maps document URIs to their full text content.
///
/// Uses `Arc<String>` so that `get_text()` returns a cheap reference-counted
/// handle instead of cloning the full document on every code-action request.
/// Documents are removed from the store on `textDocument/didClose`.
struct DocumentStore {
    documents: Mutex<HashMap<Url, Arc<String>>>,
}

impl DocumentStore {
    fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }

    fn get_text(&self, uri: &Url) -> Option<Arc<String>> {
        self.documents
            .lock()
            .ok()
            .and_then(|docs| docs.get(uri).cloned())
    }

    fn set_text(&self, uri: Url, text: String) {
        if let Ok(mut docs) = self.documents.lock() {
            docs.insert(uri, Arc::new(text));
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
    config: RwLock<Config>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // T-401: Read configuration from initializationOptions.
        if let Some(opts) = params.initialization_options {
            if let Ok(cfg) = serde_json::from_value::<Config>(opts) {
                if let Ok(mut current) = self.config.write() {
                    *current = cfg;
                }
            }
        }
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

    // T-402: Handle workspace/didChangeConfiguration for live config updates.
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Ok(cfg) = serde_json::from_value::<Config>(params.settings) {
            if let Ok(mut current) = self.config.write() {
                *current = cfg;
            }
            self.client
                .log_message(MessageType::INFO, "stringknife-lsp configuration updated")
                .await;
        }
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

        // T-411: Early size check with user-facing feedback.
        if selected.len() > MAX_INPUT_BYTES {
            self.client
                .show_message(
                    MessageType::WARNING,
                    format!(
                        "StringKnife: selection too large ({} bytes, max {} bytes). \
                         Reduce selection size to use transforms.",
                        selected.len(),
                        MAX_INPUT_BYTES,
                    ),
                )
                .await;
            return Ok(Some(Vec::new()));
        }

        let config = self.config.read().map(|c| c.clone()).unwrap_or_default();

        // T-414: Run transforms on the blocking thread pool with a 5-second timeout.
        let uri_owned = uri.clone();
        let timeout_result = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::task::spawn_blocking(move || {
                build_actions(&uri_owned, range, &selected, &config)
            }),
        )
        .await;

        match timeout_result {
            Ok(Ok(actions)) => Ok(Some(actions)),
            Ok(Err(_join_err)) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        "StringKnife: internal error computing code actions",
                    )
                    .await;
                Ok(Some(Vec::new()))
            }
            Err(_elapsed) => {
                self.client
                    .show_message(
                        MessageType::WARNING,
                        "StringKnife: code action computation timed out (5s). \
                         Try a smaller selection.",
                    )
                    .await;
                Ok(Some(Vec::new()))
            }
        }
    }
}

/// Build the full list of code actions for a given selection.
///
/// T-151: Detected decode actions appear first; encode/hash actions always appear.
/// T-152: Actions are ordered by relevance (detected decodes, then encodes, then hashes).
/// T-400: Actions are filtered by `config.enabled_categories` and truncated to
///         `config.max_code_actions`.
#[allow(clippy::too_many_lines)] // flat action registration list, splitting adds no clarity
fn build_actions(
    uri: &Url,
    range: Range,
    selected: &str,
    config: &Config,
) -> Vec<CodeActionOrCommand> {
    let detected = detect_encodings(selected);

    let mut detected_actions = Vec::new();
    let mut encode_actions = Vec::new();

    // T-400: closure that respects smart_detection config.
    // When smart_detection is true (default), decode actions only appear if detected.
    // When false, decode actions appear unconditionally (like encode actions).
    let mut try_decode =
        |title: &str,
         category: &str,
         encoding: DetectedEncoding,
         result: std::result::Result<String, stringknife_core::StringKnifeError>| {
            if !config.is_category_enabled(category) {
                return;
            }
            if let Ok(ref transformed) = result {
                if *transformed != *selected {
                    let action = build_code_action(title, uri.clone(), range, transformed);
                    if !config.smart_detection || detected.contains(&encoding) {
                        detected_actions.push(action);
                    }
                }
            }
        };

    let mut try_encode =
        |title: &str,
         category: &str,
         result: std::result::Result<String, stringknife_core::StringKnifeError>| {
            if !config.is_category_enabled(category) {
                return;
            }
            if let Ok(ref transformed) = result {
                if *transformed != *selected {
                    encode_actions.push(build_code_action(title, uri.clone(), range, transformed));
                }
            }
        };

    // --- Decode actions (shown if detected, or always if smart_detection is off) ---
    try_decode(
        "StringKnife: Base64 Decode",
        "encoding",
        DetectedEncoding::Base64,
        base64::base64_decode(selected),
    );
    try_decode(
        "StringKnife: Base64URL Decode",
        "encoding",
        DetectedEncoding::Base64,
        base64::base64url_decode(selected),
    );
    try_decode(
        "StringKnife: URL Decode",
        "encoding",
        DetectedEncoding::UrlEncoded,
        url::url_decode(selected),
    );
    try_decode(
        "StringKnife: HTML Decode",
        "encoding",
        DetectedEncoding::HtmlEntity,
        html::html_decode(selected),
    );
    try_decode(
        "StringKnife: Hex Decode",
        "encoding",
        DetectedEncoding::Hex,
        hex::hex_decode(selected),
    );
    try_decode(
        "StringKnife: Unicode Unescape",
        "encoding",
        DetectedEncoding::UnicodeEscape,
        unicode::unicode_unescape(selected),
    );

    // JWT decode (shown if JWT detected)
    try_decode(
        "StringKnife: JWT Decode Header",
        "encoding",
        DetectedEncoding::Jwt,
        jwt::jwt_decode_header(selected),
    );
    try_decode(
        "StringKnife: JWT Decode Payload",
        "encoding",
        DetectedEncoding::Jwt,
        jwt::jwt_decode_payload(selected),
    );
    try_decode(
        "StringKnife: JWT Decode (Full)",
        "encoding",
        DetectedEncoding::Jwt,
        jwt::jwt_decode_full(selected),
    );

    // --- Encode actions (always shown if category enabled) ---

    // T-400: Use config.base64_line_breaks to decide which encode variant to use.
    if config.base64_line_breaks {
        try_encode(
            "StringKnife: Base64 Encode (wrapped)",
            "encoding",
            base64::base64_encode_wrapped(selected),
        );
    } else {
        try_encode(
            "StringKnife: Base64 Encode",
            "encoding",
            base64::base64_encode(selected),
        );
    }
    try_encode(
        "StringKnife: Base64URL Encode",
        "encoding",
        base64::base64url_encode(selected),
    );
    try_encode(
        "StringKnife: URL Encode",
        "encoding",
        url::url_encode(selected),
    );
    try_encode(
        "StringKnife: URL Encode (Component)",
        "encoding",
        url::url_encode_component(selected),
    );
    try_encode(
        "StringKnife: HTML Encode",
        "encoding",
        html::html_encode(selected),
    );
    try_encode(
        "StringKnife: Hex Encode",
        "encoding",
        hex::hex_encode(selected),
    );
    try_encode(
        "StringKnife: Unicode Escape",
        "encoding",
        unicode::unicode_escape(selected),
    );
    try_encode(
        "StringKnife: Show Unicode Codepoints",
        "encoding",
        unicode::show_codepoints(selected),
    );
    try_encode(
        "StringKnife: Reverse String",
        "misc",
        misc::reverse_string(selected),
    );

    // --- Case conversion actions ---
    try_encode(
        "StringKnife: To UPPERCASE",
        "case",
        case::to_upper(selected),
    );
    try_encode(
        "StringKnife: To lowercase",
        "case",
        case::to_lower(selected),
    );
    try_encode(
        "StringKnife: To Title Case",
        "case",
        case::to_title_case(selected),
    );
    try_encode(
        "StringKnife: To Sentence Case",
        "case",
        case::to_sentence_case(selected),
    );
    try_encode(
        "StringKnife: To camelCase",
        "case",
        case::to_camel_case(selected),
    );
    try_encode(
        "StringKnife: To PascalCase",
        "case",
        case::to_pascal_case(selected),
    );
    try_encode(
        "StringKnife: To snake_case",
        "case",
        case::to_snake_case(selected),
    );
    try_encode(
        "StringKnife: To SCREAMING_SNAKE_CASE",
        "case",
        case::to_screaming_snake_case(selected),
    );
    try_encode(
        "StringKnife: To kebab-case",
        "case",
        case::to_kebab_case(selected),
    );
    try_encode(
        "StringKnife: To dot.case",
        "case",
        case::to_dot_case(selected),
    );
    try_encode(
        "StringKnife: To path/case",
        "case",
        case::to_path_case(selected),
    );
    try_encode(
        "StringKnife: To CONSTANT_CASE",
        "case",
        case::to_constant_case(selected),
    );
    try_encode(
        "StringKnife: Toggle Case",
        "case",
        case::toggle_case(selected),
    );

    // --- JSON actions ---
    // T-400: Use config.json_indent for pretty print.
    try_encode(
        "StringKnife: JSON Pretty Print",
        "json",
        json::json_pretty_print_with_indent(selected, config.json_indent),
    );
    try_encode(
        "StringKnife: JSON Minify",
        "json",
        json::json_minify(selected),
    );
    try_encode(
        "StringKnife: JSON Escape String",
        "json",
        json::json_escape(selected),
    );
    try_encode(
        "StringKnife: JSON Unescape String",
        "json",
        json::json_unescape(selected),
    );

    // --- XML actions ---
    try_encode(
        "StringKnife: XML Pretty Print",
        "xml",
        xml::xml_pretty_print(selected),
    );
    try_encode("StringKnife: XML Minify", "xml", xml::xml_minify(selected));

    // --- CSV actions ---
    try_encode(
        "StringKnife: CSV → JSON Array",
        "csv",
        csv::csv_to_json(selected),
    );

    // --- Whitespace & line actions ---
    try_encode(
        "StringKnife: Trim Whitespace",
        "whitespace",
        whitespace::trim_whitespace(selected),
    );
    try_encode(
        "StringKnife: Trim Leading",
        "whitespace",
        whitespace::trim_leading(selected),
    );
    try_encode(
        "StringKnife: Trim Trailing",
        "whitespace",
        whitespace::trim_trailing(selected),
    );
    try_encode(
        "StringKnife: Collapse Whitespace",
        "whitespace",
        whitespace::collapse_whitespace(selected),
    );
    try_encode(
        "StringKnife: Remove Blank Lines",
        "whitespace",
        whitespace::remove_blank_lines(selected),
    );
    try_encode(
        "StringKnife: Remove Duplicate Lines",
        "whitespace",
        whitespace::remove_duplicate_lines(selected),
    );
    try_encode(
        "StringKnife: Sort Lines (A→Z)",
        "whitespace",
        whitespace::sort_lines_asc(selected),
    );
    try_encode(
        "StringKnife: Sort Lines (Z→A)",
        "whitespace",
        whitespace::sort_lines_desc(selected),
    );
    try_encode(
        "StringKnife: Sort Lines (by length)",
        "whitespace",
        whitespace::sort_lines_by_length(selected),
    );
    try_encode(
        "StringKnife: Reverse Lines",
        "whitespace",
        whitespace::reverse_lines(selected),
    );
    try_encode(
        "StringKnife: Shuffle Lines",
        "whitespace",
        whitespace::shuffle_lines(selected),
    );
    try_encode(
        "StringKnife: Number Lines",
        "whitespace",
        whitespace::number_lines(selected),
    );

    // --- Hash actions (one-way) ---
    // T-400: Apply config.hash_output_format (uppercase if configured).
    let format_hash = |result: std::result::Result<String, stringknife_core::StringKnifeError>| {
        result.map(|h| {
            if config.hash_output_format == HashFormat::Uppercase {
                h.to_ascii_uppercase()
            } else {
                h
            }
        })
    };
    try_encode(
        "StringKnife: MD5 Hash",
        "hashing",
        format_hash(hash::md5(selected)),
    );
    try_encode(
        "StringKnife: SHA-1 Hash",
        "hashing",
        format_hash(hash::sha1(selected)),
    );
    try_encode(
        "StringKnife: SHA-256 Hash",
        "hashing",
        format_hash(hash::sha256(selected)),
    );
    try_encode(
        "StringKnife: SHA-512 Hash",
        "hashing",
        format_hash(hash::sha512(selected)),
    );
    try_encode(
        "StringKnife: CRC32 Checksum",
        "hashing",
        format_hash(hash::crc32(selected)),
    );

    // --- Escape actions ---
    try_encode(
        "StringKnife: Escape Backslashes",
        "escape",
        escape::escape_backslashes(selected),
    );
    try_encode(
        "StringKnife: Unescape Backslashes",
        "escape",
        escape::unescape_backslashes(selected),
    );
    try_encode(
        "StringKnife: Escape Regex",
        "escape",
        escape::escape_regex(selected),
    );
    try_encode(
        "StringKnife: Escape SQL String",
        "escape",
        escape::escape_sql(selected),
    );
    try_encode(
        "StringKnife: Escape Shell String",
        "escape",
        escape::escape_shell(selected),
    );
    try_encode(
        "StringKnife: Escape CSV Field",
        "escape",
        escape::escape_csv(selected),
    );

    // --- Inspection actions ---
    try_encode(
        "StringKnife: Count Characters",
        "inspect",
        inspect::count_chars(selected),
    );
    try_encode(
        "StringKnife: String Length (bytes)",
        "inspect",
        inspect::byte_length(selected),
    );
    try_encode(
        "StringKnife: Detect Encoding",
        "inspect",
        inspect::detect_encoding(selected),
    );

    // T-152: Detected decodes first, then all encode/misc/hash actions.
    let mut actions = detected_actions;
    actions.extend(encode_actions);

    // T-400: Enforce max_code_actions limit.
    actions.truncate(config.max_code_actions);
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
        config: RwLock::new(Config::default()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Position;

    fn test_uri() -> Url {
        Url::parse("file:///test.txt").expect("valid URL")
    }

    fn full_range() -> Range {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 100,
            },
        }
    }

    // --- Config integration tests ---

    #[test]
    fn default_config_produces_actions() {
        let config = Config::default();
        let actions = build_actions(&test_uri(), full_range(), "hello world", &config);
        assert!(!actions.is_empty(), "default config should produce actions");
    }

    #[test]
    fn disable_all_categories_produces_no_actions() {
        let config = Config {
            enabled_categories: Vec::new(),
            ..Config::default()
        };
        let actions = build_actions(&test_uri(), full_range(), "hello world", &config);
        assert!(
            actions.is_empty(),
            "no categories enabled should produce no actions"
        );
    }

    #[test]
    fn enable_only_case_category() {
        let config = Config {
            enabled_categories: vec!["case".to_string()],
            ..Config::default()
        };
        let actions = build_actions(&test_uri(), full_range(), "hello world", &config);
        assert!(!actions.is_empty());
        for action in &actions {
            if let CodeActionOrCommand::CodeAction(a) = action {
                assert!(
                    a.title.contains("Case")
                        || a.title.contains("UPPER")
                        || a.title.contains("lower")
                        || a.title.contains("camel")
                        || a.title.contains("Pascal")
                        || a.title.contains("snake")
                        || a.title.contains("SCREAMING")
                        || a.title.contains("kebab")
                        || a.title.contains("dot")
                        || a.title.contains("path")
                        || a.title.contains("CONSTANT")
                        || a.title.contains("Toggle"),
                    "expected only case actions, got: {}",
                    a.title
                );
            }
        }
    }

    #[test]
    fn max_code_actions_truncates() {
        let config = Config {
            max_code_actions: 3,
            ..Config::default()
        };
        let actions = build_actions(&test_uri(), full_range(), "hello world", &config);
        assert!(
            actions.len() <= 3,
            "expected at most 3 actions, got {}",
            actions.len()
        );
    }

    #[test]
    fn smart_detection_off_shows_decode_actions() {
        // SGVsbG8= is valid base64 for "Hello"
        let config_on = Config {
            smart_detection: true,
            ..Config::default()
        };
        let config_off = Config {
            smart_detection: false,
            ..Config::default()
        };
        let actions_on = build_actions(&test_uri(), full_range(), "SGVsbG8=", &config_on);
        let actions_off = build_actions(&test_uri(), full_range(), "SGVsbG8=", &config_off);

        // With smart detection on, decode actions should appear (input is detected as base64)
        let has_decode_on = actions_on.iter().any(|a| {
            if let CodeActionOrCommand::CodeAction(ca) = a {
                ca.title.contains("Decode")
            } else {
                false
            }
        });
        assert!(
            has_decode_on,
            "smart detection ON should show decode for base64 input"
        );

        // With smart detection off, decode actions should also appear (unconditionally)
        let has_decode_off = actions_off.iter().any(|a| {
            if let CodeActionOrCommand::CodeAction(ca) = a {
                ca.title.contains("Decode")
            } else {
                false
            }
        });
        assert!(
            has_decode_off,
            "smart detection OFF should show decode unconditionally"
        );
    }

    #[test]
    fn hash_uppercase_format() {
        let config = Config {
            enabled_categories: vec!["hashing".to_string()],
            hash_output_format: HashFormat::Uppercase,
            ..Config::default()
        };
        let actions = build_actions(&test_uri(), full_range(), "hello", &config);
        // All hash actions should produce uppercase hex
        for action in &actions {
            if let CodeActionOrCommand::CodeAction(ca) = action {
                if let Some(edit) = &ca.edit {
                    if let Some(changes) = &edit.changes {
                        for edits in changes.values() {
                            for text_edit in edits {
                                let hex_chars: Vec<char> = text_edit
                                    .new_text
                                    .chars()
                                    .filter(|c| c.is_ascii_hexdigit() && c.is_ascii_alphabetic())
                                    .collect();
                                for c in &hex_chars {
                                    assert!(
                                        c.is_ascii_uppercase(),
                                        "expected uppercase hex in '{}', found '{c}'",
                                        text_edit.new_text
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Existing extract_range tests ---

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

    // --- T-411: Size limit enforcement tests ---

    #[test]
    fn oversized_input_produces_no_actions() {
        let config = Config::default();
        // 1 byte over the limit
        let oversized = "x".repeat(MAX_INPUT_BYTES + 1);
        let actions = build_actions(&test_uri(), full_range(), &oversized, &config);
        // Individual transforms return InputTooLarge which is silently discarded,
        // so build_actions returns an empty list for oversized input.
        assert!(
            actions.is_empty(),
            "oversized input should produce no actions, got {}",
            actions.len()
        );
    }

    // --- T-412: DocumentStore Arc tests ---

    #[test]
    fn document_store_set_get_remove() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///test.txt").expect("valid URL");
        store.set_text(uri.clone(), "hello".to_string());
        assert_eq!(
            store.get_text(&uri).as_deref().map(String::as_str),
            Some("hello")
        );
        store.remove(&uri);
        assert!(store.get_text(&uri).is_none());
    }

    #[test]
    fn document_store_arc_sharing() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///test.txt").expect("valid URL");
        store.set_text(uri.clone(), "hello".to_string());
        let text1 = store.get_text(&uri).expect("text1");
        let text2 = store.get_text(&uri).expect("text2");
        assert!(
            Arc::ptr_eq(&text1, &text2),
            "repeated get_text should return the same Arc allocation"
        );
    }

    #[test]
    fn document_store_overwrite() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///test.txt").expect("valid URL");
        store.set_text(uri.clone(), "old content".to_string());
        store.set_text(uri.clone(), "new content".to_string());
        assert_eq!(
            store.get_text(&uri).as_deref().map(String::as_str),
            Some("new content")
        );
    }

    // --- T-413: Sustained operation tests ---

    #[test]
    fn sustained_build_actions_no_accumulation() {
        let uri = test_uri();
        let config = Config::default();
        for _ in 0..1000 {
            let actions = build_actions(&uri, full_range(), "hello world", &config);
            assert!(!actions.is_empty());
        }
        // build_actions is stateless — if this completes, there is no accumulation.
    }

    #[test]
    fn document_store_churn() {
        let store = DocumentStore::new();
        let content = "x".repeat(102_400); // 100KB
        for i in 0..100 {
            let uri = Url::parse(&format!("file:///doc_{i}.txt")).expect("valid URL");
            store.set_text(uri.clone(), content.clone());
            assert!(store.get_text(&uri).is_some());
            store.remove(&uri);
            assert!(store.get_text(&uri).is_none());
        }
        // All documents removed — store should be empty.
    }
}
