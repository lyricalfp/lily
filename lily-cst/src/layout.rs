use std::collections::VecDeque;

use peekmore::{PeekMore, PeekMoreIterator};

use crate::lexer::{LayoutK, Lexer, OperatorK, Token, TokenK};

#[derive(Debug)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum DelimiterK {
    Root,
    TopLevel,
}

impl DelimiterK {
    fn is_indented(&self) -> bool {
        use DelimiterK::*;
        matches!(&self, TopLevel)
    }
}

pub struct Engine<'a> {
    source: &'a str,
    offsets: Vec<usize>,
    lexer: PeekMoreIterator<Lexer<'a>>,
    current: Token,
    stack: Vec<(Position, DelimiterK)>,
    queue: VecDeque<Token>,
    keep_going: bool,
}

impl<'a> Engine<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut offset = 0;
        let mut offsets = vec![];
        for line in source.split('\n') {
            offsets.push(offset);
            offset += line.len() + 1
        }
        let mut lexer = Lexer::new(source).peekmore();
        let current = lexer.next().expect("non-empty lexer");
        let stack = vec![(
            Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            DelimiterK::Root,
        )];
        let queue = VecDeque::default();
        let keep_going = true;
        Self {
            source,
            offsets,
            lexer,
            current,
            stack,
            queue,
            keep_going,
        }
    }

    fn get_position(&self, offset: usize) -> Position {
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

    fn peek(&mut self) -> Token {
        let token = loop {
            match self.lexer.peek_next() {
                Some(token) => {
                    if let TokenK::Whitespace = token.kind {
                        continue;
                    } else {
                        break *token;
                    }
                }
                None => panic!("non-eof"),
            }
        };
        self.lexer.reset_cursor();
        token
    }
}

impl<'a> Engine<'a> {
    fn determine_end<F>(&self, predicate: F) -> (usize, usize)
    where
        F: Fn(&Position, &DelimiterK) -> bool,
    {
        let mut take_n = self.stack.len();
        let mut make_n = 0;

        for (position, delimiter) in self.stack.iter().rev() {
            if predicate(position, delimiter) {
                take_n = take_n.saturating_sub(1);
                if delimiter.is_indented() {
                    make_n += 1;
                }
            } else {
                break;
            }
        }

        (take_n, make_n)
    }

    fn insert_current(&mut self) {
        self.queue.push_front(self.current);
    }

    fn insert_begin(&mut self, delimiter: DelimiterK) {
        let next_offset = self.peek().begin;
        let next_position = self.get_position(next_offset);

        let recent_indented = self
            .stack
            .iter()
            .rfind(|(_, delimiter)| delimiter.is_indented());

        if let Some((past, _)) = recent_indented {
            if next_position.column <= past.column {
                return;
            }
        }

        self.stack.push((next_position, delimiter));
        self.queue.push_front(Token {
            begin: self.current.end,
            end: self.current.end,
            kind: TokenK::Layout(LayoutK::Begin),
        });
    }

    fn insert_separator(&mut self) {
        let current_position = self.get_position(self.current.begin);
        match self.stack.last() {
            Some((position, delimiter)) => {
                if delimiter.is_indented()
                    && current_position.column == position.column
                    && current_position.line != position.line
                {
                    self.queue.push_front(Token {
                        begin: self.current.end,
                        end: self.current.end,
                        kind: TokenK::Layout(LayoutK::Separator),
                    });
                }
            }
            _ => {}
        }
    }

    fn insert_end(&mut self) {
        let current_position = self.get_position(self.current.begin);
        let (take_n, make_n) = self.determine_end(|position, delimiter| {
            delimiter.is_indented() && current_position.column < position.column
        });
        self.stack.truncate(take_n);
        for _ in 0..make_n {
            self.queue.push_front(Token {
                begin: current_position.offset,
                end: current_position.offset,
                kind: TokenK::Layout(LayoutK::End),
            });
        }
    }

    fn insert_final(&mut self) {
        while let Some((_, delimiter)) = self.stack.pop() {
            if let DelimiterK::Root = delimiter {
                self.queue.push_front(Token {
                    begin: self.current.end,
                    end: self.current.end,
                    kind: TokenK::Eof,
                });
            } else if delimiter.is_indented() {
                self.queue.push_front(Token {
                    begin: self.current.end,
                    end: self.current.end,
                    kind: TokenK::Layout(LayoutK::End),
                });
            }
        }
    }

    fn insert_layout(&mut self) {
        use DelimiterK::*;
        use OperatorK::*;
        use TokenK::*;

        match self.current.kind {
            Operator(Bang | Pipe | Question) => {
                self.insert_current();
                self.insert_begin(TopLevel);
            }
            _ => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
            }
        }
    }
}

impl<'a> Iterator for Engine<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.keep_going {
            self.insert_layout();
            if let Some(current) = self.lexer.next() {
                self.current = current
            } else {
                self.insert_final();
                self.keep_going = false;
            }
            self.queue.pop_back()
        } else {
            self.queue.pop_back()
        }
    }
}

#[test]
fn it_works() {
    let source = r"
Identity a ?
  _ : a -> Identity a

  Equal a b !
    _ : a -> a -> True

Eq a |
  eq : a -> a -> Boolean
";
    let engine = Engine::new(source);
    for token in engine {
        if let TokenK::Layout(layout) = token.kind {
            match layout {
                LayoutK::Begin => print!("{{"),
                LayoutK::End => print!("}}"),
                LayoutK::Separator => print!(";"),
            }
        } else {
            print!("{}", &source[token.begin..token.end]);
        }
    }
    println!();
}
