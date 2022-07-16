use peekmore::PeekMoreIterator;

use crate::lexer::cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleHeader(pub Token, pub Token);

#[derive(Debug, PartialEq, Eq)]
pub enum Declaration {
    Type(Token, Token, Vec<Token>),
}

pub fn skip_spaces(tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>) {
    while let Some(Token {
        kind: TokenK::Whitespace,
        ..
    }) = tokens.peek()
    {
        tokens.next();
    }
}

pub fn skip_right(
    tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
) -> Option<Vec<Token>> {
    let mut depth = 0;
    let mut rhs = vec![];
    loop {
        match tokens.peek()?.kind {
            TokenK::Layout(LayoutK::Separator) => {
                if depth == 0 {
                    tokens.next()?;
                    break;
                } else {
                    rhs.push(tokens.next()?);
                }
            }
            TokenK::Layout(LayoutK::Begin) => {
                depth += 1;
                rhs.push(tokens.next()?);
            }
            TokenK::Layout(LayoutK::End) => {
                depth -= 1;
                rhs.push(tokens.next()?);
            }
            _ => {
                rhs.push(tokens.next()?);
            }
        }
    }
    Some(rhs)
}

pub fn module_header(
    tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
) -> Option<ModuleHeader> {
    skip_spaces(tokens);

    if let TokenK::Identifier(IdentifierK::Module) = tokens.peek()?.kind {
        let module_token = tokens.next()?;
        skip_spaces(tokens);
        if let TokenK::Identifier(IdentifierK::Upper) = tokens.peek()?.kind {
            let identifier_token = tokens.next()?;
            skip_spaces(tokens);
            if let TokenK::Layout(LayoutK::Separator) = tokens.peek()?.kind {
                return Some(ModuleHeader(module_token, identifier_token));
            }
        }
    }

    None
}

pub fn type_declaration(
    tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
) -> Option<Declaration> {
    skip_spaces(tokens);

    if let TokenK::Identifier(IdentifierK::Lower | IdentifierK::Upper) = tokens.peek()?.kind {
        let identifier_token = tokens.next()?;
        skip_spaces(tokens);
        if let TokenK::Operator(OperatorK::Colon) = tokens.peek()?.kind {
            let colon_token = tokens.next()?;
            skip_spaces(tokens);
            let rhs_tokens = skip_right(tokens)?;
            return Some(Declaration::Type(identifier_token, colon_token, rhs_tokens));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use peekmore::PeekMore;

    use crate::{
        lexer::{
            cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK},
            lex_non_empty,
        },
        parser::{module_header, Declaration, ModuleHeader},
    };

    use super::type_declaration;

    #[test]
    fn test_module_header() {
        let source = "module Main";
        let mut tokens = lex_non_empty(source).peekmore();
        assert_eq!(
            module_header(&mut tokens),
            Some(ModuleHeader(
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
        let mut tokens = lex_non_empty(source).peekmore();
        assert_eq!(
            type_declaration(&mut tokens),
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
                        kind: TokenK::Layout(LayoutK::Separator)
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
