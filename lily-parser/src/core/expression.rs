use anyhow::{bail, Context};
use lily_lexer::types::{DelimiterK, DigitK, IdentifierK, LayoutK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::{expect_token, Cursor},
    errors::ParseError,
    types::{CaseArm, DoStatement, DoStatementK, Expression, ExpressionK, LesserPattern},
};

impl<'a> Cursor<'a> {
    fn expression_atom(&mut self) -> anyhow::Result<Expression> {
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
            let expression = self.expression_core(0)?;
            let Token { end, .. } = expect_token!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(Expression {
                begin,
                end,
                kind: ExpressionK::Parenthesized(Box::new(expression)),
            });
        }

        bail!(ParseError::UnexpectedToken(kind));
    }

    fn expression_if(&mut self) -> anyhow::Result<Expression> {
        let Token { begin, .. } = expect_token!(self, TokenK::Identifier(IdentifierK::If));
        let condition = self.expression()?;

        expect_token!(self, TokenK::Identifier(IdentifierK::Then));
        let then_value = self.expression()?;

        expect_token!(self, TokenK::Identifier(IdentifierK::Else));
        let else_value @ Expression { end, .. } = self.expression()?;

        Ok(Expression {
            begin,
            end,
            kind: ExpressionK::IfThenElse(
                Box::new(condition),
                Box::new(then_value),
                Box::new(else_value),
            ),
        })
    }

    fn expression_do(&mut self) -> anyhow::Result<Expression> {
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

        let statements = self.expression_do_statements()?;
        let do_end = statements
            .last()
            .context(ParseError::InternalError(
                "Cannot determine last do statement".into(),
            ))?
            .end;

        expect_token!(self, TokenK::Layout(LayoutK::End));

        Ok(Expression {
            begin: do_begin,
            end: do_end,
            kind: ExpressionK::DoBlock(statements),
        })
    }

    fn expression_do_statement(&mut self) -> anyhow::Result<DoStatement> {
        if let TokenK::Identifier(IdentifierK::Let) = self.peek()?.kind {
            let Token {
                begin: let_begin, ..
            } = expect_token!(self, TokenK::Identifier(IdentifierK::Let));

            expect_token!(self, TokenK::Layout(LayoutK::Begin));

            let declarations = self.declaration_let_block()?;
            let let_end = declarations
                .last()
                .context(ParseError::InternalError(
                    "Cannot determine last declaration".into(),
                ))?
                .end;

            expect_token!(self, TokenK::Layout(LayoutK::End));
            expect_token!(self, TokenK::Layout(LayoutK::Separator));

            return Ok(DoStatement {
                begin: let_begin,
                end: let_end,
                kind: DoStatementK::LetStatement(declarations),
            });
        }

        if let do_statement @ Ok(_) = self.attempt(|this| this.expression_do_statement_discard()) {
            return do_statement;
        }

        if let do_statement @ Ok(_) = self.attempt(|this| this.expression_do_statement_bind()) {
            return do_statement;
        }

        bail!(ParseError::UnexpectedToken(self.peek()?.kind));
    }

    fn expression_do_statement_bind(&mut self) -> anyhow::Result<DoStatement> {
        let lesser_pattern @ LesserPattern { begin, .. } = self.lesser_pattern()?;
        expect_token!(self, TokenK::Operator(OperatorK::ArrowLeft));
        let expression @ Expression { end, .. } = self.expression()?;
        expect_token!(self, TokenK::Layout(LayoutK::Separator));
        Ok(DoStatement {
            begin,
            end,
            kind: DoStatementK::BindExpression(lesser_pattern, expression),
        })
    }

    fn expression_do_statement_discard(&mut self) -> anyhow::Result<DoStatement> {
        let expression @ Expression { begin, end, .. } = self.expression()?;
        expect_token!(self, TokenK::Layout(LayoutK::Separator));
        Ok(DoStatement {
            begin,
            end,
            kind: DoStatementK::DiscardExpression(expression),
        })
    }

    fn expression_do_statements(&mut self) -> anyhow::Result<Vec<DoStatement>> {
        let mut statements: Vec<DoStatement> = vec![];
        loop {
            if let TokenK::Layout(LayoutK::End) = self.peek()?.kind {
                break;
            }
            statements.push(self.expression_do_statement()?);
        }
        Ok(statements)
    }

    fn expression_case(&mut self) -> anyhow::Result<Expression> {
        let Token {
            begin: case_begin, ..
        } = expect_token!(self, TokenK::Identifier(IdentifierK::Case));

        let expressions = self.expression_case_expressions()?;

        expect_token!(self, TokenK::Layout(LayoutK::Begin));

        let arms = self.expression_case_arms()?;

        expect_token!(self, TokenK::Layout(LayoutK::End));

        let case_end = arms
            .last()
            .context(ParseError::InternalError(
                "Cannot determine last match arm".into(),
            ))?
            .expression
            .end;

        Ok(Expression {
            begin: case_begin,
            end: case_end,
            kind: ExpressionK::CaseOf(expressions, arms),
        })
    }

    fn expression_case_expressions(&mut self) -> anyhow::Result<Vec<Expression>> {
        let mut expressions = vec![self.expression()?];
        loop {
            if let TokenK::Operator(OperatorK::Comma) = self.peek()?.kind {
                self.take()?;
                continue;
            }
            if let TokenK::Identifier(IdentifierK::Of) = self.peek()?.kind {
                self.take()?;
                break;
            }
            expressions.push(self.expression()?);
        }
        Ok(expressions)
    }

    fn expression_case_arm(&mut self) -> anyhow::Result<CaseArm> {
        let patterns = self.greater_patterns()?;
        let condition = if let TokenK::Identifier(IdentifierK::If) = self.peek()?.kind {
            self.take()?;
            Some(self.expression()?)
        } else {
            None
        };
        expect_token!(self, TokenK::Operator(OperatorK::ArrowRight));
        let expression = self.expression()?;
        expect_token!(self, TokenK::Layout(LayoutK::Separator));
        Ok(CaseArm {
            patterns,
            condition,
            expression,
        })
    }

    fn expression_case_arms(&mut self) -> anyhow::Result<Vec<CaseArm>> {
        let mut arms = vec![self.expression_case_arm()?];
        loop {
            if let TokenK::Layout(LayoutK::End) = self.peek()?.kind {
                break;
            }
            arms.push(self.expression_case_arm()?);
        }
        Ok(arms)
    }

    fn expression_let(&mut self) -> anyhow::Result<Expression> {
        let Token {
            begin: let_begin, ..
        } = expect_token!(self, TokenK::Identifier(IdentifierK::Let));

        expect_token!(self, TokenK::Layout(LayoutK::Begin));

        let declarations = self.declaration_let_block()?;
        let let_end = declarations
            .last()
            .context(ParseError::InternalError(
                "Cannot determine last declaration".into(),
            ))?
            .end;

        expect_token!(self, TokenK::Layout(LayoutK::End));

        expect_token!(self, TokenK::Identifier(IdentifierK::In));

        let expression = self.expression()?;

        Ok(Expression {
            begin: let_begin,
            end: let_end,
            kind: ExpressionK::Let(declarations, Box::new(expression)),
        })
    }

    fn expression_core(&mut self, minimum_power: u8) -> anyhow::Result<Expression> {
        if let TokenK::Identifier(IdentifierK::If) = self.peek()?.kind {
            return self.expression_if();
        }
        if let TokenK::Identifier(IdentifierK::Do) = self.peek()?.kind {
            return self.expression_do();
        }
        if let TokenK::Identifier(IdentifierK::Case) = self.peek()?.kind {
            return self.expression_case();
        }
        if let TokenK::Identifier(IdentifierK::Let) = self.peek()?.kind {
            return self.expression_let();
        }

        let mut accumulator = self.expression_atom()?;

        loop {
            if self.peek()?.is_expression_boundary() {
                break;
            }

            if self.peek()?.is_block_argument() {
                let argument = match self.peek()?.kind {
                    TokenK::Identifier(IdentifierK::If) => self.expression_if()?,
                    TokenK::Identifier(IdentifierK::Do) => self.expression_do()?,
                    TokenK::Identifier(IdentifierK::Case) => self.expression_case()?,
                    TokenK::Identifier(IdentifierK::Let) => self.expression_let()?,
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

                let (left_power, right_power) = self.get_fixity(&operator)?;

                if left_power < minimum_power {
                    break;
                } else {
                    self.take()?;
                }

                let argument = self.expression_core(right_power)?;
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

            let argument = self.expression_atom()?;
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

    pub fn expression(&mut self) -> anyhow::Result<Expression> {
        self.expression_core(0)
    }
}
