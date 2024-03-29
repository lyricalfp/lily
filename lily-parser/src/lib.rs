mod core;
mod cursor;
mod errors;
pub mod types;

use lily_lexer::{lex, types::Token};
use types::Module;

use crate::{
    cursor::Cursor,
    types::{Domain, FixityMap},
};

pub fn parse_top_level(source: &str) -> anyhow::Result<Module> {
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

    let mut value_fixities = FixityMap::default();
    let mut type_fixities = FixityMap::default();

    for fixity_group in fixity_groups {
        let mut cursor = Cursor::new(source, fixity_group, None, None);
        let (operator, fixity) = cursor.fixity()?;
        match fixity.domain {
            Domain::Type => type_fixities.insert(operator, fixity),
            Domain::Value => value_fixities.insert(operator, fixity),
        };
        debug_assert!(cursor.is_eof());
    }

    let mut declarations = vec![];
    for declaration_group in declaration_groups {
        let mut cursor = Cursor::new(
            source,
            declaration_group,
            Some(&value_fixities),
            Some(&type_fixities),
        );
        declarations.push(cursor.declaration()?);
        debug_assert!(cursor.is_eof());
    }

    Ok(Module { declarations })
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
