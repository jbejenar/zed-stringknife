//! XML formatting transforms.
//!
//! Pretty print and minify XML/HTML content.
//! Character-level formatting — no DOM parser, no external dependencies.

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Pretty-prints XML with 2-space indentation.
///
/// Re-formats XML/HTML regardless of existing whitespace. Handles elements,
/// attributes, self-closing tags, comments, CDATA sections, processing
/// instructions, and text content.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is empty.
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn xml_pretty_print(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "xml_pretty_print".to_string(),
            reason: "empty XML input".to_string(),
        });
    }

    let tokens = tokenize_xml(trimmed);
    if tokens.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "xml_pretty_print".to_string(),
            reason: "no XML elements found".to_string(),
        });
    }

    let mut result = String::with_capacity(trimmed.len() * 2);
    let mut indent: usize = 0;

    for (i, token) in tokens.iter().enumerate() {
        match token {
            XmlToken::OpenTag(tag) => {
                if !result.is_empty() {
                    result.push('\n');
                }
                push_indent(&mut result, indent);
                result.push_str(tag);
                indent += 1;
            }
            XmlToken::CloseTag(tag) => {
                indent = indent.saturating_sub(1);
                // Inline close tag if preceded by open tag or open+text (leaf element)
                let inline = if i > 0 {
                    matches!(&tokens[i - 1], XmlToken::OpenTag(_))
                        || (i >= 2
                            && matches!(&tokens[i - 1], XmlToken::Text(_))
                            && matches!(&tokens[i - 2], XmlToken::OpenTag(_)))
                } else {
                    false
                };
                if !inline {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    push_indent(&mut result, indent);
                }
                result.push_str(tag);
            }
            XmlToken::SelfClosing(tag) => {
                if !result.is_empty() {
                    result.push('\n');
                }
                push_indent(&mut result, indent);
                result.push_str(tag);
            }
            XmlToken::Text(text) => {
                let trimmed_text = text.trim();
                if !trimmed_text.is_empty() {
                    // If previous token was an open tag, put text inline
                    let prev_was_open = if i > 0 {
                        matches!(&tokens[i - 1], XmlToken::OpenTag(_))
                    } else {
                        false
                    };
                    if prev_was_open {
                        result.push_str(trimmed_text);
                    } else {
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        push_indent(&mut result, indent);
                        result.push_str(trimmed_text);
                    }
                }
            }
            XmlToken::Comment(comment) | XmlToken::CData(comment) | XmlToken::PI(comment) => {
                if !result.is_empty() {
                    result.push('\n');
                }
                push_indent(&mut result, indent);
                result.push_str(comment);
            }
        }
    }

    Ok(result)
}

/// Minifies XML by removing unnecessary whitespace between tags.
///
/// Preserves whitespace inside text content that is not purely whitespace.
///
/// # Errors
///
/// Returns [`StringKnifeError::InvalidInput`] if the input is empty.
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn xml_minify(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "xml_minify".to_string(),
            reason: "empty XML input".to_string(),
        });
    }

    let tokens = tokenize_xml(trimmed);
    if tokens.is_empty() {
        return Err(StringKnifeError::InvalidInput {
            operation: "xml_minify".to_string(),
            reason: "no XML elements found".to_string(),
        });
    }

    let mut result = String::with_capacity(trimmed.len());
    for token in &tokens {
        match token {
            XmlToken::OpenTag(tag)
            | XmlToken::CloseTag(tag)
            | XmlToken::SelfClosing(tag)
            | XmlToken::Comment(tag)
            | XmlToken::CData(tag)
            | XmlToken::PI(tag) => {
                result.push_str(tag);
            }
            XmlToken::Text(text) => {
                let trimmed_text = text.trim();
                if !trimmed_text.is_empty() {
                    result.push_str(trimmed_text);
                }
            }
        }
    }

    Ok(result)
}

/// XML token types produced by the tokenizer.
#[derive(Debug)]
enum XmlToken {
    OpenTag(String),
    CloseTag(String),
    SelfClosing(String),
    Text(String),
    Comment(String),
    CData(String),
    PI(String),
}

/// Tokenize XML into a sequence of tokens.
fn tokenize_xml(input: &str) -> Vec<XmlToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '<' {
            // Start of a tag or special construct
            if starts_with_at(&chars, i, "<!--") {
                // Comment
                if let Some(end) = find_str(&chars, i, "-->") {
                    let comment: String = chars[i..end + 3].iter().collect();
                    tokens.push(XmlToken::Comment(comment));
                    i = end + 3;
                } else {
                    let rest: String = chars[i..].iter().collect();
                    tokens.push(XmlToken::Comment(rest));
                    break;
                }
            } else if starts_with_at(&chars, i, "<![CDATA[") {
                // CDATA section
                if let Some(end) = find_str(&chars, i, "]]>") {
                    let cdata: String = chars[i..end + 3].iter().collect();
                    tokens.push(XmlToken::CData(cdata));
                    i = end + 3;
                } else {
                    let rest: String = chars[i..].iter().collect();
                    tokens.push(XmlToken::CData(rest));
                    break;
                }
            } else if starts_with_at(&chars, i, "<?") {
                // Processing instruction
                if let Some(end) = find_str(&chars, i, "?>") {
                    let pi: String = chars[i..end + 2].iter().collect();
                    tokens.push(XmlToken::PI(pi));
                    i = end + 2;
                } else {
                    let rest: String = chars[i..].iter().collect();
                    tokens.push(XmlToken::PI(rest));
                    break;
                }
            } else {
                // Regular tag — find the closing '>'
                let tag_end = find_tag_end(&chars, i);
                let tag: String = chars[i..=tag_end].iter().collect();

                if tag.starts_with("</") {
                    tokens.push(XmlToken::CloseTag(tag));
                } else if tag.ends_with("/>") {
                    tokens.push(XmlToken::SelfClosing(tag));
                } else {
                    tokens.push(XmlToken::OpenTag(tag));
                }
                i = tag_end + 1;
            }
        } else {
            // Text content
            let start = i;
            while i < len && chars[i] != '<' {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            tokens.push(XmlToken::Text(text));
        }
    }

    tokens
}

/// Check if chars starting at `pos` match `needle`.
fn starts_with_at(chars: &[char], pos: usize, needle: &str) -> bool {
    let needle_chars: Vec<char> = needle.chars().collect();
    if pos + needle_chars.len() > chars.len() {
        return false;
    }
    chars[pos..pos + needle_chars.len()] == needle_chars
}

/// Find `needle` string in chars starting from `pos`, returning the start index.
fn find_str(chars: &[char], start: usize, needle: &str) -> Option<usize> {
    let needle_chars: Vec<char> = needle.chars().collect();
    let nlen = needle_chars.len();
    if nlen == 0 || start + nlen > chars.len() {
        return None;
    }
    for i in start..=chars.len() - nlen {
        if chars[i..i + nlen] == needle_chars {
            return Some(i);
        }
    }
    None
}

/// Find the end of a tag (the closing `>`) handling quoted attributes.
fn find_tag_end(chars: &[char], start: usize) -> usize {
    let mut i = start + 1;
    let mut in_quote: Option<char> = None;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = in_quote {
            if c == q {
                in_quote = None;
            }
        } else if c == '"' || c == '\'' {
            in_quote = Some(c);
        } else if c == '>' {
            return i;
        }
        i += 1;
    }
    // Unterminated tag — return end of input
    chars.len() - 1
}

fn push_indent(s: &mut String, level: usize) {
    for _ in 0..level {
        s.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === XML Pretty Print ===

    #[test]
    fn pretty_print_simple_element() {
        let result = xml_pretty_print("<root><child>text</child></root>").unwrap();
        assert!(result.contains("<root>"));
        assert!(result.contains("  <child>"));
        assert!(result.contains("text</child>"));
        assert!(result.contains("</root>"));
    }

    #[test]
    fn pretty_print_nested() {
        let result = xml_pretty_print("<a><b><c>val</c></b></a>").unwrap();
        assert!(result.contains("    <c>")); // 4 spaces = 2 levels
    }

    #[test]
    fn pretty_print_self_closing() {
        let result = xml_pretty_print("<root><br/><hr /></root>").unwrap();
        assert!(result.contains("  <br/>"));
        assert!(result.contains("  <hr />"));
    }

    #[test]
    fn pretty_print_attributes() {
        let input = r#"<div class="main"><span id="test">hi</span></div>"#;
        let result = xml_pretty_print(input).unwrap();
        assert!(result.contains(r#"<div class="main">"#));
        assert!(result.contains(r#"  <span id="test">"#));
    }

    #[test]
    fn pretty_print_comment() {
        let result = xml_pretty_print("<root><!-- comment --><child/></root>").unwrap();
        assert!(result.contains("  <!-- comment -->"));
    }

    #[test]
    fn pretty_print_cdata() {
        let result = xml_pretty_print("<root><![CDATA[some <data>]]></root>").unwrap();
        assert!(result.contains("  <![CDATA[some <data>]]>"));
    }

    #[test]
    fn pretty_print_processing_instruction() {
        let result = xml_pretty_print(r#"<?xml version="1.0"?><root/>"#).unwrap();
        assert!(result.contains("<?xml version=\"1.0\"?>"));
        assert!(result.contains("<root/>"));
    }

    #[test]
    fn pretty_print_already_pretty() {
        let input = "<root>\n  <child>text</child>\n</root>";
        let result = xml_pretty_print(input).unwrap();
        assert!(result.contains("<root>"));
        assert!(result.contains("  <child>text</child>"));
        assert!(result.contains("</root>"));
    }

    #[test]
    fn pretty_print_empty_input() {
        assert!(xml_pretty_print("").is_err());
    }

    #[test]
    fn pretty_print_whitespace_only() {
        assert!(xml_pretty_print("   ").is_err());
    }

    #[test]
    fn pretty_print_input_too_large() {
        let big = "<a>".repeat(MAX_INPUT_BYTES / 3 + 1);
        assert!(matches!(
            xml_pretty_print(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === XML Minify ===

    #[test]
    fn minify_simple() {
        let input = "<root>\n  <child>\n    text\n  </child>\n</root>";
        let result = xml_minify(input).unwrap();
        assert_eq!(result, "<root><child>text</child></root>");
    }

    #[test]
    fn minify_preserves_text_content() {
        let input = "<msg>hello world</msg>";
        let result = xml_minify(input).unwrap();
        assert_eq!(result, "<msg>hello world</msg>");
    }

    #[test]
    fn minify_with_attributes() {
        let input = "<div class=\"main\">\n  <span>hi</span>\n</div>";
        let result = xml_minify(input).unwrap();
        assert_eq!(result, r#"<div class="main"><span>hi</span></div>"#);
    }

    #[test]
    fn minify_self_closing() {
        let input = "<root>\n  <br />\n  <hr/>\n</root>";
        let result = xml_minify(input).unwrap();
        assert_eq!(result, "<root><br /><hr/></root>");
    }

    #[test]
    fn minify_already_minified() {
        let input = "<a><b>c</b></a>";
        assert_eq!(xml_minify(input).unwrap(), input);
    }

    #[test]
    fn minify_with_comment() {
        let input = "<root>\n  <!-- comment -->\n  <child/>\n</root>";
        let result = xml_minify(input).unwrap();
        assert_eq!(result, "<root><!-- comment --><child/></root>");
    }

    #[test]
    fn minify_empty_input() {
        assert!(xml_minify("").is_err());
    }

    #[test]
    fn minify_input_too_large() {
        let big = "<a>".repeat(MAX_INPUT_BYTES / 3 + 1);
        assert!(matches!(
            xml_minify(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_pretty_minify() {
        let input = "<root><a><b>text</b></a><c/></root>";
        let pretty = xml_pretty_print(input).unwrap();
        let minified = xml_minify(&pretty).unwrap();
        assert_eq!(minified, input);
    }

    // === Edge cases ===

    #[test]
    fn quoted_angle_bracket_in_attribute() {
        let input = r#"<div title="a>b">text</div>"#;
        let result = xml_pretty_print(input).unwrap();
        assert!(result.contains(r#"<div title="a>b">"#));
    }

    #[test]
    fn multiple_roots() {
        let input = "<a/><b/><c/>";
        let result = xml_pretty_print(input).unwrap();
        assert!(result.contains("<a/>"));
        assert!(result.contains("<b/>"));
        assert!(result.contains("<c/>"));
    }
}
