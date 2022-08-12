//! Top-level API for the lexer.
//!
//! # Basic Usage
//!
//! ```rust
//! use lily_ast::lexer::lex;
//!
//! let tokens = lex("a = 0\nb = 0");
//!
//! assert_eq!(tokens.len(), 9);
//! ```
pub use self::{cursor::tokenize, layout::with_layout, types::Token};

pub mod cursor;
pub mod layout;
pub mod types;

/// Creates a stream of tokens from a source file.
pub fn lex(source: &str) -> Vec<Token> {
    with_layout(source, tokenize(source))
}
