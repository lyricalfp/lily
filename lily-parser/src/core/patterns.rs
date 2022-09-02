use anyhow::{bail, Context};
use lily_lexer::types::{DelimiterK, DigitK, IdentifierK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{FixityMap, GreaterPattern, GreaterPatternK, LesserPattern, LesserPatternK},
};

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn lesser_patterns(&mut self) -> anyhow::Result<Vec<LesserPattern>> {
        let mut lesser_patterns = vec![];
        loop {
            if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
                break Ok(lesser_patterns);
            }

            if let TokenK::Operator(OperatorK::Underscore) = self.peek()?.kind {
                let Token { begin, end, .. } = self.take()?;
                lesser_patterns.push(LesserPattern {
                    begin,
                    end,
                    kind: LesserPatternK::Null,
                });
                continue;
            }

            if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
                let Token { begin, end, .. } = self.take()?;
                lesser_patterns.push(LesserPattern {
                    begin,
                    end,
                    kind: LesserPatternK::Variable(SmolStr::new(&self.source[begin..end])),
                });
                continue;
            }

            bail!(ParseError::UnexpectedToken(self.peek()?.kind));
        }
    }
}

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn greater_pattern_atom(&mut self, fixity_map: &FixityMap) -> anyhow::Result<GreaterPattern> {
        let Token {
            begin, end, kind, ..
        } = self.take()?;

        if let TokenK::Digit(DigitK::Int) = kind {
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Integer(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Identifier(IdentifierK::Lower) = kind {
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Variable(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Identifier(IdentifierK::Upper) = kind {
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Constructor(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Operator(OperatorK::Underscore) = kind {
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Null,
            });
        }

        if let TokenK::OpenDelimiter(DelimiterK::Round) = kind {
            let greater_pattern = self.greater_pattern_core(fixity_map, 0)?;
            let Token { end, .. } = expect_token!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Parenthesized(Box::new(greater_pattern)),
            });
        }

        bail!(ParseError::UnexpectedToken(kind));
    }

    fn greater_pattern_core(
        &mut self,
        fixity_map: &FixityMap,
        minimum_power: u8,
    ) -> anyhow::Result<GreaterPattern> {
        let mut accumulator = self.greater_pattern_atom(fixity_map)?;

        loop {
            if let TokenK::Identifier(IdentifierK::If)
            | TokenK::Operator(OperatorK::Comma | OperatorK::ArrowRight)
            | TokenK::CloseDelimiter(DelimiterK::Round) = self.peek()?.kind
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

                let argument = self.greater_pattern_core(fixity_map, right_power)?;
                accumulator = GreaterPattern {
                    begin: accumulator.begin,
                    end: argument.end,
                    kind: GreaterPatternK::BinaryOperator(
                        Box::new(accumulator),
                        operator,
                        Box::new(argument),
                    ),
                };
                continue;
            }

            let argument = self.greater_pattern_atom(fixity_map)?;
            match &mut accumulator.kind {
                GreaterPatternK::Application(spines) => {
                    accumulator.end = argument.end;
                    spines.push(argument);
                }
                _ => {
                    accumulator = GreaterPattern {
                        begin: accumulator.begin,
                        end: argument.end,
                        kind: GreaterPatternK::Application(vec![accumulator, argument]),
                    }
                }
            }
        }

        Ok(accumulator)
    }

    pub fn greater_patterns(
        &mut self,
        fixity_map: &FixityMap,
    ) -> anyhow::Result<Vec<GreaterPattern>> {
        let mut greater_patterns = vec![];

        loop {
            let greater_pattern = self.greater_pattern_core(fixity_map, 0)?;
            greater_patterns.push(greater_pattern);

            if let TokenK::Operator(OperatorK::Comma) = self.peek()?.kind {
                self.take()?;
                continue;
            }

            if let TokenK::Identifier(IdentifierK::If) | TokenK::Operator(OperatorK::ArrowRight) =
                self.peek()?.kind
            {
                break;
            }
        }

        Ok(greater_patterns)
    }
}
