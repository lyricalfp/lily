mod declaration;
mod expression;
mod fixity;
mod patterns;

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;

    use crate::{
        lexer::lex,
        parser::{
            cursor::Cursor,
            fixity::{Associativity, Fixity, FixityMap},
        },
    };

    #[test]
    fn core_it_works() {
        let source = "a * b + c * d";
        let tokens = lex(&source);
        let mut fixity_map = FixityMap::default();
        fixity_map.insert(
            SmolStr::new("+"),
            Fixity {
                begin: 0,
                end: 0,
                associativity: Associativity::Infixl,
                binding_power: 1,
                identifier: SmolStr::new("+"),
            },
        );
        fixity_map.insert(
            SmolStr::new("*"),
            Fixity {
                begin: 0,
                end: 0,
                associativity: Associativity::Infixl,
                binding_power: 3,
                identifier: SmolStr::new("*"),
            },
        );
        let mut cursor = Cursor::new(&source, tokens.into_iter());
        dbg!(cursor.expression(&mut fixity_map).unwrap());
    }
}
