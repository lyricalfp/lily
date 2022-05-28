use std::collections::HashSet;

use crate::r#type::{SourceType, Type};

use super::Visitor;

#[derive(Default)]
pub struct TypeVariablesGathered<'ast> {
    syntactic: HashSet<&'ast String>,
    skolem: HashSet<&'ast String>,
    unification: HashSet<&'ast i32>,
}

impl<'ast> TypeVariablesGathered<'ast> {
    pub fn contains_syntactic_variable(&self, value: String) -> bool {
        self.syntactic.contains(&value)
    }
    pub fn contains_skolem_variable(&self, value: String) -> bool {
        self.skolem.contains(&value)
    }
    pub fn contains_unification_variable(&self, value: i32) -> bool {
        self.unification.contains(&value)
    }
}

#[derive(Default)]
pub struct TypeVariableVisitor<'ast> {
    variables: TypeVariablesGathered<'ast>,
    in_scope: HashSet<&'ast String>,
}

impl<'ast> TypeVariableVisitor<'ast> {
    pub fn from_type(t: &'ast SourceType) -> TypeVariablesGathered {
        let mut this = Self::default();
        this.visit_type(t);
        this.variables
    }
}

impl<'ast> Visitor<'ast> for TypeVariableVisitor<'ast> {
    fn visit_type<Ann>(&mut self, t: &'ast Type<Ann>) {
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
                self.in_scope.insert(name);
                super::walk_type(self, r#type);
                self.in_scope.remove(name);
            }
            Type::Skolem { ann: _, name } => {
                self.variables.skolem.insert(name);
            }
            Type::Unsolved { ann: _, name } => {
                self.variables.unification.insert(name);
            }
            Type::Variable { ann: _, name } => {
                if !self.in_scope.contains(name) {
                    self.variables.syntactic.insert(name);
                }
            }
            Type::Constructor { ann: _, name: _ } => (),
            Type::Application {
                ann: _,
                variant: _,
                function,
                argument,
            } => {
                super::walk_type(self, function);
                super::walk_type(self, argument);
            }
        }
    }

    fn visit_expr<Ann>(&mut self, e: &'ast crate::expr::Expr<Ann>) {
        super::walk_expr(self, e)
    }
}
