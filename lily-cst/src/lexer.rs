use fancy_regex::{Captures, Regex};

use crate::token::{Token, TokenKind};

#[derive(Debug)]
pub enum Error {
    UnrecognizedToken(usize),
    UnecessaryLeadingZeroes(usize),
    InternalPanic,
}

type Cb<'a> = &'a dyn Fn(Captures<'a>) -> Result<TokenKind<'a>, Error>;

pub struct Lexer<'a> {
    offset: usize,
    source: &'a str,
    patterns: Vec<(Regex, Cb<'a>)>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let offset = 0;
        let patterns: &[(&'a str, Cb<'a>)] = &[
            (r"^\p{Lu}[\p{L}+_0-9']*", &|i| {
                Ok(TokenKind::NameUpper(i.get(0).unwrap().as_str()))
            }),
            (r"^[\p{Ll}_][\p{L}+_0-9']*", &|i| {
                Ok(TokenKind::NameLower(i.get(0).unwrap().as_str()))
            }),
            (r"^([:!#$%&*+./<=>?@\\^|~-]|(?!\p{P})\p{S})+", &|i| {
                Ok(TokenKind::NameSymbol(i.get(0).unwrap().as_str()))
            }),
            (r"^([0-9]+)(\.[0-9]+)?", &|i| {
                let m = i.get(0).unwrap();
                let s = m.as_str();
                if s.starts_with("00") {
                    Err(Error::UnecessaryLeadingZeroes(m.start()))
                } else if i.get(2).is_some() {
                    s.parse()
                        .map(TokenKind::DigitDouble)
                        .map_err(|_| Error::InternalPanic)
                } else {
                    s.parse()
                        .map(TokenKind::DigitInteger)
                        .map_err(|_| Error::InternalPanic)
                }
            }),
            (r"^--( *\|)?(.+)\n*", &|i| {
                Ok(TokenKind::CommentLine(i.get(2).unwrap().as_str().trim()))
            }),
            (r"^(::|->|=>|<-|<=)", &|i| {
                Ok(match i.get(0).unwrap().as_str() {
                    "::" => TokenKind::SymbolColon,
                    "->" => TokenKind::ArrowFunction,
                    "=>" => TokenKind::ArrowConstraint,
                    "<=" => TokenKind::NameSymbol("<="),
                    "<-" => TokenKind::NameSymbol("<-"),
                    _ => panic!("Lexer::new - this path is never taken"),
                })
            }),
            (r"^[\[\](){}@,=.|`_]", &|i| {
                Ok(match i.get(0).unwrap().as_str() {
                    "(" => TokenKind::ParenLeft,
                    ")" => TokenKind::ParenRight,
                    "[" => TokenKind::SquareLeft,
                    "]" => TokenKind::SquareRight,
                    "{" => TokenKind::BracketLeft,
                    "}" => TokenKind::BracketRight,
                    "@" => TokenKind::SymbolAt,
                    "," => TokenKind::SymbolComma,
                    "=" => TokenKind::SymbolEquals,
                    "." => TokenKind::SymbolPeriod,
                    "|" => TokenKind::SymbolPipe,
                    "`" => TokenKind::SymbolTick,
                    "_" => TokenKind::SymbolUnderscore,
                    _ => panic!("Lexer::new - this path is never taken"),
                })
            }),
        ];
        let patterns = patterns
            .iter()
            .map(|(pattern, creator)| (Regex::new(pattern).unwrap(), *creator))
            .collect();
        Self {
            offset,
            source,
            patterns,
            line: 1,
            col: 1,
        }
    }

    #[inline]
    fn window(&self) -> &'a str {
        &self.source[self.offset..]
    }

    #[inline]
    fn advance(&mut self, with: &str) -> (usize, usize, usize) {
        let current = (self.offset, self.line, self.col);
        self.offset += with.len();
        for character in with.chars() {
            if character == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        current
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(usize, Token<'a>, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // handle eof
        if self.offset >= self.source.len() {
            return None;
        }
        // skip whitespaces
        let whitespace = Regex::new(r"^\s+").unwrap();
        if let Ok(Some(m)) = whitespace.find(self.window()) {
            self.advance(m.as_str());
        }
        // everything else
        let longest_match = self
            .patterns
            .iter()
            .filter_map(|(regex, creator)| {
                if let Ok(Some(captures)) = regex.captures(self.window()) {
                    Some((captures.get(0).unwrap(), creator, captures))
                } else {
                    None
                }
            })
            .max_by_key(|(whole, _, _)| whole.end());

        match longest_match {
            Some((whole, creator, captures)) => Some(creator(captures).map(|kind| {
                let (offset, line, col) = self.advance(whole.as_str());
                (offset, Token { kind, line, col }, self.offset)
            })),
            None => Some(Err(Error::UnrecognizedToken(self.offset))),
        }
    }
}
