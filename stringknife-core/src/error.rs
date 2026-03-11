//! Structured error types for `StringKnife` transforms.

use std::fmt;

/// Error type for all `StringKnife` transform operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringKnifeError {
    /// The input is not valid for the requested operation.
    InvalidInput {
        /// The name of the operation that failed.
        operation: String,
        /// A human-readable explanation of why the input is invalid.
        reason: String,
    },
    /// The input uses an encoding that this operation does not support.
    UnsupportedEncoding {
        /// The name or description of the unsupported encoding.
        encoding: String,
    },
    /// The input exceeds the maximum allowed size.
    InputTooLarge {
        /// Maximum allowed size in bytes.
        max_bytes: usize,
        /// Actual size of the input in bytes.
        actual_bytes: usize,
    },
}

impl fmt::Display for StringKnifeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { operation, reason } => {
                write!(f, "{operation}: invalid input — {reason}")
            }
            Self::UnsupportedEncoding { encoding } => {
                write!(f, "unsupported encoding: {encoding}")
            }
            Self::InputTooLarge {
                max_bytes,
                actual_bytes,
            } => {
                write!(
                    f,
                    "input too large: {actual_bytes} bytes exceeds maximum of {max_bytes} bytes"
                )
            }
        }
    }
}

impl std::error::Error for StringKnifeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_input() {
        let err = StringKnifeError::InvalidInput {
            operation: "base64_decode".to_string(),
            reason: "contains non-base64 characters".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "base64_decode: invalid input — contains non-base64 characters"
        );
    }

    #[test]
    fn display_unsupported_encoding() {
        let err = StringKnifeError::UnsupportedEncoding {
            encoding: "Shift-JIS".to_string(),
        };
        assert_eq!(err.to_string(), "unsupported encoding: Shift-JIS");
    }

    #[test]
    fn display_input_too_large() {
        let err = StringKnifeError::InputTooLarge {
            max_bytes: 1_048_576,
            actual_bytes: 2_000_000,
        };
        assert_eq!(
            err.to_string(),
            "input too large: 2000000 bytes exceeds maximum of 1048576 bytes"
        );
    }
}
