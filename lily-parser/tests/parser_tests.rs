use lily_parser::parse_top_level;

#[test]
pub fn top_level_0() {
    let source = r"
infixl 4 add as +

example = a + b + c
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
pub fn top_level_1() {
    let source = r"
example = a + b + c
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
pub fn top_level_2() {
    let source = r"
infixl 1 add as +
infixl 2 mul as *
infixr 3 pow as ^

example = a + b * (c + d) + e ^ f
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
pub fn top_level_3() {
    let source = r"
infixl 1 add as +

example = if if a then b else c then if d then e else f else if g then h else i
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
pub fn top_level_4() {
    let source = r"
example = f a b
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
pub fn top_level_5() {
    let source = r"
example = f if a then b else c d

example = f (if a then b else c) d
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
fn top_level_6() {
    let source = "
example = do
  let
    u = 21
    v = 21
  w <- pure 21
  x <- pure 21
  attempt do
    y <- pure 21
    z <- pure 21
";
    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
fn top_level_7() {
    let source = "
example a b c = a b c
";
    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
fn top_level_8() {
    let source = "
example = case a, b of
  Cons a b, _ if hello -> do
    world
  _, Nil -> do
    example
";
    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
fn top_level_9() {
    let source = "
example = f
  let
    a = b
    c = d
  in
    a c
";
    insta::assert_debug_snapshot!(parse_top_level(source));
}

#[test]
fn top_level_10() {
    let source = "
infixr 9 type Function as ->

example : A a -> B -> C -> D (d e)
";
    insta::assert_debug_snapshot!(parse_top_level(source));
}
