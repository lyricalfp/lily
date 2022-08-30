use lily_ast::lexer::{
    lex,
    types::{LayoutK, TokenK},
};

fn lex_print(source: &str) -> String {
    let tokens = lex(source);
    let mut buffer = String::new();
    for token in tokens {
        if let TokenK::Layout(layout) = token.kind {
            match layout {
                LayoutK::Begin => buffer.push_str(format!("{{{}", token.depth).as_str()),
                LayoutK::End => buffer.push_str(format!("}}{}", token.depth).as_str()),
                LayoutK::Separator => buffer.push_str(format!(";{}", token.depth).as_str()),
            }
        } else {
            buffer.push_str(&source[token.comment_begin..token.comment_end]);
            buffer.push_str(&source[token.begin..token.end]);
        }
    }
    buffer
}

#[test]
fn layout_0() {
    let source = r"Identity : Type -> Type
Identity a ?
  _ : a -> Identity a";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_1() {
    let source = r"Equal : Type -> Type -> Boolean
Equal a b !
  _ : a -> a -> True
  _ : a -> b -> False";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_2() {
    let source = r"Eq : Type -> Constraint
Eq a |
  eq : a -> a -> Boolean";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_3() {
    let source = r"head : List a -> Maybe a
head xs = case xs of
  Cons x _ -> Just x
  Nil      -> Nothing";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_4() {
    let source = r"main : Effect Unit
main = do
  log message
  log message
  attempt do
    log message
    log message";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_5() {
    let source = r"ofCollapse : Int
ofCollapse =
  case
    do _ <- pure 0
       pure 1
  of
    Just x -> x
    Nothing -> 0";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_6() {
    let source = r"lambdaMask : List a -> Maybe a
lambdaMask xs = case xs of
  Cons x _ if (\_ -> true) x ->
    Just x
  _ ->
    Nothing";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_7() {
    let source = r"arrowFinishDo : List a -> Maybe a
arrowFinishDo xs = case xs of
  Cons x _ if do true ->
    Just x
  _ ->
    Nothing";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_8() {
    let source = r"conditionalDo : Effect Unit
conditionalDo = do
  log something
  if do true then do
    log something
  else do
    log something";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_9() {
    let source = r"letIn : Int
letIn =
  let
    x : Int
    x = 1
    y : Int
    y = 1
  in
    x + y";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_10() {
    let source = r"adoIn : Int
adoIn = ado
  x <- pure 1
  y <- pure 1
  let
    a : Int
    a = let b = c in d
    e : Int
    e = let f = g in h
  in x + y";

    insta::assert_snapshot!(lex_print(source));
}

#[test]
fn layout_11() {
    let source = r"adoLet : Effect Unit
adoLet = do
  logShow $ x + y
  let
    x : Int
    x = 1
    y : Int
    y = 1
  logShow $ x + y";

    insta::assert_snapshot!(lex_print(source));
}
