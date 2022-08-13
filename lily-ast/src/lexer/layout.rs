use super::types::{IdentifierK, LayoutK, OperatorK, Position, Token, TokenK};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelimiterK {
    KwAdo,
    KwCase,
    KwDo,
    KwIf,
    KwLetExpr,
    KwLetStmt,
    KwOf,
    KwThen,
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

pub(crate) struct LayoutEngine {
    delimiters: Vec<(Position, DelimiterK)>,
    pub depth: usize,
}

impl LayoutEngine {
    pub(crate) fn new(initial_position: Position) -> Self {
        let delimiters = vec![(initial_position, DelimiterK::MaskRoot)];
        let depth = 0;
        Self { delimiters, depth }
    }

    #[inline]
    fn push_end(&mut self, tokens: &mut Vec<Token>, current_token: Token) {
        let begin = current_token.begin;
        let end = current_token.begin;
        let depth = self.depth;
        tokens.push(Token {
            comment_begin: begin,
            comment_end: begin,
            begin,
            end,
            kind: TokenK::Layout(LayoutK::Separator),
            depth,
        });
        tokens.push(Token {
            comment_begin: begin,
            comment_end: begin,
            begin,
            end,
            kind: TokenK::Layout(LayoutK::End),
            depth,
        });
        self.depth = self.depth.saturating_sub(1);
    }

    #[inline]
    fn determine_end(&self, predicate: impl Fn(&Position, &DelimiterK) -> bool) -> (usize, usize) {
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

    #[inline]
    fn add_begin(
        &mut self,
        tokens: &mut Vec<Token>,
        current_token: Token,
        next_position: Position,
        delimiter: DelimiterK,
    ) {
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
        self.depth += 1;
        tokens.push(Token {
            comment_begin: current_token.end,
            comment_end: current_token.end,
            begin: current_token.end,
            end: current_token.end,
            kind: TokenK::Layout(LayoutK::Begin),
            depth: self.depth,
        });
    }

    #[inline]
    fn add_separator(
        &mut self,
        tokens: &mut Vec<Token>,
        current_token: Token,
        now_position: Position,
    ) {
        if let Some((position, delimiter)) = self.delimiters.last() {
            if delimiter.is_indented()
                && now_position.column == position.column
                && now_position.line > position.line
            {
                tokens.push(Token {
                    comment_begin: current_token.begin,
                    comment_end: current_token.begin,
                    begin: current_token.begin,
                    end: current_token.begin,
                    kind: TokenK::Layout(LayoutK::Separator),
                    depth: self.depth,
                });
                if let DelimiterK::KwOf = delimiter {
                    self.delimiters.push((now_position, DelimiterK::MaskPat));
                }
            }
        }
    }

    #[inline]
    fn add_end(&mut self, tokens: &mut Vec<Token>, current_token: Token, now_position: Position) {
        let (take_n, make_n) = self.determine_end(|position, delimiter| {
            delimiter.is_indented() && now_position.column < position.column
        });
        self.delimiters.truncate(take_n);
        for _ in 0..make_n {
            self.push_end(tokens, current_token);
        }
    }

    #[inline]
    pub(crate) fn add_layout(
        &mut self,
        tokens: &mut Vec<Token>,
        current_token: Token,
        now_position: Position,
        next_position: Position,
    ) {
        use DelimiterK::*;
        use IdentifierK::*;
        use OperatorK::*;
        use TokenK::*;

        macro_rules! with_end {
            ($predicate:expr, $($commit:literal ~ $pattern:pat $(if $guard:expr)? => $expression:expr,)+) => {
                {
                    let (take_n, make_n) = self.determine_end($predicate);
                    match &self.delimiters[..take_n] {
                        $($pattern $(if $guard)? => {
                            if $commit {
                                self.delimiters.truncate(take_n);
                                for _ in 0..make_n {
                                    self.push_end(tokens, current_token);
                                }
                            };
                            $expression
                        }),+
                    }
                }
            };
        }

        match current_token.kind {
            Operator(Bang | Pipe | Question) => {
                tokens.push(current_token.with_depth(self.depth));
                self.add_begin(tokens, current_token, next_position, MaskTop);
            }
            Identifier(Case) => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                self.delimiters.push((now_position, KwCase));
                tokens.push(current_token.with_depth(self.depth));
            }
            Identifier(Of) => with_end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwCase)] => {
                    self.delimiters.pop();
                    tokens.push(current_token.with_depth(self.depth));
                    self.add_begin(tokens, current_token, next_position, KwOf);
                    self.delimiters.push((next_position, MaskPat));
                },
                true ~ _ => {
                    self.add_end(tokens, current_token, now_position);
                    self.add_separator(tokens, current_token, now_position);
                    tokens.push(current_token.with_depth(self.depth));
                },
            ),
            Operator(Backslash) => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                self.delimiters.push((now_position, MaskLam));
                tokens.push(current_token.with_depth(self.depth));
            }
            Operator(ArrowRight) => with_end!(
                |position, delimiter| {
                    match delimiter {
                        KwDo => true,
                        KwOf => false,
                        _ => {
                            delimiter.is_indented() && now_position.column <= position.column
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
                    tokens.push(current_token.with_depth(self.depth));
                },
            ),
            Identifier(Ado) => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                tokens.push(current_token.with_depth(self.depth));
                self.add_begin(tokens, current_token, next_position, KwAdo);
            }
            Identifier(Do) => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                tokens.push(current_token.with_depth(self.depth));
                self.add_begin(tokens, current_token, next_position, KwDo);
            }
            Identifier(If) => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                self.delimiters.push((now_position, KwIf));
                tokens.push(current_token.with_depth(self.depth));
            }
            Identifier(Then) => with_end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwIf)] => {
                    self.delimiters.pop();
                    self.delimiters.push((now_position, KwThen));
                    tokens.push(current_token.with_depth(self.depth));
                },
                false ~ _ => {
                    self.add_end(tokens, current_token, now_position);
                    self.add_separator(tokens, current_token, now_position);
                    tokens.push(current_token.with_depth(self.depth));
                },
            ),
            Identifier(Else) => with_end!(
                |_, delimiter| delimiter.is_indented(),
                true ~ [.., (_, KwThen)] => {
                    self.delimiters.pop();
                    tokens.push(current_token.with_depth(self.depth));
                },
                false ~ _ => {
                    self.add_end(tokens, current_token, now_position);
                    self.add_separator(tokens, current_token, now_position);
                    tokens.push(current_token.with_depth(self.depth));
                },
            ),
            Identifier(Let) => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                tokens.push(current_token.with_depth(self.depth));
                self.add_begin(
                    tokens,
                    current_token,
                    next_position,
                    match self.delimiters.last() {
                        Some((_, KwAdo | KwDo)) => KwLetStmt,
                        _ => KwLetExpr,
                    },
                );
            }
            Identifier(In) => with_end!(
                |_, delimiter| {
                    match delimiter {
                        KwAdo | KwLetExpr => false,
                        _ => delimiter.is_indented(),
                    }
                },
                true ~ [.., (_, KwAdo | KwLetExpr)] => {
                    self.delimiters.pop();
                    self.push_end(tokens, current_token);
                    tokens.push(current_token.with_depth(self.depth));
                },
                false ~ _ => {
                    self.add_end(tokens, current_token, now_position);
                    self.add_separator(tokens, current_token, now_position);
                    tokens.push(current_token.with_depth(self.depth));
                },
            ),
            _ => {
                self.add_end(tokens, current_token, now_position);
                self.add_separator(tokens, current_token, now_position);
                tokens.push(current_token.with_depth(self.depth));
            }
        }
    }

    pub(crate) fn finalize_layout(&mut self, tokens: &mut Vec<Token>, eof_offset: usize) {
        while let Some((_, delimiter)) = self.delimiters.pop() {
            if let DelimiterK::MaskRoot = delimiter {
                tokens.push(Token {
                    comment_begin: eof_offset,
                    comment_end: eof_offset,
                    begin: eof_offset,
                    end: eof_offset,
                    kind: TokenK::Layout(LayoutK::Separator),
                    depth: self.depth,
                });
            } else if delimiter.is_indented() {
                tokens.push(Token {
                    comment_begin: eof_offset,
                    comment_end: eof_offset,
                    begin: eof_offset,
                    end: eof_offset,
                    kind: TokenK::Layout(LayoutK::Separator),
                    depth: self.depth,
                });
                tokens.push(Token {
                    comment_begin: eof_offset,
                    comment_end: eof_offset,
                    begin: eof_offset,
                    end: eof_offset,
                    kind: TokenK::Layout(LayoutK::End),
                    depth: self.depth,
                });
                self.depth = self.depth.saturating_sub(1);
            }
        }
    }
}
