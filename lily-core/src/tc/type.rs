use std::rc::Rc;

use lily_ast::r#type::Type;

use super::{
    context::{Context, Element},
    fresh::Fresh,
};

type SourceType = Type<()>;

pub struct TypeChecker {
    context: Context,
    fresh: Fresh,
}

impl TypeChecker {
    pub fn solve(&mut self, u: i32, t: Rc<SourceType>) -> Result<(), String> {
        match t.as_ref() {
            Type::Forall {
                ann: _,
                name: _,
                kind: _,
                r#type: _,
            } => {
                Err("solve: attempted to solve into a polytype which violates predicativity".into())
            }
            Type::Skolem { ann: _, name: _ } => Ok(self.context.unsolved_with(u, t)),
            Type::Unsolved { ann: _, name: v } => {
                if self.context.unsolved_order(u, *v) {
                    Ok(self
                        .context
                        .unsolved_with(*v, Rc::new(Type::Unsolved { ann: (), name: u })))
                } else {
                    Ok(self.context.unsolved_with(u, t))
                }
            }
            Type::Variable { ann: _, name: _ } => Ok(self.context.unsolved_with(u, t)),
            Type::Constructor { ann: _, name: _ } => Ok(self.context.unsolved_with(u, t)),
            Type::Application {
                ann: _,
                function,
                argument,
            } => {
                let kind = Rc::new(Some(Type::Constructor {
                    ann: (),
                    name: "Type".into(),
                }));

                let function_name = self.fresh.fresh() as i32;
                let function_type = Rc::new(Type::Unsolved {
                    ann: (),
                    name: function_name,
                });
                let function_elem = Box::new(Element::Unsolved {
                    name: function_name,
                    kind: Rc::clone(&kind),
                });

                let argument_name = self.fresh.fresh() as i32;
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
                    function: Rc::clone(&function_type),
                    argument: Rc::clone(&argument_type),
                });
                let application_elem = Box::new(Element::Solved {
                    name: u,
                    kind: Rc::clone(&kind),
                    r#type: Rc::clone(&application_type),
                });

                let e = vec![argument_elem, function_elem, application_elem];

                self.context.unsolved_with_elems(u, e);
                self.solve(function_name, Rc::clone(function))?;
                self.solve(argument_name, self.context.apply(Rc::clone(argument)))
            }
            Type::KindApplication {
                ann: _,
                function,
                argument,
            } => {
                let kind = Rc::new(Some(Type::Constructor {
                    ann: (),
                    name: "Type".into(),
                }));

                let function_name = self.fresh.fresh() as i32;
                let function_type = Rc::new(Type::Unsolved {
                    ann: (),
                    name: function_name,
                });
                let function_elem = Box::new(Element::Unsolved {
                    name: function_name,
                    kind: Rc::clone(&kind),
                });

                let argument_name = self.fresh.fresh() as i32;
                let argument_type = Rc::new(Type::Unsolved {
                    ann: (),
                    name: argument_name,
                });
                let argument_elem = Box::new(Element::Unsolved {
                    name: argument_name,
                    kind: Rc::clone(&kind),
                });

                let application_type = Rc::new(Type::KindApplication {
                    ann: (),
                    function: Rc::clone(&function_type),
                    argument: Rc::clone(&argument_type),
                });
                let application_elem = Box::new(Element::Solved {
                    name: u,
                    kind: Rc::clone(&kind),
                    r#type: Rc::clone(&application_type),
                });

                let e = vec![argument_elem, function_elem, application_elem];

                self.context.unsolved_with_elems(u, e);
                self.solve(function_name, Rc::clone(function))?;
                self.solve(argument_name, self.context.apply(Rc::clone(argument)))
            }
        }
    }
}
