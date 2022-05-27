pub struct Fresh {
    index: i32,
}

impl Default for Fresh {
    fn default() -> Self {
        Fresh { index: 0 }
    }
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
