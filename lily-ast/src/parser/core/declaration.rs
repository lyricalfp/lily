use anyhow::bail;
use smol_str::SmolStr;

use crate::{
    lexer::types::{IdentifierK, OperatorK, Token, TokenK},
    parser::{
        cursor::Cursor,
        errors::ParseError,
        types::{Declaration, DeclarationK, FixityMap},
    },
};

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn declaration_value(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Declaration> {
        let (declaration_begin, identifier) = {
            let Token { begin, end, .. } = self.take()?;
            (begin, SmolStr::new(&self.source[begin..end]))
        };

        let lesser_patterns = self.lesser_patterns()?;
        if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
            let _ = self.take()?;
            let (declaration_end, expression) = {
                let expression = self.expression(fixity_map)?;
                (expression.end, expression)
            };
            return Ok(Declaration {
                begin: declaration_begin,
                end: declaration_end,
                kind: DeclarationK::ValueDeclaration(identifier, lesser_patterns, expression),
            });
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }

    pub fn declaration(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Declaration> {
        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            return self.declaration_value(fixity_map);
        }
        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }
}
