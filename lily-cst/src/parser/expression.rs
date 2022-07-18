use bumpalo::Bump;

use std::{collections::HashMap, fmt::Display, iter::Peekable};

use crate::lexer::cursor::{DigitK, IdentifierK, OperatorK, Token, TokenK};

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionK<'a> {
    Application(&'a ExpressionK<'a>, &'a ExpressionK<'a>),
    BinaryOperator(&'a ExpressionK<'a>, &'a str, &'a ExpressionK<'a>),
    Int(&'a str),
    Float(&'a str),
    Variable(&'a str),
    Constructor(&'a str),
}

impl<'a> Display for ExpressionK<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionK::Application(f, x) => write!(formatter, "{{{} {}}}", f, x),
            ExpressionK::BinaryOperator(l, o, r) => write!(formatter, "[{} {} {}]", l, o, r),
            ExpressionK::Int(x) => write!(formatter, "{}", x),
            ExpressionK::Float(x) => write!(formatter, "{}", x),
            ExpressionK::Variable(x) => write!(formatter, "{}", x),
            ExpressionK::Constructor(x) => write!(formatter, "{}", x),
        }
    }
}

pub struct Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    source: &'a str,
    tokens: Peekable<I>,
    powers: HashMap<&'a str, (u8, u8)>,
    arena: &'a Bump,
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(
        source: &'a str,
        tokens: Peekable<I>,
        powers: HashMap<&'a str, (u8, u8)>,
        arena: &'a Bump,
    ) -> Self {
        Self {
            source,
            tokens,
            powers,
            arena,
        }
    }
}

impl<'a, I> Pratt<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn expression(&mut self) -> Option<&'a ExpressionK<'a>> {
        self.expressionk(0)
    }

    pub fn expressionk(&mut self, minimum_power: u8) -> Option<&'a ExpressionK<'a>> {
        let mut accumulator = self.arena.alloc(match self.tokens.next()? {
            Token { begin, end, kind } => match kind {
                TokenK::Digit(DigitK::Float) => ExpressionK::Float(&self.source[begin..end]),
                TokenK::Digit(DigitK::Int) => ExpressionK::Int(&self.source[begin..end]),
                TokenK::Identifier(IdentifierK::Lower) => {
                    ExpressionK::Variable(&self.source[begin..end])
                }
                TokenK::Identifier(IdentifierK::Upper) => {
                    ExpressionK::Constructor(&self.source[begin..end])
                }
                _ => todo!(),
            },
        });

        loop {
            match self.tokens.peek() {
                Some(Token {
                    begin,
                    end,
                    kind: TokenK::Operator(OperatorK::Source),
                }) => {
                    let operator = &self.source[*begin..*end];
                    let (left_power, right_power) =
                        *self.powers.get(operator).expect("known power");
                    if left_power < minimum_power {
                        break;
                    }
                    let operator = match self.tokens.next()? {
                        Token { begin, end, .. } => &self.source[begin..end],
                    };
                    let argument = self.expressionk(right_power)?;
                    accumulator = self.arena.alloc(ExpressionK::BinaryOperator(
                        accumulator,
                        operator,
                        argument,
                    ));
                }
                Some(_) => {
                    let argument = self.arena.alloc(match self.tokens.next()? {
                        Token { begin, end, kind } => match kind {
                            TokenK::Digit(DigitK::Float) => {
                                ExpressionK::Float(&self.source[begin..end])
                            }
                            TokenK::Digit(DigitK::Int) => {
                                ExpressionK::Int(&self.source[begin..end])
                            }
                            TokenK::Identifier(IdentifierK::Lower) => {
                                ExpressionK::Variable(&self.source[begin..end])
                            }
                            TokenK::Identifier(IdentifierK::Upper) => {
                                ExpressionK::Constructor(&self.source[begin..end])
                            }
                            _ => break,
                        },
                    });
                    accumulator = self
                        .arena
                        .alloc(ExpressionK::Application(accumulator, argument));
                }
                _ => break,
            }
        }

        Some(accumulator)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bumpalo::Bump;

    use crate::lexer::cursor::{Cursor, Token, TokenK};

    use super::Pratt;

    #[test]
    fn it_works() {
        let source = "f x y + f x y * f x y + f x y";
        let tokens = Cursor::new(source).collect::<Vec<Token>>();
        let mut powers = HashMap::new();
        let arena = Bump::new();
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
            &arena,
        );
        println!("{}", expression.expression().unwrap());
        println!("Allocated {} bytes.", arena.allocated_bytes());
    }
}
