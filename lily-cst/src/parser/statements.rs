use std::iter::Peekable;

use crate::lexer::cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

use super::{
    arena::{Interner, Symbol},
    pratt::{PowerMap, Pratt},
    types::{Binder, BinderK, ParserErr, Rhs, Statement, StatementK},
};

pub enum ModeK<'a> {
    TopLevel,
    ExprLevel(&'a PowerMap<'a>),
}

pub struct Statements<'a, I>
where
    I: Iterator<Item = Token>,
{
    source: &'a str,
    tokens: Peekable<I>,
    mode: ModeK<'a>,
    interner: &'a mut Interner<'a>,
}

impl<'a, I> Statements<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new_top_level(source: &'a str, tokens: I, interner: &'a mut Interner<'a>) -> Self {
        Self {
            source,
            tokens: tokens.peekable(),
            mode: ModeK::TopLevel,
            interner,
        }
    }

    pub fn new_expr_level(
        source: &'a str,
        tokens: I,
        powers: &'a PowerMap<'a>,
        interner: &'a mut Interner<'a>,
    ) -> Self {
        Self {
            source,
            tokens: tokens.peekable(),
            mode: ModeK::ExprLevel(powers),
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

impl<'a, I> Statements<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn skip_prefixes(&mut self) {
        while let Ok(Token {
            kind: TokenK::Comment(_) | TokenK::Whitespace,
            ..
        }) = self.peek()
        {
            self.take().unwrap();
        }
    }
}

impl<'a, I> Statements<'a, I>
where
    I: Iterator<Item = Token>,
{
    #[inline]
    pub fn intern_source(&mut self, begin: usize, end: usize) -> Symbol<'a> {
        self.interner.intern_string(&self.source[begin..end])
    }
}

impl<'a, I> Statements<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn binders(&mut self) -> Result<Vec<Binder<'a>>, ParserErr<'a>> {
        let mut binders = vec![];
        loop {
            self.skip_prefixes();
            let &Token { begin, end, kind } = self.peek()?;
            let kind = match kind {
                TokenK::Operator(OperatorK::Underscore) => BinderK::NullBinder,
                TokenK::Identifier(IdentifierK::Lower) => {
                    let Token { begin, end, .. } = self.take()?;
                    let symbol = self.intern_source(begin, end);
                    BinderK::VariableBinder(symbol)
                }
                _ => break,
            };
            binders.push(Binder { begin, end, kind })
        }
        Ok(binders)
    }

    pub fn tokens(&mut self, until: usize) -> Vec<Token> {
        let mut tokens = vec![];
        while let Ok(token) = self.peek() {
            if let TokenK::Layout(LayoutK::Separator(actual)) = token.kind {
                if actual == until {
                    break;
                } else {
                    tokens.push(self.take().unwrap())
                }
            } else {
                tokens.push(self.take().unwrap())
            }
        }
        tokens
    }

    pub fn statement(&'a mut self) -> Result<Statement<'a>, ParserErr<'a>> {
        self.skip_prefixes();
        if let TokenK::Identifier(IdentifierK::Lower) = self.peek()?.kind {
            let Token { begin, end, .. } = self.take()?;
            let identifier = self.intern_source(begin, end);
            let binders = self.binders()?;
            if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
                self.take()?;
                match self.mode {
                    ModeK::TopLevel => {
                        let tokens = self.tokens(0);
                        let Token { end, .. } = self.take()?;
                        return Ok(Statement {
                            begin,
                            end,
                            kind: StatementK::Value(identifier, binders, Rhs::Deferred(tokens)),
                        });
                    }
                    ModeK::ExprLevel(powers) => {
                        let tokens = self
                            .tokens(0)
                            .into_iter()
                            .filter(|token| !matches!(token.kind, TokenK::Whitespace));
                        let mut pratt = Pratt::new(self.source, tokens, powers, self.interner);
                        let expression = pratt.expression()?;
                        return Ok(Statement {
                            begin,
                            end,
                            kind: StatementK::Value(identifier, binders, Rhs::Done(expression)),
                        });
                    }
                }
            }
            return Err(ParserErr::UnexpectedToken(*self.peek()?));
        }
        if let TokenK::Identifier(IdentifierK::Upper) = self.peek()?.kind {
            todo!("type declarations")
        }
        return Err(ParserErr::UnexpectedToken(*self.peek()?));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::lex_non_empty,
        parser::{
            arena::{Coliseum, Interner},
            pratt::PowerMap,
            statements::Statements,
        },
    };

    #[test]
    fn expression_parsing() {
        let source = "add a b c = a + b + c";
        let tokens = lex_non_empty(source);
        let coliseum = Coliseum::default();
        let mut interner = Interner::new(&coliseum);
        let mut powers = PowerMap::default();
        powers.insert("+", (1, 2));
        let mut parser = Statements::new_expr_level(source, tokens, &mut powers, &mut interner);
        let _ = dbg!(parser.statement());
        dbg!(coliseum.allocated_bytes());
    }
}
