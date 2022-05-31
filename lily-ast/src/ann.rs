//! This module defines the annotation used for AST nodes.

/// A pair of a line and a column.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Span(u32, u32);

/// A pair of a starting and ending span.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Range(Span, Span);

/// The provenance of an AST node.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Ann {
    FromCompiler,
    FromSource(Range),
}
