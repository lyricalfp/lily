use std::{fmt::Display, hash::Hash};

use lily_interner::{Interned, InternedString, Interner};

use thiserror::Error;

use crate::lexer::cursor::Token;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ExpressionK<'a> {
    Application(Expression<'a>, Expression<'a>),
    BinaryOperator(Expression<'a>, InternedString<'a>, Expression<'a>),
    Constructor(InternedString<'a>),
    Float(InternedString<'a>),
    Int(InternedString<'a>),
    Variable(InternedString<'a>),
    Parenthesized(Expression<'a>),
}

pub type InternedExpressionK<'a> = Interned<'a, ExpressionK<'a>>;

pub type ExpressionKInterner<'a> = Interner<'a, ExpressionK<'a>>;

#[derive(Debug)]
pub struct Expression<'a> {
    // pub begin: usize,
    // pub end: usize,
    pub kind: InternedExpressionK<'a>,
}

impl<'a> PartialEq for Expression<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'a> Eq for Expression<'a> {}

impl<'a> Hash for Expression<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'a> Display for Expression<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind.0 {
            ExpressionK::Application(f, x) => write!(formatter, "{} {}", f, x),
            ExpressionK::BinaryOperator(l, o, r) => write!(formatter, "{} {} {}", l, o, r),
            ExpressionK::Constructor(x)
            | ExpressionK::Float(x)
            | ExpressionK::Int(x)
            | ExpressionK::Variable(x) => write!(formatter, "{}", x),
            ExpressionK::Parenthesized(x) => write!(formatter, "({})", x),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParserErr<'a> {
    #[error("no more tokens")]
    NoMoreTokens,
    #[error("unexpected token")]
    UnexpectedToken(Token),
    #[error("unknown operator")]
    UnknownOperator(&'a str),
}
