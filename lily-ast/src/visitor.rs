use super::expr::{Expr, Literal};
use super::r#type::Type;

pub trait Visitor: Sized {
    fn visit_expr<Ann>(&mut self, e: &Expr<Ann>) {
        walk_expr(self, e)
    }

    fn visit_type<Ann>(&mut self, t: &Type<Ann>) {
        walk_type(self, t)
    }
}

fn walk_expr<V, Ann>(visitor: &mut V, e: &Expr<Ann>)
where
    V: Visitor,
{
    match e {
        Expr::Literal { ann: _, literal } => match literal {
            Literal::Character(_) => (),
            Literal::String(_) => (),
            Literal::Int(_) => (),
            Literal::Float(_) => (),
            Literal::Array(es) => {
                for e in es.iter() {
                    visitor.visit_expr(e);
                }
            }
            Literal::Object(es) => {
                for (_, e) in es.iter() {
                    visitor.visit_expr(e);
                }
            }
        },
        Expr::Variable { ann: _, name: _ } => (),
        Expr::Lambda {
            ann: _,
            argument: _,
            expr,
        } => visitor.visit_expr(expr),
        Expr::Application {
            ann: _,
            function,
            argument,
        } => {
            visitor.visit_expr(function);
            visitor.visit_expr(argument);
        }
        Expr::Annotation {
            ann: _,
            expr,
            r#type,
            checked: _,
        } => {
            visitor.visit_expr(expr);
            visitor.visit_type(r#type);
        }
        Expr::Let {
            ann: _,
            name: _,
            value,
            expr,
        } => {
            visitor.visit_expr(value);
            visitor.visit_expr(expr);
        }
    }
}

fn walk_type<V, Ann>(visitor: &mut V, t: &Type<Ann>)
where
    V: Visitor,
{
    match t {
        Type::Forall {
            ann: _,
            name: _,
            kind,
            r#type,
        } => {
            if let Some(kind) = kind.as_ref() {
                visitor.visit_type(kind)
            };
            visitor.visit_type(r#type);
        }
        Type::Skolem { ann: _, name: _ } => {}
        Type::Unsolved { ann: _, name: _ } => {}
        Type::Variable { ann: _, name: _ } => {}
        Type::Constructor { ann: _, name: _ } => {}
        Type::Application {
            ann: _,
            variant: _,
            function,
            argument,
        } => {
            visitor.visit_type(function);
            visitor.visit_type(argument);
        }
    }
}
