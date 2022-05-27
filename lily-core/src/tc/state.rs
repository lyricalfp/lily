use super::{context::Context, fresh::Fresh};

pub struct State {
    pub context: Context,
    pub fresh: Fresh,
}
