#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: i32,
    pub column: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourceSpan,
    pub end: SourceSpan,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceAnn {
    pub range: SourceRange,
}
