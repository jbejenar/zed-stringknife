//! Whitespace and line manipulation transforms.
//!
//! Trim, collapse, sort, deduplicate, reverse, shuffle, and number lines.
//! Pure text operations — no I/O, no external dependencies.

use std::fmt::Write;

use super::common::check_size;
use crate::error::StringKnifeError;

#[cfg(test)]
use crate::MAX_INPUT_BYTES;

/// Trims leading and trailing whitespace from each line.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn trim_whitespace(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input.lines().map(str::trim).collect::<Vec<_>>().join("\n"))
}

/// Trims leading whitespace from each line.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn trim_leading(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input
        .lines()
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Trims trailing whitespace from each line.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn trim_trailing(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Collapses multiple consecutive whitespace characters into a single space.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn collapse_whitespace(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut result = String::with_capacity(input.len());
    let mut prev_was_ws = false;
    for c in input.chars() {
        if c.is_whitespace() {
            if !prev_was_ws {
                result.push(' ');
                prev_was_ws = true;
            }
        } else {
            result.push(c);
            prev_was_ws = false;
        }
    }
    Ok(result)
}

/// Removes blank lines (lines that are empty or contain only whitespace).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn remove_blank_lines(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    Ok(input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Removes duplicate lines, preserving the first occurrence and original order.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn remove_duplicate_lines(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut seen = Vec::new();
    let mut result = Vec::new();
    for line in input.lines() {
        if !seen.contains(&line) {
            seen.push(line);
            result.push(line);
        }
    }
    Ok(result.join("\n"))
}

/// Sorts lines alphabetically (A → Z, case-sensitive).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn sort_lines_asc(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut lines: Vec<&str> = input.lines().collect();
    lines.sort_unstable();
    Ok(lines.join("\n"))
}

/// Sorts lines in reverse alphabetical order (Z → A, case-sensitive).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn sort_lines_desc(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut lines: Vec<&str> = input.lines().collect();
    lines.sort_unstable();
    lines.reverse();
    Ok(lines.join("\n"))
}

/// Sorts lines by length (shortest first).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn sort_lines_by_length(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut lines: Vec<&str> = input.lines().collect();
    lines.sort_by_key(|line| line.len());
    Ok(lines.join("\n"))
}

/// Reverses the order of lines (not characters within lines).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn reverse_lines(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut lines: Vec<&str> = input.lines().collect();
    lines.reverse();
    Ok(lines.join("\n"))
}

/// Shuffles lines into a deterministic pseudo-random order.
///
/// Uses a simple hash-based shuffle (no external RNG crate) for reproducible
/// results within the same input. Not cryptographically random.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn shuffle_lines(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let mut lines: Vec<(usize, &str)> = input.lines().enumerate().collect();
    // Simple deterministic shuffle using a hash of content + index
    lines.sort_by_key(|(idx, line)| {
        let mut hash: u64 = 0x517c_c1b7_2722_0a95;
        for b in line.bytes() {
            hash = hash
                .wrapping_mul(0x0100_0000_01b3)
                .wrapping_add(u64::from(b));
        }
        hash = hash
            .wrapping_mul(0x0100_0000_01b3)
            .wrapping_add(*idx as u64);
        hash
    });
    Ok(lines
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Prefixes each line with its line number.
///
/// Format: `  1: line content` (right-aligned numbers, width adapts to total lines).
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds the size limit.
pub fn number_lines(input: &str) -> Result<String, StringKnifeError> {
    check_size(input)?;
    let lines: Vec<&str> = input.lines().collect();
    let width = lines.len().to_string().len();
    let mut result = String::with_capacity(input.len() + lines.len() * (width + 2));
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let _ = write!(result, "{:>width$}: {line}", i + 1);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Trim ===

    #[test]
    fn trim_both() {
        assert_eq!(
            trim_whitespace("  hello  \n  world  ").unwrap(),
            "hello\nworld"
        );
    }

    #[test]
    fn trim_leading_ws() {
        assert_eq!(trim_leading("  hello\n  world").unwrap(), "hello\nworld");
    }

    #[test]
    fn trim_trailing_ws() {
        assert_eq!(trim_trailing("hello  \nworld  ").unwrap(), "hello\nworld");
    }

    // === Collapse ===

    #[test]
    fn collapse_spaces() {
        assert_eq!(collapse_whitespace("hello   world").unwrap(), "hello world");
    }

    #[test]
    fn collapse_tabs_and_newlines() {
        assert_eq!(
            collapse_whitespace("hello\t\t\nworld").unwrap(),
            "hello world"
        );
    }

    // === Remove blank lines ===

    #[test]
    fn remove_blanks() {
        assert_eq!(remove_blank_lines("a\n\nb\n  \nc").unwrap(), "a\nb\nc");
    }

    #[test]
    fn remove_blanks_no_blanks() {
        assert_eq!(remove_blank_lines("a\nb\nc").unwrap(), "a\nb\nc");
    }

    // === Remove duplicates ===

    #[test]
    fn remove_dupes() {
        assert_eq!(remove_duplicate_lines("a\nb\na\nc\nb").unwrap(), "a\nb\nc");
    }

    #[test]
    fn remove_dupes_preserves_order() {
        assert_eq!(remove_duplicate_lines("c\nb\na\nc").unwrap(), "c\nb\na");
    }

    // === Sort ===

    #[test]
    fn sort_asc() {
        assert_eq!(sort_lines_asc("c\na\nb").unwrap(), "a\nb\nc");
    }

    #[test]
    fn sort_desc() {
        assert_eq!(sort_lines_desc("a\nc\nb").unwrap(), "c\nb\na");
    }

    #[test]
    fn sort_by_length() {
        assert_eq!(
            sort_lines_by_length("medium\na\nlong line").unwrap(),
            "a\nmedium\nlong line"
        );
    }

    // === Reverse lines ===

    #[test]
    fn reverse() {
        assert_eq!(reverse_lines("a\nb\nc").unwrap(), "c\nb\na");
    }

    // === Shuffle ===

    #[test]
    fn shuffle_deterministic() {
        let input = "a\nb\nc\nd\ne";
        let r1 = shuffle_lines(input).unwrap();
        let r2 = shuffle_lines(input).unwrap();
        assert_eq!(r1, r2); // deterministic
    }

    #[test]
    fn shuffle_preserves_lines() {
        let input = "x\ny\nz";
        let result = shuffle_lines(input).unwrap();
        let mut sorted: Vec<&str> = result.lines().collect();
        sorted.sort_unstable();
        assert_eq!(sorted, vec!["x", "y", "z"]);
    }

    // === Number lines ===

    #[test]
    fn number_simple() {
        let result = number_lines("a\nb\nc").unwrap();
        assert!(result.contains("1: a"));
        assert!(result.contains("2: b"));
        assert!(result.contains("3: c"));
    }

    #[test]
    fn number_alignment() {
        // With 10+ lines, numbers should be right-aligned
        let input = (1..=12)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let result = number_lines(&input).unwrap();
        assert!(result.contains(" 1: line1"));
        assert!(result.contains("12: line12"));
    }

    // === Edge cases ===

    #[test]
    fn empty_input() {
        assert_eq!(trim_whitespace("").unwrap(), "");
        assert_eq!(sort_lines_asc("").unwrap(), "");
        assert_eq!(number_lines("").unwrap(), "");
    }

    #[test]
    fn single_line() {
        assert_eq!(reverse_lines("hello").unwrap(), "hello");
        assert_eq!(sort_lines_asc("hello").unwrap(), "hello");
    }

    #[test]
    fn input_too_large() {
        let big = "a\n".repeat(MAX_INPUT_BYTES / 2 + 1);
        assert!(matches!(
            trim_whitespace(&big).unwrap_err(),
            StringKnifeError::InputTooLarge { .. }
        ));
    }
}
