use std::iter::Peekable;

use anyhow::Context;

use crate::lexer::types::Token;

use super::errors::ParseError;

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

    // pub fn peek(&mut self) -> anyhow::Result<&Token> {
    //     self.tokens.peek().context(ParseError::UnexpectedEndOfFile)
    // }

    pub fn take(&mut self) -> anyhow::Result<Token> {
        self.tokens.next().context(ParseError::UnexpectedEndOfFile)
    }
}

#[macro_export]
macro_rules! expect {
    ($self:ident, $kind:pat) => {
        {
            let token = $self.take()?;
            if matches!(token.kind, $kind) {
                token
            } else {
                bail!(ParseError::UnexpectedToken(token.kind));
            }
        }
    }
}
