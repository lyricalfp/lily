use std::rc::Rc;

use lily_ast::{make_application, r#type::Type};

use crate::make_solved;

use super::context::Element;
use super::state::State;

type SourceType = Type<()>;

macro_rules! make_some_type_type {
    ($ann:expr) => {
        Rc::new(Some(Type::Constructor {
            ann: *$ann,
            name: "Type".into(),
        }))
    };
}

pub fn solve(state: &mut State, u: i32, t: Rc<SourceType>) -> Result<(), String> {
    match t.as_ref() {
        // Trivial
        Type::Skolem { ann: _, name: _ }
        | Type::Variable { ann: _, name: _ }
        | Type::Constructor { ann: _, name: _ } => {
            state.context.unsolved_with(u, t);
            Ok(())
        }
        // Disallowed
        Type::Forall {
            ann: _,
            name: _,
            kind: _,
            r#type: _,
        } => Err("solve: attempted to solve into a polytype which violates predicativity".into()),

        // Allowed
        Type::Unsolved { ann: _, name: v } => {
            if state.context.unsolved_order(u, *v) {
                state
                    .context
                    .unsolved_with(*v, Rc::new(Type::Unsolved { ann: (), name: u }));
                Ok(())
            } else {
                state.context.unsolved_with(u, t);
                Ok(())
            }
        }

        Type::Application {
            ann,
            variant,
            function,
            argument,
        } => {
            let kind = make_some_type_type!(ann);

            let (function_name, function_type, function_elem) = state.fresh_unsolved(&kind);
            let (argument_name, argument_type, argument_elem) = state.fresh_unsolved(&kind);

            let application_type = make_application!(ann, variant, function_type, argument_type);
            let application_elem = make_solved!(u, kind, application_type);

            let e = vec![argument_elem, function_elem, application_elem];

            state.context.unsolved_with_elems(u, e);
            solve(state, function_name, Rc::clone(function))?;
            solve(
                state,
                argument_name,
                state.context.apply(Rc::clone(argument)),
            )
        }
    }
}
