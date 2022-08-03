//! Top-level API for the lexer.
//!
//! # Basic Usage
//!
//! ```rust
//! use lily_ast::lexer::lex;
//!
//! let tokens = lex("a = 0\nb = 0");
//! ```
use self::{
    cursor::{tokenize, Token},
    layout::with_layout,
};

pub mod cursor;
pub mod layout;

/// Creates a stream of tokens from a source file.
pub fn lex(source: &str) -> Vec<Token> {
    with_layout(source, tokenize(source))
}

#[cfg(test)]
mod tests {
    use crate::lexer::cursor::{LayoutK, TokenK};
    use pretty_assertions::assert_eq;

    use super::lex;

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
        let expected = r"Identity : Type -> Type;
Identity a ?{
  _ : a -> Identity a};

Equal : Type -> Type -> Boolean;
Equal a b !{
  _ : a -> a -> True;
  _ : a -> b -> False};

Eq : Type -> Constraint;
Eq a |{
  eq : a -> a -> Boolean};

head : List a -> Maybe a;
head xs = case xs of{
  Cons x _ -> Just x;
  Nil      -> Nothing};

main : Effect Unit;
main = do{
  log message;
  log message;
  attempt do{
    log message;
    log message}};

ofCollapse : Int;
ofCollapse =
  case
    do{ _ <- pure 0;
       pure 1}
  of{
    Just x -> x;
    Nothing -> 0};

lambdaMask : List a -> Maybe a;
lambdaMask xs = case xs of{
  Cons x _ if (\_ -> true) x ->
    Just x;
  _ ->
    Nothing};

arrowFinishDo : List a -> Maybe a;
arrowFinishDo xs = case xs of{
  Cons x _ if do{ true} ->
    Just x;
  _ ->
    Nothing};

conditionalDo : Effect Unit;
conditionalDo = do{
  log something;
  if do{ true} then do{
    log something}
  else do{
    log something}};

letIn : Int;
letIn =
  let{
    x : Int;
    x = 1;

    y : Int;
    y = 1}
  in
    x + y;

adoIn : Int;
adoIn = ado{
  x <- pure 1;
  y <- pure 1;
  let{
    a : Int;
    a = let{ b = c} in d;

    e : Int;
    e = let{ f = g} in h}}
  in x + y;

adoLet : Effect Unit;
adoLet = do{
  logShow $ x + y;
  let{
    x : Int;
    x = 1;

    y : Int;
    y = 1};
  logShow $ x + y};
";

        let mut collected_whitespace = vec![];

        for token in lex(SOURCE) {
            if let TokenK::Whitespace = token.kind {
                collected_whitespace.push(token);
            } else if let TokenK::Layout(layout) = token.kind {
                match layout {
                    LayoutK::Begin => actual.push('{'),
                    LayoutK::End => actual.push('}'),
                    LayoutK::Separator => actual.push(';'),
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
