use std::iter::Peekable;

use rustc_hash::FxHashMap;

use crate::lexer::cursor::{DelimiterK, DigitK, IdentifierK, OperatorK, Token, TokenK};

use super::{
    arena::{Interner, Symbol},
    types::{Expression, ExpressionK, ParserErr},
};

pub type PowerMap<'a> = FxHashMap<&'a str, (u8, u8)>;

pub struct Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    source: &'a str,
    tokens: Peekable<I>,
    powers: &'a PowerMap<'a>,
    interner: &'a mut Interner<'a>,
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(
        source: &'a str,
        tokens: I,
        powers: &'a PowerMap<'a>,
        interner: &'a mut Interner<'a>,
    ) -> Self {
        Self {
            source,
            tokens: tokens.peekable(),
            powers,
            interner,
        }
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
    pub fn intern_source(&mut self, begin: usize, end: usize) -> Symbol<'a> {
        self.interner.intern_string(&self.source[begin..end])
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
            kind: self.interner.intern_expression(kind),
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
                let Token { begin, end, .. } = self.take()?;
                let operator = self.intern_source(begin, end);
                let argument = self.expression_with(right_power)?;
                accumulator = self.intern_kind(
                    accumulator.begin,
                    argument.end,
                    ExpressionK::BinaryOperator(accumulator, operator, argument),
                );
                continue;
            }

            if self.peek().is_ok() {
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
    use rustc_hash::FxHashMap;

    use crate::{
        lexer::cursor::{Cursor, TokenK},
        parser::arena::{Coliseum, Interner},
    };

    use super::Pratt;

    #[test]
    fn simple_example() {
        let source = "(f x y + f x y) * (f x y + f x y)";
        let tokens = Cursor::new(source);
        let mut powers = FxHashMap::default();
        let coliseum = Coliseum::default();
        let mut interner = Interner::new(&coliseum);
        powers.insert("+", (1, 2));
        powers.insert("-", (1, 2));
        powers.insert("*", (3, 4));
        powers.insert("/", (3, 4));
        let mut expression = Pratt::new(
            source,
            tokens
                .into_iter()
                .filter(|token| !matches!(token.kind, TokenK::Whitespace)),
            &powers,
            &mut interner,
        );
        let result = expression.expression().unwrap();
        assert_eq!(format!("{}", result), source);
        println!("Allocated {} bytes in total", coliseum.allocated_bytes());
    }
}
