use lily_lexer::types::TokenK;
use smol_str::SmolStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected end of file.")]
    UnexpectedEndOfFile,
    #[error("Unexpected token {0:?}.")]
    UnexpectedToken(TokenK),
    #[error("Unknown binding power for operator {0:?}.")]
    UnknownBindingPower(SmolStr),
    #[error("Internal error: {0}. This incident should be reported!")]
    InternalError(String),
}
