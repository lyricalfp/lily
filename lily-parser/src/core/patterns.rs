use anyhow::bail;
use lily_lexer::types::{DelimiterK, DigitK, IdentifierK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{GreaterPattern, GreaterPatternK, LesserPattern, LesserPatternK},
};

impl<'a> Cursor<'a> {
    pub fn lesser_pattern(&mut self) -> anyhow::Result<LesserPattern> {
        if let TokenK::Operator(OperatorK::Underscore) = self.peek()?.kind {
            let Token { begin, end, .. } = self.take()?;
            return Ok(LesserPattern {
                begin,
                end,
                kind: LesserPatternK::Null,
            });
        }

        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            let Token { begin, end, .. } = self.take()?;
            return Ok(LesserPattern {
                begin,
                end,
                kind: LesserPatternK::Variable(SmolStr::new(&self.source[begin..end])),
            });
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }

    pub fn lesser_patterns(&mut self) -> anyhow::Result<Vec<LesserPattern>> {
        let mut lesser_patterns = vec![];
        loop {
            if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
                break Ok(lesser_patterns);
            }

            if let Ok(lesser_pattern) = self.lesser_pattern() {
                lesser_patterns.push(lesser_pattern);
                continue;
            }

            bail!(ParseError::UnexpectedToken(self.peek()?.kind));
        }
    }
}

impl<'a> Cursor<'a> {
    fn greater_pattern_atom(&mut self) -> anyhow::Result<GreaterPattern> {
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
            let greater_pattern = self.greater_pattern_core(0)?;
            let Token { end, .. } = expect_token!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Parenthesized(Box::new(greater_pattern)),
            });
        }

        bail!(ParseError::UnexpectedToken(kind));
    }

    fn greater_pattern_core(&mut self, minimum_power: u8) -> anyhow::Result<GreaterPattern> {
        let mut accumulator = self.greater_pattern_atom()?;

        loop {
            if self.peek()?.is_greater_pattern_boundary() {
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

                let (left_power, right_power) = self.get_fixity(&operator)?;

                if left_power < minimum_power {
                    break;
                } else {
                    self.take()?;
                }

                let argument = self.greater_pattern_core(right_power)?;
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

            let argument = self.greater_pattern_atom()?;
            match &mut accumulator.kind {
                GreaterPatternK::Application(_, arguments) => {
                    accumulator.end = argument.end;
                    arguments.push(argument);
                }
                _ => {
                    accumulator = GreaterPattern {
                        begin: accumulator.begin,
                        end: argument.end,
                        kind: GreaterPatternK::Application(Box::new(accumulator), vec![argument]),
                    }
                }
            }
        }

        Ok(accumulator)
    }

    pub fn greater_patterns(&mut self) -> anyhow::Result<Vec<GreaterPattern>> {
        let mut greater_patterns = vec![];

        loop {
            if let TokenK::Operator(OperatorK::Comma) = self.peek()?.kind {
                expect_token!(self, TokenK::Operator(OperatorK::Comma));
                continue;
            }

            if let TokenK::Identifier(IdentifierK::If) | TokenK::Operator(OperatorK::ArrowRight) =
                self.peek()?.kind
            {
                break;
            }

            greater_patterns.push(self.greater_pattern_core(0)?);
        }

        Ok(greater_patterns)
    }
}
