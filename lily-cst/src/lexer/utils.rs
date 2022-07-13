use std::{cmp::Ordering, iter};

use crate::cursor::{Token, TokenK};

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
    i: impl Iterator<Item = Token>,
    j: impl Iterator<Item = Token>,
) -> impl Iterator<Item = Token> {
    let mut i = i.peekable();
    let mut j = j.peekable();
    iter::from_fn(move || match (i.peek(), j.peek()) {
        (Some(x), Some(y)) => match x.begin.cmp(&y.begin) {
            Ordering::Less => i.next(),
            Ordering::Equal => i.next(),
            Ordering::Greater => {
                if let TokenK::Layout(_) = x.kind {
                    i.next()
                } else {
                    j.next()
                }
            }
        },
        (Some(_), None) => i.next(),
        (None, Some(_)) => j.next(),
        (None, None) => None,
    })
}
