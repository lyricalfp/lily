pub use self::{cursor::tokenize, layout::with_layout, types::Token};

pub mod cursor;
pub mod layout;
pub mod types;

pub fn lex(source: &str) -> Vec<Token> {
    with_layout(source, tokenize(source))
}
