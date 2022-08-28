mod core;
mod cursor;
mod errors;
mod fixity;

use crate::lexer::{lex, types::Token};

use self::{cursor::Cursor, fixity::FixityMap};

pub fn parse_top_level(source: &str) -> anyhow::Result<()> {
    let tokens = lex(source);

    let mut fixity_groups = vec![];
    let mut declaration_groups = vec![];
    for group in partition(&tokens) {
        let token = group.first().unwrap();
        if token.is_infix_identifier() {
            fixity_groups.push(group);
        } else {
            declaration_groups.push(group);
        }
    }

    let mut fixity_map = FixityMap::default();
    for fixity_group in fixity_groups {
        let tokens = fixity_group.into_iter().copied();
        let (operator, fixity) = Cursor::new(source, tokens).fixity()?;
        fixity_map.insert(operator, fixity);
    }

    let mut declarations = vec![];
    for declaration_group in declaration_groups {
        let tokens = declaration_group.into_iter().copied();
        let declaration = Cursor::new(source, tokens).declaration(&mut fixity_map)?;
        declarations.push(declaration);
    }

    Ok(())
}

fn partition(tokens: &[Token]) -> impl Iterator<Item = &[Token]> {
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

#[cfg(test)]
mod tests {
    use super::parse_top_level;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        const SOURCE: &str = r"
infixr 9 apply as $

infixr 5 power as ^

x : Int
x = f $ x $ y
";

        parse_top_level(SOURCE)?;

        Ok(())
    }
}
