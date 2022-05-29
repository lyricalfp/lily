use std::collections::HashSet;

use crate::r#type::{SourceType, Type};

use super::Visitor;

#[derive(Default)]
pub struct TypeVariablesGathered<'ast> {
    pub syntactic: HashSet<&'ast str>,
    pub skolem: HashSet<&'ast str>,
    pub unification: HashSet<&'ast i32>,
}

#[derive(Default)]
pub struct TypeVariableVisitor<'ast> {
    variables: TypeVariablesGathered<'ast>,
    in_scope: HashSet<&'ast str>,
}

impl<'ast> TypeVariableVisitor<'ast> {
    pub fn from_type(t: &'ast SourceType) -> TypeVariablesGathered {
        let mut this = Self::default();
        this.visit_type(t);
        this.variables
    }
}

impl<'ast, Ann> Visitor<'ast, Ann> for TypeVariableVisitor<'ast> {
    fn visit_type(&mut self, t: &'ast Type<Ann>) {
        match t {
            Type::Forall {
                ann: _,
                name,
                kind,
                r#type,
            } => {
                if let Some(kind) = kind.as_ref() {
                    super::walk_type(self, kind)
                };
                self.in_scope.insert(name.as_str());
                super::walk_type(self, r#type);
                self.in_scope.remove(name.as_str());
            }
            Type::Skolem { ann: _, name } => {
                self.variables.skolem.insert(name);
            }
            Type::Unsolved { ann: _, name } => {
                self.variables.unification.insert(name);
            }
            Type::Variable { ann: _, name } => {
                if !self.in_scope.contains(name.as_str()) {
                    self.variables.syntactic.insert(name);
                }
            }
            _ => super::walk_type(self, t),
        }
    }
}
