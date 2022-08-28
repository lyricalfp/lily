use thiserror::Error;

use crate::lexer::types::TokenK;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected end of file.")]
    UnexpectedEndOfFile,
    #[error("Unexpected token {0:?}.")]
    UnexpectedToken(TokenK),
    #[error("Internal error: {0}. This incident should be reported!")]
    InternalError(String),
}
