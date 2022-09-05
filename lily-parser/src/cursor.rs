use anyhow::{bail, Context};
use lily_lexer::types::Token;
use smol_str::SmolStr;

use crate::{
    errors::ParseError,
    types::FixityMap,
};

pub struct Cursor<'a> {
    pub source: &'a str,
    tokens: &'a [Token],
    index: usize,
    fixity_map: Option<&'a FixityMap>,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str, tokens: &'a [Token], fixity_map: Option<&'a FixityMap>) -> Self {
        Self {
            source,
            tokens,
            index: 0,
            fixity_map,
        }
    }

    pub fn peek(&mut self) -> anyhow::Result<&Token> {
        if self.is_eof() {
            bail!(ParseError::UnexpectedEndOfFile);
        }
        Ok(&self.tokens[self.index])
    }

    pub fn take(&mut self) -> anyhow::Result<Token> {
        if self.is_eof() {
            bail!(ParseError::UnexpectedEndOfFile);
        }
        let token = self.tokens[self.index];
        self.index += 1;
        Ok(token)
    }

    pub fn is_eof(&mut self) -> bool {
        self.index == self.tokens.len()
    }

    pub fn attempt<T>(
        &mut self,
        callback: impl FnOnce(&mut Self) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        let index = self.index;
        match callback(self) {
            Ok(ok) => Ok(ok),
            Err(err) => {
                self.index = index;
                Err(err)
            }
        }
    }

    pub fn get_fixity(&self, operator: &SmolStr) -> anyhow::Result<(u8, u8)> {
        Ok(self
            .fixity_map
            .context(ParseError::UnknownBindingPower(operator.clone()))?
            .get(operator)
            .context(ParseError::UnknownBindingPower(operator.clone()))?
            .as_pair())
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
