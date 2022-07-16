use peekmore::{PeekMore, PeekMoreIterator};

use crate::lexer::cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

#[derive(Debug, PartialEq, Eq)]
pub struct Header(pub Token, pub Token);

#[derive(Debug, PartialEq, Eq)]
pub enum Declaration {
    Type(Domain, Token, Token, Vec<Token>),
    Value(Token, Vec<Token>, Token, Vec<Token>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Domain {
    Type,
    Value,
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

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    #[must_use]
    pub fn expect(&mut self, kind: TokenK) -> Option<Token> {
        if self.peek()?.kind == kind {
            self.advance()
        } else {
            None
        }
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn skip_whitespace(&mut self) {
        while let Some(Token {
            kind: TokenK::Whitespace,
            ..
        }) = self.peek()
        {
            self.advance().unwrap();
        }
    }
    pub fn collect_comments(&mut self) -> Vec<Token> {
        let mut comments = vec![];
        while let Some(Token {
            kind: TokenK::Comment(_),
            ..
        }) = self.peek()
        {
            comments.push(self.advance().unwrap())
        }
        comments
    }
    pub fn collect_until_separator(&mut self, depth: usize) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(token) = self.peek() {
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
    pub fn collect_prefixes(&mut self) -> Vec<Token> {
        self.skip_whitespace();
        let comments = self.collect_comments();
        self.skip_whitespace();
        comments
    }
    pub fn collect_binders(&mut self) -> Option<Vec<Token>> {
        let mut binders = vec![];
        while let TokenK::Operator(OperatorK::Underscore) | TokenK::Identifier(IdentifierK::Lower) =
            self.peek()?.kind
        {
            binders.push(self.advance()?);
            self.collect_prefixes();
        }
        Some(binders)
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn header(&mut self) -> Option<Header> {
        self.collect_prefixes();
        let module_token = self.expect(TokenK::Identifier(IdentifierK::Module))?;
        self.collect_prefixes();
        let module_identifier = self.expect(TokenK::Identifier(IdentifierK::Upper))?;
        self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
        Some(Header(module_token, module_identifier))
    }
    pub fn declaration(&mut self) -> Option<Declaration> {
        self.collect_prefixes();
        match self.peek()?.kind {
            TokenK::Identifier(IdentifierK::Lower) => {
                let identifier = self.advance()?;
                self.collect_prefixes();

                if let TokenK::Operator(OperatorK::Colon) = self.peek()?.kind {
                    let colon = self.advance()?;
                    self.collect_prefixes();
                    let typ = self.collect_until_separator(0);
                    return Some(Declaration::Type(Domain::Value, identifier, colon, typ));
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
                    let value = self.collect_until_separator(0);
                    return Some(Declaration::Value(identifier, binders, operator, value));
                }

                None
            }
            TokenK::Identifier(IdentifierK::Upper) => {
                let identifier = self.advance()?;
                self.collect_prefixes();

                if let TokenK::Operator(OperatorK::Colon) = self.peek()?.kind {
                    let colon = self.advance()?;
                    self.collect_prefixes();
                    let typ = self.collect_until_separator(0);
                    return Some(Declaration::Type(Domain::Type, identifier, colon, typ));
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

                None
            }
            token => {
                dbg!("unexpected token {}", &token);
                None
            }
        }
    }
}

#[test]
fn it_works() {
    let source = "f x y = 0";
    let mut parser = Parser::new(crate::lexer::lex_non_empty(source));
    dbg!(parser.declaration());
}

#[cfg(test)]
mod tests {
    use peekmore::PeekMore;

    use crate::{
        lexer::{
            cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK},
            lex_non_empty,
        },
        parser::{Declaration, Domain, Header, Parser},
    };

    #[test]
    fn test_module_header() {
        let source = "module Main";
        let tokens = lex_non_empty(source).peekmore();
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.header(),
            Some(Header(
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
            Some(Declaration::Type(
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
                vec![
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
                ]
            ))
        );
    }
}
