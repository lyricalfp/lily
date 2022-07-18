use std::{fmt::Display, iter::Peekable};

use lily_interner::{Interned, InternedString, Interner, StringInterner};
use rustc_hash::FxHashMap;

use crate::lexer::cursor::{DigitK, IdentifierK, OperatorK, Token, TokenK};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ExpressionK<'a> {
    Application(Expression<'a>, Expression<'a>),
    BinaryOperator(Expression<'a>, InternedString<'a>, Expression<'a>),
    Constructor(InternedString<'a>),
    Float(InternedString<'a>),
    Int(InternedString<'a>),
    Variable(InternedString<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Expression<'a>(Interned<'a, ExpressionK<'a>>);

impl<'a> Display for Expression<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 .0 {
            ExpressionK::Application(f, x) => write!(formatter, "{} {}", f, x),
            ExpressionK::BinaryOperator(l, o, r) => write!(formatter, "{} {} {}", l, o, r),
            ExpressionK::Constructor(x)
            | ExpressionK::Float(x)
            | ExpressionK::Int(x)
            | ExpressionK::Variable(x) => write!(formatter, "{}", x),
        }
    }
}

pub struct Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    source: &'a str,
    tokens: Peekable<I>,
    powers: FxHashMap<&'a str, (u8, u8)>,
    interner: Interner<'a, ExpressionK<'a>>,
    strings: StringInterner<'a>,
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(
        source: &'a str,
        tokens: Peekable<I>,
        powers: FxHashMap<&'a str, (u8, u8)>,
        interner: Interner<'a, ExpressionK<'a>>,
        strings: StringInterner<'a>,
    ) -> Self {
        Self {
            source,
            tokens,
            powers,
            interner,
            strings,
        }
    }

    pub fn reclaim(self) -> (Interner<'a, ExpressionK<'a>>, StringInterner<'a>) {
        (self.interner, self.strings)
    }
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn expression(&mut self) -> Option<Expression<'a>> {
        self.expression_with_power(0)
    }

    pub fn expression_with_power(&mut self, minimum_power: u8) -> Option<Expression<'a>> {
        let mut accumulator = match self.tokens.next()? {
            Token { begin, end, kind } => {
                let text = self.from_source(begin, end);
                self.from_kind(match kind {
                    TokenK::Digit(DigitK::Float) => ExpressionK::Float(text),
                    TokenK::Digit(DigitK::Int) => ExpressionK::Int(text),
                    TokenK::Identifier(IdentifierK::Lower) => ExpressionK::Variable(text),
                    TokenK::Identifier(IdentifierK::Upper) => ExpressionK::Constructor(text),
                    _ => panic!("bad token {:?}", kind),
                })
            }
        };

        loop {
            if let Some(&Token {
                begin,
                end,
                kind: TokenK::Operator(OperatorK::Source),
            }) = self.tokens.peek()
            {
                let operator = &self.source[begin..end];
                let (left_power, right_power) = *self.powers.get(operator).expect("known power");
                if left_power < minimum_power {
                    break;
                }
                let operator = match self.tokens.next()? {
                    Token { begin, end, .. } => self.from_source(begin, end),
                };
                let argument = self.expression_with_power(right_power)?;
                accumulator =
                    self.from_kind(ExpressionK::BinaryOperator(accumulator, operator, argument));
                continue;
            };

            if let Some(_) = self.tokens.peek() {
                let argument = match self.tokens.next()? {
                    Token { begin, end, kind } => {
                        let text = self.from_source(begin, end);
                        self.from_kind(match kind {
                            TokenK::Digit(DigitK::Float) => ExpressionK::Float(text),
                            TokenK::Digit(DigitK::Int) => ExpressionK::Int(text),
                            TokenK::Identifier(IdentifierK::Lower) => ExpressionK::Variable(text),
                            TokenK::Identifier(IdentifierK::Upper) => {
                                ExpressionK::Constructor(text)
                            }
                            _ => panic!("bad token {:?}", kind),
                        })
                    }
                };
                accumulator = self.from_kind(ExpressionK::Application(accumulator, argument));
                continue;
            };

            break;
        }

        Some(accumulator)
    }

    #[inline]
    pub fn from_source(&mut self, begin: usize, end: usize) -> InternedString<'a> {
        self.strings.intern(&self.source[begin..end])
    }

    #[inline]
    pub fn from_kind(&mut self, kind: ExpressionK<'a>) -> Expression<'a> {
        Expression(self.interner.intern(kind))
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use lily_interner::{Interner, StringInterner};
    use rustc_hash::FxHashMap;

    use crate::lexer::cursor::{Cursor, Token, TokenK};

    use super::Pratt;

    #[test]
    fn it_works() {
        let source = "f x y + f x y * f x y + f x y";
        let tokens = Cursor::new(source).collect::<Vec<Token>>();
        let mut powers = FxHashMap::default();
        let arena = Bump::new();
        let interner = Interner::new(&arena);
        let strings = StringInterner::new(&arena);
        powers.insert("+", (1, 2));
        powers.insert("-", (1, 2));
        powers.insert("*", (3, 4));
        powers.insert("/", (3, 4));
        let mut expression = Pratt::new(
            source,
            tokens
                .into_iter()
                .filter(|token| !matches!(token.kind, TokenK::Whitespace))
                .peekable(),
            powers,
            interner,
            strings,
        );
        println!("{}", expression.expression().unwrap());
        println!("Allocated {} bytes.", arena.allocated_bytes());
    }
}
