use fancy_regex::Regex;

#[derive(Debug)]
pub enum Error {
    UnrecognizedToken(usize),
}

#[derive(Debug)]
pub enum Token<'a> {
    DigitDouble(f64),
    DigitInteger(i64),
    NameLower(&'a str),
    NameUpper(&'a str),
    NameSymbol(&'a str),
    ParenLeft,
    ParenRight,
    BracketLeft,
    BracketRight,
    SquareLeft,
    SquareRight,
    SymbolAt,
    SymbolComma,
    SymbolEquals,
    SymbolPeriod,
    SymbolPipe,
    SymbolTick,
    SymbolUnderscore,
    CommentLine(&'a str),
}

type Pattern<'a> = &'a str;
type Creator<'a> = &'a dyn Fn(&'a str) -> Result<Token<'a>, Error>;

#[derive(Default)]
struct Builder<'a>(Vec<(Pattern<'a>, Creator<'a>)>);

impl<'a> Builder<'a> {
    fn push(mut self, p: Pattern<'a>, c: Creator<'a>) -> Self {
        self.0.push((p, c));
        self
    }

    fn build(self) -> Vec<(Regex, Creator<'a>)> {
        self.0
            .into_iter()
            .map(|(pattern, creator)| {
                (
                    Regex::new(format!("^{}", pattern).as_str()).unwrap(),
                    creator,
                )
            })
            .collect()
    }
}

pub struct Lexer<'a> {
    offset: usize,
    source: &'a str,
    patterns: Vec<(Regex, Creator<'a>)>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let offset = 0;
        let patterns = Builder::default()
            .push(r"\p{Lu}[\p{L}+_0-9']*", &|i| Ok(Token::NameUpper(i)))
            .push(r"[\p{Ll}_][\p{L}+_0-9']*", &|i| Ok(Token::NameLower(i)))
            .push(r"([:!#$%&*+./<=>?@\\^|~-]|(?!\p{P})\p{S})+", &|i| {
                Ok(Token::NameSymbol(i))
            })
            .push(r"--( \|)?.+\n*", &|i| {
                Ok(Token::CommentLine(i.trim_start_matches("-- |").trim_start_matches("--").trim()))
            })
            .push(r"[\[\](){}@,=.|`_]", &|i| {
                Ok(match i {
                    "(" => Token::ParenLeft,
                    ")" => Token::ParenRight,
                    "[" => Token::SquareLeft,
                    "]" => Token::SquareRight,
                    "{" => Token::BracketLeft,
                    "}" => Token::BracketRight,
                    "@" => Token::SymbolAt,
                    "," => Token::SymbolComma,
                    "=" => Token::SymbolEquals,
                    "." => Token::SymbolPeriod,
                    "|" => Token::SymbolPipe,
                    "`" => Token::SymbolTick,
                    "_" => Token::SymbolUnderscore,
                    _ => panic!("Lexer::new - this path is never taken"),
                })
            })
            .build();
        Self {
            offset,
            source,
            patterns,
        }
    }

    #[inline]
    fn window(&self) -> &'a str {
        &self.source[self.offset..]
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(usize, Token<'a>, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // handle eof
        if self.offset >= self.source.len() {
            return None;
        }
        // skip whitespaces
        let whitespace = Regex::new(r"^\s+").unwrap();
        if let Ok(Some(m)) = whitespace.find(self.window()) {
            self.offset += m.end();
        }
        // everything else
        let longest_match = self
            .patterns
            .iter()
            .filter_map(|(regex, creator)| {
                if let Ok(Some(m)) = regex.find(self.window()) {
                    Some((m.end(), creator(m.as_str())))
                } else {
                    None
                }
            })
            .max_by_key(|(length, _)| *length);

        match longest_match {
            Some((length, created)) => match created {
                Ok(token) => {
                    let left_offset = self.offset;
                    self.offset += length;
                    Some(Ok((left_offset, token, self.offset)))
                }
                Err(error) => Some(Err(error)),
            },
            None => Some(Err(Error::UnrecognizedToken(self.offset))),
        }
    }
}

#[test]
pub fn it_works_as_intended() {
    for token in Lexer::new("let x = x in [x + x, x * x, x - x, x -- | x, x == x]\n\nid a = a").take(30) {
        println!("{:?}", token);
    }
}
