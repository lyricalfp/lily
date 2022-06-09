use pest::{Parser, Span};

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Lexer;

#[derive(Debug)]
pub enum TokenKind<'a> {
    CurlyLeft,
    CurlyRight,
    HardAtSign,
    HardBackslash,
    HardColon,
    HardComma,
    HardDoubleColon,
    HardEqual,
    HardPeriod,
    HardQuestion,
    HardTick,
    HardUnderscore,
    LineComment(&'a str),
    LiteralInt(usize),
    LiteralFloat(f64),
    NameUpper(&'a str),
    NameLower(&'a str),
    NameSymbol(&'a str),
    ParenLeft,
    ParenRight,
    SquareLeft,
    SquareRight,
}

#[derive(Debug)]
pub struct Token<'a>(Span<'a>, TokenKind<'a>);

pub fn lex<'a>(source: &'a str) -> impl Iterator<Item = Token<'a>> {
    let pairs = Lexer::parse(Rule::tokens, source).unwrap();
    pairs.flatten().into_iter().map(move |pair| {
        Token(
            pair.as_span(),
            match pair.as_rule() {
                Rule::line_comment => TokenKind::LineComment(pair.as_str()),
                Rule::syntax_braces => match pair.as_str() {
                    "{" => TokenKind::CurlyLeft,
                    "}" => TokenKind::CurlyRight,
                    "(" => TokenKind::ParenLeft,
                    ")" => TokenKind::ParenRight,
                    "[" => TokenKind::SquareLeft,
                    "]" => TokenKind::SquareRight,
                    _ => panic!("syntax_braces"),
                },
                Rule::name_upper => TokenKind::NameUpper(pair.as_str()),
                Rule::name_lower => TokenKind::NameLower(pair.as_str()),
                Rule::name_symbol => match pair.as_str() {
                    "." => TokenKind::HardPeriod,
                    "=" => TokenKind::HardEqual,
                    "?" => TokenKind::HardQuestion,
                    "@" => TokenKind::HardAtSign,
                    "_" => TokenKind::HardUnderscore,
                    "\\" => TokenKind::HardBackslash,
                    "::" => TokenKind::HardDoubleColon,
                    "," => TokenKind::HardComma,
                    "`" => TokenKind::HardTick,
                    sy => TokenKind::NameSymbol(sy),
                },
                Rule::literal_digit_hex => TokenKind::LiteralInt(
                    usize::from_str_radix(&pair.as_str()[2..], 16).expect("literal_digit_hex"),
                ),
                Rule::literal_digit_int => {
                    TokenKind::LiteralInt(pair.as_str().parse().expect("literal_digit_int"))
                }
                Rule::literal_digit_flt => {
                    TokenKind::LiteralFloat(pair.as_str().parse().expect("literal_digit_flt"))
                }
                Rule::block_comment => TokenKind::LineComment(pair.as_str()),
                _ => unreachable!(),
            },
        )
    })
}

    }
}
