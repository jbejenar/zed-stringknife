//! Case conversion transforms.
//!
//! Converts text between 13 naming conventions: `UPPER`, `lower`, `Title`,
//! `Sentence`, `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE`,
//! `kebab-case`, `dot.case`, `path/case`, `CONSTANT_CASE`, and toggle case.
//!
//! The core algorithm splits input into words by detecting boundaries at:
//! - Whitespace, underscores, hyphens, dots, slashes
//! - `camelCase` transitions (lowercase → uppercase)
//! - Acronym boundaries (`HTTPSConn` → `HTTPS`, `Conn`)
//! - Number boundaries (`var2name` → `var`, `2`, `name`)

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Split input into words by detecting case boundaries, separators, and numbers.
fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        let c = chars[i];

        // Separators: whitespace, underscore, hyphen, dot, slash
        if c.is_whitespace() || c == '_' || c == '-' || c == '.' || c == '/' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            i += 1;
            continue;
        }

        // Number boundary: transition between digit and non-digit
        if !current.is_empty() {
            let prev_is_digit = current.chars().last().is_some_and(|p| p.is_ascii_digit());
            let curr_is_digit = c.is_ascii_digit();
            if prev_is_digit != curr_is_digit {
                words.push(current.clone());
                current.clear();
            }
        }

        // Case boundary detection
        if c.is_uppercase() && !current.is_empty() {
            let prev = current.chars().last().unwrap_or(' ');
            if prev.is_lowercase() || prev.is_ascii_digit() {
                // camelCase boundary: "camelC" → "camel", "C"
                words.push(current.clone());
                current.clear();
            } else if prev.is_uppercase() {
                // Check for acronym end: "HTTPSConn" → at 'C' (after S),
                // look ahead to see if next char is lowercase
                if i + 1 < len && chars[i + 1].is_lowercase() {
                    // This uppercase starts a new word after an acronym
                    words.push(current.clone());
                    current.clear();
                }
                // Otherwise continue the acronym run
            }
        }

        current.push(c);
        i += 1;
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Converts text to `UPPERCASE`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_upper(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input.to_uppercase())
}

/// Converts text to `lowercase`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_lower(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input.to_lowercase())
}

/// Converts text to `Title Case` (capitalize first letter of each word).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_title_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words
        .iter()
        .map(|w| capitalize(w))
        .collect::<Vec<_>>()
        .join(" "))
}

/// Converts text to `Sentence case` (capitalize first letter only).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_sentence_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    if words.is_empty() {
        return Ok(String::new());
    }
    let mut parts: Vec<String> = Vec::with_capacity(words.len());
    parts.push(capitalize(&words[0]));
    for w in &words[1..] {
        parts.push(w.to_lowercase());
    }
    Ok(parts.join(" "))
}

/// Converts text to `camelCase`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_camel_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    if words.is_empty() {
        return Ok(String::new());
    }
    let mut result = words[0].to_lowercase();
    for w in &words[1..] {
        result.push_str(&capitalize(w));
    }
    Ok(result)
}

/// Converts text to `PascalCase`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_pascal_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words.iter().map(|w| capitalize(w)).collect::<String>())
}

/// Converts text to `snake_case`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_snake_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_"))
}

/// Converts text to `SCREAMING_SNAKE_CASE`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_screaming_snake_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words
        .iter()
        .map(|w| w.to_uppercase())
        .collect::<Vec<_>>()
        .join("_"))
}

/// Converts text to `kebab-case`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_kebab_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-"))
}

/// Converts text to `dot.case`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_dot_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("."))
}

/// Converts text to `path/case`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_path_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let words = split_words(input);
    Ok(words
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("/"))
}

/// Alias for [`to_screaming_snake_case`].
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn to_constant_case(input: &str) -> Result<String, StringKnifeError> {
    to_screaming_snake_case(input)
}

/// Toggles the case of each character (upper ↔ lower).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn toggle_case(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().collect::<String>()
            } else if c.is_lowercase() {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect())
}

/// Capitalize first letter, lowercase the rest.
fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            let lower: String = chars.collect::<String>().to_lowercase();
            format!("{upper}{lower}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Word splitting ===

    #[test]
    fn split_camel_case() {
        assert_eq!(split_words("camelCase"), vec!["camel", "Case"]);
    }

    #[test]
    fn split_pascal_case() {
        assert_eq!(split_words("PascalCase"), vec!["Pascal", "Case"]);
    }

    #[test]
    fn split_snake_case() {
        assert_eq!(split_words("snake_case"), vec!["snake", "case"]);
    }

    #[test]
    fn split_kebab_case() {
        assert_eq!(split_words("kebab-case"), vec!["kebab", "case"]);
    }

    #[test]
    fn split_acronym() {
        assert_eq!(split_words("HTTPSConnection"), vec!["HTTPS", "Connection"]);
    }

    #[test]
    fn split_numbers() {
        assert_eq!(split_words("myVar2Name"), vec!["my", "Var", "2", "Name"]);
    }

    #[test]
    fn split_mixed_separators() {
        assert_eq!(
            split_words("hello world_foo-bar"),
            vec!["hello", "world", "foo", "bar"]
        );
    }

    // === UPPERCASE ===

    #[test]
    fn upper_simple() {
        assert_eq!(to_upper("hello world").unwrap(), "HELLO WORLD");
    }

    #[test]
    fn upper_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(matches!(
            to_upper(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }

    // === lowercase ===

    #[test]
    fn lower_simple() {
        assert_eq!(to_lower("HELLO WORLD").unwrap(), "hello world");
    }

    // === Title Case ===

    #[test]
    fn title_from_snake() {
        assert_eq!(to_title_case("hello_world").unwrap(), "Hello World");
    }

    #[test]
    fn title_from_camel() {
        assert_eq!(to_title_case("camelCase").unwrap(), "Camel Case");
    }

    // === Sentence Case ===

    #[test]
    fn sentence_simple() {
        assert_eq!(
            to_sentence_case("hello_world_foo").unwrap(),
            "Hello world foo"
        );
    }

    // === camelCase ===

    #[test]
    fn camel_from_snake() {
        assert_eq!(to_camel_case("hello_world").unwrap(), "helloWorld");
    }

    #[test]
    fn camel_from_pascal() {
        assert_eq!(to_camel_case("PascalCase").unwrap(), "pascalCase");
    }

    #[test]
    fn camel_from_kebab() {
        assert_eq!(to_camel_case("kebab-case-name").unwrap(), "kebabCaseName");
    }

    // === PascalCase ===

    #[test]
    fn pascal_from_snake() {
        assert_eq!(to_pascal_case("hello_world").unwrap(), "HelloWorld");
    }

    #[test]
    fn pascal_from_camel() {
        assert_eq!(to_pascal_case("camelCase").unwrap(), "CamelCase");
    }

    // === snake_case ===

    #[test]
    fn snake_from_camel() {
        assert_eq!(to_snake_case("camelCase").unwrap(), "camel_case");
    }

    #[test]
    fn snake_from_pascal() {
        assert_eq!(to_snake_case("PascalCase").unwrap(), "pascal_case");
    }

    #[test]
    fn snake_from_screaming() {
        assert_eq!(to_snake_case("SCREAMING_SNAKE").unwrap(), "screaming_snake");
    }

    #[test]
    fn snake_acronym() {
        assert_eq!(
            to_snake_case("HTTPSConnection").unwrap(),
            "https_connection"
        );
    }

    #[test]
    fn snake_with_numbers() {
        assert_eq!(to_snake_case("myVar2Name").unwrap(), "my_var_2_name");
    }

    // === SCREAMING_SNAKE_CASE ===

    #[test]
    fn screaming_from_camel() {
        assert_eq!(to_screaming_snake_case("camelCase").unwrap(), "CAMEL_CASE");
    }

    // === kebab-case ===

    #[test]
    fn kebab_from_camel() {
        assert_eq!(to_kebab_case("camelCase").unwrap(), "camel-case");
    }

    #[test]
    fn kebab_from_snake() {
        assert_eq!(to_kebab_case("snake_case").unwrap(), "snake-case");
    }

    // === dot.case ===

    #[test]
    fn dot_from_camel() {
        assert_eq!(to_dot_case("camelCase").unwrap(), "camel.case");
    }

    // === path/case ===

    #[test]
    fn path_from_camel() {
        assert_eq!(to_path_case("camelCase").unwrap(), "camel/case");
    }

    // === CONSTANT_CASE ===

    #[test]
    fn constant_is_screaming_snake() {
        assert_eq!(
            to_constant_case("myVariable").unwrap(),
            to_screaming_snake_case("myVariable").unwrap()
        );
    }

    // === Toggle Case ===

    #[test]
    fn toggle_simple() {
        assert_eq!(toggle_case("Hello World").unwrap(), "hELLO wORLD");
    }

    #[test]
    fn toggle_numbers_unchanged() {
        assert_eq!(toggle_case("abc123DEF").unwrap(), "ABC123def");
    }

    // === Empty input ===

    #[test]
    fn empty_input() {
        assert_eq!(to_snake_case("").unwrap(), "");
        assert_eq!(to_camel_case("").unwrap(), "");
        assert_eq!(to_pascal_case("").unwrap(), "");
        assert_eq!(toggle_case("").unwrap(), "");
    }

    // === Single word ===

    #[test]
    fn single_word() {
        assert_eq!(to_snake_case("hello").unwrap(), "hello");
        assert_eq!(to_camel_case("hello").unwrap(), "hello");
        assert_eq!(to_pascal_case("hello").unwrap(), "Hello");
        assert_eq!(to_kebab_case("hello").unwrap(), "hello");
    }

    // === Roundtrip ===

    #[test]
    fn roundtrip_snake_to_camel_to_snake() {
        let original = "my_variable_name";
        let camel = to_camel_case(original).unwrap();
        assert_eq!(camel, "myVariableName");
        let back = to_snake_case(&camel).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_pascal_to_kebab_to_pascal() {
        let original = "MyVariableName";
        let kebab = to_kebab_case(original).unwrap();
        assert_eq!(kebab, "my-variable-name");
        let back = to_pascal_case(&kebab).unwrap();
        assert_eq!(back, original);
    }
}
