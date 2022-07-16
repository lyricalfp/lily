use peekmore::{PeekMore, PeekMoreIterator};

use crate::lexer::cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

#[derive(Debug, PartialEq, Eq)]
pub struct Header(pub Token, pub Token);

#[derive(Debug, PartialEq, Eq)]
pub enum Declaration {
    Type(Token, Token, Vec<Token>),
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
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn header(&mut self) -> Option<Header> {
        self.skip_whitespace();
        self.collect_comments();
        self.skip_whitespace();
        let module_token = self.expect(TokenK::Identifier(IdentifierK::Module))?;
        self.skip_whitespace();
        self.collect_comments();
        self.skip_whitespace();
        let module_identifier = self.expect(TokenK::Identifier(IdentifierK::Upper))?;
        self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
        Some(Header(module_token, module_identifier))
    }
    pub fn declaration(&mut self) -> Option<Declaration> {
        self.skip_whitespace();
        self.collect_comments();
        self.skip_whitespace();
        let type_identifier = self.expect(TokenK::Identifier(IdentifierK::Upper))?;
        self.skip_whitespace();
        self.collect_comments();
        self.skip_whitespace();
        let colon_token = self.expect(TokenK::Operator(OperatorK::Colon))?;
        self.skip_whitespace();
        self.collect_comments();
        self.skip_whitespace();
        let rhs_tokens = self.collect_until_separator(0);
        self.expect(TokenK::Layout(LayoutK::Separator(0)))?;
        Some(Declaration::Type(type_identifier, colon_token, rhs_tokens))
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
        parser::{Declaration, Header, Parser},
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
