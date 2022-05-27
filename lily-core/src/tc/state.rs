use std::rc::Rc;

use lily_ast::{r#type::{Type, SourceType}, ann::SourceAnn};

use super::{
    context::{Context, Element},
    fresh::Fresh,
};

pub struct State {
    pub context: Context,
    fresh: Fresh,
}

pub type FreshUnsolved = (i32, Rc<SourceType>, Element);

impl State {
    pub fn fresh_unsolved(&mut self, ann: SourceAnn, kind: &Rc<Option<SourceType>>) -> FreshUnsolved {
        let name = self.fresh.fresh();
        let r#type = Rc::new(Type::Unsolved { ann, name });
        let elem = Element::Unsolved {
            name,
            kind: Rc::clone(kind),
        };
        (name, r#type, elem)
    }
}
