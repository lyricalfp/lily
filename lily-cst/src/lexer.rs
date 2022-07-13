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

    #[test]
    fn basic_layout_test() {
        let source = r"
module Main

Identity a ?
  _ : a -> Identity a

Equal a b !
  _ : a -> a -> True
  _ : a -> b -> False

Eq a |
  eq : a -> a -> Boolean
";
        for token in lex(source) {
            if let TokenK::Layout(layout) = token.kind {
                match layout {
                    LayoutK::Begin => print!("{{"),
                    LayoutK::End => print!("}}"),
                    LayoutK::Separator => print!(";"),
                }
            } else {
                print!("{}", &source[token.begin..token.end]);
            }
        }
        println!();
    }
}
