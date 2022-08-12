//! Type definitions for the lexer.

/// The kinds of comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentK {
    Block,
    Line,
}

/// The kinds of identifiers.
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

/// The kinds of brackets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelimiterK {
    Round,
    Square,
    Brace,
}

/// The kinds of operators.
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

/// The kinds of digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitK {
    Float,
    Int,
}

/// The kinds of invalid tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnknownK {
    UnfinishedComment,
    UnfinishedFloat,
    UnknownToken,
    EndOfFile,
}

/// The kinds of layout tokens.
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
    Digit(DigitK),
    Identifier(IdentifierK),
    Layout(LayoutK),
    OpenDelimiter(DelimiterK),
    Operator(OperatorK),
    Unknown(UnknownK),
}

/// A token produced by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    /// The beginning byte offset of the prefix.
    pub comment_begin: usize,
    /// The ending byte offset of the prefix.
    pub comment_end: usize,
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
    /// Returns `true` if the [`Token`] is EOF.
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenK::Unknown(UnknownK::EndOfFile))
    }

    /// Sets a new value for a [`Token`]'s `depth` field.
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}
