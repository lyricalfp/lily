mod cursor;
mod errors;
mod fixity;
mod group;

use crate::lexer::lex;

use self::{cursor::Cursor, fixity::FixityMap, group::partition};

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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_top_level;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        const SOURCE: &str = r"
infixr 9 apply as $

infixr 5 power as ^

x = f $ x $ y
";

        parse_top_level(SOURCE)?;

        Ok(())
    }
}
