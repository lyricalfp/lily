use logos::{Lexer, Logos};
use smol_str::SmolStr;
use unicode_categories::UnicodeCategories;

fn as_smol_str(lex: &mut Lexer<Token>) -> SmolStr {
    SmolStr::new(lex.slice())
}

fn name_symbol_0(lex: &mut Lexer<Token>) -> Option<SmolStr> {
    Some(as_smol_str(lex))
}

fn name_symbol_1(lex: &mut Lexer<Token>) -> Option<SmolStr> {
    if let Some(character) = lex.slice().chars().nth(0) {
        if !character.is_punctuation() {
            Some(as_smol_str(lex))
        } else {
            None
        }
    } else {
        None
    }
}

fn comment_line(lex: &mut Lexer<Token>) -> SmolStr {
    SmolStr::new(lex.slice().trim())
}

#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[regex(r"\p{Lu}[\p{L}+_0-9']*", callback = as_smol_str)]
    NameUpper(SmolStr),

    #[regex(r"[\p{Ll}_][\p{L}+_0-9']*", callback = as_smol_str)]
    NameLower(SmolStr),

    #[regex(r"[:!#$%&*+./<=>?@\\^|~-]+", priority = 2, callback = name_symbol_0)]
    #[regex(r"\p{S}+", callback = name_symbol_1)]
    NameSymbol(SmolStr),

    #[token("@", priority = 3)]
    SymbolAtSign,

    #[token("::", priority = 3)]
    SymbolColon,

    #[token(",", priority = 3)]
    SymbolComma,

    #[token("=", priority = 3)]
    SymbolEqual,

    #[token(".", priority = 3)]
    SymbolPeriod,

    #[token("?", priority = 3)]
    SymbolQuestion,

    #[token("_", priority = 3)]
    SymbolUnderscore,

    #[token("->", priority = 3)]
    SymbolFunctionArrow,

    #[token("=>", priority = 3)]
    SymbolConstraintArrow,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("[")]
    LeftSquare,

    #[token("]")]
    RightSquare,

    #[token("{")]
    LeftCurly,

    #[token("}")]
    RightCurly,

    #[regex(r"--.+\n*", callback = comment_line)]
    CommentLine(SmolStr),

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}
