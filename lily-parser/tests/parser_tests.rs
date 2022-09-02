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
infixl 2 add as *

example = a + b * (c + d) + e
";

    insta::assert_debug_snapshot!(parse_top_level(source));
}
