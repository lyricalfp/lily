#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentK {
    Block,
    Line,
}

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

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}
