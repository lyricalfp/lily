pub struct Fresh {
    index: usize,
}

impl Default for Fresh {
    fn default() -> Self {
        Fresh { index: 0 }
    }
}

impl Fresh {
    pub fn from(index: usize) -> Self {
        Fresh { index }
    }

    pub fn fresh(&mut self) -> usize {
        let index = self.index;
        self.index += 1;
        index
    }
}
