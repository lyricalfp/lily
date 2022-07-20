use std::{fmt::Display, hash::Hash};

use thiserror::Error;

use crate::lexer::cursor::Token;

use super::arena::{Interned, Symbol};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ExpressionK<'a> {
    Application(Expression<'a>, Expression<'a>),
    BinaryOperator(Expression<'a>, Symbol<'a>, Expression<'a>),
    Constructor(Symbol<'a>),
    Float(Symbol<'a>),
    Int(Symbol<'a>),
    Variable(Symbol<'a>),
    Parenthesized(Expression<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Expression<'a> {
    pub begin: usize,
    pub end: usize,
    pub kind: Interned<'a, ExpressionK<'a>>,
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BinderK<'a> {
    NullBinder,
    VariableBinder(Symbol<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Binder<'a> {
    pub begin: usize,
    pub end: usize,
    pub kind: BinderK<'a>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Rhs<T> {
    Deferred(Vec<Token>),
    Done(T),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum StatementK<'a> {
    Value(Symbol<'a>, Vec<Binder<'a>>, Rhs<Expression<'a>>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Statement<'a> {
    pub begin: usize,
    pub end: usize,
    pub kind: StatementK<'a>,
}
