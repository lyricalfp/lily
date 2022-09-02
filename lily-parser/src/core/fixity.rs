use anyhow::Context;
use lily_lexer::types::{DigitK, IdentifierK, LayoutK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{Associativity, Fixity},
};

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn fixity(&mut self) -> anyhow::Result<(SmolStr, Fixity)> {
        let Token {
            begin: fixity_begin,
            kind,
            ..
        } = expect_token!(
            self,
            TokenK::Identifier(IdentifierK::Infixl | IdentifierK::Infixr)
        );
        let associativity = match kind {
            TokenK::Identifier(IdentifierK::Infixl) => Associativity::Infixl,
            TokenK::Identifier(IdentifierK::Infixr) => Associativity::Infixr,
            _ => unreachable!(),
        };

        let Token { begin, end, .. } = expect_token!(self, TokenK::Digit(DigitK::Int));
        let binding_power = self.source[begin..end]
            .parse()
            .context(ParseError::InternalError(
                "Malformed digit token.".to_string(),
            ))?;

        let Token { begin, end, .. } = expect_token!(self, TokenK::Identifier(IdentifierK::Lower));
        let identifier = SmolStr::new(&self.source[begin..end]);

        expect_token!(self, TokenK::Identifier(IdentifierK::As));

        let Token {
            begin,
            end: fixity_end,
            ..
        } = expect_token!(self, TokenK::Operator(_));
        let operator = SmolStr::new(&self.source[begin..fixity_end]);

        expect_token!(self, TokenK::Layout(LayoutK::Separator));

        Ok((
            operator,
            Fixity {
                begin: fixity_begin,
                end: fixity_end,
                associativity,
                binding_power,
                identifier,
            },
        ))
    }
}
