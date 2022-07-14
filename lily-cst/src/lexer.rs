mod utils;

use crate::{
    cursor::{Cursor, Token},
    layout::Layout,
    lines::Lines,
};

pub fn lex(source: &str) -> impl Iterator<Item = Token> + '_ {
    let cursor = Cursor::new(source);
    let lines = Lines::new(source);
    let (tokens, annotations) = utils::split(cursor);
    let with_layout = Layout::new(lines, tokens);
    utils::join(with_layout, annotations)
}

#[cfg(test)]
mod tests {
    use crate::{
        cursor::{LayoutK, TokenK},
        lexer::lex,
    };

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
";

    #[test]
    fn ascending_position() {
        let tokens = lex(SOURCE).collect::<Vec<_>>();
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
<eof>
";

        for token in lex(SOURCE) {
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
