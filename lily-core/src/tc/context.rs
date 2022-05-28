//! Defines the type of the ordered context consumed by the type
//! checker.
use std::{collections::HashSet, rc::Rc};

use lily_ast::{r#type::SourceType, visitor::common::TypeVariableVisitor};

/// The default allocation size for the context. This helps reduce the
/// number of allocations needed when we shift elements around. At
/// most, we don't expect more than 100 context elements to exist at a
/// time, unless we're type checking a large, unannotated piece of
/// code.
const DEFAULT_CONTEXT_CAPACITY: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub enum VariableVariant {
    Skolem,
    Syntactic,
}

/// The type of context elements.
#[derive(Debug, PartialEq, Eq)]
pub enum Element {
    /// Type variables in scope.
    Variable {
        name: String,
        kind: Rc<Option<SourceType>>,
        variant: VariableVariant,
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
    elements: Vec<Element>,
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
    /// Determines the index and kind of an [`Unsolved`] variable.
    ///
    /// # Panics
    ///
    /// Panics if the variable does not exist in the context.
    ///
    /// [`Unsolved`]: Element::Unsolved
    pub fn unsafe_unsolved_position(&self, expected_name: i32) -> (usize, Rc<Option<SourceType>>) {
        self.elements
            .iter()
            .enumerate()
            .find_map(|(index, current)| {
                if let Element::Unsolved { name, kind } = current {
                    if *name == expected_name {
                        Some((index, Rc::clone(kind)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("todo!")
    }

    /// Determines whether an [`Unsolved`] variable appears before another.
    ///
    /// # Panics
    ///
    /// Panics if the variable does not exist in the context.
    ///
    /// [`Unsolved`]: Element::Unsolved
    pub fn unsafe_unsolved_appears_before(&self, a: i32, b: i32) -> bool {
        let (a, _) = self.unsafe_unsolved_position(a);
        let (b, _) = self.unsafe_unsolved_position(b);
        a < b
    }

    /// Replaces an [`Unsolved`] variable with a [`Solved`] one.
    ///
    /// # Panics
    ///
    /// Panics if the variable does not exist in the context.
    ///
    /// [`Unsolved`]: Element::Unsolved
    /// [`Solved`]: Element::Solved
    pub fn unsafe_solve(&mut self, name: i32, r#type: Rc<SourceType>) {
        let (index, kind) = self.unsafe_unsolved_position(name);
        self.elements[index] = Element::Solved { name, kind, r#type }
    }

    /// Replaces an [`Unsolved`] variable with other [`Element`] elements.
    ///
    /// # Panics
    ///
    /// Panics if the variable does not exist in the context.
    ///
    /// [`Unsolved`]: Element::Unsolved
    /// [`Element`]: Element
    pub fn unsafe_unsolved_replace(&mut self, name: i32, elements: Vec<Element>) {
        let (index, _) = self.unsafe_unsolved_position(name);
        self.elements.splice(index..index + 1, elements);
    }

    pub fn apply(&self, r#_type: Rc<SourceType>) -> Rc<SourceType> {
        todo!()
    }

    fn type_is_well_formed_common<'a, I>(&self, r#type: &SourceType, elements: I)
    where
        I: Iterator<Item = &'a Element>,
    {
        let variables = TypeVariableVisitor::from_type(r#type);

        let mut syntactic: HashSet<&String> = HashSet::default();
        let mut skolem: HashSet<&String> = HashSet::default();
        let mut unification: HashSet<&i32> = HashSet::default();

        for element in elements {
            match element {
                Element::Variable {
                    name,
                    kind: _,
                    variant,
                } => match variant {
                    VariableVariant::Skolem => {
                        skolem.insert(name);
                    }
                    VariableVariant::Syntactic => {
                        syntactic.insert(name);
                    }
                },
                Element::Unsolved { name, kind: _ } => {
                    unification.insert(name);
                }
                Element::Solved {
                    name,
                    kind: _,
                    r#type: _,
                } => {
                    unification.insert(name);
                }
                _ => (),
            }
        }

        for _ in variables.syntactic.difference(&syntactic) {
            panic!("unknown type variable");
        }

        for _ in variables.skolem.difference(&skolem) {
            panic!("escaped skolem variable");
        }

        for _ in variables.unification.difference(&unification) {
            panic!("unknown unification variable");
        }
    }

    /// Determines whether a [`SourceType`] is well-formed.
    ///
    /// # Panics
    ///
    /// Panics if the type is not well-formed.
    pub fn type_is_well_formed(&self, r#type: &SourceType) {
        self.type_is_well_formed_common(r#type, self.elements.iter());
    }

    /// Determines whether a [`SourceType`] is well-formed before an
    /// [`Unsolved`] variable.
    ///
    /// # Panics
    ///
    /// Panics if the variable does not exist in the context, or if
    /// the type is not well-formed.
    ///
    /// [`Unsolved`]: Element::Unsolved
    /// [`SourceType`]: lily_ast::type::SourceType
    pub fn type_is_well_formed_before_unsolved(&self, r#type: &SourceType, until: i32) {
        self.type_is_well_formed_common(
            r#type,
            self.elements.iter().take_while(|element| match element {
                Element::Unsolved { name, kind: _ } => *name != until,
                _ => true,
            }),
        )
    }
}
