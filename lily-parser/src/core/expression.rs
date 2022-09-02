use anyhow::{bail, Context};
use lily_lexer::types::{DelimiterK, DigitK, IdentifierK, LayoutK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{Expression, ExpressionK, FixityMap},
};

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn expression_atom(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Expression> {
        let Token {
            begin, end, kind, ..
        } = self.take()?;

        if let TokenK::Digit(DigitK::Int) = kind {
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Integer(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Digit(DigitK::Float) = kind {
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Float(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Identifier(IdentifierK::Lower) = kind {
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Variable(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Identifier(IdentifierK::Upper) = kind {
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Constructor(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::OpenDelimiter(DelimiterK::Round) = kind {
            let expression = self.expression_core(fixity_map, 0)?;
            let Token { end, .. } = expect_token!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Parenthesized(Box::new(expression)),
            });
        }

        bail!(ParseError::UnexpectedToken(kind));
    }

    fn expression_core(
        &mut self,
        fixity_map: &FixityMap,
        minimum_power: u8,
    ) -> anyhow::Result<Expression> {
        let mut accumulator = self.expression_atom(fixity_map)?;

        loop {
            if let TokenK::Layout(LayoutK::Separator) | TokenK::CloseDelimiter(DelimiterK::Round) =
                self.peek()?.kind
            {
                break;
            }

            if let Token {
                begin,
                end,
                kind: TokenK::Operator(OperatorK::Source),
                ..
            } = self.peek()?
            {
                let source_range = *begin..*end;
                let operator = SmolStr::new(&self.source[source_range]);

                let (left_power, right_power) = fixity_map
                    .get(&operator)
                    .context(ParseError::UnknownBindingPower(operator.clone()))?
                    .as_pair();

                if left_power < minimum_power {
                    break;
                } else {
                    self.take()?;
                }

                let argument = self.expression_core(fixity_map, right_power)?;
                accumulator = Expression {
                    begin: accumulator.begin,
                    end: argument.end,
                    kind: ExpressionK::BinaryOperator(
                        Box::new(accumulator),
                        operator,
                        Box::new(argument),
                    ),
                };
                continue;
            }

            let argument = self.expression_atom(fixity_map)?;
            match &mut accumulator.kind {
                ExpressionK::Application(spines) => {
                    accumulator.end = argument.end;
                    spines.push(argument);
                }
                _ => {
                    accumulator = Expression {
                        begin: accumulator.begin,
                        end: argument.end,
                        kind: ExpressionK::Application(vec![accumulator, argument]),
                    }
                }
            }
        }

        Ok(accumulator)
    }

    pub fn expression(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Expression> {
        self.expression_core(fixity_map, 0)
    }
}