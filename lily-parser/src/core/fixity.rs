use anyhow::Context;
use lily_lexer::types::{DigitK, IdentifierK, LayoutK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{Associativity, Domain, Fixity},
};

impl<'a> Cursor<'a> {
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

        let (domain, identifier) = if let TokenK::Identifier(IdentifierK::Type) = self.peek()?.kind
        {
            self.take()?;
            let Token { begin, end, .. } =
                expect_token!(self, TokenK::Identifier(IdentifierK::Upper));
            let identifier = SmolStr::new(&self.source[begin..end]);
            (Domain::Type, identifier)
        } else {
            let Token { begin, end, .. } =
                expect_token!(self, TokenK::Identifier(IdentifierK::Lower));
            let identifier = SmolStr::new(&self.source[begin..end]);
            (Domain::Value, identifier)
        };

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
                domain,
                binding_power,
                identifier,
            },
        ))
    }
}
