use std::iter::Peekable;

use anyhow::Context;
use lily_lexer::types::Token;

use crate::errors::ParseError;

pub struct Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub source: &'a str,
    tokens: Peekable<I>,
}

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(source: &'a str, tokens: I) -> Self {
        Self {
            source,
            tokens: tokens.peekable(),
        }
    }

    pub fn peek(&mut self) -> anyhow::Result<&Token> {
        self.tokens.peek().context(ParseError::UnexpectedEndOfFile)
    }

    pub fn take(&mut self) -> anyhow::Result<Token> {
        self.tokens.next().context(ParseError::UnexpectedEndOfFile)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! expect_token {
    ($self:ident, $kind:pat) => {{
        let token = $self.take()?;
        if matches!(token.kind, $kind) {
            token
        } else {
            anyhow::bail!(ParseError::UnexpectedToken(token.kind));
        }
    }};
}

pub use expect_token;
