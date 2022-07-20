pub mod arena;
pub mod pratt;
pub mod types;

use peekmore::{PeekMore, PeekMoreIterator};

use crate::lexer::cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Header(pub Token, pub Token);

#[derive(Debug, PartialEq, Eq)]
pub enum Rhs<T> {
    Deferred(Vec<Token>),
    Done(T),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Declaration {
    Type(Domain, Token, Token, Rhs<Type>),
    Value(Token, Vec<Token>, Token, Rhs<Value>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Domain {
    Type,
    Value,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParserErr {
    #[error("no more tokens")]
    NoMoreTokens,
    #[error("unexpected token")]
    UnexpectedToken(Token),
}

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    tokens: PeekMoreIterator<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(tokens: I) -> Self {
        let tokens = tokens.peekmore();
        Self { tokens }
    }

    pub fn peek(&mut self) -> Result<&Token, ParserErr> {
        self.tokens.peek().ok_or(ParserErr::NoMoreTokens)
    }

    pub fn advance(&mut self) -> Result<Token, ParserErr> {
        self.tokens.next().ok_or(ParserErr::NoMoreTokens)
    }

    pub fn expect(&mut self, kind: TokenK) -> Result<Token, ParserErr> {
        let token = self.peek()?;
        if token.kind == kind {
            self.advance()
        } else {
            Err(ParserErr::UnexpectedToken(*token))
        }
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn collect_binders(&mut self) -> Result<Vec<Token>, ParserErr> {
        let mut binders = vec![];
        while let TokenK::Operator(OperatorK::Underscore) | TokenK::Identifier(IdentifierK::Lower) =
            self.peek()?.kind
        {
            binders.push(self.advance()?);
            self.collect_prefixes();
        }
        Ok(binders)
    }
    pub fn collect_prefixes(&mut self) -> Vec<Token> {
        let mut prefixes = vec![];
        while let Ok(Token {
            kind: TokenK::Comment(_) | TokenK::Whitespace,
            ..
        }) = self.peek()
        {
            prefixes.push(self.advance().unwrap())
        }
        prefixes
    }
    pub fn collect_until_separator(&mut self, depth: usize) -> Vec<Token> {
        let mut tokens = vec![];
        while let Ok(token) = self.peek() {
            if let TokenK::Layout(LayoutK::Separator(actual)) = token.kind {
                if actual == depth {
                    break;
                } else {
                    tokens.push(self.advance().unwrap())
                }
            } else {
                tokens.push(self.advance().unwrap())
            }
        }
        tokens
    }
    pub fn skip_to_after_separator(&mut self, depth: usize) {
        while let Ok(token) = self.peek() {
            if let TokenK::Layout(LayoutK::Separator(actual)) = token.kind {
                let _ = self.advance();
                if actual == depth {
                    break;
                }
            }
            let _ = self.advance();
        }
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn header(&mut self) -> Result<Header, ParserErr> {
        self.collect_prefixes();
        let module_token = self.expect(TokenK::Identifier(IdentifierK::Module))?;
        self.collect_prefixes();
        let module_identifier = self.expect(TokenK::Identifier(IdentifierK::Upper))?;
        self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
        Ok(Header(module_token, module_identifier))
    }
    pub fn declaration(&mut self) -> Result<Declaration, ParserErr> {
        self.collect_prefixes();
        match self.peek()?.kind {
            TokenK::Identifier(IdentifierK::Lower) => {
                let identifier = self.advance()?;
                self.collect_prefixes();

                if let TokenK::Operator(OperatorK::Colon) = self.peek()?.kind {
                    let colon = self.advance()?;
                    self.collect_prefixes();
                    let typ = Rhs::Deferred(self.collect_until_separator(0));
                    self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
                    return Ok(Declaration::Type(Domain::Value, identifier, colon, typ));
                }

                if let TokenK::Operator(OperatorK::LessThan) = self.peek()?.kind {
                    todo!("instance chain")
                }

                if let TokenK::Operator(OperatorK::Pipe) = self.peek()?.kind {
                    todo!("instance declaration")
                }

                let binders = self.collect_binders()?;

                if let TokenK::Operator(OperatorK::Equal) = self.peek()?.kind {
                    let operator = self.advance()?;
                    self.collect_prefixes();
                    let value = Rhs::Deferred(self.collect_until_separator(0));
                    self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
                    return Ok(Declaration::Value(identifier, binders, operator, value));
                }

                Err(ParserErr::UnexpectedToken(*self.peek()?))
            }
            TokenK::Identifier(IdentifierK::Upper) => {
                let identifier = self.advance()?;
                self.collect_prefixes();

                if let TokenK::Operator(OperatorK::Colon) = self.peek()?.kind {
                    let colon = self.advance()?;
                    self.collect_prefixes();
                    let typ = Rhs::Deferred(self.collect_until_separator(0));
                    self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
                    return Ok(Declaration::Type(Domain::Type, identifier, colon, typ));
                }

                let _ = self.collect_binders()?;

                if let TokenK::Operator(OperatorK::Question) = self.peek()?.kind {
                    todo!("data declaration")
                }

                if let TokenK::Operator(OperatorK::Bang) = self.peek()?.kind {
                    todo!("type family")
                }

                if let TokenK::Operator(OperatorK::Pipe | OperatorK::GreaterThan) =
                    self.peek()?.kind
                {
                    todo!("type class")
                }

                Err(ParserErr::UnexpectedToken(*self.peek()?))
            }
            _ => Err(ParserErr::UnexpectedToken(*self.peek()?)),
        }
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn declarations(&mut self) -> Vec<Result<Declaration, ParserErr>> {
        let mut declarations = vec![];
        loop {
            match self.declaration() {
                Ok(declaration) => declarations.push(Ok(declaration)),
                Err(
                    ParserErr::UnexpectedToken(Token {
                        kind: TokenK::Eof, ..
                    })
                    | ParserErr::NoMoreTokens,
                ) => break,
                Err(error) => {
                    declarations.push(Err(error));
                    self.skip_to_after_separator(0);
                }
            }
        }
        declarations
    }
}

#[cfg(test)]
mod tests {
    use peekmore::PeekMore;

    use crate::{
        lexer::{
            cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK},
            lex_non_empty,
        },
        parser::{Declaration, Domain, Header, Parser, Rhs},
    };

    #[test]
    fn test_module_header() {
        let source = "module Main";
        let tokens = lex_non_empty(source).peekmore();
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.header(),
            Ok(Header(
                Token {
                    begin: 0,
                    end: 6,
                    kind: TokenK::Identifier(IdentifierK::Module),
                },
                Token {
                    begin: 7,
                    end: 11,
                    kind: TokenK::Identifier(IdentifierK::Upper),
                },
            ),)
        );
    }

    #[test]
    fn test_type_declaration() {
        let source = r"
Type : do
  hello
  world
";
        let tokens = lex_non_empty(source).peekmore();
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.declaration(),
            Ok(Declaration::Type(
                Domain::Type,
                Token {
                    begin: 1,
                    end: 5,
                    kind: TokenK::Identifier(IdentifierK::Upper)
                },
                Token {
                    begin: 6,
                    end: 7,
                    kind: TokenK::Operator(OperatorK::Colon)
                },
                Rhs::Deferred(vec![
                    Token {
                        begin: 8,
                        end: 10,
                        kind: TokenK::Identifier(IdentifierK::Do)
                    },
                    Token {
                        begin: 10,
                        end: 10,
                        kind: TokenK::Layout(LayoutK::Begin)
                    },
                    Token {
                        begin: 10,
                        end: 13,
                        kind: TokenK::Whitespace
                    },
                    Token {
                        begin: 13,
                        end: 18,
                        kind: TokenK::Identifier(IdentifierK::Lower)
                    },
                    Token {
                        begin: 18,
                        end: 18,
                        kind: TokenK::Layout(LayoutK::Separator(1))
                    },
                    Token {
                        begin: 18,
                        end: 21,
                        kind: TokenK::Whitespace
                    },
                    Token {
                        begin: 21,
                        end: 26,
                        kind: TokenK::Identifier(IdentifierK::Lower)
                    },
                    Token {
                        begin: 26,
                        end: 26,
                        kind: TokenK::Layout(LayoutK::End)
                    }
                ])
            ))
        );
    }

    #[test]
    fn best_effort_parsing() {
        let source = r"
main : Effect Unit
main & Effect Unit
main : Effect Unit
";
        let mut parser = Parser::new(
            crate::lexer::lex_non_empty(source).filter(|token| !matches!(token.kind, TokenK::Eof)),
        );
        let mut declarations = parser.declarations().into_iter();
        assert!(declarations.next().unwrap().is_ok());
        assert!(declarations.next().unwrap().is_err());
        assert!(declarations.next().unwrap().is_ok());
    }
}
