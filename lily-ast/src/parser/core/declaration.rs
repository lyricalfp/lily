use anyhow::bail;

use crate::{
    lexer::types::{IdentifierK, OperatorK, Token, TokenK},
    parser::{cursor::Cursor, errors::ParseError, fixity::FixityMap},
};

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn declaration(&mut self, _: &FixityMap) -> anyhow::Result<()> {
        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            self.take()?;

            if let TokenK::Operator(OperatorK::Colon) = self.peek()?.kind {
                self.take()?;
                return Ok(());
            }

            let _ = self.lesser_patterns()?;

            if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
                self.take()?;
                return Ok(());
            }

            bail!(ParseError::UnexpectedToken(self.peek()?.kind));
        }

        if let TokenK::Identifier(IdentifierK::Upper) = self.peek()?.kind {
            self.take()?;
            return Ok(());
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }
}
