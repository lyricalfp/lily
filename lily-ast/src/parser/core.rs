use anyhow::bail;
use anyhow::Context;
use smol_str::SmolStr;

use crate::{
    expect,
    lexer::types::{DigitK, IdentifierK, OperatorK, Token, TokenK},
    parser::{
        cursor::Cursor,
        errors::ParseError,
        fixity::{Associativity, Fixity, FixityMap},
    },
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
        } = expect!(
            self,
            TokenK::Identifier(IdentifierK::Infixl | IdentifierK::Infixr)
        );
        let associativity = match kind {
            TokenK::Identifier(IdentifierK::Infixl) => Associativity::Infixl,
            TokenK::Identifier(IdentifierK::Infixr) => Associativity::Infixr,
            _ => unreachable!(),
        };

        let Token { begin, end, .. } = expect!(self, TokenK::Digit(DigitK::Int));
        let binding_power = self.source[begin..end]
            .parse()
            .context(ParseError::InternalError(
                "Malformed digit token.".to_string(),
            ))?;

        let Token { begin, end, .. } = expect!(self, TokenK::Identifier(IdentifierK::Lower));
        let identifier = SmolStr::new(&self.source[begin..end]);

        expect!(self, TokenK::Identifier(IdentifierK::As));

        let Token {
            begin,
            end: fixity_end,
            ..
        } = expect!(self, TokenK::Operator(_));
        let operator = SmolStr::new(&self.source[begin..fixity_end]);

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

    pub fn declaration(&mut self, _: &mut FixityMap) -> anyhow::Result<()> {
        let Token { kind, .. } = self.take()?;

        if let TokenK::Identifier(IdentifierK::Lower) = kind {
            let Token { kind, .. } = self.take()?;

            if let TokenK::Operator(OperatorK::Colon) = kind {
                return Ok(());
            }

            if let TokenK::Operator(OperatorK::Equal) = kind {
                return Ok(());
            }

            bail!(ParseError::UnexpectedToken(kind));
        }

        if let TokenK::Identifier(IdentifierK::Upper) = kind {
            return Ok(());
        }

        bail!(ParseError::UnexpectedToken(kind));
    }
}
