use anyhow::bail;
use lily_lexer::types::{DelimiterK, IdentifierK, OperatorK, Token, TokenK};
use smol_str::SmolStr;

use crate::{
    cursor::Cursor,
    errors::ParseError,
    expect_token,
    types::{Ty, TyK},
};

impl<'a> Cursor<'a> {
    fn ty_atom(&mut self) -> anyhow::Result<Ty> {
        let token @ Token {
            begin, end, kind, ..
        } = self.take()?;

        if let TokenK::Identifier(IdentifierK::Upper) = kind {
            return Ok(Ty {
                begin,
                end,
                kind: TyK::Constructor(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::Identifier(IdentifierK::Lower) = kind {
            return Ok(Ty {
                begin,
                end,
                kind: TyK::Variable(SmolStr::new(&self.source[begin..end])),
            });
        }

        if let TokenK::OpenDelimiter(DelimiterK::Round) = kind {
            let ty = self.ty_core(0)?;
            let Token { end, .. } = expect_token!(self, TokenK::CloseDelimiter(DelimiterK::Round));
            return Ok(Ty {
                begin,
                end,
                kind: TyK::Parenthesized(Box::new(ty)),
            });
        }

        bail!(ParseError::UnexpectedToken(token.kind));
    }

    fn ty_core(&mut self, minimum_power: u8) -> anyhow::Result<Ty> {
        let mut accumulator = self.ty_atom()?;

        loop {
            if self.peek()?.is_ty_boundary() {
                break;
            }

            if let Token {
                begin,
                end,
                kind: TokenK::Operator(OperatorK::ArrowRight | OperatorK::Source),
                ..
            } = self.peek()?
            {
                let source_range = *begin..*end;
                let operator = SmolStr::new(&self.source[source_range]);

                let (left_power, right_power) = self.get_type_fixity(&operator)?;

                if left_power < minimum_power {
                    break;
                } else {
                    self.take()?;
                }

                let argument = self.ty_core(right_power)?;
                accumulator = Ty {
                    begin: accumulator.begin,
                    end: argument.end,
                    kind: TyK::BinaryOperator(Box::new(accumulator), operator, Box::new(argument)),
                };
                continue;
            }

            let argument = self.ty_atom()?;
            match &mut accumulator.kind {
                TyK::Application(_, arguments) => {
                    accumulator.end = argument.end;
                    arguments.push(argument);
                }
                _ => {
                    accumulator = Ty {
                        begin: accumulator.begin,
                        end: accumulator.end,
                        kind: TyK::Application(Box::new(accumulator), vec![argument]),
                    }
                }
            }
        }

        Ok(accumulator)
    }

    pub fn ty(&mut self) -> anyhow::Result<Ty> {
        self.ty_core(0)
    }
}
