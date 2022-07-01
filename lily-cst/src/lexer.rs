//! A tokenizer implementation in the spirit of `rustc_lexer`.
//!
//! This tokenizer is simplistic in that the `TokenSpan` type encodes
//! the `begin` and `end` offsets of a token in the source file, as
//! opposed to directly storing the string slices. Furthermore, the
//! `TokenKind` type is also designed to be more general with its
//! categories; for instance, it encodes both lowercase names (used in
//! values) and uppercase names (used in types) under the `Identifier`
//! variant.
use std::str::Chars;

use unicode_categories::UnicodeCategories;

/// An error for an unknown token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenError {
    UnfinishedBlockComment,
    UnfinishedCharacter,
    UnfinishedNumber,
    UnfinishedString,
    UnknownToken,
}

/// The kind of the spanned token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    /// A backslash: `\`
    Backslash,
    /// A character: `'a'`
    Character,
    /// A colon: `:`
    Colon,
    /// A comma: `,`
    Comma,
    /// A comment block: `{- hey! -}`
    CommentBlock,
    /// A comment line: `-- listen!`
    CommentLine,
    /// An equal sign: `=`
    Equal,
    /// An integer: `0`, `1`, `2`
    Integer,
    /// Left curly braces: `{`
    LeftBrace,
    /// Left parenthesis: `(`
    LeftParen,
    /// Left square bracket: `[`
    LeftSquare,
    /// A lowercase identifier: `_erin_'`
    LowerIdentifier,
    /// A float: `1.0`, `42.0`
    Number,
    /// A period: `.`
    Period,
    /// Right curly braces: `}`
    RightBrace,
    /// Right parenthesis: `)`
    RightParen,
    /// Right squre bracket: `]`
    RightSquare,
    /// A semicolon: `;`
    Semicolon,
    /// A string: `"let's all love lain"`
    String,
    /// An operator: `$`, `+`, `..`
    Symbol,
    /// An unknown token.
    Unknown(TokenError),
    /// An uppercase identifier: `Erin_'`
    UpperIdentifier,
    /// Whitespace characters.
    Whitespace,
}

/// A token in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenSpan {
    /// The beginning offset (inclusive).
    pub begin: usize,
    /// The ending offset (exclusive).
    pub end: usize,
    /// The kind of the token.
    pub kind: TokenKind,
}

/// The current state of the tokenizer.
pub struct Cursor<'a> {
    /// The length of the source file.
    length: usize,
    /// The characters in the source file.
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Cursor<'a> {
        Cursor {
            length: source.len(),
            chars: source.chars(),
        }
    }

    /// Determines if the cursor is at the end.
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// The number of characters already consumed.
    pub fn consumed_len(&self) -> usize {
        self.length - self.chars.as_str().len()
    }

    /// Peeks a single character into the future, returning `'\0'` if
    /// there's no more characters.
    pub fn peek_1(&mut self) -> char {
        self.chars.clone().next().unwrap_or('\0')
    }

    /// Peeks 2 characters into the future, returning `'\0'` if
    /// there's no more characters.
    pub fn peek_2(&mut self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or('\0')
    }

    /// Takes a single character.
    pub fn take(&mut self) -> Option<char> {
        self.chars.next()
    }

    /// Takes characters while a predicate matches and the cursor is
    /// not at the end of the file.
    pub fn take_while(&mut self, predicate: impl Fn(char) -> bool) {
        while predicate(self.peek_1()) && !self.is_eof() {
            self.take();
        }
    }
}

impl<'a> Cursor<'a> {
    pub fn take_token(&mut self) -> TokenSpan {
        let begin = self.consumed_len();
        let kind = match self.take().unwrap() {
            // block comments
            '{' if self.peek_1() == '-' => {
                self.take();
                loop {
                    self.take_while(|c| c != '-');
                    if self.peek_1() == '-' && self.peek_2() == '}' {
                        self.take();
                        self.take();
                        break TokenKind::CommentBlock;
                    } else if self.take() == None {
                        break TokenKind::Unknown(TokenError::UnfinishedBlockComment);
                    }
                }
            }

            // strings
            '"' => {
                self.take_while(|c| c != '"');
                if self.take() == Some('"') {
                    TokenKind::String
                } else {
                    TokenKind::Unknown(TokenError::UnfinishedString)
                }
            }

            // characters
            '\'' => {
                self.take();
                if self.peek_1() == '\'' {
                    self.take();
                    TokenKind::Character
                } else {
                    TokenKind::Unknown(TokenError::UnfinishedCharacter)
                }
            }

            // reserved syntax
            initial @ (',' | ';' | '(' | ')' | '[' | ']' | '{' | '}') => match initial {
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '[' => TokenKind::LeftSquare,
                ']' => TokenKind::RightSquare,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                _ => unreachable!(),
            },

            // reserved syntax that can also be symbols if repeated
            initial @ (':' | '=' | '.') => {
                if self.peek_1() == initial {
                    self.take_while(|c| c.is_symbol() || c.is_punctuation() || c == initial);
                    TokenKind::Symbol
                } else {
                    match initial {
                        ':' => TokenKind::Colon,
                        '=' => TokenKind::Equal,
                        '.' => TokenKind::Period,
                        _ => unreachable!(),
                    }
                }
            }

            // comment lines
            '-' if self.peek_1() == '-' => {
                self.take_while(|c| c != '\n');
                TokenKind::CommentLine
            }

            // identifiers
            initial if initial.is_letter_lowercase() || initial == '_' => {
                self.take_while(|c| c.is_letter() || c.is_number() || c == '\'' || c == '_');
                TokenKind::LowerIdentifier
            }

            initial if initial.is_letter_uppercase() => {
                self.take_while(|c| c.is_letter() || c.is_number() || c == '\'' || c == '_');
                TokenKind::UpperIdentifier
            }

            // whitespace
            initial if initial.is_whitespace() => {
                self.take_while(|c| c.is_whitespace());
                TokenKind::Whitespace
            }

            // integers and floats
            initial if initial.is_number() => {
                self.take_while(|c| c.is_number());
                if self.peek_1() == '.' {
                    self.take();
                    if self.peek_1().is_number() {
                        self.take_while(|c| c.is_number());
                        TokenKind::Number
                    } else {
                        TokenKind::Unknown(TokenError::UnfinishedNumber)
                    }
                } else {
                    TokenKind::Integer
                }
            }

            // operators
            initial if initial.is_symbol() || initial.is_punctuation() => {
                self.take_while(|c| c.is_symbol() || c.is_punctuation());
                TokenKind::Symbol
            }

            // everything else
            _ => TokenKind::Unknown(TokenError::UnknownToken),
        };

        let end = self.consumed_len();
        TokenSpan { begin, end, kind }
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let cursor = Cursor::new(source);
        Self { cursor }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = TokenSpan;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_eof() {
            None
        }else {
            Some(self.cursor.take_token())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tokenizes_without_spaces() {
        let source = "main=logShow\"a\"";
        let tokens = vec![
            TokenSpan {
                begin: 0,
                end: 4,
                kind: TokenKind::LowerIdentifier,
            },
            TokenSpan {
                begin: 4,
                end: 5,
                kind: TokenKind::Equal,
            },
            TokenSpan {
                begin: 5,
                end: 12,
                kind: TokenKind::LowerIdentifier,
            },
            TokenSpan {
                begin: 12,
                end: 15,
                kind: TokenKind::String,
            },
        ];
        assert_eq!(Lexer::new(source).collect::<Vec<_>>(), tokens);
    }
}
