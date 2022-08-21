use self::{cursor::Cursor, layout::LayoutEngine, types::Token};

pub mod cursor;
pub mod layout;
pub mod types;

pub fn lex(source: &str) -> Vec<Token> {
    let tokens = {
        let mut cursor = Cursor::new(source);
        let mut tokens = vec![];
        loop {
            let token = cursor.take_token();
            tokens.push(token);
            if token.is_eof() {
                break tokens;
            }
        }
    };

    if let [token] = &tokens[..] {
        if token.is_eof() {
            return tokens;
        }
    }

    let tokens = {
        let input_tokens = tokens;

        let get_position = |offset| {
            let mut line = 1;
            let mut column = 1;

            for (current, character) in source.chars().enumerate() {
                if current == offset {
                    break;
                }
                if character == '\n' {
                    column = 1;
                    line += 1
                } else {
                    column += 1;
                }
            }

            types::Position { line, column }
        };

        let initial_position = if let Some(token) = input_tokens.first() {
            get_position(token.begin)
        } else {
            return input_tokens;
        };

        let mut output_tokens = vec![];
        let mut layout_engine = LayoutEngine::new(initial_position);

        for (index, &token) in input_tokens.iter().enumerate() {
            let next_begin = match input_tokens.get(index + 1) {
                Some(next) => next.begin,
                None => {
                    layout_engine.finalize_layout(&mut output_tokens, source.len());
                    output_tokens.push(token.with_depth(layout_engine.depth));
                    break;
                }
            };
            layout_engine.add_layout(
                &mut output_tokens,
                token,
                get_position(token.begin),
                get_position(next_begin),
            )
        }

        output_tokens
    };

    tokens
}
