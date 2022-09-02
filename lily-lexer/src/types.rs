#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentK {
    Block,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierK {
    Ado,
    As,
    Case,
    Do,
    Else,
    If,
    In,
    Infixl,
    Infixr,
    Let,
    Lower,
    Of,
    Then,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelimiterK {
    Round,
    Square,
    Brace,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitK {
    Float,
    Int,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnknownK {
    UnfinishedComment,
    UnfinishedFloat,
    UnknownToken,
    EndOfFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutK {
    Begin,
    End,
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenK {
    CloseDelimiter(DelimiterK),
    Digit(DigitK),
    Identifier(IdentifierK),
    Layout(LayoutK),
    OpenDelimiter(DelimiterK),
    Operator(OperatorK),
    Unknown(UnknownK),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    pub comment_begin: usize,
    pub comment_end: usize,
    pub begin: usize,
    pub end: usize,
    pub kind: TokenK,
    pub depth: usize,
}

impl Token {
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenK::Unknown(UnknownK::EndOfFile))
    }

    pub fn is_separator_zero(&self) -> bool {
        self.depth == 0 && matches!(self.kind, TokenK::Layout(LayoutK::Separator))
    }

    pub fn is_infix_identifier(&self) -> bool {
        matches!(
            self.kind,
            TokenK::Identifier(IdentifierK::Infixl | IdentifierK::Infixr)
        )
    }

    pub fn is_expression_end(&self) -> bool {
        matches!(
            self.kind,
            TokenK::Identifier(IdentifierK::If | IdentifierK::Then | IdentifierK::Else)
                | TokenK::Layout(LayoutK::Separator)
                | TokenK::CloseDelimiter(DelimiterK::Round)
        )
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
