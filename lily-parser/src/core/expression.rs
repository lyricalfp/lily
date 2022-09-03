use anyhow::{bail, Context};
use lily_lexer::types::{DelimiterK, DigitK, IdentifierK, LayoutK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{
        Declaration, DoStatement, DoStatementK, Expression, ExpressionK, FixityMap, LesserPattern,
    },
};

impl<'a> Cursor<'a> {
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

    fn expression_if(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Expression> {
        let Token { begin, .. } = expect_token!(self, TokenK::Identifier(IdentifierK::If));
        let condition = self.expression(fixity_map)?;

        expect_token!(self, TokenK::Identifier(IdentifierK::Then));
        let then_value = self.expression(fixity_map)?;

        expect_token!(self, TokenK::Identifier(IdentifierK::Else));
        let else_value @ Expression { end, .. } = self.expression(fixity_map)?;

        return Ok(Expression {
            begin,
            end,
            kind: ExpressionK::IfThenElse(
                Box::new(condition),
                Box::new(then_value),
                Box::new(else_value),
            ),
        });
    }

    fn expression_do(&mut self, fixity_map: &FixityMap) -> anyhow::Result<Expression> {
        let Token {
            begin: do_begin,
            end: do_end,
            ..
        } = expect_token!(self, TokenK::Identifier(IdentifierK::Do));

        if let TokenK::Layout(LayoutK::Separator) = self.peek()?.kind {
            return Ok(Expression {
                begin: do_begin,
                end: do_end,
                kind: ExpressionK::DoBlock(vec![]),
            });
        }

        expect_token!(self, TokenK::Layout(LayoutK::Begin));
        let mut statements: Vec<DoStatement> = vec![];
        let argument_end = loop {
            if let TokenK::Layout(LayoutK::End) = self.peek()?.kind {
                expect_token!(self, TokenK::Layout(LayoutK::End));
                match statements.last() {
                    Some(last) => break last.end,
                    None => break do_end,
                }
            }
            statements.push(self.expression_do_statement(fixity_map)?);
        };

        Ok(Expression {
            begin: do_begin,
            end: argument_end,
            kind: ExpressionK::DoBlock(statements),
        })
    }

    fn expression_do_statement(&mut self, fixity_map: &FixityMap) -> anyhow::Result<DoStatement> {
        if let TokenK::Identifier(IdentifierK::Let) = self.peek()?.kind {
            let Token { begin, .. } = expect_token!(self, TokenK::Identifier(IdentifierK::Let));

            expect_token!(self, TokenK::Layout(LayoutK::Begin));

            let declaration @ Declaration { mut end, .. } = self.declaration(fixity_map)?;
            let mut declarations = vec![declaration];
            loop {
                if let TokenK::Layout(LayoutK::End) = self.peek()?.kind {
                    break;
                }
                let declaration = self.declaration(fixity_map)?;
                end = declaration.end;
                declarations.push(declaration);
            }

            expect_token!(self, TokenK::Layout(LayoutK::End));
            expect_token!(self, TokenK::Layout(LayoutK::Separator));

            return Ok(DoStatement {
                begin,
                end,
                kind: DoStatementK::LetStatement(declarations),
            });
        }

        if let do_statement @ Ok(_) =
            self.attempt(|this| this.expression_do_statement_discard(fixity_map))
        {
            return do_statement;
        }

        if let do_statement @ Ok(_) =
            self.attempt(|this| this.expression_do_statement_bind(fixity_map))
        {
            return do_statement;
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }

    fn expression_do_statement_bind(
        &mut self,
        fixity_map: &FixityMap,
    ) -> anyhow::Result<DoStatement> {
        let lesser_pattern @ LesserPattern { begin, .. } = self.lesser_pattern()?;
        expect_token!(self, TokenK::Operator(OperatorK::ArrowLeft));
        let expression @ Expression { end, .. } = self.expression(fixity_map)?;
        expect_token!(self, TokenK::Layout(LayoutK::Separator));
        return Ok(DoStatement {
            begin,
            end,
            kind: DoStatementK::BindExpression(lesser_pattern, expression),
        });
    }

    fn expression_do_statement_discard(
        &mut self,
        fixity_map: &FixityMap,
    ) -> anyhow::Result<DoStatement> {
        let expression @ Expression { begin, end, .. } = self.expression(fixity_map)?;
        expect_token!(self, TokenK::Layout(LayoutK::Separator));
        Ok(DoStatement {
            begin,
            end,
            kind: DoStatementK::DiscardExpression(expression),
        })
    }

    fn expression_core(
        &mut self,
        fixity_map: &FixityMap,
        minimum_power: u8,
    ) -> anyhow::Result<Expression> {
        if let TokenK::Identifier(IdentifierK::If) = self.peek()?.kind {
            return self.expression_if(fixity_map);
        }
        if let TokenK::Identifier(IdentifierK::Do) = self.peek()?.kind {
            return self.expression_do(fixity_map);
        }

        let mut accumulator = self.expression_atom(fixity_map)?;

        loop {
            if self.peek()?.is_expression_boundary() {
                break;
            }

            if self.peek()?.is_block_argument() {
                let argument = match self.peek()?.kind {
                    TokenK::Identifier(IdentifierK::If) => self.expression_if(fixity_map)?,
                    TokenK::Identifier(IdentifierK::Do) => self.expression_do(fixity_map)?,
                    kind => bail!(ParseError::InternalError(format!(
                        "Unhandled block argument '{:?}'",
                        kind
                    ))),
                };
                accumulator = Expression {
                    begin: accumulator.begin,
                    end: argument.end,
                    kind: ExpressionK::Application(Box::new(accumulator), vec![argument]),
                };
                continue;
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
                ExpressionK::Application(_, arguments) => {
                    accumulator.end = argument.end;
                    arguments.push(argument);
                }
                _ => {
                    accumulator = Expression {
                        begin: accumulator.begin,
                        end: argument.end,
                        kind: ExpressionK::Application(Box::new(accumulator), vec![argument]),
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
