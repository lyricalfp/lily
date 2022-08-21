use anyhow::bail;
use anyhow::Context;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::{
    lexer::types::{DigitK, IdentifierK, Token, TokenK},
    parser::errors::ParseError,
};

use super::cursor::Cursor;

#[derive(Debug)]
pub enum Associativity {
    Infixl,
    Infixr,
}

#[derive(Debug)]
pub struct Fixity {
    pub begin: usize,
    pub end: usize,
    pub associativity: Associativity,
    pub binding_power: u8,
    pub identifier: SmolStr,
}

pub type FixityMap = FxHashMap<SmolStr, Fixity>;

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn fixity(&mut self) -> anyhow::Result<(SmolStr, Fixity)> {
        let associativity = self.take()?;
        let begin = associativity.begin;
        let associativity = match associativity.kind {
            TokenK::Identifier(IdentifierK::Infixl) => Associativity::Infixl,
            TokenK::Identifier(IdentifierK::Infixr) => Associativity::Infixr,
            _ => bail!(ParseError::UnexpectedToken(associativity.kind)),
        };

        let binding_power = self.take()?;
        let binding_power = match binding_power.kind {
            TokenK::Digit(DigitK::Int) => self.source[binding_power.begin..binding_power.end]
                .parse::<u8>()
                .context("Malformed digit token")?,
            _ => bail!(ParseError::UnexpectedToken(binding_power.kind)),
        };

        let identifier = self.take()?;
        let identifier = match identifier.kind {
            TokenK::Identifier(IdentifierK::Lower) => {
                SmolStr::new(&self.source[identifier.begin..identifier.end])
            }
            _ => bail!(ParseError::UnexpectedToken(identifier.kind)),
        };

        let as_t = self.take()?;
        match as_t.kind {
            TokenK::Identifier(IdentifierK::As) => (),
            _ => bail!(ParseError::UnexpectedToken(as_t.kind)),
        };

        let operator = self.take()?;
        let end = operator.end;
        let operator = match operator.kind {
            TokenK::Operator(_) => SmolStr::new(&self.source[operator.begin..operator.end]),
            _ => bail!(ParseError::UnexpectedToken(operator.kind)),
        };

        Ok((
            operator,
            Fixity {
                begin,
                end,
                associativity,
                binding_power,
                identifier,
            },
        ))
    }
}
