use crate::lexer::types::Token;

pub fn partition(tokens: &[Token]) -> impl Iterator<Item = &[Token]> {
    let mut tokens_iter = tokens.iter();
    let mut last_start = 0;
    std::iter::from_fn(move || {
        let start = last_start;
        let mut end = last_start;
        loop {
            match tokens_iter.next() {
                Some(token) => {
                    if token.is_eof() {
                        break None;
                    }
                    end += 1;
                    if token.is_separator_zero() {
                        last_start = end;
                        break Some(&tokens[start..end]);
                    }
                }
                None => {
                    if end - start == 0 {
                        break None;
                    } else {
                        last_start = end;
                        break Some(&tokens[start..end]);
                    }
                }
            }
        }
    })
}
