//! Implements the tokenizer.
//!
//! This module handles turning source files into a stream of tokens
//! to be consumed by the layout engine and eventually the parser.
//! Most of the core state and logic is handled within the [`Cursor`]
//! type, hence the module name, and a top-level [`tokenize`] function
//! is also exposed.
//!
//! # Usage
//!
//! ```rust
//! use lily_ast::lexer::cursor::{Cursor, tokenize};
//!
//! let mut cursor = Cursor::new("a b");
//! assert!(cursor.next().is_some());
//! assert!(cursor.next().is_some());
//! assert!(cursor.next().is_some());
//! assert!(cursor.next().is_none());
//!
//! let tokens = tokenize("a b");
//! assert_eq!(tokens.len(), 3);
//! ```
use std::str::Chars;

use unicode_categories::UnicodeCategories;

/// The kinds of comments, either block or line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentK {
    Block,
    Line,
}

/// The kinds of identifiers, both reserved and user-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierK {
    Ado,
    Case,
    Do,
    Else,
    If,
    In,
    Let,
    Lower,
    Of,
    Then,
    Upper,
}

/// The kinds of delimiters or brackets e.g. `(`, `[`, `{`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelimiterK {
    Round,
    Square,
    Brace,
}

/// The kinds of operators, both reserved and user-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorK {
    ArrowLeft,
    ArrowRight,
    Backslash,
    Bang,
    Colon,
    Comma,
    Equal,
    GreaterThan,
    LessThan,
    Period,
    Pipe,
    Question,
    Source,
    Underscore,
}

/// The kinds of numbers, either float or int.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitK {
    Float,
    Int,
}

/// The kinds of unrecognized tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnknownK {
    UnfinishedComment,
    UnfinishedFloat,
    UnknownToken,
}

/// The kinds of layout tokens, inserted by the layout engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutK {
    Begin,
    End,
    Separator,
}

/// The kind of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenK {
    CloseDelimiter(DelimiterK),
    Comment(CommentK),
    Digit(DigitK),
    Identifier(IdentifierK),
    Layout(LayoutK),
    OpenDelimiter(DelimiterK),
    Operator(OperatorK),
    Unknown(UnknownK),
    Whitespace,
}

impl TokenK {
    /// Returns `true` if the token is irrelevant for layout.
    pub fn is_annotation(&self) -> bool {
        matches!(
            self,
            TokenK::Comment(_) | TokenK::Layout(_) | TokenK::Whitespace
        )
    }

    /// Returns `true` if the token is relevant for layout.
    pub fn is_syntax(&self) -> bool {
        !self.is_annotation()
    }
}

/// A token produced by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    /// The beginning byte offset.
    pub begin: usize,
    /// The ending byte offset.
    pub end: usize,
    /// The kind of the token.
    pub kind: TokenK,
    /// The "layout depth" of the token.
    pub depth: usize,
}

impl Token {
    /// Returns `true` if the token is irrelevant for layout.
    pub fn is_annotation(&self) -> bool {
        self.kind.is_annotation()
    }

    /// Returns `true` if the token is relevant for layout.
    pub fn is_syntax(&self) -> bool {
        self.kind.is_syntax()
    }

    /// Creates a new [`Token`] with a given `depth`.
    pub fn with_depth(&self, depth: usize) -> Self {
        Self { depth, ..*self }
    }
}

/// An iterator that yields tokens.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    length: usize,
    source: &'a str,
    chars: Chars<'a>,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    /// Creates a new [`Cursor`] given the source file.
    pub fn new(source: &'a str) -> Self {
        Self {
            length: source.len(),
            source,
            chars: source.chars(),
        }
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn consumed(&self) -> usize {
        self.length - self.chars.as_str().len()
    }

    fn peek_1(&mut self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn peek_2(&mut self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }

    fn take(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) {
        while predicate(self.peek_1()) && !self.is_eof() {
            self.take();
        }
    }
}

impl<'a> Cursor<'a> {
    fn take_token(&mut self) -> Token {
        let begin = self.consumed();
        let kind = match self.take().unwrap() {
            // Block Comments
            '{' if self.peek_1() == '-' => {
                self.take();
                loop {
                    if self.is_eof() {
                        break TokenK::Unknown(UnknownK::UnfinishedComment);
                    } else if self.peek_1() == '-' && self.peek_2() == '}' {
                        self.take();
                        self.take();
                        break TokenK::Comment(CommentK::Block);
                    } else {
                        self.take();
                    }
                }
            }
            // Open Parentheses
            '(' => TokenK::OpenDelimiter(DelimiterK::Round),
            '[' => TokenK::OpenDelimiter(DelimiterK::Square),
            '{' => TokenK::OpenDelimiter(DelimiterK::Brace),
            // Close Parentheses
            ')' => TokenK::CloseDelimiter(DelimiterK::Round),
            ']' => TokenK::CloseDelimiter(DelimiterK::Square),
            '}' => TokenK::CloseDelimiter(DelimiterK::Brace),
            // Built-in Symbols
            ',' => TokenK::Operator(OperatorK::Comma),
            '\\' => TokenK::Operator(OperatorK::Backslash),
            // Comment Line
            '-' if self.peek_1() == '-' => {
                self.take_while(|c| c != '\n');
                TokenK::Comment(CommentK::Line)
            }
            // Identifiers
            initial if initial.is_letter_lowercase() || initial == '_' && self.peek_1() == '_' => {
                self.take_while(|c| c.is_letter() || c.is_number() || "'_".contains(c));
                let end = self.consumed();
                TokenK::Identifier(match &self.source[begin..end] {
                    "ado" => IdentifierK::Ado,
                    "case" => IdentifierK::Case,
                    "do" => IdentifierK::Do,
                    "else" => IdentifierK::Else,
                    "if" => IdentifierK::If,
                    "in" => IdentifierK::In,
                    "let" => IdentifierK::Let,
                    "of" => IdentifierK::Of,
                    "then" => IdentifierK::Then,
                    _ => IdentifierK::Lower,
                })
            }
            initial if initial.is_letter_uppercase() => {
                self.take_while(|c| c.is_letter() || c.is_number() || "'_".contains(c));
                TokenK::Identifier(IdentifierK::Upper)
            }
            // Compound Symbols
            initial if initial.is_symbol() || initial.is_punctuation() => {
                self.take_while(|c| !"(){}[]".contains(c) && (c.is_symbol() || c.is_punctuation()));
                let end = self.consumed();
                TokenK::Operator(match &self.source[begin..end] {
                    "->" => OperatorK::ArrowRight,
                    "<-" => OperatorK::ArrowLeft,
                    "=" => OperatorK::Equal,
                    ":" => OperatorK::Colon,
                    "." => OperatorK::Period,
                    "|" => OperatorK::Pipe,
                    "?" => OperatorK::Question,
                    "!" => OperatorK::Bang,
                    "_" => OperatorK::Underscore,
                    "<" => OperatorK::LessThan,
                    ">" => OperatorK::GreaterThan,
                    _ => OperatorK::Source,
                })
            }
            // Digits
            initial if initial.is_ascii_digit() => {
                self.take_while(|c| c.is_ascii_digit());
                if self.peek_1() == '.' {
                    // 1..
                    if self.peek_2() == '.' {
                        TokenK::Digit(DigitK::Int)
                    // 1.2
                    } else if self.peek_2().is_ascii_digit() {
                        self.take();
                        self.take_while(|c| c.is_ascii_digit());
                        TokenK::Digit(DigitK::Float)
                    // 1.
                    } else {
                        self.take();
                        TokenK::Unknown(UnknownK::UnfinishedFloat)
                    }
                } else {
                    TokenK::Digit(DigitK::Int)
                }
            }
            // Whitespace
            initial if initial.is_whitespace() => {
                self.take_while(|c| c.is_whitespace());
                TokenK::Whitespace
            }
            // Unknown Token
            _ => TokenK::Unknown(UnknownK::UnknownToken),
        };
        let end = self.consumed();
        let depth = 1;
        Token {
            begin,
            end,
            kind,
            depth,
        }
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_eof() {
            None
        } else {
            Some(self.take_token())
        }
    }
}

/// Converts a source file into a stream of tokens.
pub fn tokenize(source: &str) -> Vec<Token> {
    Cursor::new(source).collect()
}

#[cfg(test)]
mod tests {
    use super::{CommentK, Cursor, DigitK, IdentifierK, OperatorK, Token, TokenK};

    #[test]
    fn double_period_after_int() {
        let source = "1..2";
        let cursor = Cursor::new(source);
        let expected = vec![
            Token {
                begin: 0,
                end: 1,
                kind: TokenK::Digit(DigitK::Int),
                depth: 1,
            },
            Token {
                begin: 1,
                end: 3,
                kind: TokenK::Operator(OperatorK::Source),
                depth: 1,
            },
            Token {
                begin: 3,
                end: 4,
                kind: TokenK::Digit(DigitK::Int),
                depth: 1,
            },
        ];
        assert_eq!(cursor.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn block_comment_in_between() {
        let source = "1{-hello-}2";
        let cursor = Cursor::new(source);
        let expected = vec![
            Token {
                begin: 0,
                end: 1,
                kind: TokenK::Digit(DigitK::Int),
                depth: 1,
            },
            Token {
                begin: 1,
                end: 10,
                kind: TokenK::Comment(CommentK::Block),
                depth: 1,
            },
            Token {
                begin: 10,
                end: 11,
                kind: TokenK::Digit(DigitK::Int),
                depth: 1,
            },
        ];
        assert_eq!(cursor.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn underscore_disambiguation() {
        let source = "__";
        let mut cursor = Cursor::new(source);
        assert_eq!(
            cursor.next(),
            Some(Token {
                begin: 0,
                end: 2,
                kind: TokenK::Identifier(IdentifierK::Lower),
                depth: 1,
            })
        )
    }
}
