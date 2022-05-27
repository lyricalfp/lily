use std::rc::Rc;

use lily_ast::r#type::Type;

use super::{context::Element, state::State};

type SourceType = Type<()>;

pub fn solve(state: &mut State, u: i32, t: Rc<SourceType>) -> Result<(), String> {
    match t.as_ref() {
        Type::Forall {
            ann: _,
            name: _,
            kind: _,
            r#type: _,
        } => Err("solve: attempted to solve into a polytype which violates predicativity".into()),
        Type::Skolem { ann: _, name: _ } => Ok(state.context.unsolved_with(u, t)),
        Type::Unsolved { ann: _, name: v } => {
            if state.context.unsolved_order(u, *v) {
                Ok(state
                    .context
                    .unsolved_with(*v, Rc::new(Type::Unsolved { ann: (), name: u })))
            } else {
                Ok(state.context.unsolved_with(u, t))
            }
        }
        Type::Variable { ann: _, name: _ } => Ok(state.context.unsolved_with(u, t)),
        Type::Constructor { ann: _, name: _ } => Ok(state.context.unsolved_with(u, t)),
        Type::Application {
            ann: _,
            variant,
            function,
            argument,
        } => {
            let kind = Rc::new(Some(Type::Constructor {
                ann: (),
                name: "Type".into(),
            }));

            let function_name = state.fresh.fresh() as i32;
            let function_type = Rc::new(Type::Unsolved {
                ann: (),
                name: function_name,
            });
            let function_elem = Box::new(Element::Unsolved {
                name: function_name,
                kind: Rc::clone(&kind),
            });

            let argument_name = state.fresh.fresh() as i32;
            let argument_type = Rc::new(Type::Unsolved {
                ann: (),
                name: argument_name,
            });
            let argument_elem = Box::new(Element::Unsolved {
                name: argument_name,
                kind: Rc::clone(&kind),
            });

            let application_type = Rc::new(Type::Application {
                ann: (),
                variant: *variant,
                function: Rc::clone(&function_type),
                argument: Rc::clone(&argument_type),
            });
            let application_elem = Box::new(Element::Solved {
                name: u,
                kind: Rc::clone(&kind),
                r#type: Rc::clone(&application_type),
            });

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
