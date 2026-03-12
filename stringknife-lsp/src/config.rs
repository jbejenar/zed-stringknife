//! Configuration schema for the `StringKnife` LSP.
//!
//! Defines the typed configuration structure read from Zed's
//! `initializationOptions` and updated via `workspace/didChangeConfiguration`.

use serde::Deserialize;

/// All available transform categories.
pub const ALL_CATEGORIES: &[&str] = &[
    "encoding",
    "hashing",
    "case",
    "json",
    "xml",
    "csv",
    "whitespace",
    "escape",
    "inspect",
    "misc",
];

/// Hash output format: lowercase or uppercase hex digits.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HashFormat {
    /// Lowercase hex digits (default).
    #[default]
    Lowercase,
    /// Uppercase hex digits.
    Uppercase,
}

/// `StringKnife` LSP configuration.
///
/// All fields have sensible defaults. When no configuration is provided,
/// the server behaves identically to pre-configuration versions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// Which transform categories are enabled. Default: all categories.
    ///
    /// Valid values: `encoding`, `hashing`, `case`, `json`, `xml`, `csv`,
    /// `whitespace`, `escape`, `inspect`, `misc`.
    pub enabled_categories: Vec<String>,

    /// Maximum number of code actions to show in the context menu.
    /// Default: 50 (effectively unlimited for current action count).
    pub max_code_actions: usize,

    /// Whether to use smart encoding detection to surface likely decode
    /// actions first. When false, decode actions are shown unconditionally.
    /// Default: true.
    pub smart_detection: bool,

    /// Output format for hash digests.
    /// Default: `"lowercase"`.
    pub hash_output_format: HashFormat,

    /// Number of spaces per indent level for JSON pretty print.
    /// Default: 2.
    pub json_indent: usize,

    /// Whether to wrap Base64 output at 76 characters per line (MIME style).
    /// Default: false.
    pub base64_line_breaks: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled_categories: ALL_CATEGORIES.iter().map(|&s| s.to_string()).collect(),
            max_code_actions: 50,
            smart_detection: true,
            hash_output_format: HashFormat::default(),
            json_indent: 2,
            base64_line_breaks: false,
        }
    }
}

impl Config {
    /// Returns true if the given category is enabled.
    pub fn is_category_enabled(&self, category: &str) -> bool {
        self.enabled_categories.iter().any(|c| c == category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_all_categories() {
        let config = Config::default();
        for &cat in ALL_CATEGORIES {
            assert!(
                config.is_category_enabled(cat),
                "category {cat} should be enabled by default"
            );
        }
    }

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.max_code_actions, 50);
        assert!(config.smart_detection);
        assert_eq!(config.hash_output_format, HashFormat::Lowercase);
        assert_eq!(config.json_indent, 2);
        assert!(!config.base64_line_breaks);
    }

    #[test]
    fn deserialize_partial_config() {
        let json = r#"{"maxCodeActions": 10, "smartDetection": false}"#;
        let config: Config = serde_json::from_str(json).expect("should parse");
        assert_eq!(config.max_code_actions, 10);
        assert!(!config.smart_detection);
        // Defaults for unset fields
        assert_eq!(config.json_indent, 2);
        assert!(config.is_category_enabled("encoding"));
    }

    #[test]
    fn deserialize_full_config() {
        let json = r#"{
            "enabledCategories": ["encoding", "case"],
            "maxCodeActions": 5,
            "smartDetection": false,
            "hashOutputFormat": "uppercase",
            "jsonIndent": 4,
            "base64LineBreaks": true
        }"#;
        let config: Config = serde_json::from_str(json).expect("should parse");
        assert_eq!(config.enabled_categories, vec!["encoding", "case"]);
        assert_eq!(config.max_code_actions, 5);
        assert!(!config.smart_detection);
        assert_eq!(config.hash_output_format, HashFormat::Uppercase);
        assert_eq!(config.json_indent, 4);
        assert!(config.base64_line_breaks);
    }

    #[test]
    fn deserialize_empty_config() {
        let json = "{}";
        let config: Config = serde_json::from_str(json).expect("should parse");
        assert_eq!(config.max_code_actions, 50);
        assert!(config.smart_detection);
    }

    #[test]
    fn category_filtering() {
        let config = Config {
            enabled_categories: vec!["encoding".to_string(), "case".to_string()],
            ..Config::default()
        };
        assert!(config.is_category_enabled("encoding"));
        assert!(config.is_category_enabled("case"));
        assert!(!config.is_category_enabled("hashing"));
        assert!(!config.is_category_enabled("json"));
    }
}
