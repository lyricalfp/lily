use anyhow::bail;
use lily_lexer::types::{IdentifierK, LayoutK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::Cursor,
    errors::ParseError,
    expect_token,
    types::{Declaration, DeclarationK},
};

impl<'a> Cursor<'a> {
    fn declaration_value(&mut self) -> anyhow::Result<Declaration> {
        let (declaration_begin, identifier) = {
            let Token { begin, end, .. } = self.take()?;
            (begin, SmolStr::new(&self.source[begin..end]))
        };

        let lesser_patterns = self.lesser_patterns()?;
        if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
            let _ = self.take()?;
            let (declaration_end, expression) = {
                let expression = self.expression()?;
                (expression.end, expression)
            };
            expect_token!(self, TokenK::Layout(LayoutK::Separator));
            return Ok(Declaration {
                begin: declaration_begin,
                end: declaration_end,
                kind: DeclarationK::ValueDeclaration(identifier, lesser_patterns, expression),
            });
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }

    pub fn declaration_let(&mut self) -> anyhow::Result<Declaration> {
        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            return self.declaration_value();
        }
        bail!(ParseError::UnexpectedToken(self.peek()?.kind))
    }

    pub fn declaration_let_block(&mut self) -> anyhow::Result<Vec<Declaration>> {
        let mut declarations = vec![self.declaration_let()?];
        loop {
            if let TokenK::Layout(LayoutK::End) = self.peek()?.kind {
                break;
            }
            declarations.push(self.declaration_let()?);
        }
        Ok(declarations)
    }

    pub fn declaration(&mut self) -> anyhow::Result<Declaration> {
        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            return self.declaration_value();
        }
        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }
}
