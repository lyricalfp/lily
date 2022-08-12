//! Implements off-side rules.
//!
//! This module handles the insertion of [`LayoutK`] tokens into the
//! token stream, and in effect, allows the parser to be implemented
//! in terms of a context-free grammar. Likewise, this also adjusts
//! the [`depth`] field of each [`Token`], which is also used further
//! by the parser to disambiguate boundaries between declarations and
//! expressions.
//!
//! Since most of the implementation is specific to `lily`, only the
//! top-level [`with_layout`] function is exposed.
//!
//! # Basic Usage
//!
//! ```rust
//! use lily_ast::lexer::cursor::tokenize;
//! use lily_ast::lexer::layout::with_layout;
//!
//! let source = "a = 0\nb = 0";
//! let tokens = with_layout(source, tokenize(source));
//! ```
//!
//! [`depth`]: Token::depth
use super::types::{IdentifierK, LayoutK, OperatorK, Token, TokenK};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    pub line: usize,
    pub column: usize,
}

struct LayoutEngine {
    delimiters: Vec<(Position, DelimiterK)>,
    depth: usize,
}

impl LayoutEngine {
    fn new(initial_position: Position) -> Self {
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
    fn add_layout(
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

    fn finalize_layout(&mut self, tokens: &mut Vec<Token>, eof_offset: usize) {
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

/// Inserts layout rules to a stream of tokens by consuming it and
/// constructing a new one.
pub fn with_layout(source: &str, input_tokens: Vec<Token>) -> Vec<Token> {
    if input_tokens.is_empty() {
        return input_tokens;
    }

    let get_position = |offset| {
        let mut line = 1;
        let mut column = 1;

        for (current, character) in source.chars().enumerate() {
            if current == offset {
                break;
            }
            if character == '\n' {
                column = 1;
                line += 1
            } else {
                column += 1;
            }
        }

        Position { line, column }
    };

    let initial_position = if let Some(initial_token) = input_tokens.first() {
        get_position(initial_token.begin)
    } else {
        return input_tokens;
    };

    let mut output_tokens = vec![];
    let mut layout_engine = LayoutEngine::new(initial_position);

    for (index, &token) in input_tokens.iter().enumerate() {
        let next_begin = match input_tokens.get(index + 1) {
            Some(next) => next.begin,
            None => {
                layout_engine.finalize_layout(&mut output_tokens, source.len());
                output_tokens.push(token);
                break;
            }
        };
        layout_engine.add_layout(
            &mut output_tokens,
            token,
            get_position(token.begin),
            get_position(next_begin),
        )
    }

    output_tokens
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::lexer::{
        tokenize,
        types::{LayoutK, TokenK},
    };

    use super::with_layout;

    const SOURCE: &str = r"Identity : Type -> Type
Identity a ?
  _ : a -> Identity a

Equal : Type -> Type -> Boolean
Equal a b !
  _ : a -> a -> True
  _ : a -> b -> False

Eq : Type -> Constraint
Eq a |
  eq : a -> a -> Boolean

head : List a -> Maybe a
head xs = case xs of
  Cons x _ -> Just x
  Nil      -> Nothing

main : Effect Unit
main = do
  log message
  log message
  attempt do
    log message
    log message

ofCollapse : Int
ofCollapse =
  case
    do _ <- pure 0
       pure 1
  of
    Just x -> x
    Nothing -> 0

lambdaMask : List a -> Maybe a
lambdaMask xs = case xs of
  Cons x _ if (\_ -> true) x ->
    Just x
  _ ->
    Nothing

arrowFinishDo : List a -> Maybe a
arrowFinishDo xs = case xs of
  Cons x _ if do true ->
    Just x
  _ ->
    Nothing

conditionalDo : Effect Unit
conditionalDo = do
  log something
  if do true then do
    log something
  else do
    log something

letIn : Int
letIn =
  let
    x : Int
    x = 1

    y : Int
    y = 1
  in
    x + y

adoIn : Int
adoIn = ado
  x <- pure 1
  y <- pure 1
  let
    a : Int
    a = let b = c in d

    e : Int
    e = let f = g in h
  in x + y

adoLet : Effect Unit
adoLet = do
  logShow $ x + y
  let
    x : Int
    x = 1

    y : Int
    y = 1
  logShow $ x + y
";

    #[test]
    fn ascending_position() {
        let tokens = with_layout(SOURCE, tokenize(SOURCE));
        for window in tokens.windows(2) {
            assert!(window[0].begin <= window[1].begin);
            assert!(window[0].end <= window[1].end);
        }
    }

    #[test]
    fn basic_layout_test() {
        let mut actual = String::new();
        let expected = r"Identity : Type -> Type;0
Identity a ?{1
  _ : a -> Identity a;1}1;0

Equal : Type -> Type -> Boolean;0
Equal a b !{1
  _ : a -> a -> True;1
  _ : a -> b -> False;1}1;0

Eq : Type -> Constraint;0
Eq a |{1
  eq : a -> a -> Boolean;1}1;0

head : List a -> Maybe a;0
head xs = case xs of{1
  Cons x _ -> Just x;1
  Nil      -> Nothing;1}1;0

main : Effect Unit;0
main = do{1
  log message;1
  log message;1
  attempt do{2
    log message;2
    log message;2}2;1}1;0

ofCollapse : Int;0
ofCollapse =
  case
    do{1 _ <- pure 0;1
       pure 1;1}1
  of{1
    Just x -> x;1
    Nothing -> 0;1}1;0

lambdaMask : List a -> Maybe a;0
lambdaMask xs = case xs of{1
  Cons x _ if (\_ -> true) x ->
    Just x;1
  _ ->
    Nothing;1}1;0

arrowFinishDo : List a -> Maybe a;0
arrowFinishDo xs = case xs of{1
  Cons x _ if do{2 true;2}2 ->
    Just x;1
  _ ->
    Nothing;1}1;0

conditionalDo : Effect Unit;0
conditionalDo = do{1
  log something;1
  if do{2 true;2}2 then do{2
    log something;2}2
  else do{2
    log something;2}2;1}1;0

letIn : Int;0
letIn =
  let{1
    x : Int;1
    x = 1;1

    y : Int;1
    y = 1;1}1
  in
    x + y;0

adoIn : Int;0
adoIn = ado{1
  x <- pure 1;1
  y <- pure 1;1
  let{2
    a : Int;2
    a = let{3 b = c;3}3 in d;2

    e : Int;2
    e = let{3 f = g;3}3 in h;2}2;1}1
  in x + y;0

adoLet : Effect Unit;0
adoLet = do{1
  logShow $ x + y;1
  let{2
    x : Int;2
    x = 1;2

    y : Int;2
    y = 1;2}2;1
  logShow $ x + y;1}1;0
";

        for token in with_layout(SOURCE, tokenize(SOURCE)) {
            if let TokenK::Layout(layout) = token.kind {
                match layout {
                    LayoutK::Begin => actual.push_str(format!("{{{}", token.depth).as_str()),
                    LayoutK::End => actual.push_str(format!("}}{}", token.depth).as_str()),
                    LayoutK::Separator => actual.push_str(format!(";{}", token.depth).as_str()),
                }
            } else {
                actual.push_str(&SOURCE[token.comment_begin..token.comment_end]);
                actual.push_str(&SOURCE[token.begin..token.end]);
            }
        }

        assert_eq!(actual, expected);
    }
}
