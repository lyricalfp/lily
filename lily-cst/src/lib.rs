pub mod lexer;
pub mod token;

#[test]
pub fn it_works_as_intended() {
    for token in lexer::Lexer::new("main :: Effect Unit\nmain = do\n  pure unit").take(30) {
        println!("{:?}", token);
    }
}
