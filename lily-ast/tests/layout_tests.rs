use std::{env, fs, path::Path};

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
fn golden_layout_tests() -> anyhow::Result<()> {
    let layout_files = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("tests")
        .join("layout_tests");

    for file_entry in fs::read_dir(layout_files.as_path())? {
        let file_entry = file_entry?;
        let file_contents = fs::read_to_string(file_entry.path())?;
        insta::assert_snapshot!(lex_print(&file_contents));
    }

    Ok(())
}
