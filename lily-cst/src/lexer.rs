pub mod cursor;
pub mod layout;
pub mod partition;

use self::{
    cursor::{Cursor, Token},
    layout::Layout,
};

use crate::{error::CstErr, lines::Lines};

pub fn lex(source: &str) -> Result<impl Iterator<Item = Token> + '_, CstErr> {
    if source.is_empty() {
        Err(CstErr::EmptySourceFile)
    } else {
        Ok(lex_non_empty(source))
    }
}

pub fn lex_non_empty(source: &str) -> impl Iterator<Item = Token> + '_ {
    assert!(!source.is_empty());
    let cursor = Cursor::new(source);
    let lines = Lines::new(source);
    let (tokens, annotations) = partition::split(cursor);
    let with_layout = Layout::new(lines, tokens);
    partition::join(lines, with_layout, annotations)
}

#[cfg(test)]
mod tests {
    use crate::lexer::{
        cursor::{LayoutK, TokenK},
        lex_non_empty,
    };
    use pretty_assertions::assert_eq;

    const SOURCE: &str = r"module Main

Identity : Type -> Type
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
        let tokens = lex_non_empty(SOURCE).collect::<Vec<_>>();
        for window in tokens.windows(2) {
            assert!(window[0].begin <= window[1].begin);
            assert!(window[0].end <= window[1].end);
        }
    }

    #[test]
    fn basic_layout_test() {
        let mut actual = String::new();
        let expected = r"module Main;

Identity : Type -> Type;
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
<eof>
";

        for token in lex_non_empty(SOURCE) {
            if let TokenK::Layout(layout) = token.kind {
                match layout {
                    LayoutK::Begin => actual.push('{'),
                    LayoutK::End => actual.push('}'),
                    LayoutK::Separator => actual.push(';'),
                }
            } else if let TokenK::Eof = token.kind {
                actual.push_str("<eof>");
            } else {
                actual.push_str(&SOURCE[token.begin..token.end]);
            }
        }
        actual.push('\n');
        print!("{}", actual);
        assert_eq!(actual, expected);
    }
}
