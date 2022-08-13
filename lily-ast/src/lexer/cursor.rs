use std::str::Chars;

use unicode_categories::UnicodeCategories;

use super::types::{DelimiterK, DigitK, IdentifierK, OperatorK, Token, TokenK, UnknownK};

#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    length: usize,
    source: &'a str,
    chars: Chars<'a>,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
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

    fn take(&mut self) -> char {
        self.chars.next().unwrap_or(EOF_CHAR)
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) {
        while predicate(self.peek_1()) && !self.is_eof() {
            self.take();
        }
    }
}

impl<'a> Cursor<'a> {
    pub(crate) fn take_token(&mut self) -> Token {
        let comment_begin = self.consumed();
        loop {
            match (self.peek_1(), self.peek_2()) {
                ('-', '-') => {
                    self.take_while(|c| c != '\n');
                }
                ('{', '-') => loop {
                    if self.is_eof() {
                        break;
                    } else if self.peek_1() == '-' && self.peek_2() == '}' {
                        self.take();
                        self.take();
                        break;
                    } else {
                        self.take();
                    }
                },
                (i, _) if i.is_whitespace() => {
                    self.take_while(|c| c.is_whitespace());
                }
                _ => break,
            }
        }
        let comment_end = self.consumed();
        let begin = self.consumed();
        let kind = match self.take() {
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
            // End of file
            '\0' => TokenK::Unknown(UnknownK::EndOfFile),
            // Unknown Token
            _ => TokenK::Unknown(UnknownK::UnknownToken),
        };
        let end = self.consumed();
        let depth = 0;
        Token {
            comment_begin,
            comment_end,
            begin,
            end,
            kind,
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::types::UnknownK;

    use super::{Cursor, DigitK, IdentifierK, OperatorK, Token, TokenK};
    use pretty_assertions::assert_eq;

    #[test]
    fn double_period_after_int() {
        let source = "1..2";
        let mut cursor = Cursor::new(source);
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 0,
                comment_end: 0,
                begin: 0,
                end: 1,
                kind: TokenK::Digit(DigitK::Int),
                depth: 0,
            }
        );
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 1,
                comment_end: 1,
                begin: 1,
                end: 3,
                kind: TokenK::Operator(OperatorK::Source),
                depth: 0,
            }
        );
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 3,
                comment_end: 3,
                begin: 3,
                end: 4,
                kind: TokenK::Digit(DigitK::Int),
                depth: 0,
            }
        );
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 4,
                comment_end: 4,
                begin: 4,
                end: 4,
                kind: TokenK::Unknown(UnknownK::EndOfFile),
                depth: 0,
            }
        );
    }

    #[test]
    fn block_comment_in_between() {
        let source = "1 {-hello-} 2";
        let mut cursor = Cursor::new(source);
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 0,
                comment_end: 0,
                begin: 0,
                end: 1,
                kind: TokenK::Digit(DigitK::Int),
                depth: 0,
            }
        );
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 1,
                comment_end: 12,
                begin: 12,
                end: 13,
                kind: TokenK::Digit(DigitK::Int),
                depth: 0,
            }
        );
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 13,
                comment_end: 13,
                begin: 13,
                end: 13,
                kind: TokenK::Unknown(UnknownK::EndOfFile),
                depth: 0,
            }
        );
    }

    #[test]
    fn underscore_disambiguation() {
        let source = "__";
        let mut cursor = Cursor::new(source);
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 0,
                comment_end: 0,
                begin: 0,
                end: 2,
                kind: TokenK::Identifier(IdentifierK::Lower),
                depth: 0,
            }
        );
        assert_eq!(
            cursor.take_token(),
            Token {
                comment_begin: 2,
                comment_end: 2,
                begin: 2,
                end: 2,
                kind: TokenK::Unknown(UnknownK::EndOfFile),
                depth: 0,
            }
        )
    }
}
