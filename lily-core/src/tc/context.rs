//! Defines the type of the ordered context consumed by the type
//! checker.
use std::rc::Rc;

use lily_ast::r#type::Type;

/// The default allocation size for the context. This helps reduce the
/// number of allocations needed when we shift elements around. At
/// most, we don't expect more than 100 context elements to exist at a
/// time, unless we're type checking a large, unannotated piece of
/// code.
const DEFAULT_CONTEXT_CAPACITY: usize = 512;

/// TODO: remove this once we have source annotations
type SourceType = Type<()>;

/// The type of context elements.
#[derive(Debug, PartialEq, Eq)]
pub enum Element {
    /// Type variables in scope.
    Variable {
        name: String,
        kind: Rc<Option<SourceType>>,
    },
    /// Unification variables.
    Unsolved {
        name: i32,
        kind: Rc<Option<SourceType>>,
    },
    /// Solved unification variables.
    Solved {
        name: i32,
        kind: Rc<Option<SourceType>>,
        r#type: Rc<SourceType>,
    },
    /// Metasyntactic markers.
    Marker { name: String },
}

/// The context consumed by the type checker.
#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    elements: Vec<Box<Element>>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            elements: Vec::with_capacity(DEFAULT_CONTEXT_CAPACITY),
        }
    }
}

/// Associated operations for the ordered context.
impl Context {
    /// Inserts a `value` to the context.
    pub fn push(&mut self, value: Element) -> () {
        self.elements.push(Box::new(value));
    }

    /// Removes entries up to a provided `value`.
    pub fn discard(&mut self, value: Element) -> () {
        match self.position(&value) {
            Some(index) => {
                self.elements.truncate(index);
            }
            None => {
                eprintln!("Context::discard - {:?} does not exist in the context. No operation has been performed.", value);
            }
        }
    }

    /// Determines the position of a provided `value`.
    pub fn position(&self, value: &Element) -> Option<usize> {
        self.elements
            .iter()
            .position(|current| current.as_ref() == value)
    }

    /// Determines the position of an unsolved element.
    pub fn unsolved_position(&self, value: i32) -> Option<(usize, Rc<Option<SourceType>>)> {
        self.elements
            .iter()
            .enumerate()
            .find_map(|(index, current)| match current.as_ref() {
                Element::Unsolved { name, kind } => {
                    if name == &value {
                        Some((index, Rc::clone(kind)))
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }

    /// Determines whether an unsolved variable appears before another.
    pub fn unsolved_order(&self, a: i32, b: i32) -> bool {
        self.unsolved_position(a).map(|(i, _)| i).unwrap_or(0)
            < self.unsolved_position(b).map(|(i, _)| i).unwrap_or(0)
    }

    /// Replace an unsolved variable in the context with a solved one.
    pub fn unsolved_with(&mut self, name: i32, r#type: Rc<SourceType>) -> () {
        match self.unsolved_position(name) {
            Some((index, kind)) => {
                *self.elements[index] = Element::Solved { name, kind, r#type };
            }
            None => {
                eprintln!("Context::discard - {:?} does not exist in the context. No operation has been performed.", name);
            }
        }
    }

    /// Replace an unsolved variable in the context with other elements.
    pub fn unsolved_with_elems(&mut self, name: i32, elems: Vec<Box<Element>>) -> () {
        match self.unsolved_position(name) {
            Some((index, _)) => {
                self.elements.splice(index..index + 1, elems);
            }
            None => {
                eprintln!("Context::discard - {:?} does not exist in the context. No operation has been performed.", name);
            }
        }
    }

    pub fn apply(&self, r#_type: Rc<Type<()>>) -> Rc<Type<()>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_context() {
        let _ = Context::default();
    }

    #[test]
    fn push_context() {
        let mut context = Context::default();

        context.push(Element::Marker { name: "a".into() });
        context.push(Element::Marker { name: "b".into() });

        assert_eq!(context.elements.len(), 2);
    }

    #[test]
    fn discard_context() {
        let mut context = Context::default();

        context.push(Element::Marker { name: "a".into() });
        context.push(Element::Marker { name: "b".into() });
        context.discard(Element::Marker { name: "a".into() });

        assert_eq!(context.elements.len(), 0);
    }
}
