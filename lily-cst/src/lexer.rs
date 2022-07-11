use std::str::Chars;

use unicode_categories::UnicodeCategories;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentK {
    Block,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierK {
    Do,
    Let,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterK {
    Round,
    Square,
    Brace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorK {
    ArrowLeft,
    ArrowRight,
    Backslash,
    Bang,
    Colon,
    Comma,
    Equal,
    Normal,
    Period,
    Pipe,
    Question,
    Underscore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitK {
    Float,
    Int,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownK {
    UnfinishedComment,
    UnfinishedFloat,
    UnknownToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutK {
    Begin,
    End,
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub begin: usize,
    pub end: usize,
    pub kind: TokenK,
}

struct Cursor<'a> {
    length: usize,
    source: &'a str,
    chars: Chars<'a>,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
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
                    _ => OperatorK::Normal,
                })
            }
            // Identifiers
            initial if initial.is_letter_lowercase() || initial == '_' => {
                self.take_while(|c| c.is_letter() || c.is_number() || "'_".contains(c));
                let end = self.consumed();
                TokenK::Identifier(match &self.source[begin..end] {
                    "let" => IdentifierK::Let,
                    "do" => IdentifierK::Do,
                    _ => IdentifierK::Lower,
                })
            }
            initial if initial.is_letter_uppercase() => {
                self.take_while(|c| c.is_letter() || c.is_number() || "'_".contains(c));
                TokenK::Identifier(IdentifierK::Upper)
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
        Token { begin, end, kind }
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_eof() {
            None
        } else {
            let mut token = self.cursor.take_token();
            loop {
                if let TokenK::Whitespace = token.kind {
                    token = self.cursor.take_token();
                } else {
                    break Some(token);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommentK, DigitK, Lexer, OperatorK, Token, TokenK};

    #[test]
    fn double_period_after_int() {
        let source = "1..2";
        let lexer = Lexer::new(source);
        let expected = vec![
            Token {
                begin: 0,
                end: 1,
                kind: TokenK::Digit(DigitK::Int),
            },
            Token {
                begin: 1,
                end: 3,
                kind: TokenK::Operator(OperatorK::Normal),
            },
            Token {
                begin: 3,
                end: 4,
                kind: TokenK::Digit(DigitK::Int),
            },
        ];
        assert_eq!(lexer.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn block_comment_in_between() {
        let source = "1{-hello-}2";
        let lexer = Lexer::new(source);
        let expected = vec![
            Token {
                begin: 0,
                end: 1,
                kind: TokenK::Digit(DigitK::Int),
            },
            Token {
                begin: 1,
                end: 10,
                kind: TokenK::Comment(CommentK::Block),
            },
            Token {
                begin: 10,
                end: 11,
                kind: TokenK::Digit(DigitK::Int),
            },
        ];
        assert_eq!(lexer.collect::<Vec<_>>(), expected);
    }
}
