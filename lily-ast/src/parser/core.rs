use anyhow::bail;
use anyhow::Context;
use smol_str::SmolStr;

use crate::{
    expect,
    lexer::types::{DelimiterK, DigitK, IdentifierK, LayoutK, OperatorK, Token, TokenK},
    parser::{
        cursor::Cursor,
        errors::ParseError,
        fixity::{Associativity, Fixity, FixityMap},
        types::{
            Expression, ExpressionK, GreaterPattern, GreaterPatternK, LesserPattern, LesserPatternK,
        },
    },
};

impl<'a, I> Cursor<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn fixity(&mut self) -> anyhow::Result<(SmolStr, Fixity)> {
        let Token {
            begin: fixity_begin,
            kind,
            ..
        } = expect!(
            self,
            TokenK::Identifier(IdentifierK::Infixl | IdentifierK::Infixr)
        );
        let associativity = match kind {
            TokenK::Identifier(IdentifierK::Infixl) => Associativity::Infixl,
            TokenK::Identifier(IdentifierK::Infixr) => Associativity::Infixr,
            _ => unreachable!(),
        };

        let Token { begin, end, .. } = expect!(self, TokenK::Digit(DigitK::Int));
        let binding_power = self.source[begin..end]
            .parse()
            .context(ParseError::InternalError(
                "Malformed digit token.".to_string(),
            ))?;

        let Token { begin, end, .. } = expect!(self, TokenK::Identifier(IdentifierK::Lower));
        let identifier = SmolStr::new(&self.source[begin..end]);

        expect!(self, TokenK::Identifier(IdentifierK::As));

        let Token {
            begin,
            end: fixity_end,
            ..
        } = expect!(self, TokenK::Operator(_));
        let operator = SmolStr::new(&self.source[begin..fixity_end]);

        Ok((
            operator,
            Fixity {
                begin: fixity_begin,
                end: fixity_end,
                associativity,
                binding_power,
                identifier,
            },
        ))
    }

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

    pub fn greater_pattern_atom(
        &mut self,
        fixity_map: &FixityMap,
    ) -> anyhow::Result<GreaterPattern> {
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
            let greater_pattern = self.greater_pattern_zero(fixity_map)?;
            let Token { end, .. } = expect!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(GreaterPattern {
                begin,
                end,
                kind: GreaterPatternK::Parenthesized(Box::new(greater_pattern)),
            });
        }

        bail!(ParseError::UnexpectedToken(kind));
    }

    pub fn greater_pattern_core(
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
                    .context(ParseError::InternalError(
                        "Unknown operator binding power!".to_string(),
                    ))?
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

    pub fn greater_pattern_zero(
        &mut self,
        fixity_map: &FixityMap,
    ) -> anyhow::Result<GreaterPattern> {
        self.greater_pattern_core(fixity_map, 0)
    }

    pub fn greater_patterns(
        &mut self,
        fixity_map: &FixityMap,
    ) -> anyhow::Result<Vec<GreaterPattern>> {
        let mut greater_patterns = vec![];

        loop {
            let greater_pattern = self.greater_pattern_zero(fixity_map)?;
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

    pub fn expression_atom(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Expression> {
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
            let expression = self.expression_zero(fixity_map)?;
            let Token { end, .. } = expect!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Parenthesized(Box::new(expression)),
            });
        }

        bail!(ParseError::UnexpectedToken(kind));
    }

    pub fn expression_core(
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
                    .context(ParseError::InternalError(
                        "Unknown operator binding power!".to_string(),
                    ))?
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

    pub fn expression_zero(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Expression> {
        self.expression_core(fixity_map, 0)
    }

    pub fn declaration(&mut self, _: &FixityMap) -> anyhow::Result<()> {
        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            self.take()?;

            if let TokenK::Operator(OperatorK::Colon) = self.peek()?.kind {
                self.take()?;
                return Ok(());
            }

            let _ = self.lesser_patterns()?;

            if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
                self.take()?;
                return Ok(());
            }

            bail!(ParseError::UnexpectedToken(self.peek()?.kind));
        }

        if let TokenK::Identifier(IdentifierK::Upper) = self.peek()?.kind {
            self.take()?;
            return Ok(());
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }
}

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;

    use crate::{
        lexer::lex,
        parser::{
            cursor::Cursor,
            fixity::{Associativity, Fixity, FixityMap},
        },
    };

    #[test]
    fn core_it_works() {
        let source = "Just a, Just b ->";
        let tokens = lex(&source);
        let mut fixity_map = FixityMap::default();
        fixity_map.insert(
            SmolStr::new("+"),
            Fixity {
                begin: 0,
                end: 0,
                associativity: Associativity::Infixl,
                binding_power: 1,
                identifier: SmolStr::new("+"),
            },
        );
        let mut cursor = Cursor::new(&source, tokens.into_iter());
        dbg!(cursor.greater_patterns(&mut fixity_map).unwrap());
    }
}
