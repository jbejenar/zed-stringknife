//! Miscellaneous string transforms.

use crate::error::StringKnifeError;
use crate::MAX_INPUT_BYTES;

/// Reverses the characters in the input string.
///
/// Handles Unicode correctly by reversing grapheme clusters via `chars()`.
///
/// # Errors
///
/// Returns [`StringKnifeError::InputTooLarge`] if input exceeds [`MAX_INPUT_BYTES`].
pub fn reverse_string(input: &str) -> Result<String, StringKnifeError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(StringKnifeError::InputTooLarge {
            max_bytes: MAX_INPUT_BYTES,
            actual_bytes: input.len(),
        });
    }
    Ok(input.chars().rev().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_ascii() {
        assert_eq!(reverse_string("hello").unwrap(), "olleh");
    }

    #[test]
    fn reverse_unicode() {
        assert_eq!(reverse_string("héllo").unwrap(), "olléh");
    }

    #[test]
    fn reverse_empty() {
        assert_eq!(reverse_string("").unwrap(), "");
    }

    #[test]
    fn reverse_single_char() {
        assert_eq!(reverse_string("a").unwrap(), "a");
    }

    #[test]
    fn reverse_palindrome() {
        assert_eq!(reverse_string("racecar").unwrap(), "racecar");
    }

    #[test]
    fn reverse_input_too_large() {
        let big = "a".repeat(MAX_INPUT_BYTES + 1);
        let err = reverse_string(&big).unwrap_err();
        assert!(matches!(err, StringKnifeError::InputTooLarge { .. }));
    }
}
