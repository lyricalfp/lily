//! Defines the type of expressions in `lily` along with associated operations.
use crate::r#type::Type;
use derivative::Derivative;
use std::collections::HashMap;
use std::rc::Rc;

/// The type of syntactic literals in `lily`.
#[derive(Debug, PartialEq)]
pub enum Literal<E> {
    Character(char),
    String(String),
    Int(i64),
    Float(f64),
    Array(Vec<E>),
    Object(HashMap<String, E>),
}

/// The type of expressions in `lily`.
#[derive(Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub enum Expr<Ann> {
    /// Syntactic literals.
    Literal {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        literal: Literal<Expr<Ann>>,
    },
    /// Term variables.
    Variable {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: String,
    },
    /// Anonymous functions.
    Lambda {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        argument: String,
        expression: Rc<Expr<Ann>>,
    },
    /// Function application.
    Application {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        function: Rc<Expr<Ann>>,
        argument: Rc<Expr<Ann>>,
    },
    /// Annotated expressions.
    Annotation {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        expression: Rc<Expr<Ann>>,
        r#type: Rc<Type<Ann>>,
        /// Determines whether or not an annotated expression needs to
        /// be checked by the type checker. If not, these expressions
        /// are omitted from further processing.
        checked: bool,
    },
    /// Let bindings.
    Let {
        #[derivative(PartialEq = "ignore")]
        ann: Ann,
        name: String,
        value: Rc<Expr<Ann>>,
        expression: Rc<Expr<Ann>>,
    },
}

impl<Ann> Expr<Ann> {
    pub fn ann(&self) -> &Ann {
        match self {
            Expr::Literal { ann, literal: _ } => ann,
            Expr::Variable { ann, name: _ } => ann,
            Expr::Lambda {
                ann,
                argument: _,
                expression: _,
            } => ann,
            Expr::Application {
                ann,
                function: _,
                argument: _,
            } => ann,
            Expr::Annotation {
                ann,
                expression: _,
                r#type: _,
                checked: _,
            } => ann,
            Expr::Let {
                ann,
                name: _,
                value: _,
                expression: _,
            } => ann,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ann_is_ignored_in_eq() {
        let x = Expr::Lambda {
            ann: 0,
            argument: "a".into(),
            expression: Rc::new(Expr::Variable {
                ann: 0,
                name: "a".into(),
            }),
        };
        let y = Expr::Lambda {
            ann: 1,
            argument: "a".into(),
            expression: Rc::new(Expr::Variable {
                ann: 2,
                name: "a".into(),
            }),
        };

        assert_eq!(x, y);
    }
}
