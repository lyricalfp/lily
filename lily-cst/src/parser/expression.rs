use std::{fmt::Display, iter::Peekable};

use lily_interner::{Interned, InternedString, Interner, StringInterner};
use rustc_hash::FxHashMap;

use crate::lexer::cursor::{DelimiterK, DigitK, IdentifierK, OperatorK, Token, TokenK};

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
            ExpressionK::Parenthesized(x) => write!(formatter, "({})", x),
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
        let mut accumulator = self.advance()?;
        loop {
            if let Some(Token {
                kind: TokenK::CloseDelimiter(_),
                ..
            }) = self.tokens.peek()
            {
                break;
            }

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
                let argument = self.advance()?;
                accumulator = self.from_kind(ExpressionK::Application(accumulator, argument));
                continue;
            };

            break;
        }

        Some(accumulator)
    }

    pub fn advance(&mut self) -> Option<Expression<'a>> {
        Some(match self.tokens.next()? {
            Token { begin, end, kind } => {
                let kind = match kind {
                    TokenK::Digit(DigitK::Float) => {
                        ExpressionK::Float(self.from_source(begin, end))
                    }
                    TokenK::Digit(DigitK::Int) => ExpressionK::Int(self.from_source(begin, end)),
                    TokenK::Identifier(IdentifierK::Lower) => {
                        ExpressionK::Variable(self.from_source(begin, end))
                    }
                    TokenK::Identifier(IdentifierK::Upper) => {
                        ExpressionK::Constructor(self.from_source(begin, end))
                    }
                    TokenK::OpenDelimiter(DelimiterK::Round) => {
                        let initial = self.expression()?;
                        assert_eq!(
                            self.tokens.next()?.kind,
                            TokenK::CloseDelimiter(DelimiterK::Round)
                        );
                        ExpressionK::Parenthesized(initial)
                    }
                    _ => panic!("bad token {:?}", kind),
                };
                self.from_kind(kind)
            }
        })
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
    fn simple_example() {
        let source = "(f x y + f x y) * (f x y + f x y)";
        let tokens = Cursor::new(source).collect::<Vec<Token>>();
        let mut powers = FxHashMap::default();
        let arena_0 = Bump::new();
        let arena_1 = Bump::new();
        let interner = Interner::new(&arena_0);
        let strings = StringInterner::new(&arena_1);
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
        let result = expression.expression().unwrap();
        assert_eq!(format!("{}", result), source);
        println!(
            "Allocated {} bytes on expressions",
            arena_0.allocated_bytes()
        );
        println!(
            "Allocated {} bytes on string slices",
            arena_1.allocated_bytes()
        );
    }
}
