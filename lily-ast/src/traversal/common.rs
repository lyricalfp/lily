use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::r#type::Type;

use super::Traversal;

#[derive(Default)]
pub struct TypeVariableTraversal<'ast, Ann> {
    syntactic: HashMap<&'ast str, Rc<Type<Ann>>>,
    unification: HashMap<&'ast i32, Rc<Type<Ann>>>,
    in_scope: HashSet<String>,
}

impl<'ast, Ann> TypeVariableTraversal<'ast, Ann>
where
    Ann: Copy,
{
    pub fn with_syntactic<const N: usize>(syntactic: [(&'ast str, Rc<Type<Ann>>); N]) -> Self {
        TypeVariableTraversal {
            syntactic: HashMap::from(syntactic),
            unification: HashMap::default(),
            in_scope: HashSet::default(),
        }
    }

    pub fn with_unification<const N: usize>(unification: [(&'ast i32, Rc<Type<Ann>>); N]) -> Self {
        TypeVariableTraversal {
            syntactic: HashMap::default(),
            unification: HashMap::from(unification),
            in_scope: HashSet::default(),
        }
    }

    pub fn on_type(mut self, t: Rc<Type<Ann>>) -> Rc<Type<Ann>> {
        self.traverse_type(t)
    }
}

impl<Ann> Traversal<Ann> for TypeVariableTraversal<'_, Ann> {
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
                let kind = kind
                    .as_ref()
                    .map(|kind| super::walk_type(self, Rc::clone(kind)));

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
            _ => super::walk_type(self, t),
        }
    }
}
