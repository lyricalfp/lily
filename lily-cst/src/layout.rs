use std::{collections::VecDeque, iter::Peekable};

use crate::{
    cursor::{LayoutK, OperatorK, Token, TokenK},
    lines::{Lines, Position},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterK {
    Root,
    TopLevel,
}

impl DelimiterK {
    fn is_indented(&self) -> bool {
        use DelimiterK::*;
        matches!(&self, Root | TopLevel)
    }
}

pub struct Layout<'a, I: Iterator> {
    lines: Lines<'a>,
    tokens: Peekable<I>,
    current: Token,
    delimiters: Vec<(Position, DelimiterK)>,
    token_queue: VecDeque<Token>,
    should_keep_going: bool,
}

impl<'a, I> Layout<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(lines: Lines<'a>, tokens: I) -> Self {
        let mut tokens = tokens.peekable();
        let start = lines.get_position(tokens.peek().expect("non-empty tokens").begin);
        let current = tokens.next().expect("non-empty tokens");
        let delimiters = vec![(start, DelimiterK::Root)];
        let token_queue = VecDeque::default();
        let should_keep_going = true;
        Self {
            lines,
            tokens,
            current,
            delimiters,
            token_queue,
            should_keep_going,
        }
    }
}

impl<'a, I> Layout<'a, I>
where
    I: Iterator<Item = Token>,
{
    fn determine_end<F>(&self, predicate: F) -> (usize, usize)
    where
        F: Fn(&Position, &DelimiterK) -> bool,
    {
        let mut take_n = self.delimiters.len();
        let mut make_n = 0;

        for (position, delimiter) in self.delimiters.iter().rev() {
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
        self.token_queue.push_front(self.current);
    }

    fn insert_begin(&mut self, delimiter: DelimiterK) {
        let next_offset = self.tokens.peek().expect("non-eof").begin;
        let next_position = self.lines.get_position(next_offset);

        let recent_indented = self
            .delimiters
            .iter()
            .rfind(|(_, delimiter)| delimiter.is_indented());

        if let Some((past, _)) = recent_indented {
            if next_position.column <= past.column {
                return;
            }
        }

        self.delimiters.push((next_position, delimiter));
        self.token_queue.push_front(Token {
            begin: self.current.end,
            end: self.current.end,
            kind: TokenK::Layout(LayoutK::Begin),
        });
    }

    fn insert_separator(&mut self) {
        let current_position = self.lines.get_position(self.current.begin);
        match self.delimiters.last() {
            Some((position, delimiter)) => {
                if delimiter.is_indented()
                    && current_position.column == position.column
                    && current_position.line > position.line
                {
                    self.token_queue.push_front(Token {
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
        let current_position = self.lines.get_position(self.current.begin);
        let (take_n, make_n) = self.determine_end(|position, delimiter| {
            delimiter.is_indented() && current_position.column < position.column
        });
        self.delimiters.truncate(take_n);
        for _ in 0..make_n {
            self.token_queue.push_front(Token {
                begin: current_position.offset,
                end: current_position.offset,
                kind: TokenK::Layout(LayoutK::End),
            });
        }
    }

    fn insert_final(&mut self) {
        let eof_offset = self.lines.eof_offset();
        while let Some((_, delimiter)) = self.delimiters.pop() {
            if let DelimiterK::Root = delimiter {
                self.token_queue.push_front(Token {
                    begin: self.current.end,
                    end: self.current.end,
                    kind: TokenK::Layout(LayoutK::Separator),
                });
                self.token_queue.push_front(Token {
                    begin: eof_offset,
                    end: eof_offset,
                    kind: TokenK::Eof,
                });
            } else if delimiter.is_indented() {
                self.token_queue.push_front(Token {
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

impl<'a, I> Iterator for Layout<'a, I>
where
    I: Iterator<Item = Token>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.should_keep_going {
            self.insert_layout();
            if let Some(current) = self.tokens.next() {
                self.current = current
            } else {
                self.insert_final();
                self.should_keep_going = false;
            }
            self.token_queue.pop_back()
        } else {
            self.token_queue.pop_back()
        }
    }
}
