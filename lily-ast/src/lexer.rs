//! Top-level API for the lexer.
//!
//! # Basic Usage
//!
//! ```rust
//! use lily_ast::lexer::lex;
//!
//! let tokens = lex("a = 0\nb = 0");
//!
//! assert_eq!(tokens.len(), 13);
//! ```
pub use self::{cursor::tokenize, layout::with_layout, types::Token};

pub mod cursor;
pub mod layout;
pub mod types;

/// Creates a stream of tokens from a source file.
pub fn lex(source: &str) -> Vec<Token> {
    with_layout(source, tokenize(source))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{
        lex,
        types::{LayoutK, TokenK},
    };

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
        let tokens = lex(SOURCE);
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

        let mut collected_whitespace = vec![];

        for token in lex(SOURCE) {
            if let TokenK::Whitespace = token.kind {
                collected_whitespace.push(token);
            } else if let TokenK::Layout(layout) = token.kind {
                match layout {
                    LayoutK::Begin => actual.push_str(format!("{{{}", token.depth).as_str()),
                    LayoutK::End => actual.push_str(format!("}}{}", token.depth).as_str()),
                    LayoutK::Separator => actual.push_str(format!(";{}", token.depth).as_str()),
                }
            } else {
                while let Some(whitespace) = collected_whitespace.pop() {
                    actual.push_str(&SOURCE[whitespace.begin..whitespace.end]);
                }
                actual.push_str(&SOURCE[token.begin..token.end]);
            }
        }

        actual.push('\n');
        print!("{}", actual);
        assert_eq!(actual, expected);
    }
}
