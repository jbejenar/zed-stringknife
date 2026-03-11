//! String transformation modules.
//!
//! Each submodule contains pure functions with the signature:
//! `fn(&str) -> Result<String, StringKnifeError>`
//!
//! No I/O, no side effects, no LSP dependencies.

pub mod misc;
