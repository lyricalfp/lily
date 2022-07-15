use std::{collections::VecDeque, iter::Peekable};

use super::cursor::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

use crate::lines::{Lines, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterK {
    KwAdo,
    KwCase,
    KwDo,
    KwIf,
    KwLetExpr,
    KwLetStmt,
    KwOf,
    KwThen,
    MaskElse,
    MaskLam,
    MaskPat,
    MaskRoot,
    MaskTop,
}

impl DelimiterK {
    fn is_indented(&self) -> bool {
        use DelimiterK::*;
        matches!(
            &self,
            KwAdo | KwDo | KwLetExpr | KwLetStmt | KwOf | MaskRoot | MaskTop
        )
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
        let delimiters = vec![(start, DelimiterK::MaskRoot)];
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
        if let Some((position, delimiter)) = self.delimiters.last() {
            if delimiter.is_indented()
                && current_position.column == position.column
                && current_position.line > position.line
            {
                self.token_queue.push_front(Token {
                    begin: self.current.end,
                    end: self.current.end,
                    kind: TokenK::Layout(LayoutK::Separator),
                });
                if let DelimiterK::KwOf = delimiter {
                    self.delimiters.push((
                        self.lines.get_position(self.current.begin),
                        DelimiterK::MaskPat,
                    ));
                }
            }
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
        while let Some((_, delimiter)) = self.delimiters.pop() {
            if let DelimiterK::MaskRoot = delimiter {
                self.token_queue.push_front(Token {
                    begin: self.current.end,
                    end: self.current.end,
                    kind: TokenK::Layout(LayoutK::Separator),
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
        use IdentifierK::*;
        use OperatorK::*;
        use TokenK::*;

        macro_rules! end {
            ($predicate:expr, $($commit:literal ~ $pattern:pat $(if $guard:expr)? => $expression:expr,)+) => {
                {
                    let (take_n, make_n) = self.determine_end($predicate);
                    match &self.delimiters[..take_n] {
                        $($pattern $(if $guard)? => {
                            if $commit {
                                self.delimiters.truncate(take_n);
                                for _ in 0..make_n {
                                    self.token_queue.push_front(Token {
                                        begin: self.current.begin,
                                        end: self.current.begin,
                                        kind: TokenK::Layout(LayoutK::End),
                                    });
                                }
                            };
                            $expression
                        }),+
                    }
                }
            };
        }

        match self.current.kind {
            Operator(Bang | Pipe | Question) => {
                self.insert_current();
                self.insert_begin(MaskTop);
            }
            Identifier(Case) => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
                self.delimiters
                    .push((self.lines.get_position(self.current.begin), KwCase));
            }
            Identifier(Of) => end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwCase)] => {
                    self.delimiters.pop();
                    self.insert_current();
                    self.insert_begin(KwOf);
                    let next = self.tokens.peek().expect("non-eof");
                    self.delimiters
                        .push((self.lines.get_position(next.begin), MaskPat));
                },
                true ~ _ => {
                    self.insert_end();
                    self.insert_separator();
                    self.insert_current();
                },
            ),
            Operator(Backslash) => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
                self.delimiters
                    .push((self.lines.get_position(self.current.begin), MaskLam));
            }
            Operator(ArrowRight) => end!(
                |position, delimiter| {
                    match delimiter {
                        KwDo => true,
                        KwOf => false,
                        _ => {
                            let current_position = self.lines.get_position(self.current.begin);
                            delimiter.is_indented() && current_position.column <= position.column
                        },
                    }
                },
                true ~ _ => {
                    self.insert_current();
                    if let Some((_, KwIf)) = self.delimiters.last() {
                        self.delimiters.pop();
                    }
                    if let Some((_, MaskLam | MaskPat)) = self.delimiters.last() {
                        self.delimiters.pop();
                    }
                },
            ),
            Identifier(Ado) => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
                self.insert_begin(KwAdo);
            }
            Identifier(Do) => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
                self.insert_begin(KwDo);
            }
            Identifier(If) => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
                self.delimiters
                    .push((self.lines.get_position(self.current.begin), KwIf));
            }
            Identifier(Then) => end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwIf)] => {
                    self.delimiters.pop();
                    self.insert_current();
                    self.delimiters
                        .push((self.lines.get_position(self.current.begin), KwThen));
                },
                false ~ _ => {
                    self.insert_end();
                    self.insert_separator();
                    self.insert_current();
                },
            ),
            Identifier(Else) => end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwThen)] => {
                    self.delimiters.pop();
                    self.insert_current();
                },
                false ~ _ => {
                    self.insert_end();
                    self.insert_separator();
                    self.insert_current();
                },
            ),
            Identifier(Let) => {
                self.insert_end();
                self.insert_separator();
                self.insert_current();
                self.insert_begin(match self.delimiters.last() {
                    Some((_, KwAdo | KwDo)) => KwLetStmt,
                    _ => KwLetExpr,
                });
            }
            Identifier(In) => end!(
                |_, delimiter| {
                    match delimiter {
                        KwAdo | KwLetExpr => false,
                        _ => delimiter.is_indented(),
                    }
                },
                true ~ [.., (_, KwAdo | KwLetExpr)] => {
                    self.delimiters.pop();
                    self.token_queue.push_front(Token {
                        begin: self.current.begin,
                        end: self.current.begin,
                        kind: Layout(LayoutK::End),
                    });
                    self.insert_current();
                },
                false ~ _ => {
                    self.insert_end();
                    self.insert_separator();
                    self.insert_current();
                },
            ),
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
