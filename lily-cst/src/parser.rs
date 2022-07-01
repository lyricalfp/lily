use std::iter::Peekable;

use crate::lexer::{Lexer, TokenKind, TokenSpan};

#[derive(Debug)]
pub struct LowerName<'a>(TokenSpan, &'a str);

#[derive(Debug)]
pub struct UpperName<'a>(TokenSpan, &'a str);

#[derive(Debug)]
pub struct SyntaxToken<'a>(TokenSpan, &'a str);

#[derive(Debug)]
pub enum Expression {}

#[derive(Debug)]
pub enum Type {}

#[derive(Debug)]
pub enum TypeRhs {
    Deferred(Vec<TokenSpan>),
    Finished(Type),
}

#[derive(Debug)]
pub enum ValueRhs {
    Deferred(Vec<TokenSpan>),
    Finished(Expression),
}

#[derive(Debug)]
pub enum Declaration<'a> {
    ValueType(LowerName<'a>, SyntaxToken<'a>, TypeRhs, SyntaxToken<'a>),
    ValueExpr(LowerName<'a>, SyntaxToken<'a>, ValueRhs, SyntaxToken<'a>),
}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokens = Lexer::new(source).peekable();
        Self { source, tokens }
    }
}

impl<'a> Parser<'a> {
    pub fn skip_whitespace(&mut self) {
        while let Some(TokenSpan {
            kind: TokenKind::Whitespace,
            ..
        }) = self.tokens.peek()
        {
            self.tokens.next();
        }
    }

    pub fn lower_ident(&mut self) -> Option<LowerName<'a>> {
        let token = self.tokens.next()?;
        if matches!(token.kind, TokenKind::LowerIdentifier) {
            Some(LowerName(token, &self.source[token.begin..token.end]))
        } else {
            None
        }
    }

    pub fn upper_ident(&mut self) -> Option<UpperName<'a>> {
        let token = self.tokens.next()?;
        if matches!(token.kind, TokenKind::UpperIdentifier) {
            Some(UpperName(token, &self.source[token.begin..token.end]))
        } else {
            None
        }
    }

    pub fn declaration(&mut self) -> Option<Declaration<'a>> {
        let lower_ident = self.lower_ident()?;
        self.skip_whitespace();
        match self.tokens.peek()?.kind {
            TokenKind::Colon => {
                let colon_span = self.tokens.next()?;
                let colon_token =
                    SyntaxToken(colon_span, &self.source[colon_span.begin..colon_span.end]);
                self.skip_whitespace();
                let mut deferred_spans = vec![];
                loop {
                    if let TokenSpan {
                        kind: TokenKind::Semicolon,
                        ..
                    } = self.tokens.peek()?
                    {
                        break;
                    } else {
                        deferred_spans.push(self.tokens.next()?);
                    }
                }
                let deferred_tokens = TypeRhs::Deferred(deferred_spans);
                let semicolon_span = self.tokens.next()?;
                let semicolon_token = SyntaxToken(
                    semicolon_span,
                    &self.source[semicolon_span.begin..semicolon_span.end],
                );
                Some(Declaration::ValueType(
                    lower_ident,
                    colon_token,
                    deferred_tokens,
                    semicolon_token,
                ))
            }
            TokenKind::Equal => {
                let equal_span = self.tokens.next()?;
                let equal_token =
                    SyntaxToken(equal_span, &self.source[equal_span.begin..equal_span.end]);
                self.skip_whitespace();
                let mut deferred_spans = vec![];
                loop {
                    if let TokenSpan {
                        kind: TokenKind::Semicolon,
                        ..
                    } = self.tokens.peek()?
                    {
                        break;
                    } else {
                        deferred_spans.push(self.tokens.next()?);
                    }
                }
                let deferred_tokens = TypeRhs::Deferred(deferred_spans);
                let semicolon_span = self.tokens.next()?;
                let semicolon_token = SyntaxToken(
                    semicolon_span,
                    &self.source[semicolon_span.begin..semicolon_span.end],
                );
                Some(Declaration::ValueType(
                    lower_ident,
                    equal_token,
                    deferred_tokens,
                    semicolon_token,
                ))
            }
            _ => None,
        }
    }
}

#[test]
fn it_works() {
    let source = "main : Effect Unit;";
    let mut parser = Parser::new(source);
    dbg!(parser.declaration());
}
