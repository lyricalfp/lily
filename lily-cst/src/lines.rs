#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Lines<'a> {
    source: &'a str,
}

impl<'a> Lines<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn get_position(&self, offset: usize) -> Position {
        let mut line = 1;
        let mut column = 1;

        for character in self.source[..offset].chars() {
            if character == '\n' {
                column = 1;
                line += 1
            } else {
                column += 1;
            }
        }

        Position {
            line,
            column,
        }
    }

    pub fn eof_offset(&self) -> usize {
        self.source.len()
    }
}
