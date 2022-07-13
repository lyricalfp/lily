#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

pub struct Lines<'a> {
    source: &'a str,
    offsets: Vec<usize>,
}

impl<'a> Lines<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut offset = 0;
        let mut offsets = vec![];
        for line in source.split('\n') {
            offsets.push(offset);
            offset += line.len() + 1;
        }
        Self { source, offsets }
    }

    pub fn get_position(&self, offset: usize) -> Position {
        assert!(
            offset <= self.source.len(),
            "offset cannot be greater than source"
        );
        let closest_index = self
            .offsets
            .binary_search_by_key(&offset, |&offset| offset)
            .unwrap_or_else(|index| index.saturating_sub(1));
        let line_offset = self.offsets[closest_index];
        let line = closest_index + 1;
        let column = offset - line_offset + 1;
        Position {
            offset,
            line,
            column,
        }
    }

    pub fn eof_offset(&self) -> usize {
        self.source.len()
    }
}
