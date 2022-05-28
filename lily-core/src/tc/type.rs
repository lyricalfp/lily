use std::rc::Rc;

use lily_ast::ann::SourceAnn;
use lily_ast::r#type::{SourceType, Type};

use super::context::Element;
use super::state::State;

pub fn subsumes(state: &mut State, t1: Rc<SourceType>, t2: Rc<SourceType>) -> Result<(), String> {
    match (t1.as_ref(), t2.as_ref()) {
        (
            Type::Function {
                ann: _,
                argument: t1_argument,
                result: t1_result,
            },
            Type::Function {
                ann: _,
                argument: t2_argument,
                result: t2_result,
            },
        ) => {
            subsumes(state, Rc::clone(t2_argument), Rc::clone(t1_argument))?;
            subsumes(
                state,
                state.context.apply(Rc::clone(t1_result)),
                state.context.apply(Rc::clone(t2_result)),
            )
        }
        (
            _,
            Type::Forall {
                ann: _,
                name: _,
                kind: _,
                r#type: _,
            },
        ) => {
            todo!()
        }
        (
            Type::Forall {
                ann: _,
                name: _,
                kind: _,
                r#type: _,
            },
            _,
        ) => {
            todo!()
        }
        _ => unify(state, t1, t2),
    }
}

pub fn unify(_state: &mut State, _t1: Rc<SourceType>, _t2: Rc<SourceType>) -> Result<(), String> {
    todo!()
}

pub fn solve(state: &mut State, a: SourceAnn, u: i32, t: Rc<SourceType>) -> Result<(), String> {
    match t.as_ref() {
        // Trivial
        Type::Skolem { ann: _, name: _ }
        | Type::Variable { ann: _, name: _ }
        | Type::Constructor { ann: _, name: _ } => {
            state.context.type_is_well_formed_before_unsolved(&t, u);
            state.context.unsafe_solve(u, t);
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
            if state.context.unsafe_unsolved_appears_before(u, *v) {
                state
                    .context
                    .unsafe_solve(*v, Rc::new(Type::Unsolved { ann: a, name: u }));
                Ok(())
            } else {
                state.context.unsafe_solve(u, t);
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

            let (function_name, function_type, function_elem) =
                state.fresh_unsolved(*ann, Rc::clone(&kind));
            let (argument_name, argument_type, argument_elem) =
                state.fresh_unsolved(*ann, Rc::clone(&kind));

            let application_elem = Element::Solved {
                name: u,
                kind: Rc::clone(&kind),
                r#type: Rc::new(Type::Application {
                    ann: *ann,
                    variant: *variant,
                    argument: argument_type,
                    function: function_type,
                }),
            };

            state.context.unsafe_unsolved_replace(
                u,
                Vec::from([argument_elem, function_elem, application_elem]),
            );
            solve(state, *ann, function_name, Rc::clone(function))?;
            solve(
                state,
                *ann,
                argument_name,
                state.context.apply(Rc::clone(argument)),
            )
        }

        Type::Function {
            ann,
            argument,
            result,
        } => {
            let kind = Rc::new(Some(Type::Constructor {
                ann: *ann,
                name: "Type".into(),
            }));

            let (argument_name, argument_type, argument_elem) =
                state.fresh_unsolved(*ann, Rc::clone(&kind));
            let (result_name, result_type, result_elem) =
                state.fresh_unsolved(*ann, Rc::clone(&kind));

            let application_elem = Element::Solved {
                name: u,
                kind: Rc::clone(&kind),
                r#type: Rc::new(Type::Function {
                    ann: *ann,
                    argument: argument_type,
                    result: result_type,
                }),
            };

            state.context.unsafe_unsolved_replace(
                u,
                Vec::from([result_elem, argument_elem, application_elem]),
            );
            solve(state, *ann, argument_name, Rc::clone(argument))?;
            solve(
                state,
                *ann,
                result_name,
                state.context.apply(Rc::clone(result)),
            )
        }
    }
}
