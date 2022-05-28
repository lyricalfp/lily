use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::r#type::Type;

use super::Traversal;

#[derive(Default)]
pub struct ReplaceVariablesTraversal<'a, Ann> {
    syntactic: HashMap<&'a str, Rc<Type<Ann>>>,
    unification: HashMap<&'a i32, Rc<Type<Ann>>>,
    in_scope: HashSet<String>,
}

impl<Ann> Traversal<Ann> for ReplaceVariablesTraversal<'_, Ann> {
    fn traverse_type(&mut self, t: Rc<Type<Ann>>) -> Rc<Type<Ann>>
    where
        Ann: Copy,
    {
        match t.as_ref() {
            Type::Forall {
                ann,
                name,
                kind,
                r#type,
            } => {
                let kind = kind.as_ref().map(|kind| super::walk_type(self, Rc::clone(kind)));

                self.in_scope.insert(name.to_string());
                let r#type = super::walk_type(self, Rc::clone(r#type));
                self.in_scope.remove(name);

                Rc::new(Type::Forall {
                    ann: *ann,
                    name: name.to_string(),
                    kind,
                    r#type,
                })
            }
            Type::Skolem { ann: _, name: _ } => t,
            Type::Unsolved { ann: _, name } => {
                if let Some(value) = self.unification.get(name) {
                    Rc::clone(value)
                } else {
                    t
                }
            }
            Type::Variable { ann: _, name } => {
                if let Some(value) = self.syntactic.get(name.as_str()) {
                    Rc::clone(value)
                } else {
                    t
                }
            }
            Type::Constructor { ann: _, name: _ } => t,
            Type::Application {
                ann,
                variant,
                function,
                argument,
            } => Rc::new(Type::Application {
                ann: *ann,
                variant: *variant,
                function: super::walk_type(self, Rc::clone(function)),
                argument: super::walk_type(self, Rc::clone(argument)),
            }),
            Type::Function {
                ann,
                argument,
                result,
            } => Rc::new(Type::Function {
                ann: *ann,
                argument: super::walk_type(self, Rc::clone(argument)),
                result: super::walk_type(self, Rc::clone(result)),
            }),
        }
    }
}
