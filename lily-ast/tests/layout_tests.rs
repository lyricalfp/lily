use std::{env, fs, path::Path};

use anyhow::bail;
use lily_ast::lexer::{
    lex,
    types::{LayoutK, TokenK},
};
use pretty_assertions::assert_eq;

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

    let mut input_paths = vec![];

    for entry in fs::read_dir(layout_files.as_path())? {
        let path = entry?.path();
        let extension = path.extension().unwrap();
        match extension.to_str().unwrap() {
            "lily" => input_paths.push(path),
            "out" => (),
            _ => {
                bail!("Unrecognized file: {:?}", path.as_path());
            }
        }
    }

    for input_path in input_paths {
        let mut output_path = input_path.clone();
        output_path.set_extension("lily.out");

        let input_file = fs::read_to_string(input_path)?;
        let output_file = fs::read_to_string(output_path)?;

        assert_eq!(lex_print(&input_file), output_file);
    }

    Ok(())
}
