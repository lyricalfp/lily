use std::rc::Rc;

use lily_ast::r#type::Type;

use super::{
    context::{Context, Element},
    fresh::Fresh,
};

type SourceType = Type<()>;

pub struct State {
    pub context: Context,
    fresh: Fresh,
}

pub type FreshUnsolved = (i32, Rc<SourceType>, Box<Element>);

impl State {
    pub fn fresh_unsolved(&mut self, kind: &Rc<Option<SourceType>>) -> FreshUnsolved {
        let name = self.fresh.fresh();
        let r#type = Rc::new(Type::Unsolved { ann: (), name });
        let elem = Box::new(Element::Unsolved {
            name,
            kind: Rc::clone(kind),
        });
        (name, r#type, elem)
    }
}
