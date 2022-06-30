use std::iter::Peekable;

use crate::lexer::{TokenKind, TokenSpan};

#[derive(Debug)]
pub struct LowerName<'a>(TokenSpan, &'a str);

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

pub fn parse_skip_whitespace(tokens: &mut Peekable<impl Iterator<Item = TokenSpan>>) {
    while let Some(TokenSpan {
        kind: TokenKind::Whitespace,
        ..
    }) = tokens.peek()
    {
        tokens.next();
    }
}

pub fn parse_lower_name<'a>(
    source: &'a str,
    tokens: &mut Peekable<impl Iterator<Item = TokenSpan>>,
) -> Option<LowerName<'a>> {
    let token = tokens.next()?;
    if matches!(token.kind, TokenKind::LowerIdentifier) {
        Some(LowerName(token, &source[token.begin..token.end]))
    } else {
        None
    }
}

pub fn parse_syntax_token<'a>(
    source: &'a str,
    tokens: &mut Peekable<impl Iterator<Item = TokenSpan>>,
    expected: TokenKind,
) -> Option<SyntaxToken<'a>> {
    let token = tokens.next()?;
    match token.kind {
        syntax_kind if syntax_kind == expected => {
            Some(SyntaxToken(token, &source[token.begin..token.end]))
        }
        _ => None,
    }
}

pub fn parse_equal_or_colon<'a>(
    source: &'a str,
    tokens: &mut Peekable<impl Iterator<Item = TokenSpan>>,
) -> Option<SyntaxToken<'a>> {
    let token = tokens.next()?;
    match token.kind {
        TokenKind::Equal => Some(SyntaxToken(token, &source[token.begin..token.end])),
        TokenKind::Colon => Some(SyntaxToken(token, &source[token.begin..token.end])),
        _ => None,
    }
}

pub fn parse_declaration<'a>(
    source: &'a str,
    tokens: &mut Peekable<impl Iterator<Item = TokenSpan>>,
) -> Option<Declaration<'a>> {
    let identifier = parse_lower_name(source, tokens)?;
    parse_skip_whitespace(tokens);
    let equal_or_colon = parse_equal_or_colon(source, tokens)?;
    match equal_or_colon {
        SyntaxToken(_, "=") => {
            parse_skip_whitespace(tokens);
            let mut deferred = vec![];
            loop {
                if let TokenSpan {
                    kind: TokenKind::Semicolon,
                    ..
                } = tokens.peek()?
                {
                    break;
                }
                deferred.push(tokens.next()?);
            }
            let semicolon = parse_syntax_token(source, tokens, TokenKind::Semicolon)?;
            Some(Declaration::ValueExpr(
                identifier,
                equal_or_colon,
                ValueRhs::Deferred(deferred),
                semicolon,
            ))
        }
        SyntaxToken(_, ":") => {
            parse_skip_whitespace(tokens);
            let mut deferred = vec![];
            loop {
                if let TokenSpan {
                    kind: TokenKind::Semicolon,
                    ..
                } = tokens.peek()?
                {
                    break;
                }
                deferred.push(tokens.next()?);
            }
            let semicolon = parse_syntax_token(source, tokens, TokenKind::Semicolon)?;
            Some(Declaration::ValueType(
                identifier,
                equal_or_colon,
                TypeRhs::Deferred(deferred),
                semicolon,
            ))
        }
        _ => None,
    }
}

#[test]
fn it_works() {
    let source = "main : Effect Unit;\nmain = logShow $ 1 + 2;";
    let mut tokens = crate::lexer::lex(source).peekable();
    dbg!(parse_declaration(source, &mut tokens));
    parse_skip_whitespace(&mut tokens);
    dbg!(parse_declaration(source, &mut tokens));
}
