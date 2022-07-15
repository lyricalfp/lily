use std::iter::Peekable;

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

pub struct Layout<'a, I>
where
    I: Iterator,
{
    lines: Lines<'a>,
    tokens: Peekable<I>,
    current: Token,
    delimiters: Vec<(Position, DelimiterK)>,
    layouts: Vec<(usize, LayoutK)>,
}

impl<'a, I> Layout<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(lines: Lines<'a>, tokens: I) -> Self {
        let mut tokens = tokens.peekable();
        let current = tokens.next().expect("tokens must be non-empty");
        let delimiters = vec![(lines.get_position(current.begin), DelimiterK::MaskRoot)];
        let layouts = vec![];
        Self {
            lines,
            tokens,
            current,
            delimiters,
            layouts,
        }
    }

    pub fn iter(mut self) -> impl Iterator<Item = Token> {
        loop {
            self.insert_layout();
            if let Some(current) = self.tokens.next() {
                self.current = current;
            } else {
                self.insert_final();
                break;
            }
        }
        self.layouts.into_iter().map(|(offset, layout)| Token {
            begin: offset,
            end: offset,
            kind: TokenK::Layout(layout),
        })
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
        self.layouts.push((self.current.end, LayoutK::Begin));
    }

    fn insert_separator(&mut self) {
        let current_position = self.lines.get_position(self.current.begin);
        if let Some((position, delimiter)) = self.delimiters.last() {
            if delimiter.is_indented()
                && current_position.column == position.column
                && current_position.line > position.line
            {
                self.layouts.push((self.current.begin, LayoutK::Separator));
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
            self.layouts.push((self.current.begin, LayoutK::End))
        }
    }

    fn insert_final(&mut self) {
        let eof_offset = self.lines.eof_offset();
        while let Some((_, delimiter)) = self.delimiters.pop() {
            if let DelimiterK::MaskRoot = delimiter {
                self.layouts.push((eof_offset, LayoutK::Separator));
            } else if delimiter.is_indented() {
                self.layouts.push((eof_offset, LayoutK::End));
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
                                    self.layouts.push((self.current.begin, LayoutK::End));
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
                self.insert_begin(MaskTop);
            }
            Identifier(Case) => {
                self.insert_end();
                self.insert_separator();
                self.delimiters
                    .push((self.lines.get_position(self.current.begin), KwCase));
            }
            Identifier(Of) => end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwCase)] => {
                    self.delimiters.pop();
                    self.insert_begin(KwOf);
                    let next = self.tokens.peek().expect("non-eof");
                    self.delimiters
                        .push((self.lines.get_position(next.begin), MaskPat));
                },
                true ~ _ => {
                    self.insert_end();
                    self.insert_separator();

                },
            ),
            Operator(Backslash) => {
                self.insert_end();
                self.insert_separator();
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
                self.insert_begin(KwAdo);
            }
            Identifier(Do) => {
                self.insert_end();
                self.insert_separator();
                self.insert_begin(KwDo);
            }
            Identifier(If) => {
                self.insert_end();
                self.insert_separator();
                self.delimiters
                    .push((self.lines.get_position(self.current.begin), KwIf));
            }
            Identifier(Then) => end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwIf)] => {
                    self.delimiters.pop();
                    self.delimiters
                        .push((self.lines.get_position(self.current.begin), KwThen));
                },
                false ~ _ => {
                    self.insert_end();
                    self.insert_separator();
                },
            ),
            Identifier(Else) => end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwThen)] => {
                    self.delimiters.pop();

                },
                false ~ _ => {
                    self.insert_end();
                    self.insert_separator();

                },
            ),
            Identifier(Let) => {
                self.insert_end();
                self.insert_separator();
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
                    self.layouts.push((self.current.begin, LayoutK::End))

                },
                false ~ _ => {
                    self.insert_end();
                    self.insert_separator();
                },
            ),
            _ => {
                self.insert_end();
                self.insert_separator();
            }
        }
    }
}
