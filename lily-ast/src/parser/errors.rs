use thiserror::Error;

use crate::lexer::types::TokenK;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected end of file.")]
    UnexpectedEndOfFile,
    #[error("Unexpected token {0:?}.")]
    UnexpectedToken(TokenK),
}
