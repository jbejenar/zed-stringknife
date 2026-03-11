//! Shared helpers used across transform modules.

use crate::error::StringKnifeError;
use crate::MAX_INPUT_BYTES;

/// Check that input does not exceed the maximum allowed size.
pub fn check_size(input: &str) -> Result<(), StringKnifeError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(StringKnifeError::InputTooLarge {
            max_bytes: MAX_INPUT_BYTES,
            actual_bytes: input.len(),
        });
    }
    Ok(())
}
