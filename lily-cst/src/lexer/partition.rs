use std::{cmp::Ordering, iter};

use super::cursor::{Token, TokenK};

pub fn split(
    i: impl Iterator<Item = Token> + Clone,
) -> (impl Iterator<Item = Token>, impl Iterator<Item = Token>) {
    let j = i.clone();
    (
        i.filter(|token| !matches!(token.kind, TokenK::Comment(_) | TokenK::Whitespace)),
        j.filter(|token| matches!(token.kind, TokenK::Comment(_) | TokenK::Whitespace)),
    )
}

pub fn join(
    s: &str,
    i: impl Iterator<Item = Token>,
    j: impl Iterator<Item = Token>,
) -> impl Iterator<Item = Token> {
    let mut i = i.peekable();
    let mut j = j.peekable();
    let mut k = iter::once(Token {
        begin: s.len(),
        end: s.len(),
        kind: TokenK::Eof,
    });
    iter::from_fn(move || match (i.peek(), j.peek()) {
        (Some(x), Some(y)) => match x.kind {
            // layout tokens have priority over annotation tokens
            TokenK::Layout(k) => {
                i.next();
                Some(Token {
                    begin: y.begin,
                    end: y.begin,
                    kind: TokenK::Layout(k),
                })
            }
            // on non-zero-width tokens, choose what comes first
            _ => match x.begin.cmp(&y.begin) {
                Ordering::Less => i.next(),
                Ordering::Greater => j.next(),
                Ordering::Equal => panic!("uncaught zero-width token {:?} {:?}", x, y),
            },
        },
        (Some(_), None) => i.next(),
        (None, Some(_)) => j.next(),
        (None, None) => k.next(),
    })
}
