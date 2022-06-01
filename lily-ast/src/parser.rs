#[test]
fn it_works_as_intended() {
    use crate::{colosseum::Colosseum, grammar};

    let arena = bumpalo::Bump::new();
    let mut colosseum = Colosseum::new(&arena);

    let input = "Function Int Int";
    let parsed = grammar::Type0Parser::new().parse(&mut colosseum, input);

    println!("{:?}", parsed);
}
