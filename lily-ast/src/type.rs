//! Defines the type of types in `lily` along with associated operations.
use derivative::Derivative;
use std::rc::Rc;

use super::ann::SourceAnn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationVariant {
    Type,
    Kind,
}

/// The type of types in `lily`.
#[derive(Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub enum Type<Ann> {
    /// Universal quantification.
    Forall {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: String,
        kind: Rc<Option<Type<Ann>>>,
        r#type: Rc<Type<Ann>>,
    },
    /// Skolem type variables.
    ///
    /// This type is usually synthesized by the compiler through skolemization.
    Skolem {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: String,
    },
    /// Unification variables.
    ///
    /// This type is usually synthesized by the compiler through unification.
    Unsolved {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: i32,
    },
    /// Syntactic type variables.
    Variable {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: String,
    },
    /// Type constructors.
    Constructor {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: String,
    },
    /// Type or kind application.
    Application {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        variant: ApplicationVariant,
        function: Rc<Type<Ann>>,
        argument: Rc<Type<Ann>>,
    },
}

pub type SourceType = Type<SourceAnn>;

impl<Ann> Type<Ann> {
    /// Returns a reference to a type's annotation.
    pub fn ann(&self) -> &Ann {
        match self {
            Type::Forall {
                ann,
                name: _,
                kind: _,
                r#type: _,
            } => ann,
            Type::Skolem { ann, name: _ } => ann,
            Type::Unsolved { ann, name: _ } => ann,
            Type::Variable { ann, name: _ } => ann,
            Type::Constructor { ann, name: _ } => ann,
            Type::Application {
                ann,
                variant: _,
                function: _,
                argument: _,
            } => ann,
        }
    }

    /// Returns `true` if the type is [`Forall`].
    ///
    /// [`Forall`]: Type::Forall
    #[must_use]
    pub fn is_forall(&self) -> bool {
        matches!(self, Self::Forall { .. })
    }
}
