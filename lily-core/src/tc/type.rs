use std::rc::Rc;

use lily_ast::r#type::Type;

use super::context::Element;
use super::state::State;

type SourceType = Type<()>;

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
            let kind = Rc::new(Some(Type::Constructor {
                ann: *ann,
                name: "Type".into(),
            }));

            let (function_name, function_type, function_elem) = state.fresh_unsolved(&kind);
            let (argument_name, argument_type, argument_elem) = state.fresh_unsolved(&kind);

            let application_type = Rc::new(Type::Application {
                ann: *ann,
                variant: *variant,
                function: Rc::clone(&function_type),
                argument: Rc::clone(&argument_type),
            });
            let application_elem = Element::Solved {
                name: u,
                kind: Rc::clone(&kind),
                r#type: Rc::clone(&application_type),
            };

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
