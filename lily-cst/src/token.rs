#[derive(Debug)]
pub enum TokenKind<'a> {
    DigitDouble(f64),
    DigitInteger(i64),
    NameLower(&'a str),
    NameUpper(&'a str),
    NameSymbol(&'a str),
    ParenLeft,
    ParenRight,
    BracketLeft,
    BracketRight,
    SquareLeft,
    SquareRight,
    SymbolAt,
    SymbolColon,
    SymbolComma,
    SymbolEquals,
    SymbolPeriod,
    SymbolPipe,
    SymbolTick,
    SymbolUnderscore,
    CommentLine(&'a str),
    ArrowFunction,
    ArrowConstraint,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
    pub col: usize,
}
