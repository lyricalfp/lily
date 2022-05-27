#[derive(Default)]
pub struct Fresh {
    index: i32,
}

impl Fresh {
    pub fn from(index: i32) -> Self {
        Fresh { index }
    }

    pub fn fresh(&mut self) -> i32 {
        let index = self.index;
        self.index += 1;
        index
    }
}
