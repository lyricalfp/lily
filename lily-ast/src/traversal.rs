use std::rc::Rc;

use super::expr::{Expr, Literal};
use super::r#type::Type;

pub mod common;

pub trait Traversal<Ann>: Sized {
    fn traverse_expr(&mut self, e: Rc<Expr<Ann>>) -> Rc<Expr<Ann>>
    where
        Ann: Copy,
    {
        walk_expr(self, e)
    }

    fn traverse_type(&mut self, t: Rc<Type<Ann>>) -> Rc<Type<Ann>>
    where
        Ann: Copy,
    {
        walk_type(self, t)
    }
}

pub fn walk_expr<T, Ann>(traversal: &mut T, e: Rc<Expr<Ann>>) -> Rc<Expr<Ann>>
where
    T: Traversal<Ann>,
    Ann: Copy,
{
    match e.as_ref() {
        Expr::Literal { ann, literal } => match literal {
            Literal::Character(_) => e,
            Literal::String(_) => e,
            Literal::Int(_) => e,
            Literal::Float(_) => e,
            Literal::Array(es) => Rc::new(Expr::Literal {
                ann: *ann,
                literal: Literal::Array(
                    es.iter()
                        .map(|e| traversal.traverse_expr(Rc::clone(e)))
                        .collect(),
                ),
            }),
            Literal::Object(es) => Rc::new(Expr::Literal {
                ann: *ann,
                literal: Literal::Object(
                    es.iter()
                        .map(|(k, e)| (k.to_string(), traversal.traverse_expr(Rc::clone(e))))
                        .collect(),
                ),
            }),
        },
        Expr::Variable { ann: _, name: _ } => e,
        Expr::Lambda {
            ann,
            argument,
            expr,
        } => Rc::new(Expr::Lambda {
            ann: *ann,
            argument: argument.to_string(),
            expr: traversal.traverse_expr(Rc::clone(expr)),
        }),
        Expr::Application {
            ann,
            function,
            argument,
        } => Rc::new(Expr::Application {
            ann: *ann,
            function: traversal.traverse_expr(Rc::clone(function)),
            argument: traversal.traverse_expr(Rc::clone(argument)),
        }),
        Expr::Annotation {
            ann,
            expr,
            r#type,
            checked,
        } => Rc::new(Expr::Annotation {
            ann: *ann,
            expr: traversal.traverse_expr(Rc::clone(expr)),
            r#type: traversal.traverse_type(Rc::clone(r#type)),
            checked: *checked,
        }),
        Expr::Let {
            ann,
            name,
            value,
            expr,
        } => Rc::new(Expr::Let {
            ann: *ann,
            name: name.to_string(),
            value: traversal.traverse_expr(Rc::clone(value)),
            expr: traversal.traverse_expr(Rc::clone(expr)),
        }),
    }
}

pub fn walk_type<T, Ann>(traversal: &mut T, t: Rc<Type<Ann>>) -> Rc<Type<Ann>>
where
    T: Traversal<Ann>,
    Ann: Copy,
{
    match t.as_ref() {
        Type::Forall {
            ann,
            name,
            kind,
            r#type,
        } => Rc::new(Type::Forall {
            ann: *ann,
            name: name.to_string(),
            kind: kind.as_ref().map(|t| traversal.traverse_type(Rc::clone(t))),
            r#type: traversal.traverse_type(Rc::clone(r#type)),
        }),
        Type::Skolem { ann: _, name: _ } => t,
        Type::Unsolved { ann: _, name: _ } => t,
        Type::Variable { ann: _, name: _ } => t,
        Type::Constructor { ann: _, name: _ } => t,
        Type::Application {
            ann,
            variant,
            function,
            argument,
        } => Rc::new(Type::Application {
            ann: *ann,
            variant: *variant,
            function: traversal.traverse_type(Rc::clone(function)),
            argument: traversal.traverse_type(Rc::clone(argument)),
        }),
        Type::Function {
            ann,
            argument,
            result,
        } => Rc::new(Type::Function {
            ann: *ann,
            argument: traversal.traverse_type(Rc::clone(argument)),
            result: traversal.traverse_type(Rc::clone(result)),
        }),
    }
}
