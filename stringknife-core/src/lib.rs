//! `StringKnife` Core — Pure string transformation engine.
//!
//! Every transform is a pure function:
//! `fn(&str) -> Result<String, StringKnifeError>`
//!
//! No I/O, no side effects, no shared state, no LSP dependencies.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod transforms;

pub use error::StringKnifeError;

/// Maximum input size in bytes (1 MB).
pub const MAX_INPUT_BYTES: usize = 1_048_576;
