//! This module defines the annotation used for AST nodes.

/// A pair of starting and ending byte offsets.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Span(pub u32, pub u32);

/// The provenance of an AST node.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Ann {
    FromCompiler,
    FromSource(Span),  // TODO: add module name here
}
