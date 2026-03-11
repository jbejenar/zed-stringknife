//! String transformation modules.
//!
//! Each submodule contains pure functions with the signature:
//! `fn(&str) -> Result<String, StringKnifeError>`
//!
//! No I/O, no side effects, no LSP dependencies.

pub mod base64;
pub mod case;
pub(crate) mod common;
pub mod csv;
pub mod hash;
pub mod hex;
pub mod html;
pub mod json;
pub mod jwt;
pub mod misc;
pub mod unicode;
pub mod url;
pub mod whitespace;
pub mod xml;
