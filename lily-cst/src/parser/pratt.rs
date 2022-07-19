use std::iter::Peekable;

use lily_interner::{InternedString, StringInterner};
use rustc_hash::FxHashMap;

use crate::lexer::cursor::{DelimiterK, DigitK, IdentifierK, OperatorK, Token, TokenK};

use super::types::{Expression, ExpressionK, ExpressionKInterner, ParserErr};

type PowerMap<'a> = FxHashMap<&'a str, (u8, u8)>;

pub struct Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    source: &'a str,
    tokens: Peekable<I>,
    powers: PowerMap<'a>,
    expressions: ExpressionKInterner<'a>,
    strings: StringInterner<'a>,
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(
        source: &'a str,
        tokens: Peekable<I>,
        powers: PowerMap<'a>,
        expressions: ExpressionKInterner<'a>,
        strings: StringInterner<'a>,
    ) -> Self {
        Self {
            source,
            tokens,
            powers,
            expressions,
            strings,
        }
    }

    pub fn reclaim(self) -> (ExpressionKInterner<'a>, StringInterner<'a>) {
        (self.expressions, self.strings)
    }

    pub fn peek(&mut self) -> Result<&Token, ParserErr<'a>> {
        self.tokens.peek().ok_or(ParserErr::NoMoreTokens)
    }

    pub fn take(&mut self) -> Result<Token, ParserErr<'a>> {
        self.tokens.next().ok_or(ParserErr::NoMoreTokens)
    }

    pub fn expect(&mut self, kind: TokenK) -> Result<Token, ParserErr<'a>> {
        let token = self.peek()?;
        if token.kind == kind {
            self.take()
        } else {
            Err(ParserErr::UnexpectedToken(*token))
        }
    }
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    #[inline]
    pub fn intern_source(&mut self, begin: usize, end: usize) -> InternedString<'a> {
        self.strings.intern(&self.source[begin..end])
    }

    #[inline]
    pub fn intern_kind(
        &mut self,
        begin: usize,
        end: usize,
        kind: ExpressionK<'a>,
    ) -> Expression<'a> {
        Expression {
            begin,
            end,
            kind: self.expressions.intern(kind),
        }
    }
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn atom(&mut self) -> Result<Expression<'a>, ParserErr<'a>> {
        let token @ Token { begin, end, kind } = self.take()?;

        let (begin, end, kind) = match kind {
            TokenK::Digit(DigitK::Float) => Ok((
                begin,
                end,
                ExpressionK::Float(self.intern_source(begin, end)),
            )),
            TokenK::Digit(DigitK::Int) => {
                Ok((begin, end, ExpressionK::Int(self.intern_source(begin, end))))
            }
            TokenK::Identifier(IdentifierK::Lower) => Ok((
                begin,
                end,
                ExpressionK::Variable(self.intern_source(begin, end)),
            )),
            TokenK::Identifier(IdentifierK::Upper) => Ok((
                begin,
                end,
                ExpressionK::Constructor(self.intern_source(begin, end)),
            )),
            TokenK::OpenDelimiter(DelimiterK::Round) => {
                let expression = self.expression()?;
                let closing = self.expect(TokenK::CloseDelimiter(DelimiterK::Round))?;
                Ok((begin, closing.end, ExpressionK::Parenthesized(expression)))
            }
            _ => Err(ParserErr::UnexpectedToken(token)),
        }?;

        Ok(self.intern_kind(begin, end, kind))
    }

    #[inline]
    pub fn expression(&mut self) -> Result<Expression<'a>, ParserErr<'a>> {
        self.expression_with(0)
    }

    pub fn expression_with(&mut self, minimum_power: u8) -> Result<Expression<'a>, ParserErr<'a>> {
        let mut accumulator = self.atom()?;
        loop {
            if let Ok(Token {
                kind: TokenK::CloseDelimiter(_),
                ..
            }) = self.peek()
            {
                break;
            }

            if let Ok(&Token {
                begin,
                end,
                kind: TokenK::Operator(OperatorK::Source),
            }) = self.peek()
            {
                let operator = &self.source[begin..end];
                let (left_power, right_power) = *self
                    .powers
                    .get(operator)
                    .ok_or(ParserErr::UnknownOperator(operator))?;
                if left_power < minimum_power {
                    break;
                }
                let operator = match self.take()? {
                    Token { begin, end, .. } => self.intern_source(begin, end),
                };
                let argument = self.expression_with(right_power)?;
                accumulator = self.intern_kind(
                    accumulator.begin,
                    argument.end,
                    ExpressionK::BinaryOperator(accumulator, operator, argument),
                );
                continue;
            }

            if let Ok(_) = self.peek() {
                let argument = self.atom()?;
                accumulator = self.intern_kind(
                    accumulator.begin,
                    argument.end,
                    ExpressionK::Application(accumulator, argument),
                );
                continue;
            }

            break;
        }

        Ok(accumulator)
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
